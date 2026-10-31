//! The Driver: a case's steps run against a launched world, with layouts read and idleness waited on.

use super::*;

#[derive(Debug)]
pub(crate) struct Timed {
    pub(crate) frames: u64,
    pub(crate) budget: u64,
}

pub(crate) struct Driver<'a> {
    pub(crate) mcp: &'a mut Mcp,
    pub(crate) case: &'a Case,
    pub(crate) workspace: PathBuf,
    pub(crate) snaps: Snaps,
    /// the frame the last input (or write) landed before
    pub(crate) at: Option<u64>,
    pub(crate) timed: Vec<Timed>,
    /// the longest a settle may take (0: never wait for one)
    pub(crate) settle_ms: u64,
    /// the last step that ran was an input: the next input is the same gesture (two quick presses), not settled before
    pub(crate) after_input: bool,
    /// the latest tree read since the last input: a step after it is checked against it first, so a frame that already
    /// showed what's expected counts as that frame (a substrate answering each `ui` with a newer frame would otherwise
    /// time it a frame late)
    pub(crate) seen: Option<Value>,
    /// what a pass or a slow result says beside it (a settle that accepted stuck stale panels)
    pub(crate) notes: Vec<String>,
}

/// What says whether a tree's layout has settled: the window, each node's id, rect and visibility, and each panel's
/// box and texture. Values (a playing video's time, a hover) may go on changing; these may not.
pub(crate) fn layout_of(tree: &Value) -> Value {
    let nodes: Vec<Value> = pred::nodes(tree).iter().map(|n| json!([n["id"], n["rect"], n["visible"]])).collect();
    let panels: Vec<Value> = tree["panels"].as_array().map(|ps| ps.iter().map(|p| json!([p["name"], p["shown"], p["rect"], p["texture"]])).collect()).unwrap_or_default();
    json!([tree["window"], nodes, panels])
}

/// The panels a tree reports as stale (presenting a texture of an older layout).
pub(crate) fn stale_panels(tree: &Value) -> Vec<String> {
    tree["panels"].as_array().map(|ps| ps.iter().filter(|p| p["stale"].as_bool() == Some(true)).map(|p| match p["stale_why"].as_str() {
        Some(why) => format!("{} ({why})", p["name"].as_str().unwrap_or("?")),
        None => p["name"].as_str().unwrap_or("?").to_string(),
    }).collect()).unwrap_or_default()
}

/// What differs between two layouts (for a settle that never came): the window, node ids, the panels.
pub(crate) fn moving(a: &Value, b: &Value) -> Vec<String> {
    let ids = |l: &Value| -> Vec<(String, String)> { l[1].as_array().map(|v| v.iter().map(|n| (n[0].as_str().unwrap_or("?").to_string(), n.to_string())).collect()).unwrap_or_default() };
    let (x, y) = (ids(a), ids(b));
    let mut out: Vec<String> = vec![];
    if a[0] != b[0] {
        out.push("window".into());
    }
    for n in x.iter().filter(|n| !y.contains(n)).chain(y.iter().filter(|n| !x.contains(n))) {
        if !out.contains(&n.0) {
            out.push(n.0.clone());
        }
    }
    if a[2] != b[2] {
        out.push("panels".into());
    }
    out
}

/// A substrate that presents no new frame for this long is idle: the same tree that long counts as settled.
pub(crate) const IDLE_MS: u64 = 250;
/// Panels still flagged stale with every rect, the window and the stale flags themselves unchanged this long: the
/// substrate has nothing more to present, so waiting can't help. The tree counts as settled, with a note saying which
/// panels were stale (their nodes say so themselves, and a case that needs them judges them).
pub(crate) const STUCK_MS: u64 = 1500;

