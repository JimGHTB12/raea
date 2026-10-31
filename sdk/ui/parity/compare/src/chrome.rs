//! The oracle: React DOM in headless Chrome (Chrome for Testing's chrome-headless-shell, pinned in chrome.sh), driven
//! over the DevTools protocol. One browser per worker, a fresh page (target) per case, at a fixed viewport and a device
//! scale of 1. Input goes in as Input.dispatchMouseEvent / dispatchKeyEvent / insertText; what the page shows comes
//! back as every data-testid's box and state (STATE_JS) and a screenshot (Page.captureScreenshot: print-to-pdf hangs on
//! this machine, screenshots don't).

use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use base64::Engine as _;
use serde_json::{Value, json};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

use crate::compare::{Snap, chrome_node};
use crate::Input;

pub struct Browser {
    child: Child,
    ws: WebSocket<MaybeTlsStream<TcpStream>>,
    next: u64,
    pub version: String,
    _profile: PathBuf,
}

impl Drop for Browser {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self._profile);
    }
}

impl Browser {
    pub fn launch(bin: &Path, tmp: &Path) -> Result<Browser, String> {
        let profile = tmp.join(format!("chrome-{}-{}", std::process::id(), crate::uniq()));
        std::fs::create_dir_all(&profile).map_err(|e| e.to_string())?;
        let mut child = Command::new(bin)
            .args([
                "--headless",
                "--remote-debugging-port=0",
                &format!("--user-data-dir={}", profile.display()),
                "--no-first-run",
                "--no-default-browser-check",
                "--hide-scrollbars",
                "--force-device-scale-factor=1",
                "--font-render-hinting=none",
                "--disable-lcd-text",
                "--disable-gpu",
                "--disable-smooth-scrolling",
                "--disable-background-timer-throttling",
                "--disable-renderer-backgrounding",
                "--disable-backgrounding-occluded-windows",
                "--allow-file-access-from-files",
                "--force-color-profile=srgb",
                "about:blank",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("launching {}: {e}", bin.display()))?;
        // "DevTools listening on ws://127.0.0.1:PORT/devtools/browser/ID"
        let err = child.stderr.take().ok_or("no stderr")?;
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut sent = false;
            for line in BufReader::new(err).lines().map_while(Result::ok) {
                if !sent && let Some(u) = line.split("DevTools listening on ").nth(1) {
                    let _ = tx.send(u.trim().to_string());
                    sent = true;
                }
            }
        });
        let url = match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(u) => u,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("chrome didn't say where DevTools listens in 30 s".into());
            }
        };
        let (ws, _) = tungstenite::connect(&url).map_err(|e| format!("devtools {url}: {e}"))?;
        if let MaybeTlsStream::Plain(s) = ws.get_ref() {
            s.set_read_timeout(Some(Duration::from_secs(60))).ok();
        }
        let mut b = Browser { child, ws, next: 1, version: String::new(), _profile: profile };
        b.version = b.send("Browser.getVersion", json!({}), None)?["product"].as_str().unwrap_or("").to_string();
        Ok(b)
    }

    /// One command, its result (events in between are dropped: nothing here listens for them).
    pub fn send(&mut self, method: &str, params: Value, session: Option<&str>) -> Result<Value, String> {
        let id = self.next;
        self.next += 1;
        let mut msg = json!({"id": id, "method": method, "params": params});
        if let Some(s) = session {
            msg["sessionId"] = json!(s);
        }
        self.ws.send(Message::Text(msg.to_string().into())).map_err(|e| format!("{method}: {e}"))?;
        let t = Instant::now();
        loop {
            let m = self.ws.read().map_err(|e| format!("{method}: no answer ({e}) after {} ms", t.elapsed().as_millis()))?;
            let Message::Text(txt) = m else { continue };
            let v: Value = serde_json::from_str(&txt).map_err(|e| e.to_string())?;
            if v["id"].as_u64() == Some(id) {
                if let Some(e) = v.get("error") {
                    return Err(format!("{method}: {}", e["message"].as_str().unwrap_or(&e.to_string())));
                }
                return Ok(v["result"].clone());
            }
        }
    }

    /// A fresh page for a case: its own target, the viewport set, `side` given to the probe before anything runs.
    pub fn page(&mut self, w: u32, h: u32, side: &str) -> Result<Page, String> {
        let t = self.send("Target.createTarget", json!({"url": "about:blank"}), None)?;
        let target = t["targetId"].as_str().ok_or("no targetId")?.to_string();
        let s = self.send("Target.attachToTarget", json!({"targetId": target, "flatten": true}), None)?;
        let session = s["sessionId"].as_str().ok_or("no sessionId")?.to_string();
        let ss = Some(session.as_str());
        self.send("Page.enable", json!({}), ss)?;
        self.send("Runtime.enable", json!({}), ss)?;
        self.send("Emulation.setDeviceMetricsOverride", json!({"width": w, "height": h, "deviceScaleFactor": 1, "mobile": false}), ss)?;
        // the generic families desktop Chrome on macOS has (its platform prefs); headless shell has Blink's built-in
        // Times New Roman, Arial and Courier New, so a list of missing faces fell to a face no user's Chrome draws
        #[cfg(target_vendor = "apple")]
        self.send("Page.setFontFamilies", json!({"fontFamilies": {"standard": "Times", "serif": "Times", "sansSerif": "Helvetica", "fixed": "Courier"}}), ss)?;
        self.send("Page.addScriptToEvaluateOnNewDocument", json!({"source": format!("window.__parity = {{side: {side:?}, trace: []}};")}), ss)?;
        // the async clipboard as an app in a focused window has it (P17's clipboard case); headless Chrome refuses it
        // otherwise. Best effort: an older Chrome without these names keeps its refusal
        let _ = self.send("Browser.grantPermissions", json!({"permissions": ["clipboardReadWrite", "clipboardSanitizedWrite"]}), None);
        Ok(Page { target, session, w, h, pending_wait: 0, held: 0, at: None, a11y: false })
    }
}

