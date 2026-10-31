//! The runner's own tests: the fake-cases suites run against conformance-fake (fake/), a substrate whose knobs make
//! cases slow or failing.

use conformance_fake::{Fake, Knobs};
use serde_json::Value;
use crate::run::{self, Opts};

/// Runs the case files matching `only` (fake-cases/: the legacy app's cases, whose tree the fake serves; they test the
/// runner, not an app) against a fresh fake each, and returns each one's results line.
fn suite(only: &[&str], k: Knobs) -> Vec<Value> {
    suite_with(only, k, &[])
}

/// `suite` with more runner arguments.
fn suite_with(only: &[&str], k: Knobs, extra: &[&str]) -> Vec<Value> {
    let cases = crate::here().join("fake-cases");
    let all = crate::case::all(&cases).unwrap();
    // the quick tier's cases (what the runner chooses with no --tier)
    let names: Vec<String> = all.iter().filter(|c| !c.full_only).map(|c| c.name.clone()).filter(|n| only.iter().any(|g| crate::pred::glob(g, n))).collect();
    assert!(!names.is_empty(), "no cases match {only:?}");
    let mut out = vec![];
    for name in names {
        let fake = Fake::start(k);
        // the fake's port too: tests run in parallel and may run the same case
        let dir = std::env::temp_dir().join(format!("conformance-test-{}-{}-{}", std::process::id(), fake.port, name.replace('/', "-")));
        let _ = std::fs::remove_dir_all(&dir);
        let mut args: Vec<String> = ["--attach", &fake.port.to_string(), "--cases", cases.to_str().unwrap(), "--only", &name, "--out", dir.to_str().unwrap(), "--substrate", "fake"].iter().map(|s| s.to_string()).collect();
        args.extend(extra.iter().map(|s| s.to_string()));
        let code = run::main(Opts::parse(&args).unwrap());
        let lines = std::fs::read_to_string(dir.join("results.jsonl")).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        for l in lines.lines() {
            let v: Value = serde_json::from_str(l).unwrap();
            assert_eq!(code == 1, v["status"] == "fail", "{name}: exit {code} for {v}");
            out.push(v);
        }
        drop(fake);
    }
    out
}

fn statuses(rs: &[Value]) -> Vec<String> {
    rs.iter().map(|r| format!("{} {} {}", r["case"].as_str().unwrap(), r["status"].as_str().unwrap(), r["detail"].as_str().unwrap())).collect()
}

/// Every case the toy feed can answer passes on it, within budget.
#[test]
fn fake_passes() {
    let rs = suite(
        &[
            "pointer/card-hover*",
            "pointer/left-feed-select*",
            "pointer/chat-toggle-twice",
            "scroll/feed-*",
            "scroll/chat-transcript",
            "scroll/only-under-pointer",
            "text/search-*",
            "text/composer-*",
            "text/ime-*",
            "focus/*-press-focuses",
            "focus/focus-moves",
            "panels/chat-toggle",
            "panels/left-toggle",
            "layout/feed*",
            "responsive/middle-min",
            "live/chat-message-arrives",
            "live/transcript-scrolls",
        ],
        Knobs::default(),
    );
    let bad: Vec<String> = statuses(&rs).into_iter().filter(|s| !s.contains(" pass ")).collect();
    assert!(bad.is_empty(), "{bad:#?}");
}

/// A substrate that shows things late is slow by how many frames, not failing.
#[test]
fn lag_is_slow() {
    let rs = suite(&["pointer/card-hover"], Knobs { lag: 5, ..Default::default() });
    assert_eq!(rs[0]["status"], "slow", "{rs:?}");
    assert!(rs[0]["over"].as_u64().unwrap() >= 3, "{rs:?}");
}

/// A substrate that lays a new size or a toggled panel out frames after the window reports it: the start snapshot
/// and each step's targets wait for the settled tree, so a panel toggle compares against the layout at the case's
/// size, and the wait isn't counted (2 frames of reflow, within the panels' budget of 3, still pass)...
#[test]
fn settles_before_start_and_targets() {
    let k = Knobs { reflow: 6, ..Default::default() };
    let rs = suite(&["panels/chat-toggle", "panels/left-toggle", "pointer/left-feed-select"], Knobs { reflow: 2, ..k });
    let bad: Vec<String> = statuses(&rs).into_iter().filter(|s| !s.contains(" pass ")).collect();
    assert!(bad.is_empty(), "{bad:#?}");
    // ...where a start snapshot taken without the settle is of the old layout, and the toggle back never matches it
    let rs = suite_with(&["panels/chat-toggle"], k, &["--settle-ms", "0"]);
    assert_eq!(rs[0]["status"], "fail", "{rs:?}");
    assert!(rs[0]["detail"].as_str().unwrap().contains("middle"), "{rs:?}");
}

/// A tree that never settles fails within the bound, and says what kept moving.
#[test]
fn never_settles_fails() {
    let t = std::time::Instant::now();
    let rs = suite_with(&["panels/chat-toggle"], Knobs { jitter: true, ..Default::default() }, &["--settle-ms", "400"]);
    assert_eq!(rs[0]["status"], "fail", "{rs:?}");
    let d = rs[0]["detail"].as_str().unwrap();
    assert!(d.contains("never settled in 400 ms") && d.contains("middle"), "{d}");
    assert!(t.elapsed() < std::time::Duration::from_secs(5), "took {:?}", t.elapsed());
}

/// A panel flagged stale forever in a layout that doesn't move counts as settled after a grace, noted in the result.
#[test]
fn stuck_stale_settles_with_a_note() {
    let rs = suite(&["panels/chat-toggle"], Knobs { stuck: true, ..Default::default() });
    assert_eq!(rs[0]["status"], "pass", "{rs:?}");
    assert!(rs[0]["detail"].as_str().unwrap().contains("settled with panels stale"), "{rs:?}");
}

/// A substrate that never hovers fails, and says what the tree showed.
#[test]
fn broken_fails() {
    let rs = suite(&["pointer/card-hover"], Knobs { no_hover: true, ..Default::default() });
    assert_eq!(rs[0]["status"], "fail", "{rs:?}");
    let d = rs[0]["detail"].as_str().unwrap();
    assert!(d.contains("feed.card.0.hovered is false"), "{d}");
}
