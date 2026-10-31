//! The side under test: the case's bundle on @raea/ui's React adapter, run headless (its window an image the UI camera
//! renders into, no window on screen), one process per case, driven over its Bevy Remote Protocol: input in, ui+watch
//! (the tree after a frame), frame_png+watch and the probe's action `parity.read` (the trace) out. The host is
//! raea-page-host (engine/page/host: `raea-page-host BUNDLE --css F --size WxH`, its methods under `page.`).

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use base64::Engine as _;
use serde_json::{Value, json};

use crate::compare::{Snap, bevy_node};
use crate::Input;

pub struct Host {
    child: Child,
    port: u16,
    pub log: PathBuf,
    /// the frame the latest input landed before
    last: u64,
    pending_wait: u32,
    req: u64,
    /// the JS engine the probe reports (jsc, quickjs)
    pub engine: String,
    /// read the accessibility tree too (P16)
    pub a11y: bool,
    /// how long (ms) Chrome's page ran between its previous read and this one: this side's JS clock runs at least as
    /// long before it's read, so a timer that fired by Chrome's read has fired here (a 30 ms lazy at mount)
    pub catch_up: f64,
    /// the JS clock at this side's previous read
    js_prev: f64,
}

impl Drop for Host {
    fn drop(&mut self) {
        // by PID: the process this run started
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn free_port() -> Result<u16, String> {
    std::net::TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map(|a| a.port()).map_err(|e| e.to_string())
}

/// One BRP call: its result, or its error's message. A +watch method answers as an event stream: its first event is
/// the answer and the connection is dropped after it (bevy_remote then stops watching), as raea-page-host's MCP bridge does.
pub fn brp(port: u16, method: &str, params: Value, limit: Duration) -> Result<Value, String> {
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).to_string();
    let mut sock = std::net::TcpStream::connect(("127.0.0.1", port)).map_err(|e| format!("brp on {port}: {e}"))?;
    sock.set_read_timeout(Some(limit)).ok();
    write!(sock, "POST / HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).map_err(|e| e.to_string())?;
    let mut r = BufReader::new(sock);
    let (mut stream, mut chunked, mut len) = (false, false, None::<usize>);
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).map_err(|e| format!("{method}: {e}"))? == 0 {
            return Err(format!("{method}: the app closed the connection"));
        }
        let l = line.trim_end().to_ascii_lowercase();
        if l.is_empty() {
            break;
        }
        stream |= l.starts_with("content-type:") && l.contains("event-stream");
        chunked |= l.starts_with("transfer-encoding:") && l.contains("chunked");
        if let Some(n) = l.strip_prefix("content-length:") {
            len = n.trim().parse().ok();
        }
    }
    let text = if stream {
        loop {
            let mut line = String::new();
            if r.read_line(&mut line).map_err(|_| format!("{method}: no answer in {} s", limit.as_secs()))? == 0 {
                return Err(format!("{method}: the stream ended without an answer"));
            }
            if let Some(d) = line.trim_end().strip_prefix("data:") {
                break d.trim().to_string();
            }
        }
    } else if let Some(n) = len.filter(|_| !chunked) {
        let mut b = vec![0; n];
        r.read_exact(&mut b).map_err(|e| e.to_string())?;
        String::from_utf8_lossy(&b).into_owned()
    } else {
        let mut all = String::new();
        r.read_to_string(&mut all).map_err(|e| e.to_string())?;
        all.lines().filter(|l| l.trim_start().starts_with('{')).collect::<Vec<_>>().join("")
    };
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{method}: {e}: {}", &text[..text.len().min(200)]))?;
    match (v.get("result"), v.get("error")) {
        (_, Some(e)) => Err(format!("{method}: {}", e["message"].as_str().map(str::to_string).unwrap_or_else(|| e.to_string()))),
        (Some(r), _) => Ok(r.clone()),
        _ => Err(format!("{method}: {text}")),
    }
}

