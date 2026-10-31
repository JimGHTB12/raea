//! conformance-fake: a tiny fake substrate for the conformance runner's own tests (src/selftest.rs there): an MCP server answering `input` and `ui` over HTTP JSON-RPC as
//! the contract says, with a toy feed (header toggles, a left panel of feeds, cards, a search box, a chat with an echo)
//! presented once a frame (16 ms). `lag` delays what's presented by that many frames (to make cases slow) and
//! `no_hover` breaks hovering (to make them fail). `reflow` lays a new window size or a toggled panel out that many
//! frames after the window reports it, its panel stale till then (the current stack's reflow: a snapshot taken as soon
//! as the window has its size is of the old layout), and `jitter` never lets the layout settle.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};

const FEEDS: &[&str] = &["everything", "needs-review", "intro", "concepts", "storyboards", "swipe-file"];
const STEMS: &[&str] = &["v2-g", "board", "v2-f", "s3b", "v2-e", "v2-d", "s3", "i1-room", "v2-c", "v2-b", "v2-a", "concept-2", "concept-1", "swipe-1"];

#[derive(Default, Clone)]
struct Ui {
    w: f64,
    h: f64,
    pointer: Option<(f64, f64)>,
    down: bool,
    left: bool,
    chat: bool,
    feed: usize,
    focus: Option<&'static str>,
    search: String,
    query: String,
    composer: String,
    marked: String,
    messages: Vec<String>,
    top: f64,
    left_top: f64,
    chat_top: f64,
}

pub struct Fake {
    pub port: u16,
}

/// The ui as input left it, the frame counter, the trees kept for `lag`, the layout as it's laid out
type State = (Ui, u64, Vec<Value>, Laid);

/// The geometry (window w, h, left shown, chat shown) the nodes are laid out at, and frames till the next catches up.
#[derive(Clone, Copy, Default)]
struct Laid {
    geo: (f64, f64, bool, bool),
    wait: usize,
}

#[derive(Clone, Copy, Default)]
pub struct Knobs {
    pub lag: usize,
    pub no_hover: bool,
    pub reflow: usize,
    pub jitter: bool,
    /// the feed panel reports stale forever, its layout stable (a substrate's false positive, or a repaint that never comes)
    pub stuck: bool,
}

