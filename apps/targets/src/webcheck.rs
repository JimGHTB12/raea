//! `raea app check` on the web target: the app on raea-page-host headless, driven over BRP as ui-parity drives it. After
//! the mount and after each step, `--settle` frames run and then until two frames lay out the same, and the frame is
//! written as NN-name.png with its elements (by data-testid) in tree.json, as raea-rune-host writes them. unmapped.json
//! is empty on web: Blitz takes the page's CSS as Chrome does, with no table of mapped props (what native wouldn't draw
//! is T4a's report). A page that throws, or a host that exits, fails the check with its log's tail.

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use base64::Engine as _;
use serde_json::{Value, json};

use crate::brp::{call, free_port};
use crate::steps::{self, Step};
use crate::verbs::Ask;

struct Host {
    child: Child,
    port: u16,
    log: std::path::PathBuf,
    /// the frame the latest input landed before
    last: u64,
    req: u64,
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

const S: Duration = Duration::from_secs(20);

impl Host {
    fn tail(&self) -> String {
        let t = std::fs::read_to_string(&self.log).unwrap_or_default();
        let t: Vec<&str> = t.lines().rev().take(8).collect();
        t.into_iter().rev().collect::<Vec<_>>().join(" | ")
    }

    fn call(&mut self, m: &str, p: Value) -> Result<Value, String> {
        call(self.port, m, p, S).map_err(|e| match self.child.try_wait() {
            Ok(Some(st)) => format!("raea-page-host exited ({st}): {}", self.tail()),
            _ => e,
        })
    }

    fn send(&mut self, ev: Value) -> Result<(), String> {
        let r = self.call("page.input", ev)?;
        self.last = self.last.max(r["frame"].as_u64().unwrap_or(0));
        Ok(())
    }

    fn frame(&mut self) -> Result<u64, String> {
        Ok(self.call("page.state", json!({}))?["frame"].as_u64().unwrap_or(0))
    }

    fn tree_after(&mut self, f: u64) -> Result<Value, String> {
        self.call("page.ui+watch", json!({"after": f}))
    }

    /// the middle of the element `tid` in the latest tree
    fn middle(&mut self, tid: &str) -> Result<(f64, f64), String> {
        let f = self.frame()?;
        let t = self.tree_after(f)?;
        let n = t["nodes"].as_array().into_iter().flatten().find(|n| n["id"].as_str() == Some(tid)).ok_or_else(|| format!("no element with data-testid {tid:?}"))?;
        let r = &n["rect"];
        let g = |k: &str| r[k].as_f64().unwrap_or(0.0);
        Ok((g("x") + g("w") / 2.0, g("y") + g("h") / 2.0))
    }

    fn press_at(&mut self, (x, y): (f64, f64)) -> Result<(), String> {
        self.send(json!({"type": "move", "x": x, "y": y}))?;
        self.send(json!({"type": "down", "x": x, "y": y, "clicks": 1, "button": "left"}))?;
        self.send(json!({"type": "up", "x": x, "y": y, "button": "left"}))
    }

    fn play(&mut self, s: &Step) -> Result<u32, String> {
        match s {
            Step::Press(t) => {
                let at = self.middle(t)?;
                self.press_at(at)?;
            }
            Step::Type { text, into } => {
                let at = self.middle(into)?;
                self.press_at(at)?;
                self.send(json!({"type": "key", "key": "End", "mods": []}))?;
                for c in text.chars() {
                    self.send(json!({"type": "text", "text": c.to_string()}))?;
                }
            }
            Step::Wheel { over, dy } => {
                let (x, y) = self.middle(over)?;
                self.send(json!({"type": "move", "x": x, "y": y}))?;
                self.send(json!({"type": "wheel", "x": x, "y": y, "dx": 0, "dy": -dy, "unit": "pixel"}))?;
            }
            Step::Key(k) => self.send(json!({"type": "key", "key": k, "mods": []}))?,
            Step::Frames(n) => return Ok(*n),
            // queued in the page host's raea-io-os for the app's next pick, save or dialog
            Step::Os(v) => {
                call(self.port, "page.os", v.clone(), Duration::from_secs(10))?;
            }
        }
        Ok(0)
    }

