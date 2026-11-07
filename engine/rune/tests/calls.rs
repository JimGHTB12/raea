//! T2c (docs/targets.feature): a call the native target can't answer says so. world.* is refused with a typed error
//! naming T6, any other unknown call as one this host lacks, a sync call the same way, and every one is kept for the
//! check's unanswered.json.
use raea_rune_host::js::App;

const SCRIPT: &str = r#"
  globalThis.__ui_deliver = (json) => { const m = JSON.parse(json); if (m.t === "answer") console.log("answer " + m.id + " " + m.ok + " " + m.error); };
  __ui_call(1, "world.spawn_entity", JSON.stringify({ components: {} }));
  __ui_call(2, "files.open", "{}");
  __ui_call(3, "world.spawn_entity", "{}");
  console.log("sync " + __ui_sync("layout.now", "null"));
"#;

#[test]
fn a_call_the_native_target_can_t_answer_says_so_and_is_listed() {
    let app = App::new(SCRIPT, (100.0, 100.0)).unwrap();
    let logs = app.logs.borrow().join("\n");
    assert!(logs.contains("answer 1 false world.spawn_entity: no world on the native target (targets.feature T6)"), "{logs}");
    assert!(logs.contains("answer 2 false the Rune host has no files.open"), "{logs}");
    assert!(logs.contains("the Rune host has no layout.now"), "{logs}");
    let refused = app.refused.borrow();
    let listed: Vec<(&str, u32)> = refused.iter().map(|(k, (_, n))| (k.as_str(), *n)).collect();
    assert_eq!(listed, [("files.open", 1), ("layout.now", 1), ("world.spawn_entity", 2)]);
}
