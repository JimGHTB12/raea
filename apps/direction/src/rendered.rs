//! How `raea render` ends (docs/direction.feature DR3): what the render found, counted on the last line it prints
//! whatever its output is piped through, and its exit: 1 while an error stands that nothing lets through. An error is
//! let through by `--despite CHECK WHY` (a check's name, or `all`), or for a harness that renders many pieces and
//! compares pixels, not scenes, by RAEA_DESPITE="CHECK: why" (which a raea before this ignores, as it would refuse the
//! flag). What was let through, and why, is written into the render's manifest.

use raea_core_facts::{Finding, Level, finding};
use serde_json::{Value, json};

/// Each `--despite CHECK WHY`, then RAEA_DESPITE's.
fn despite(args: &[String], env: Option<String>) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = args.windows(3).filter(|w| w[0] == "--despite").map(|w| (w[1].clone(), w[2].clone())).collect();
    if let Some((check, why)) = env.as_deref().and_then(|e| e.split_once(':')) {
        out.push((check.trim().to_string(), why.trim().to_string()));
    }
    out
}

/// The last line and the exit, for `found` with `through` letting errors past.
fn ending(found: &[Finding], through: &[(String, String)]) -> (String, i32) {
    let excused = |f: &Finding| through.iter().any(|(c, _)| c == "all" || *c == f.check);
    let errors = |past: bool| found.iter().filter(|f| f.level == Level::Error && excused(f) == past).count();
    let mut line = format!("raea render: {}", finding::count(found));
    if errors(true) > 0 {
        let why: Vec<String> = through.iter().map(|(c, w)| format!("--despite {c} {w:?}")).collect();
        line.push_str(&format!("; {} let through by {}", errors(true), why.join(", ")));
    }
    (line, i32::from(errors(false) > 0))
}

/// `raea render`'s exit, once its findings' count is printed; 0 for any other command.
pub fn rendered(args: &[String]) -> i32 {
    if args.first().map(String::as_str) != Some("render") {
        return 0;
    }
    let found = finding::taken();
    let through = despite(args, std::env::var("RAEA_DESPITE").ok());
    if let (Some(m), false) = (raea_runtime_bundle::flag(args, "--manifest"), through.is_empty()) {
        // the reasons beside the findings they let through
        if let Some(mut v) = std::fs::read(&m).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()) {
            v["despite"] = Value::Array(through.iter().map(|(c, w)| json!({"check": c, "why": w})).collect());
            let _ = std::fs::write(&m, v.to_string());
        }
    }
    let (line, code) = ending(&found, &through);
    println!("{line}");
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn an_error_fails_the_render_unless_it_is_let_through_and_says_by_what() {
        let found = vec![Finding::new("clips", Level::Error, "rider in seat", "raea render"), Finding::new("tilted", Level::Warn, "vase leans", "raea render")];
        assert_eq!(ending(&found, &[]), ("raea render: 1 error, 1 warn, 0 notes".into(), 1));
        let through = despite(&s(&["render", "a.tsx", "--despite", "clips", "the rider is posed through it"]), None);
        assert_eq!(ending(&found, &through), ("raea render: 1 error, 1 warn, 0 notes; 1 let through by --despite clips \"the rider is posed through it\"".into(), 0));
        // another check's despite lets nothing through
        assert_eq!(ending(&found, &despite(&s(&["render", "--despite", "floats", "x"]), None)).1, 1);
        // a harness's
        assert_eq!(ending(&found, &despite(&s(&["render"]), Some("all: P26 compares pixels".into()))).1, 0);
        assert_eq!(ending(&[], &[]), ("raea render: 0 errors, 0 warns, 0 notes".into(), 0));
    }
}