impl Driver<'_> {
    pub(crate) fn tree(&mut self) -> Result<Value, String> {
        let t = self.mcp.ui()?;
        self.seen = Some(t.clone());
        Ok(t)
    }

    /// Polls the tree until `p` holds, for at most `ms`.
    pub(crate) fn until(&mut self, p: &Value, ms: u64) -> Result<Value, (String, Value)> {
        let t = Instant::now();
        let mut last: (String, Value);
        let mut first = self.seen.take();
        loop {
            // a read that fails (no new frame in the substrate's own wait: llvmpipe takes seconds over its first frames)
            // is retried like a predicate that doesn't hold yet, until the deadline
            match first.take().map(Ok).unwrap_or_else(|| self.tree()) {
                Ok(tree) => match pred::holds(p, &tree, &self.snaps) {
                    Ok(()) => return Ok(tree),
                    Err(e) => last = (e, tree),
                },
                Err(e) => last = (e, Value::Null),
            }
            if t.elapsed() > Duration::from_millis(ms) {
                return Err(last);
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    /// Polls the tree until its layout has settled: no panel stale, and the same rects on two consecutive frames (or,
    /// from a substrate presenting no new frames, on one frame for IDLE_MS; or stale panels that haven't changed in
    /// STUCK_MS, noted). Not timed: it runs before an input is sent, so no case's latency counts it. Fails, rather than
    /// waits on, a tree that doesn't settle in `settle_ms`.
    pub(crate) fn settle(&mut self) -> Result<Value, String> {
        if self.settle_ms == 0 {
            return self.tree();
        }
        let t = Instant::now();
        // the last tree polled: its frame, its layout, when that frame was first seen
        let mut prev: Option<(u64, Value, Instant)> = None;
        // stale panels and the layout they were stale in, since when
        let mut stuck: Option<(Vec<String>, Value, Instant)> = None;
        let mut why = String::new();
        loop {
            let tree = self.tree()?;
            let f = frame_of(&tree).unwrap_or(0);
            let lay = layout_of(&tree);
            let stale = stale_panels(&tree);
            if !stale.is_empty() {
                why = format!("panels still stale at frame {f}: {}", stale.join(", "));
                prev = None;
                match &stuck {
                    Some((s, l, since)) if *s == stale && *l == lay => {
                        if since.elapsed() >= Duration::from_millis(STUCK_MS) {
                            let note = format!("settled with panels stale and unchanged for {STUCK_MS} ms: {}", stale.join(", "));
                            if !self.notes.contains(&note) {
                                self.notes.push(note);
                            }
                            return Ok(tree);
                        }
                    }
                    _ => stuck = Some((stale, lay, Instant::now())),
                }
            } else {
                stuck = None;
                match &prev {
                    Some((pf, pl, _)) if *pf != f && *pl == lay => return Ok(tree),
                    Some((pf, pl, seen)) if *pf == f && *pl == lay && seen.elapsed() >= Duration::from_millis(IDLE_MS) => return Ok(tree),
                    Some((pf, _, _)) if *pf == f => {}
                    Some((pf, pl, _)) => {
                        let mut m = moving(pl, &lay);
                        m.truncate(12);
                        why = format!("rects still changing from frame {pf} to {f}: {}", m.join(", "));
                        prev = Some((f, lay, Instant::now()));
                    }
                    None => prev = Some((f, lay, Instant::now())),
                }
            }
            if t.elapsed() > Duration::from_millis(self.settle_ms) {
                return Err(format!("the tree never settled in {} ms ({})", self.settle_ms, if why.is_empty() { "no two frames to compare" } else { &why }));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    pub(crate) fn point(&self, tree: &Value, at: &Value) -> Result<(f64, f64), String> {
        let (id, spec) = match at {
            Value::String(s) => (s.as_str(), &Value::Null),
            Value::Object(_) => (at["id"].as_str().ok_or("at needs an id")?, at),
            _ => return Err(format!("at {at}: a node id or {{id, where?, fx?, fy?, dx?, dy?}}")),
        };
        let n = pred::nodes(tree)
            .iter()
            .filter(|n| pred::glob(id, n["id"].as_str().unwrap_or("")))
            .find(|n| spec["where"].as_object().is_none_or(|w| w.iter().all(|(k, v)| pred::get(n, k) == v || pred::get(n, k).as_str() == v.as_str() && v.is_string())))
            .ok_or_else(|| {
                let mut ids: Vec<&str> = pred::nodes(tree).iter().filter_map(|n| n["id"].as_str()).collect();
                ids.truncate(40);
                format!("no node {id}{} to point at (the tree has: {})", if spec["where"].is_null() { String::new() } else { format!(" where {}", spec["where"]) }, ids.join(", "))
            })?;
        let r = Rect::of(n).ok_or(format!("{id} has no rect"))?;
        let f = |k: &str, d: f64| spec[k].as_f64().unwrap_or(d);
        Ok((r.x + r.w * f("fx", 0.5) + f("dx", 0.0), r.y + r.h * f("fy", 0.5) + f("dy", 0.0)))
    }

    /// An input step as the contract's events (sugar expanded, `at` resolved).
    pub(crate) fn events(&self, tree: &Value, i: &Value) -> Result<Vec<Value>, String> {
        let t = i["type"].as_str().unwrap_or("");
        let xy = |k: &str| -> Result<(f64, f64), String> {
            match i.get(k) {
                Some(at) => self.point(tree, at),
                None => Ok((i["x"].as_f64().ok_or(format!("{t} needs x,y or {k}"))?, i["y"].as_f64().ok_or(format!("{t} needs x,y or {k}"))?)),
            }
        };
        let r1 = |v: f64| (v * 10.0).round() / 10.0;
        let button = i["button"].as_str().unwrap_or("left");
        let mods = i.get("mods").cloned().unwrap_or(json!([]));
        let ptr = |ty: &str, (x, y): (f64, f64), clicks: u64| match ty {
            "move" => json!({"type": "move", "x": r1(x), "y": r1(y)}),
            _ => json!({"type": ty, "x": r1(x), "y": r1(y), "button": button, "clicks": clicks}),
        };
        let key = |k: &str, state: &str| json!({"type": "key", "key": k, "mods": mods, "state": state});
        Ok(match t {
            "move" => vec![ptr("move", xy("at")?, 0)],
            "down" | "up" => vec![ptr(t, xy("at")?, i["clicks"].as_u64().unwrap_or(1))],
            "click" => {
                let p = xy("at")?;
                vec![ptr("move", p, 0), ptr("down", p, 1), ptr("up", p, 1)]
            }
            "dblclick" => {
                let p = xy("at")?;
                vec![ptr("move", p, 0), ptr("down", p, 1), ptr("up", p, 1), ptr("down", p, 2), ptr("up", p, 2)]
            }
            "drag" => {
                let (a, b) = (xy("from")?, xy("to")?);
                let n = i["moves"].as_u64().unwrap_or(8).max(1);
                let mut v = vec![ptr("move", a, 0), ptr("down", a, 1)];
                for k in 1..=n {
                    let f = k as f64 / n as f64;
                    v.push(ptr("move", (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f), 0));
                }
                v.push(ptr("up", b, 1));
                v
            }
            "wheel" => {
                let (x, y) = xy("at")?;
                let mut e = json!({"type": "wheel", "x": r1(x), "y": r1(y), "dx": i["dx"].as_f64().unwrap_or(0.0), "dy": i["dy"].as_f64().ok_or("wheel needs dy")?,
                                   "unit": i["unit"].as_str().unwrap_or("pixel"), "natural": i["natural"].as_bool().unwrap_or(true)});
                if let Some(p) = i.get("phase") {
                    e["phase"] = p.clone();
                }
                vec![e]
            }
            "key" => vec![key(i["key"].as_str().ok_or("key needs key")?, i["state"].as_str().unwrap_or("down"))],
            "press" => {
                let k = i["key"].as_str().ok_or("press needs key")?;
                vec![key(k, "down"), key(k, "up")]
            }
            "keys" => {
                let ks: Vec<String> = match (i["text"].as_str(), i["keys"].as_array()) {
                    (Some(s), _) => s.chars().map(|c| c.to_string()).collect(),
                    (None, Some(a)) => a.iter().filter_map(|k| k.as_str().map(str::to_string)).collect(),
                    _ => return Err("keys needs text or keys".into()),
                };
                ks.iter().flat_map(|k| [key(k, "down"), key(k, "up")]).collect()
            }
            "text" => vec![json!({"type": "text", "text": i["text"].as_str().ok_or("text needs text")?})],
            "ime" => {
                let mut e = json!({"type": "ime"});
                for k in ["marked", "commit"] {
                    if let Some(v) = i.get(k) {
                        e[k] = v.clone();
                    }
                }
                vec![e]
            }
            "leave" => vec![json!({"type": "leave"})],
            "resize" => vec![json!({"type": "resize", "w": i["w"], "h": i["h"]})],
            other => return Err(format!("no input type {other}")),
        })
    }

    pub(crate) fn step(&mut self, s: &Value) -> Result<(), String> {
        if let Some(cond) = s.get("if") {
            let tree = self.tree()?;
            if pred::holds(cond, &tree, &self.snaps).is_err() {
                return Ok(());
            }
        }
        let was_input = std::mem::replace(&mut self.after_input, s.get("input").is_some());
        if let Some(i) = s.get("input") {
            // a step's targets are resolved on a settled tree, unless it's the same gesture as the input just before
            let tree = if was_input { self.tree()? } else { self.settle()? };
            let evs = self.events(&tree, i)?;
            self.snaps.insert("last".into(), tree);
            for e in evs {
                let f = self.mcp.input(e)?;
                self.at = Some(f);
            }
            self.seen = None;
        } else if let Some(p) = s.get("wait") {
            let ms = s["wait_ms"].as_u64().unwrap_or(10_000);
            self.until(p, ms).map_err(|(e, _)| format!("never got there in {ms} ms: wait {}: {e}", clip(&p.to_string(), 160)))?;
        } else if let Some(p) = s.get("expect") {
            let budget = s["budget"].as_u64().unwrap_or(self.case.budget);
            let ms = s["timeout_ms"].as_u64().unwrap_or(self.case.timeout_ms);
            let tree = match self.until(p, ms) {
                Ok(t) => t,
                Err((e, tree)) => {
                    let since = match (frame_of(&tree), self.at) {
                        (Some(f), Some(a)) => format!(" ({} frames after the input)", f.saturating_sub(a) + 1),
                        _ => String::new(),
                    };
                    return Err(format!("expected {}: never held in {ms} ms; the tree showed {e}{since}", clip(&p.to_string(), 200)));
                }
            };
            let f = frame_of(&tree).unwrap_or(0);
            let at = *self.at.get_or_insert(f);
            let frames = (f + 1).saturating_sub(at);
            if let Some(hold) = s["hold"].as_u64() {
                let t = Instant::now();
                loop {
                    let tree = self.tree()?;
                    let g = frame_of(&tree).unwrap_or(0);
                    if let Err(e) = pred::holds(p, &tree, &self.snaps) {
                        return Err(format!("expected {}: held at frame {f}, then not at frame {g}: {e}", clip(&p.to_string(), 200)));
                    }
                    // an idle substrate may present no new frames: then wall time stands in for them
                    if g >= f + hold || t.elapsed() > Duration::from_millis((hold * 100).max(250)) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(4));
                }
            }
            self.timed.push(Timed { frames, budget });
        } else if let Some(name) = s["snap"].as_str() {
            let tree = self.settle()?;
            self.snaps.insert(name.into(), tree);
        } else if let Some(w) = s.get("write") {
            let (from, to) = (w["from"].as_str().ok_or("write needs from")?, w["to"].as_str().ok_or("write needs to")?);
            let (src, dst) = (self.workspace.join(from), self.workspace.join(to));
            let part = dst.with_file_name(format!(".{}.part", dst.file_name().unwrap_or_default().to_string_lossy()));
            std::fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
            std::fs::write(&part, std::fs::read(&src).map_err(|e| format!("write: {}: {e}", src.display()))?).map_err(|e| e.to_string())?;
            let tree = self.tree()?;
            std::fs::rename(&part, &dst).map_err(|e| format!("write: {e}"))?;
            self.at = Some(frame_of(&tree).unwrap_or(0) + 1);
            self.seen = None;
            self.snaps.insert("last".into(), tree);
        } else if let Some(r) = s.get("repeat") {
            let steps = r["steps"].as_array().cloned().unwrap_or_default();
            for i in 0..r["times"].as_u64().unwrap_or(1) {
                for s in &steps {
                    self.step(&case::subst(s, i))?;
                }
            }
        }
        Ok(())
    }
}

// --- a case at one window size