pub struct Page {
    target: String,
    session: String,
    pub w: u32,
    pub h: u32,
    pending_wait: u32,
    /// the mouse buttons held (CDP's `buttons` bitmask), carried on every mouse event as a real mouse's are
    held: u32,
    /// read the accessibility tree too (P16)
    pub a11y: bool,
    /// where the mouse is: a move to where it already is isn't sent, as a real mouse sends none (CDP's mouseMoved
    /// there fires a pointermove anyway, which a press held under capture would hear)
    at: Option<(f64, f64)>,
}

/// Every data-testid's box and state, once the page has settled: at least `min` animation frames, then until two
/// frames running report the same (at most 90).
const STATE_JS: &str = r#"(async (min) => {
  const raf = () => new Promise((r) => requestAnimationFrame(() => r()));
  await document.fonts.ready;
  const snap = () => [...document.querySelectorAll("[data-testid]")].map((el) => {
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    const tag = el.tagName;
    let value = el.hasAttribute("data-value") ? el.getAttribute("data-value") : null;
    if (tag === "INPUT" && (el.type === "checkbox" || el.type === "radio")) value = el.checked;
    else if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") value = el.value;
    const scrolls = /(auto|scroll)/.test(cs.overflowY + " " + cs.overflowX) && (el.scrollHeight > el.clientHeight || el.scrollWidth > el.clientWidth);
    const n = {
      id: el.getAttribute("data-testid"),
      rect: { x: r.x, y: r.y, w: r.width, h: r.height },
      shown: el.getClientRects().length > 0,
      value,
      focused: document.activeElement === el,
      hovered: el.matches(":hover"),
    };
    if (scrolls) n.scroll = el.scrollTop;
    // a field's selection (null where the type has none, as a number field's)
    if ((tag === "INPUT" || tag === "TEXTAREA") && el.selectionStart != null) n.caret = [el.selectionStart, el.selectionEnd];
    return n;
  });
  let last = null, out = null;
  for (let i = 0; i < 90; i++) {
    await raf();
    if (i + 1 < min) continue;
    out = JSON.stringify(snap());
    if (out === last) break;
    last = out;
  }
  return JSON.stringify({ nodes: JSON.parse(out), trace: (window.__parity && window.__parity.trace) || [], now: performance.now() });
})"#;