/// The tree presented at `frame`: the window at its size at once, the nodes laid out at `laid`'s geometry, which
/// catches up `reflow` frames after a change; one panel (the feed), stale while they differ.
fn present(ui: &Ui, frame: u64, laid: &mut Laid, k: Knobs) -> Value {
    let now = (ui.w, ui.h, ui.left, ui.chat);
    if k.reflow == 0 {
        laid.geo = now;
    } else if laid.geo == now {
        laid.wait = k.reflow;
    } else if laid.wait > 1 {
        laid.wait -= 1;
    } else {
        laid.geo = now;
    }
    let mut view = ui.clone();
    (view.w, view.h, view.left, view.chat) = laid.geo;
    if k.jitter && frame % 2 == 1 {
        view.w += 8.0;
    }
    let mut t = view.tree(frame, k);
    t["window"] = json!({"w": ui.w, "h": ui.h});
    let mid = t["nodes"].as_array().unwrap().iter().find(|n| n["id"] == "middle").map(|n| n["rect"].clone()).unwrap_or(Value::Null);
    let tex = json!({"w": mid["w"].as_f64().unwrap_or(0.0) * 2.0, "h": mid["h"].as_f64().unwrap_or(0.0) * 2.0, "scale": 2});
    t["panels"] = json!([{"name": "feed", "shown": true, "rect": mid, "texture": tex, "stale": laid.geo != now || k.stuck}]);
    t
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> Value {
    json!({"x": x, "y": y, "w": w, "h": h})
}

impl Ui {
    fn cards(&self) -> Vec<&'static str> {
        STEMS
            .iter()
            .copied()
            .filter(|s| match FEEDS[self.feed] {
                "concepts" => s.starts_with("concept"),
                "storyboards" => *s == "board",
                "swipe-file" => s.starts_with("swipe"),
                "intro" => s.starts_with("v2") || s.starts_with("i1"),
                _ => true,
            })
            .filter(|s| s.contains(self.query.as_str()))
            .collect()
    }
    fn left_w(&self) -> f64 {
        if self.left { 220.0 } else { 0.0 }
    }
    fn chat_w(&self) -> f64 {
        if self.chat { 340.0 } else { 0.0 }
    }
    fn transcript_h(&self) -> f64 {
        self.h - 40.0 - 60.0
    }
    fn chat_max(&self) -> f64 {
        (self.messages.len() as f64 * 64.0 - self.transcript_h()).max(0.0)
    }
    fn left_max(&self) -> f64 {
        (FEEDS.len() as f64 * 32.0 + 16.0 - (self.h - 40.0)).max(0.0)
    }
    fn feed_max(&self) -> f64 {
        (16.0 + self.cards().len() as f64 * 216.0 - (self.h - 40.0)).max(0.0)
    }

    /// The tree's nodes (without hover), topmost last.
    fn nodes(&self) -> Vec<Value> {
        let (w, h) = (self.w, self.h);
        let mut v = vec![];
        let n = |id: &str, role: &str, r: Value, extra: Value| {
            let mut o = json!({"id": id, "role": role, "name": id, "rect": r, "visible": true, "hovered": false, "focused": false, "pressed": false, "selected": false, "value": null});
            for (k, x) in extra.as_object().unwrap() {
                o[k] = x.clone();
            }
            o
        };
        let (mx, mw) = (self.left_w(), w - self.left_w() - self.chat_w());
        v.push(n("middle", "region", rect(mx, 40.0, mw, h - 40.0), json!({})));
        v.push(n("feed.list", "list", rect(mx, 40.0, mw, h - 40.0), json!({"scroll": {"top": self.top, "max": self.feed_max()}})));
        for (i, s) in self.cards().iter().enumerate() {
            let y = 40.0 + 16.0 + i as f64 * 216.0 - self.top;
            if y + 200.0 > 40.0 && y < h {
                v.push(n(&format!("feed.card.{i}"), "button", rect(mx + 24.0, y, mw - 48.0, 200.0), json!({"value": s, "text": s})));
            }
        }
        if self.left {
            v.push(n("left", "complementary", rect(0.0, 40.0, 220.0, h - 40.0), json!({})));
            v.push(n("left.feeds", "list", rect(0.0, 40.0, 220.0, h - 40.0), json!({"scroll": {"top": self.left_top, "max": self.left_max()}})));
            for (i, f) in FEEDS.iter().enumerate() {
                let y = 48.0 + i as f64 * 32.0 - self.left_top;
                v.push(n(&format!("left.feed.{f}"), "tab", rect(8.0, y, 204.0, 30.0), json!({"selected": i == self.feed, "text": f})));
                if *f == "intro" {
                    v.push(n("left.feed.intro.badge", "status", rect(180.0, y + 6.0, 20.0, 18.0), json!({"value": STEMS.len()})));
                }
            }
        }
        if self.chat {
            let x = w - 340.0;
            v.push(n("chat", "complementary", rect(x, 40.0, 340.0, h - 40.0), json!({})));
            v.push(n("chat.transcript", "log", rect(x, 40.0, 340.0, self.transcript_h()), json!({"scroll": {"top": self.chat_top, "max": self.chat_max()}})));
            for (i, m) in self.messages.iter().enumerate() {
                let y = 40.0 + i as f64 * 64.0 - self.chat_top;
                let vis = y + 60.0 > 40.0 && y < 40.0 + self.transcript_h();
                v.push(n(&format!("chat.message.{i}"), "article", rect(x + 8.0, y, 324.0, 60.0), json!({"text": m, "visible": vis})));
            }
            let val = format!("{}{}", self.composer, self.marked);
            v.push(n("chat.composer", "textbox", rect(x + 8.0, h - 52.0, 324.0, 44.0), json!({"value": val, "focused": self.focus == Some("chat.composer")})));
        }
        v.push(n("header.toggle.files", "button", rect(8.0, 6.0, 28.0, 28.0), json!({"pressed": self.left})));
        let val = if self.focus == Some("feed.search") { format!("{}{}", self.search, self.marked) } else { self.search.clone() };
        v.push(n("feed.search", "searchbox", rect(w - 240.0, 6.0, 180.0, 28.0), json!({"value": val, "focused": self.focus == Some("feed.search")})));
        v.push(n("header.toggle.chat", "button", rect(w - 36.0, 6.0, 28.0, 28.0), json!({"pressed": self.chat})));
        v
    }

    fn hit(&self, x: f64, y: f64) -> Option<String> {
        let ns = self.nodes();
        ns.iter()
            .rev()
            .filter(|n| n["visible"] == json!(true) && ["button", "tab", "textbox", "searchbox"].contains(&n["role"].as_str().unwrap()))
            .find(|n| {
                let r = &n["rect"];
                let (rx, ry, rw, rh) = (r["x"].as_f64().unwrap(), r["y"].as_f64().unwrap(), r["w"].as_f64().unwrap(), r["h"].as_f64().unwrap());
                x >= rx && x < rx + rw && y >= ry && y < ry + rh
            })
            .map(|n| n["id"].as_str().unwrap().to_string())
    }

    fn scroller(&self, x: f64, y: f64) -> Option<&'static str> {
        if self.chat && x >= self.w - 340.0 && y >= 40.0 && y < 40.0 + self.transcript_h() {
            Some("chat")
        } else if self.left && x < 220.0 && y >= 40.0 {
            Some("left")
        } else if y >= 40.0 {
            Some("feed")
        } else {
            None
        }
    }

    fn tree(&self, frame: u64, k: Knobs) -> Value {
        let mut ns = self.nodes();
        if let (Some((x, y)), false) = (self.pointer, k.no_hover) {
            if let Some(id) = self.hit(x, y) {
                for n in ns.iter_mut().filter(|n| n["id"] == json!(id)) {
                    n["hovered"] = json!(true);
                    n["pressed"] = json!(self.down || n["pressed"] == json!(true));
                }
            }
        }
        json!({"frame": frame, "window": {"w": self.w, "h": self.h}, "nodes": ns})
    }

    fn type_key(&mut self, key: &str) {
        let Some(f) = self.focus else { return };
        let search = f == "feed.search";
        let text = if search { &mut self.search } else { &mut self.composer };
        match key {
            "Backspace" => {
                text.pop();
            }
            "Escape" => {
                if search {
                    text.clear();
                    self.query.clear();
                }
                self.focus = None;
            }
            "Enter" => {
                if search {
                    self.query = text.clone();
                    self.focus = None;
                } else if !text.is_empty() {
                    let t = std::mem::take(text);
                    self.messages.push(format!("you: {t}"));
                    self.messages.push(format!("echo: {t}"));
                    self.chat_top = self.chat_max();
                }
            }
            k if k.chars().count() == 1 => text.push_str(k),
            _ => {}
        }
    }

    fn input(&mut self, e: &Value) -> Result<(), String> {
        let xy = || (e["x"].as_f64().unwrap_or(0.0), e["y"].as_f64().unwrap_or(0.0));
        match e["type"].as_str().unwrap_or("") {
            "move" => self.pointer = Some(xy()),
            "leave" => self.pointer = None,
            "down" => {
                self.pointer = Some(xy());
                self.down = true;
            }
            "up" => {
                self.down = false;
                let (x, y) = xy();
                if e["clicks"].as_u64() == Some(2) {
                    return Ok(());
                }
                match self.hit(x, y).as_deref() {
                    Some("header.toggle.chat") => self.chat = !self.chat,
                    Some("header.toggle.files") => self.left = !self.left,
                    Some("feed.search") => self.focus = Some("feed.search"),
                    Some("chat.composer") => self.focus = Some("chat.composer"),
                    Some(id) if id.starts_with("left.feed.") => {
                        self.feed = FEEDS.iter().position(|f| id == format!("left.feed.{f}")).unwrap_or(0);
                        self.top = 0.0;
                    }
                    _ => self.focus = None,
                }
            }
            "wheel" => {
                let (x, y) = xy();
                let per = if e["unit"] == json!("line") { 40.0 } else { 1.0 };
                let d = -e["dy"].as_f64().unwrap_or(0.0) * per;
                match self.scroller(x, y) {
                    Some("chat") => self.chat_top = (self.chat_top + d).clamp(0.0, self.chat_max()),
                    Some("left") => self.left_top = (self.left_top + d).clamp(0.0, self.left_max()),
                    Some("feed") => self.top = (self.top + d).clamp(0.0, self.feed_max()),
                    _ => {}
                }
            }
            "key" => {
                if e["state"] == json!("down") {
                    self.type_key(e["key"].as_str().unwrap_or(""));
                }
            }
            "text" => {
                for c in e["text"].as_str().unwrap_or("").chars() {
                    self.type_key(&c.to_string());
                }
            }
            "ime" => {
                if let Some(m) = e["marked"].as_str() {
                    self.marked = m.to_string();
                }
                if let Some(c) = e["commit"].as_str() {
                    self.marked.clear();
                    for ch in c.chars() {
                        self.type_key(&ch.to_string());
                    }
                }
            }
            "resize" => {
                self.w = e["w"].as_f64().ok_or("resize needs w")?;
                self.h = e["h"].as_f64().ok_or("resize needs h")?;
            }
            other => return Err(format!("no input type {other}")),
        }
        self.top = self.top.clamp(0.0, self.feed_max());
        Ok(())
    }
}