    /// the page once settled: `min` frames after the latest input, then until two frames lay out the same (90 at most);
    /// the tree and the frame's PNG
    fn read(&mut self, min: u32) -> Result<(Value, Vec<u8>), String> {
        let now = self.frame()?;
        let mut at = self.last.max(now) + min as u64;
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
        self.req += 1;
        let shot = self.call("page.frame_png+watch", json!({"req": format!("check-{}", self.req)}))?;
        let png = base64::engine::general_purpose::STANDARD.decode(shot["png"].as_str().unwrap_or("")).map_err(|e| e.to_string())?;
        Ok((tree, png))
    }
}

fn launch(js: &Path, css: &Path, out: &Path, size: (u32, u32)) -> Result<Host, String> {
    let port = free_port()?;
    let log = out.join("page-host.log");
    let f = std::fs::File::create(&log).map_err(|e| e.to_string())?;
    let child = Command::new(crate::web::bin()?)
        .arg(js)
        .arg("--css")
        .arg(css)
        .arg("--size")
        .arg(format!("{}x{}", size.0, size.1))
        .env("RAEA_PAGE_BRP_PORT", port.to_string())
        .env("RAEA_PAGE_MCP_PORT", "0")
        .env("RAEA_AUDIO_MUTE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::from(f.try_clone().map_err(|e| e.to_string())?))
        .stderr(Stdio::from(f))
        .spawn()
        .map_err(|e| format!("raea-page-host: {e}"))?;
    let mut h = Host { child, port, log, last: 0, req: 0 };
    // its first frame, or its exit (a host that never answers fails at 60 s)
    let t = Instant::now();
    loop {
        if let Ok(Some(st)) = h.child.try_wait() {
            return Err(format!("raea-page-host exited ({st}) before its first frame: {}", h.tail()));
        }
        if call(port, "page.state", json!({}), Duration::from_secs(2)).is_ok() {
            return Ok(h);
        }
        if t.elapsed() > Duration::from_secs(60) {
            return Err(format!("raea-page-host's BRP didn't answer in 60 s: {}", h.tail()));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// A page's own errors in the host's log (the page's console.error and uncaught throws)
fn page_errors(log: &Path) -> Vec<String> {
    let t = std::fs::read_to_string(log).unwrap_or_default();
    t.lines().filter(|l| l.contains("Uncaught") || l.contains("[page] error") || l.contains("console.error")).map(str::to_string).collect()
}

pub fn check(a: &Ask) -> Result<String, String> {
    let steps = match &a.script {
        Some(p) => steps::parse(&std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?)?,
        None => Vec::new(),
    };
    let out = &a.png;
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let built = crate::web::build(&a.app, &out.join("page"))?;
    let mut h = launch(&built.js, &built.css, out, a.size)?;
    let mut trees = Vec::new();
    let mut shoot = |h: &mut Host, i: usize, name: &str, extra: u32| -> Result<(), String> {
        let (tree, png) = h.read(a.settle + extra)?;
        let file = format!("{i:02}-{name}.png");
        std::fs::write(out.join(&file), png).map_err(|e| format!("{file}: {e}"))?;
        trees.push(json!({"step": i, "name": name, "png": file, "frame": tree["frame"], "tree": tree["nodes"]}));
        Ok(())
    };
    shoot(&mut h, 0, "mount", 0)?;
    for (i, s) in steps.iter().enumerate() {
        let wait = h.play(s).map_err(|e| format!("step {}: {e}", i + 1))?;
        shoot(&mut h, i + 1, &steps::name(s), wait)?;
    }
    let errors = page_errors(&h.log);
    drop(h);
    let tree = out.join("tree.json");
    std::fs::write(&tree, serde_json::to_string_pretty(&trees).unwrap()).map_err(|e| format!("{}: {e}", tree.display()))?;
    std::fs::write(out.join("unmapped.json"), "[]\n").map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err(format!("the page reported {} error(s) (page-host.log): {}", errors.len(), errors.join(" | ")));
    }
    Ok(format!("{} frames written to {} (web: Blitz on Bevy)", trees.len(), out.display()))
}