/// CDP's modifiers bitmask from the script's names.
fn mods_bits(mods: &[String]) -> u32 {
    mods.iter().map(|m| match m.as_str() { "alt" => 1, "ctrl" => 2, "meta" => 4, "shift" => 8, _ => 0 }).sum()
}

/// A key's DOM code, Windows virtual key code and the text it types.
fn key_desc(key: &str) -> (String, u32, Option<String>) {
    let named = |code: &str, vk: u32, text: Option<&str>| (code.to_string(), vk, text.map(str::to_string));
    match key {
        "Enter" => named("Enter", 13, Some("\r")),
        "Tab" => named("Tab", 9, None),
        "Backspace" => named("Backspace", 8, None),
        "Escape" => named("Escape", 27, None),
        " " | "Space" => named("Space", 32, Some(" ")),
        "ArrowLeft" => named("ArrowLeft", 37, None),
        "ArrowUp" => named("ArrowUp", 38, None),
        "ArrowRight" => named("ArrowRight", 39, None),
        "ArrowDown" => named("ArrowDown", 40, None),
        "Delete" => named("Delete", 46, None),
        "Home" => named("Home", 36, None),
        "End" => named("End", 35, None),
        "PageUp" => named("PageUp", 33, None),
        "PageDown" => named("PageDown", 34, None),
        k if k.chars().count() == 1 => {
            let c = k.chars().next().unwrap();
            if c.is_ascii_alphabetic() {
                (format!("Key{}", c.to_ascii_uppercase()), c.to_ascii_uppercase() as u32, Some(k.into()))
            } else if c.is_ascii_digit() {
                (format!("Digit{c}"), c as u32, Some(k.into()))
            } else {
                (String::new(), 0, Some(k.into()))
            }
        }
        k => (k.to_string(), 0, None),
    }
}

fn key_name(key: &str) -> &str {
    if key == "Space" { " " } else { key }
}

impl Page {
    /// The page's DevTools session, for commands of a caller's own (src/wpt.rs).
    pub fn session(&self) -> &str {
        &self.session
    }

