//! R4a: a bundled TSX app (the R5 counter) run by raea-runtime-jsengine in the host, clicked by a headless script
//! through Rune's input, re-rendering through React and Rune.

use raea_rune_host::js::App;
use raea_rune_host::session::Session;

mod common;


/// The text under element `t` (by data-testid), as the host's tree has it.
fn text(s: &Session, t: &str) -> String {
    let tree = &s.rt.state.tree;
    let mut out = String::new();
    let mut stack = vec![s.tid(t).unwrap()];
    while let Some(id) = stack.pop() {
        let n = tree.get(id).unwrap();
        out.push_str(n.text.as_deref().unwrap_or(""));
        stack.extend(n.children.iter().rev());
    }
    out
}

#[test]
fn the_counter_mounts_and_a_click_re_renders_it() {
    let mut app = common::app("counter.tsx");
    assert_eq!(text(&app.session, "count"), "Count: 0");
    let inc = app.session.tid("inc").unwrap();
    assert!(app.session.rect(inc).is_some_and(|r| r[2] > 0.0), "Rune laid the button out");
    for want in 1..=3 {
        app.input(|s| s.click_el(inc).unwrap()).unwrap();
        app.frame().unwrap();
        assert_eq!(text(&app.session, "count"), format!("Count: {want}"));
    }
    // past 2 a reset appears, laid out by Rune, and pressing it goes back to 0
    let reset = app.session.tid("reset").expect("the reset button");
    assert!(app.session.rect(reset).is_some_and(|r| r[2] > 0.0));
    app.input(|s| s.click_el(reset).unwrap()).unwrap();
    app.frame().unwrap();
    assert_eq!(text(&app.session, "count"), "Count: 0");
    assert!(app.session.tid("reset").is_none());
    assert!(!app.logs.borrow().iter().any(|l| !l.starts_with("0:")), "{:?}", app.logs.borrow());
    // sdk/ui reads localStorage's kept values before the page's code, synchronously: answered, not refused
    assert!(!app.refused.borrow().contains_key("net.storage.get"), "{:?}", app.refused.borrow());
}

#[test]
fn emits_calls_syncs_and_timers_go_through_the_host() {
    let src = common::script(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/apps/asks.tsx"));
    let mut app = App::new(&src, (400.0, 300.0)).unwrap();
    // effects run after the mount's commit, in a task of their own: the first frame has them
    app.frame().unwrap();
    let set = app.session.clock() * 1e3;
    assert_eq!(text(&app.session, "sync"), "err: the Rune host has no nothing");
    assert_eq!(text(&app.session, "answer"), "err: the Rune host has no nothing");
    assert!(app.session.rt.state.ticking, "the page asked for frames");
    app.frame().unwrap();
    assert_eq!(text(&app.session, "frames"), "ticking");
    // the timer fires on the virtual clock, 100 ms after the effect set it, and not before
    while app.session.clock() * 1e3 + 1e-6 < set + 100.0 - 1000.0 / 60.0 {
        assert_eq!(text(&app.session, "timer"), "waiting", "at {} ms", app.session.clock() * 1e3);
        app.frame().unwrap();
    }
    app.frame().unwrap();
    app.frame().unwrap();
    let fired = text(&app.session, "timer");
    let at: f64 = fired.strip_prefix("fired at ").expect(&fired).parse().unwrap();
    assert!((at - (set + 100.0)).abs() <= 17.0, "fired at {at} ms, set at {set} ms");
}
