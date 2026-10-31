//! A case file (sdk/ui/conformance/cases/AREA/NAME.toml): its area, window size(s), steps and budget. Steps are a
//! list of one-key tables run in order:
//!   input  = {type, ...}   one event as MCP `input` takes it, or sugar for a few: click, dblclick, drag, keys;
//!                           x/y can come from `at` (a node id, or {id, where?, fx?, fy?, dx?, dy?}: a point in its rect)
//!   wait   = PREDICATE     not timed: the tree gets there within wait_ms (default 10 s) or the case fails
//!   expect = PREDICATE     timed: frames from the last input (or write) to the first tree where it holds; options
//!                           beside it: budget (frames), hold (frames it must keep holding), timeout_ms
//!   snap   = "name"        keep the tree as it is now, for `since`/`eq_snap` ("start" and "last" are kept for you)
//!   write  = {from, to}    copy a file of the run's workspace to another path in it (made beside, then renamed in)
//!   repeat = {times, steps} the steps again, "{i}" in their strings replaced by 0, 1, ...
//! Any step can carry `if = PREDICATE`: it runs only when that holds on the tree now.

use std::path::{Path, PathBuf};

use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Case {
    /// area/name, from its path under cases/
    pub name: String,
    pub area: String,
    pub title: String,
    pub space: Option<String>,
    /// the page's hash route at launch on raea-page-host (the feed's `/feed/<id>`, ...)
    pub route: Option<String>,
    pub windows: Vec<(u32, u32)>,
    pub budget: u64,
    pub timeout_ms: u64,
    /// only in the full tier
    pub full_only: bool,
    pub env: Vec<(String, String)>,
    pub steps: Vec<Value>,
    pub path: PathBuf,
}

/// The contract's default budgets: 2 frames at 60 Hz for pointer and text, 1 for a live update, 3 for a resize or a
/// panel reflow.
pub fn default_budget(area: &str) -> u64 {
    match area {
        "live" => 1,
        "layout" | "responsive" | "panels" => 3,
        _ => 2,
    }
}

pub const AREAS: &[&str] = &["pointer", "scroll", "text", "focus", "layout", "responsive", "live", "panels", "review", "chat"];

pub fn load(path: &Path, root: &Path) -> Result<Case, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: Value = if path.extension().is_some_and(|e| e == "json") {
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?
    } else {
        toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?
    };
    let name = path.strip_prefix(root).unwrap_or(path).with_extension("").to_string_lossy().to_string();
    let area = v["area"].as_str().ok_or(format!("{name}: no area"))?.to_string();
    if !AREAS.contains(&area.as_str()) {
        return Err(format!("{name}: area {area:?} isn't one of {}", AREAS.join(", ")));
    }
    let size = |w: &Value| -> Option<(u32, u32)> { Some((w[0].as_u64()? as u32, w[1].as_u64()? as u32)) };
    let windows: Vec<(u32, u32)> = match (v["windows"].as_array(), size(&v["window"])) {
        (Some(ws), _) => ws.iter().filter_map(size).collect(),
        (None, Some(w)) => vec![w],
        (None, None) => vec![(1440, 900)],
    };
    if windows.is_empty() {
        return Err(format!("{name}: windows is empty"));
    }
    let steps = v["steps"].as_array().cloned().ok_or(format!("{name}: no steps"))?;
    for (i, s) in steps.iter().enumerate() {
        check_step(s).map_err(|e| format!("{name}: step {}: {e}", i + 1))?;
    }
    if !steps.iter().any(has_expect) {
        return Err(format!("{name}: no expect step: a case has to expect something"));
    }
    Ok(Case {
        title: v["title"].as_str().unwrap_or("").to_string(),
        space: v["space"].as_str().map(str::to_string),
        route: v["route"].as_str().map(str::to_string),
        budget: v["budget"].as_u64().unwrap_or_else(|| default_budget(&area)),
        timeout_ms: v["timeout_ms"].as_u64().unwrap_or(3000),
        full_only: v["tier"].as_str() == Some("full"),
        env: v["env"].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_str().map(str::to_string).unwrap_or(v.to_string()))).collect()).unwrap_or_default(),
        name,
        area,
        windows,
        steps,
        path: path.to_path_buf(),
    })
}

const KINDS: &[&str] = &["input", "wait", "expect", "snap", "write", "repeat"];

fn has_expect(s: &Value) -> bool {
    s.get("expect").is_some() || s["repeat"]["steps"].as_array().is_some_and(|ss| ss.iter().any(has_expect))
}

fn check_step(s: &Value) -> Result<(), String> {
    let kinds: Vec<&str> = KINDS.iter().copied().filter(|k| s.get(*k).is_some()).collect();
    if kinds.len() != 1 {
        return Err(format!("a step is exactly one of {} (this has {:?}): {s}", KINDS.join(", "), kinds));
    }
    if let Some(r) = s.get("repeat") {
        for (i, s) in r["steps"].as_array().ok_or("repeat needs steps")?.iter().enumerate() {
            check_step(s).map_err(|e| format!("repeat step {}: {e}", i + 1))?;
        }
    }
    if let Some(i) = s.get("input") {
        let t = i["type"].as_str().ok_or("input needs a type")?;
        const TYPES: &[&str] = &["move", "down", "up", "leave", "wheel", "key", "text", "ime", "resize", "click", "dblclick", "drag", "keys", "press"];
        if !TYPES.contains(&t) {
            return Err(format!("no input type {t:?} (one of {})", TYPES.join(", ")));
        }
    }
    Ok(())
}

/// Every case file under `root`, sorted by name.
pub fn all(root: &Path) -> Result<Vec<Case>, String> {
    let mut files = vec![];
    walk(root, &mut files);
    files.sort();
    files.iter().map(|f| load(f, root)).collect()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|e| e == "toml" || e == "json") {
            out.push(p);
        }
    }
}

/// "{i}" replaced by `i` in every string of `v`.
pub fn subst(v: &Value, i: u64) -> Value {
    match v {
        Value::String(s) => Value::String(s.replace("{i}", &i.to_string())),
        Value::Array(a) => Value::Array(a.iter().map(|x| subst(x, i)).collect()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, x)| (k.clone(), subst(x, i))).collect()),
        other => other.clone(),
    }
}