    pub fn open(&mut self, b: &mut Browser, url: &str) -> Result<(), String> {
        let s = Some(self.session.as_str());
        b.send("Page.navigate", json!({"url": url}), s)?;
        let t = Instant::now();
        loop {
            let r = b.send("Runtime.evaluate", json!({"expression": "document.readyState", "returnByValue": true}), s)?;
            if r["result"]["value"] == json!("complete") {
                break;
            }
            if t.elapsed() > Duration::from_secs(20) {
                return Err("the page didn't load in 20 s".into());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    }

    fn mouse(&mut self, b: &mut Browser, kind: &str, x: f64, y: f64, extra: Value) -> Result<(), String> {
        self.at = Some((x, y));
        let mut p = json!({"type": kind, "x": x, "y": y});
        if let (Some(o), Some(e)) = (p.as_object_mut(), extra.as_object()) {
            o.extend(e.clone());
        }
        b.send("Input.dispatchMouseEvent", p, Some(&self.session)).map(|_| ())
    }

    pub fn input(&mut self, b: &mut Browser, i: &Input) -> Result<(), String> {
        match i {
            Input::Move(x, y) if self.at == Some((*x, *y)) => Ok(()),
            Input::Move(x, y) => self.mouse(b, "mouseMoved", *x, *y, json!({"button": if self.held & 2 != 0 { "right" } else if self.held & 1 != 0 { "left" } else { "none" }, "buttons": self.held})),
            Input::Down(x, y, clicks, right) => {
                self.held |= if *right { 2 } else { 1 };
                self.mouse(b, "mousePressed", *x, *y, json!({"button": if *right { "right" } else { "left" }, "buttons": self.held, "clickCount": clicks}))
            }
            Input::Up(x, y, clicks, right) => {
                self.held &= if *right { !2 } else { !1 };
                self.mouse(b, "mouseReleased", *x, *y, json!({"button": if *right { "right" } else { "left" }, "buttons": self.held, "clickCount": clicks}))
            }
            Input::Leave => self.mouse(b, "mouseMoved", -50.0, -50.0, json!({"button": "none", "buttons": self.held})),
            Input::Wheel(x, y, dx, dy) => self.mouse(b, "mouseWheel", *x, *y, json!({"deltaX": dx, "deltaY": dy})),
            Input::Key(key, mods) => {
                let (code, vk, text) = key_desc(key);
                let m = mods_bits(mods);
                let chord = m & (2 | 4) != 0;
                let mut down = json!({"type": if text.is_some() && !chord { "keyDown" } else { "rawKeyDown" }, "key": key_name(key), "code": code, "windowsVirtualKeyCode": vk, "modifiers": m});
                if let Some(t) = text.as_ref().filter(|_| !chord) {
                    down["text"] = json!(t);
                    down["unmodifiedText"] = json!(t);
                }
                b.send("Input.dispatchKeyEvent", down, Some(&self.session))?;
                b.send("Input.dispatchKeyEvent", json!({"type": "keyUp", "key": key_name(key), "code": code, "windowsVirtualKeyCode": vk, "modifiers": m}), Some(&self.session)).map(|_| ())
            }
            Input::Text(t) => b.send("Input.insertText", json!({"text": t}), Some(&self.session)).map(|_| ()),
            Input::Resize(w, h) => {
                (self.w, self.h) = (*w as u32, *h as u32);
                b.send("Emulation.setDeviceMetricsOverride", json!({"width": self.w, "height": self.h, "deviceScaleFactor": 1, "mobile": false}), Some(&self.session)).map(|_| ())
            }
            Input::Wait(n) => {
                self.pending_wait = self.pending_wait.max(*n);
                Ok(())
            }
        }
    }

    /// The page once settled: its elements, the trace and a screenshot.
    pub fn read(&mut self, b: &mut Browser, min: u32) -> Result<Snap, String> {
        let min = min.max(std::mem::take(&mut self.pending_wait));
        let s = Some(self.session.as_str());
        let r = b.send("Runtime.evaluate", json!({"expression": format!("{STATE_JS}({min})"), "awaitPromise": true, "returnByValue": true}), s)?;
        if let Some(e) = r.get("exceptionDetails") {
            return Err(format!("the page threw: {}", e["exception"]["description"].as_str().unwrap_or(&e.to_string())));
        }
        let v: Value = serde_json::from_str(r["result"]["value"].as_str().unwrap_or("{}")).map_err(|e| e.to_string())?;
        let shot = b.send("Page.captureScreenshot", json!({"format": "png", "fromSurface": true, "captureBeyondViewport": false}), s)?;
        let png = base64::engine::general_purpose::STANDARD.decode(shot["data"].as_str().unwrap_or("")).map_err(|e| e.to_string())?;
        let png = image::load_from_memory(&png).map_err(|e| e.to_string())?.to_rgba8();
        let nodes = v["nodes"].as_array().cloned().unwrap_or_default().iter().filter_map(|n| Some((n["id"].as_str()?.to_string(), chrome_node(n)))).collect();
        let a11y = if self.a11y { Some(self.a11y_tree(b)?) } else { None };
        Ok(Snap { nodes, trace: v["trace"].as_array().cloned().unwrap_or_default(), png, a11y, now: v["now"].as_f64().unwrap_or(0.0) })
    }

    /// Each data-testid's node in Chrome's accessibility tree (src/a11y.rs).
    fn a11y_tree(&self, b: &mut Browser) -> Result<crate::a11y::Tree, String> {
        let s = Some(self.session.as_str());
        let doc = b.send("DOM.getDocument", json!({"depth": -1}), s)?;
        let mut ids = vec![];
        crate::a11y::testids(&doc["root"], &mut ids);
        let mut out = crate::a11y::Tree::new();
        for (id, backend) in ids {
            let r = b.send("Accessibility.getPartialAXTree", json!({"backendNodeId": backend, "fetchRelatives": false}), s)?;
            let n = r["nodes"].as_array().and_then(|a| a.iter().find(|n| n["backendDOMNodeId"].as_u64() == Some(backend)).or(a.first())).cloned();
            out.insert(id, n.map(|n| crate::a11y::chrome_record(&n)).unwrap_or_else(|| json!({"role": "hidden"})));
        }
        Ok(out)
    }

    pub fn close(self, b: &mut Browser) {
        let _ = b.send("Target.closeTarget", json!({"targetId": self.target}), None);
    }
}
