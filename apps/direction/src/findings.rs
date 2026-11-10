//! `raea findings` (docs/direction.feature DR3): what renders and checks found, read from the project's record
//! (core/facts record.rs). For each thing rendered or checked, its latest entry against the one before (or the one
//! `--since` names): what was fixed, what is new and what is carried, and since which render.

use std::path::{Path, PathBuf};

use raea_core_facts::record::{self, Entry};
use raea_core_facts::{Finding, finding};
use raea_runtime_bundle::flag;
use serde_json::{Value, json};

pub const USAGE: &str = "raea findings [PROJECT | EDIT.tsx] [--about NAME] [--check ID] [--open] [--since RENDER] [--json]   (what each render and raea check found, from the project's record: fixed, new and carried since the one before, or since RENDER; --open: only what stands)";

/// The oldest entry of the unbroken run up to the latest that all found `f`.
fn carried_since(entries: &[&Entry], f: &Finding) -> String {
    let key = record::same(f);
    let run = entries.iter().rev().take_while(|e| e.findings.iter().any(|g| record::same(g) == key));
    run.last().map(|e| e.id.clone()).unwrap_or_default()
}

/// One thing's latest entry against the one before it (or `since`), each list filtered by `keep`.
fn compare(entries: &[&Entry], since: Option<&str>, keep: &dyn Fn(&Finding) -> bool, open: bool) -> Option<Value> {
    let (now, earlier) = entries.split_last()?;
    let before = match since {
        Some(id) => earlier.iter().rev().find(|e| e.id.starts_with(id)),
        None => earlier.last(),
    };
    let none = Vec::new();
    let ch = record::changes(before.map_or(&none, |b| &b.findings), &now.findings);
    let list = |fs: Vec<&Finding>| fs.into_iter().filter(|f| keep(f)).cloned().collect::<Vec<Finding>>();
    let carried: Vec<Value> = list(ch.carried).into_iter().map(|f| json!({"since": carried_since(entries, &f), "finding": f})).collect();
    let fixed = if open { Vec::new() } else { list(ch.fixed) };
    Some(json!({"of": now.of, "id": now.id, "by": now.by, "commit": now.commit, "out": now.out, "against": before.map(|b| &b.id),
        "new": list(ch.new), "carried": carried, "fixed": fixed, "despite": now.despite}))
}

/// One thing's lines: its entry, then what is new, carried and fixed.
fn lines(r: &Value) -> Vec<String> {
    let read = |v: &Value| serde_json::from_value::<Finding>(v.clone()).ok();
    let mut t = vec![format!(
        "{} ({} {}{}, against {})",
        r["of"].as_str().unwrap_or(""),
        r["by"].as_str().unwrap_or(""),
        r["id"].as_str().unwrap_or(""),
        r["commit"].as_str().map(|c| format!(" at {c}")).unwrap_or_default(),
        r["against"].as_str().unwrap_or("nothing: its first")
    )];
    t.extend(r["new"].as_array().into_iter().flatten().filter_map(read).map(|f| format!("  new      {}", f.line())));
    for c in r["carried"].as_array().into_iter().flatten() {
        if let Some(f) = read(&c["finding"]) {
            t.push(format!("  carried  {} (since {})", f.line(), c["since"].as_str().unwrap_or("")));
        }
    }
    t.extend(r["fixed"].as_array().into_iter().flatten().filter_map(read).map(|f| format!("  fixed    {}", f.line())));
    t
}

/// What stands across the reports: each one's new and carried.
fn standing(reports: &[Value]) -> Vec<Finding> {
    let mut out = Vec::new();
    for r in reports {
        out.extend(r["new"].as_array().into_iter().flatten().filter_map(|v| serde_json::from_value(v.clone()).ok()));
        out.extend(r["carried"].as_array().into_iter().flatten().filter_map(|c| serde_json::from_value(c["finding"].clone()).ok()));
    }
    out
}

