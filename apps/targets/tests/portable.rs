//! T4a: the check's portability report, from the files raea-rune-host writes beside its frames.

use raea_app_targets::portable::{note, report, verdict};

fn dir(unmapped: &str, unanswered: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("raea-t4a-{}-{}", std::process::id(), unmapped.len() * 31 + unanswered.len()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("unmapped.json"), unmapped).unwrap();
    std::fs::write(d.join("unanswered.json"), unanswered).unwrap();
    d
}

#[test]
fn each_unmapped_prop_and_unanswered_call_is_named() {
    let d = dir(
        r#"[{"prop": "transform", "value": "skewX(10deg)", "path": "/div[0]/span[0]", "why": "no skew"}]"#,
        r#"[{"call": "files.open", "why": "the Rune host has no files.open", "times": 2}]"#,
    );
    let lines = report(&d).unwrap();
    assert_eq!(lines, ["unmapped transform: skewX(10deg) at /div[0]/span[0] (no skew)", "unanswered files.open (the Rune host has no files.open), 2 times"]);
    // native fails on them, naming each; web only says them
    let e = verdict(&lines, &d).unwrap_err();
    assert!(e.starts_with("not portable to native: 2 things") && e.contains("/div[0]/span[0]") && e.contains("files.open"), "{e}");
    assert!(note(&lines).contains("the web check still passes"));
}

#[test]
fn nothing_unmapped_passes() {
    let d = dir("[]", "[]");
    let lines = report(&d).unwrap();
    assert_eq!(verdict(&lines, &d), Ok(String::new()));
    assert_eq!(note(&lines), "native: the Rune host draws and answers all of it");
}

#[test]
fn a_run_with_no_report_is_an_error_not_a_pass() {
    let d = std::env::temp_dir().join(format!("raea-t4a-none-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    assert!(report(&d).is_err());
}