impl Fake {
    pub fn start(k: Knobs) -> Fake {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let ui = Ui { w: 1200.0, h: 800.0, left: true, ..Default::default() };
        // frame 0 is presented at once, so a `ui` before the clock's first tick shows what this fake (its knobs) draws
        let mut laid = Laid { geo: (ui.w, ui.h, ui.left, ui.chat), wait: k.reflow };
        let first = present(&ui, 0, &mut laid, k);
        let state = Arc::new(Mutex::new((ui, 0u64, vec![first], laid)));
        // the frame clock: every 16 ms the state as it was `lag` frames ago is presented
        let st = state.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(16));
            let mut s = st.lock().unwrap();
            s.1 += 1;
            let (ui, frame, mut laid) = (s.0.clone(), s.1, s.3);
            let t = present(&ui, frame, &mut laid, k);
            s.3 = laid;
            s.2.push(t);
            let keep = k.lag + 1;
            if s.2.len() > keep {
                let n = s.2.len() - keep;
                s.2.drain(..n);
            }
            if Arc::strong_count(&st) == 1 {
                return;
            }
        });
        let st = state.clone();
        std::thread::spawn(move || {
            for c in listener.incoming().flatten() {
                let st = st.clone();
                std::thread::spawn(move || serve(c, &st));
            }
        });
        Fake { port }
    }
}

