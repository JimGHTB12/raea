//! R6a: @raea/rune's props and components (sdk/rune) drawn by the host: sdk/rune/examples/motion.tsx bundled and run.
//! A toast enters on its transition while everything without Rune motion lands at once; the kit's components stand
//! in for their DOM stand-ins; the switch's press reaches React; nothing is left unmapped.


use raea_rune_host::js::App;

mod common;


fn info(app: &App, id: u64) -> rune::ElementInfo {
    let k = raea_rune_host::session::Session::key_of(id);
    app.session.rt.elements().into_iter().find(|e| e.key.as_deref() == Some(&k)).unwrap()
}

#[test]
fn a_toast_enters_on_its_spring_and_the_rest_lands_at_once() {
    let mut app = common::example("motion.tsx");
    app.frame().unwrap();
    let add = app.session.tid("add").unwrap();
    app.input(|s| { let _ = s.click_el(add); }).unwrap();
    let toast = app.session.tid("toast-1").expect("a toast");
    // the frame it mounts: entering (pushed in from the top, fading), where a plain element would show whole
    let first = info(&app, toast);
    assert!(first.opacity < 0.9 || first.rect[1] < first.target[1] - 0.5, "entering: {first:?}");
    let panel = app.session.tid("panel").unwrap();
    assert!(info(&app, panel).opacity > 0.999, "the panel (no motion) whole");
    for _ in 0..90 {
        app.frame().unwrap();
    }
    let rest = info(&app, toast);
    assert!(rest.opacity > 0.999 && (rest.rect[1] - rest.target[1]).abs() < 0.5, "at rest: {rest:?}");
    assert!(app.session.rt.state.report.borrow().is_empty(), "{:?}", app.session.rt.state.report.borrow().all().collect::<Vec<_>>());
}

#[test]
fn kit_components_stand_in_and_the_switch_reaches_react() {
    let mut app = common::example("motion.tsx");
    app.frame().unwrap();
    let texts = |app: &App| app.session.rt.elements().into_iter().filter_map(|e| e.text).collect::<Vec<_>>();
    assert!(texts(&app).iter().any(|t| t == "normal"), "the badge: {:?}", texts(&app));
    let sw = app.session.rt.elements().into_iter().find(|e| e.role == "switch").expect("rune-kit's switch");
    let r = raea_rune_host::session::shown(&sw);
    app.input(|s| s.click(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0)).unwrap();
    for _ in 0..5 {
        app.frame().unwrap();
    }
    assert!(texts(&app).iter().any(|t| t == "urgent"), "the badge after the switch: {:?}", texts(&app));
}

#[test]
fn rolling_text_keeps_its_edge_padding_so_resting_glyphs_are_never_faded() {
    // the span's own style once overwrote the kit's padding, leaving the glyphs in the fade (R8b, a 108 px total)
    let mut app = common::example("motion.tsx");
    for _ in 0..30 {
        app.frame().unwrap();
    }
    let els = app.session.rt.elements();
    let cells: Vec<_> = els.iter().filter(|e| e.key.as_deref().is_some_and(|k| k.starts_with("rolling-") && k.contains('.'))).collect();
    let row = els.iter().find(|e| e.key.as_deref().is_some_and(|k| k.starts_with("rolling-") && !k.contains('.'))).expect("the rolling row");
    assert!(!cells.is_empty(), "rolling cells");
    let side = row.rect[0] + 0.5;
    assert!(cells.iter().all(|c| c.rect[0] >= side), "glyphs inside the row's padding: row {:?}, first cell {:?}", row.rect, cells[0].rect);
}