/// OUT/tw.css for a case's bundle at OUT/<area>/<case>/bevy.js.
fn tw_css(bundle: &Path) -> std::ffi::OsString {
    bundle.ancestors().map(|d| d.join("tw.css")).find(|p| p.is_file()).map(|p| p.into_os_string()).unwrap_or_default()
}

impl Host {
    /// The host running `bundle` headless at `w`x`h`, once its first frame has ended, with the page's stylesheet `css`
    /// (else the corpus's tw.css above the bundle) and `env` over the corpus's own (RAEA_PAGE_ROOT: src/app.rs).
    #[allow(clippy::too_many_arguments)]
    pub fn launch(bin: &Path, bundle: &Path, dir: &Path, w: u32, h: u32, engine: &str, css: Option<std::ffi::OsString>, env: &[(&str, std::ffi::OsString)]) -> Result<Host, String> {
        let port = free_port()?;
        let name = bin.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let log = dir.join(format!("{name}.log"));
        let f = std::fs::File::create(&log).map_err(|e| e.to_string())?;
        // the corpus's Tailwind build (build.ts), which the Blitz page takes as Chrome's page does
        let css = css.unwrap_or_else(|| tw_css(bundle));
        let mut cmd = Command::new(bin);
        cmd.current_dir(dir).env("RAEA_AUDIO_MUTE", "1");
        cmd.arg(bundle).arg("--size").arg(format!("{w}x{h}")).env("RAEA_PAGE_BRP_PORT", port.to_string()).env("RAEA_PAGE_MCP_PORT", "0");
        if !css.is_empty() {
            cmd.arg("--css").arg(&css);
        }
        let child = cmd
            // raea-runtime-js's engine: JavaScriptCore or QuickJS (the build must have it: --features portable)
            .env("BEVY_TSX_ENGINE", engine)
            .envs(env.iter().map(|(k, v)| (*k, v)))
            .stdin(Stdio::null())
            .stdout(Stdio::from(f.try_clone().map_err(|e| e.to_string())?))
            .stderr(Stdio::from(f))
            .spawn()
            .map_err(|e| format!("launching {}: {e}", bin.display()))?;
        let mut h_ = Host { child, port, log, last: 0, pending_wait: 0, req: 0, engine: String::new(), a11y: false, catch_up: 0.0, js_prev: 0.0 };
        let t = Instant::now();
        loop {
            if let Ok(Some(st)) = h_.child.try_wait() {
                let tail = std::fs::read_to_string(&h_.log).unwrap_or_default();
                let tail: Vec<&str> = tail.lines().rev().take(6).collect();
                return Err(format!("{name} exited ({st}) before its first frame: {}", tail.into_iter().rev().collect::<Vec<_>>().join(" | ")));
            }
            if brp(port, &h_.m("state"), json!({}), Duration::from_secs(2)).is_ok() {
                break;
            }
            if t.elapsed() > Duration::from_secs(60) {
                return Err(format!("{name}'s BRP didn't answer in 60 s"));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(h_)
    }

    /// `name` under the host's prefix
    fn m(&self, name: &str) -> String {
        format!("page.{name}")
    }

    /// The probe's `parity.read`: the trace, the JS engine and its clock.
    fn parity_read(&mut self) -> Result<Value, String> {
        self.req += 1;
        brp(self.port, "page.action+watch", json!({"req": format!("parity-{}", self.req), "name": "parity.read"}), Duration::from_secs(20))
    }

    fn send(&mut self, ev: Value) -> Result<(), String> {
        let r = brp(self.port, &self.m("input"), ev, Duration::from_secs(20))?;
        self.last = self.last.max(r["frame"].as_u64().unwrap_or(0));
        Ok(())
    }

    pub fn input(&mut self, i: &Input) -> Result<(), String> {
        match i {
            Input::Move(x, y) => self.send(json!({"type": "move", "x": x, "y": y})),
            Input::Down(x, y, clicks, right) => self.send(json!({"type": "down", "x": x, "y": y, "clicks": clicks, "button": if *right { "right" } else { "left" }})),
            Input::Up(x, y, _, right) => self.send(json!({"type": "up", "x": x, "y": y, "button": if *right { "right" } else { "left" }})),
            Input::Leave => self.send(json!({"type": "leave"})),
            // the DOM's sign (positive scrolls the content up, toward its end) to winit's (positive moves it down)
            Input::Wheel(x, y, dx, dy) => self.send(json!({"type": "wheel", "x": x, "y": y, "dx": -dx, "dy": -dy, "unit": "pixel"})),
            Input::Key(k, mods) => self.send(json!({"type": "key", "key": k, "mods": mods})),
            Input::Text(t) => self.send(json!({"type": "text", "text": t})),
            Input::Resize(w, h) => self.send(json!({"type": "resize", "w": w, "h": h})),
            Input::Wait(n) => {
                self.pending_wait = self.pending_wait.max(*n);
                Ok(())
            }
        }
    }

    /// The bundle's clock (performance.now, ms), from the probe.
    fn js_now(&mut self) -> Result<f64, String> {
        let tr = self.parity_read()?;
        Ok(tr["value"]["now"].as_f64().unwrap_or(f64::MAX))
    }

    fn tree_after(&self, frame: u64) -> Result<Value, String> {
        brp(self.port, &self.m("ui+watch"), json!({"after": frame}), Duration::from_secs(20))
    }

    /// The window once settled: at least `min` frames after the latest input, then until two frames running lay out
    /// the same (at most 90); the probe's trace; the frame read back.
    pub fn read(&mut self, min: u32) -> Result<Snap, String> {
        let min = min.max(std::mem::take(&mut self.pending_wait)) as u64;
        let until = self.js_prev + std::mem::take(&mut self.catch_up);
        // a frame at a time until the JS clock is there (10 s at most)
        for _ in 0..600 {
            if self.js_now()? >= until {
                break;
            }
            let f = brp(self.port, &self.m("state"), json!({}), Duration::from_secs(5))?["frame"].as_u64().unwrap_or(0);
            self.tree_after(f + 1)?;
        }
        let now = brp(self.port, &self.m("state"), json!({}), Duration::from_secs(5))?["frame"].as_u64().unwrap_or(0);
        let mut at = self.last.max(now) + min;
        let mut tree = self.tree_after(at)?;
        for _ in 0..90 {
            at = tree["frame"].as_u64().unwrap_or(at) + 1;
            let next = self.tree_after(at)?;
            let same = next["nodes"] == tree["nodes"];
            tree = next;
            if same {
                break;
            }
        }
        let tr = self.parity_read()?;
        self.req += 1;
        let shot = brp(self.port, &self.m("frame_png+watch"), json!({"req": format!("parity-png-{}", self.req)}), Duration::from_secs(20))?;
        self.engine = tr["value"]["engine"].as_str().unwrap_or("").to_string();
        let png = base64::engine::general_purpose::STANDARD.decode(shot["png"].as_str().unwrap_or("")).map_err(|e| e.to_string())?;
        let png = image::load_from_memory(&png).map_err(|e| e.to_string())?.to_rgba8();
        let nodes = tree["nodes"].as_array().cloned().unwrap_or_default().iter().filter_map(|n| Some((n["id"].as_str()?.to_string(), bevy_node(n)))).collect();
        // the AccessKit tree, read through accesskit_consumer (a11y: systems/a11y)
        let a11y = if self.a11y {
            let t = brp(self.port, &self.m("a11y"), json!({}), Duration::from_secs(20))?;
            Some(t["nodes"].as_object().cloned().unwrap_or_default().into_iter().collect())
        } else {
            None
        };
        let now = tr["value"]["now"].as_f64().unwrap_or(0.0);
        self.js_prev = now;
        Ok(Snap { nodes, trace: tr["value"]["trace"].as_array().cloned().unwrap_or_default(), png, a11y, now })
    }
}