/// `raea findings ...` (args from "findings").
pub fn cli(args: &[String]) -> Result<String, String> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        return Ok(USAGE.into());
    }
    let target = args.get(1).filter(|a| !a.starts_with("--")).map_or_else(|| PathBuf::from("."), PathBuf::from);
    // made absolute first: a bare name's parent is "", which names no folder
    let from = if target.is_file() { target.canonicalize().unwrap_or_else(|_| target.clone()).parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf) } else { target.clone() };
    let project = record::up(&from, ".raea/project").ok_or_else(|| format!("raea findings: {} isn't in a raea project (no .raea/project at or above it), so nothing is recorded\n{USAGE}", target.display()))?;
    let (about, check, since) = (flag(args, "--about"), flag(args, "--check"), flag(args, "--since"));
    let keep = |f: &Finding| about.as_deref().is_none_or(|n| f.about.iter().any(|a| a.name == n || a.key.as_deref() == Some(n))) && check.as_deref().is_none_or(|c| f.check == c);
    let all = record::read(&project);
    let only = target.is_file().then(|| record::rel(&project, &target));
    let mut ofs: Vec<&str> = all.iter().map(|e| e.of.as_str()).filter(|o| only.as_deref().is_none_or(|w| w == *o)).collect();
    ofs.sort();
    ofs.dedup();
    let open = args.iter().any(|a| a == "--open");
    let reports: Vec<Value> = ofs.iter().filter_map(|of| compare(&all.iter().filter(|e| e.of == *of).collect::<Vec<_>>(), since.as_deref(), &keep, open)).collect();
    let count = finding::count(&standing(&reports));
    if args.iter().any(|a| a == "--json") {
        return Ok(serde_json::to_string_pretty(&json!({"project": project, "entries": all.len(), "reports": reports, "standing": count})).unwrap_or_default());
    }
    let mut t = vec![format!("raea findings: {} ({} entries in .raea/record)", project.display(), all.len())];
    if reports.is_empty() {
        t.push("  nothing recorded yet: a render or raea check of a file in this project keeps what it found".into());
    }
    t.extend(reports.iter().flat_map(lines));
    t.push(format!("raea findings: {count} standing"));
    Ok(t.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use raea_core_facts::Level;

    fn entry(id: &str, fs: Vec<Finding>) -> Entry {
        Entry { id: id.into(), at: 0, by: "raea render".into(), of: "edit.tsx".into(), out: None, commit: None, findings: fs, despite: vec![] }
    }

    #[test]
    fn the_latest_render_against_the_one_before_says_fixed_new_and_carried_since_when() {
        let clips = Finding::new("clips", Level::Error, "rider in seat", "raea render").about("rider", None);
        let tilt = Finding::new("tilted", Level::Warn, "vase leans", "raea render").about("vase", None);
        let float = Finding::new("floats", Level::Error, "lamp floats", "raea render").about("lamp", None);
        let es = [entry("a", vec![clips.clone(), tilt.clone()]), entry("b", vec![clips.clone(), tilt]), entry("c", vec![clips, float])];
        let refs: Vec<&Entry> = es.iter().collect();
        let all = |_: &Finding| true;
        let r = compare(&refs, None, &all, false).unwrap();
        assert_eq!((r["against"].as_str(), r["new"][0]["check"].as_str(), r["fixed"][0]["check"].as_str()), (Some("b"), Some("floats"), Some("tilted")));
        assert_eq!((r["carried"][0]["finding"]["check"].as_str(), r["carried"][0]["since"].as_str()), (Some("clips"), Some("a")));
        // against a named render, and only what stands
        let r = compare(&refs, Some("a"), &all, true).unwrap();
        assert_eq!((r["against"].as_str(), r["fixed"].as_array().map(Vec::len)), (Some("a"), Some(0)));
        assert_eq!(finding::count(&standing(&[r])), "2 errors, 0 warns, 0 notes");
        let lamp = |f: &Finding| f.about.iter().any(|a| a.name == "lamp");
        assert_eq!(compare(&refs, None, &lamp, false).unwrap()["carried"].as_array().map(Vec::len), Some(0));
    }
}