fn serve(c: TcpStream, st: &Arc<Mutex<State>>) {
    let mut r = BufReader::new(c.try_clone().unwrap());
    let mut len = 0;
    let mut line = String::new();
    r.read_line(&mut line).ok();
    loop {
        line.clear();
        if r.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
            break;
        }
        if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            len = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0; len];
    r.read_exact(&mut body).ok();
    let msg: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    let reply = |result: Value| json!({"jsonrpc": "2.0", "id": msg["id"], "result": result});
    let text = |v: Value| json!({"content": [{"type": "text", "text": v.to_string()}]});
    let answer = match msg["method"].as_str().unwrap_or("") {
        "tools/list" => reply(json!({"tools": [{"name": "input"}, {"name": "ui"}]})),
        "tools/call" => {
            let args = &msg["params"]["arguments"];
            let mut s = st.lock().unwrap();
            match msg["params"]["name"].as_str().unwrap_or("") {
                "input" => match s.0.input(args) {
                    Ok(()) => reply(text(json!({"frame": s.1 + 1}))),
                    Err(e) => reply(json!({"content": [{"type": "text", "text": e}], "isError": true})),
                },
                "ui" => {
                    // the oldest kept state (`lag` frames old), presented now
                    let mut t = s.2.first().cloned().unwrap_or(Value::Null);
                    t["frame"] = json!(s.1);
                    reply(text(t))
                }
                other => reply(json!({"content": [{"type": "text", "text": format!("no tool {other}")}], "isError": true})),
            }
        }
        _ => json!({"jsonrpc": "2.0", "id": msg["id"], "error": {"code": -32601, "message": "no method"}}),
    }
    .to_string();
    let mut c = c;
    let _ = write!(c, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}", answer.len());
}
