//! `raea app check`'s portability report (docs/targets.feature T4a): what the native host couldn't draw or answer,
//! read from the files raea-rune-host writes beside its frames: unmapped.json (each prop or value with the path of the
//! first element it was seen on, and why) and unanswered.json (each call the app made that the host refused, why, and
//! how many times). On the native target any line fails the check; on a web app the native host runs too, and its
//! report says what native wouldn't draw, without failing.

use std::path::Path;

use serde_json::Value;

/// One line per unmapped prop and per unanswered call, as the host found them in `dir`.
pub fn report(dir: &Path) -> Result<Vec<String>, String> {
    let read = |name: &str| -> Result<Vec<Value>, String> {
        let p = dir.join(name);
        let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))
    };
    let s = |v: &Value, k: &str| v[k].as_str().unwrap_or("").to_string();
    let mut lines: Vec<String> = read("unmapped.json")?.iter().map(|u| format!("unmapped {}: {} at {} ({})", s(u, "prop"), s(u, "value"), s(u, "path"), s(u, "why"))).collect();
    for c in read("unanswered.json")? {
        let n = c["times"].as_u64().unwrap_or(1);
        lines.push(format!("unanswered {} ({}), {n} time{}", s(&c, "call"), s(&c, "why"), if n == 1 { "" } else { "s" }));
    }
    Ok(lines)
}

/// The native check's verdict on `lines`: ok when there are none, else an error naming each.
pub fn verdict(lines: &[String], dir: &Path) -> Result<String, String> {
    if lines.is_empty() {
        return Ok(String::new());
    }
    Err(format!("not portable to native: {} thing{} the Rune host didn't draw or answer (unmapped.json, unanswered.json in {}):\n  {}", lines.len(), if lines.len() == 1 { "" } else { "s" }, dir.display(), lines.join("\n  ")))
}

/// A web app's note: what the native target wouldn't draw, or that it would draw everything.
pub fn note(lines: &[String]) -> String {
    if lines.is_empty() {
        return "native: the Rune host draws and answers all of it".into();
    }
    format!("native wouldn't draw or answer {} thing{} (the web check still passes):\n  {}", lines.len(), if lines.len() == 1 { "" } else { "s" }, lines.join("\n  "))
}

/// The Rune props a web app has that the web target ignores (T4b): @raea/rune names them on each element in
/// `data-rune-web-ignored`, and the native host's run lists them by path in web-ignored.json.
pub fn web_ignored(dir: &Path) -> Result<Vec<String>, String> {
    let p = dir.join("web-ignored.json");
    let text = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let v: Vec<Value> = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(v.iter().map(|w| format!("ignored on web: {} at {}", w["ignored"].as_str().unwrap_or(""), w["path"].as_str().unwrap_or(""))).collect())
}

/// A web app's Rune props the web ignores, said; empty when there are none.
pub fn web_note(lines: &[String]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    format!("Rune props the web target doesn't draw (@raea/rune has no CSS for them):\n  {}", lines.join("\n  "))
}
