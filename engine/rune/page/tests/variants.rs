//! T3b (docs/targets.feature): Tailwind's variants resolve per element and per state. The cascade follows Tailwind
//! 4.3.3's order, and through a headless session hover, active, focus, focus-visible, disabled, dark and the
//! breakpoints restyle what they're on (seen here as the widths they set).

use raea_rune_page::page::Page;
use raea_rune_page::session::Session;
use raea_rune_page::tailwind::style_in;
use raea_rune_page::variants::State;
use serde_json::json;

#[test]
fn variants_hold_per_state_in_tailwinds_order() {
    let at = |s: State| style_in("w-1 hover:w-2 focus:w-3 md:w-4 md:hover:w-5 lg:w-6 dark:w-7 dark:hover:w-8", &s).0["width"].clone();
    let w = |n: f64| json!(n * 4.0);
    assert_eq!(at(State::default()), w(1.0));
    assert_eq!(at(State { hover: true, ..State::default() }), w(2.0));
    // focus comes after hover in Tailwind's CSS, so it wins when both hold
    assert_eq!(at(State { hover: true, focus: true, ..State::default() }), w(3.0));
    // breakpoints are min-widths (md: 48rem), after the pseudo-classes; md:hover: sits between md: and lg:
    assert_eq!(at(State { focus: true, width: 768.0, ..State::default() }), w(4.0));
    assert_eq!(at(State { hover: true, width: 800.0, ..State::default() }), w(5.0));
    assert_eq!(at(State { hover: true, width: 1024.0, ..State::default() }), w(6.0));
    assert_eq!(at(State { width: 767.0, ..State::default() }), w(1.0));
    // dark: comes last, and dark:hover: after it
    assert_eq!(at(State { dark: true, width: 1100.0, ..State::default() }), w(7.0));
    assert_eq!(at(State { dark: true, hover: true, width: 1100.0, ..State::default() }), w(8.0));
    // focus-visible needs the focus; enabled and disabled only a control
    let fv = |s: State| style_in("w-1 focus-visible:w-2 disabled:w-3", &s).0["width"].clone();
    assert_eq!(fv(State { focus_visible: true, ..State::default() }), w(1.0));
    assert_eq!(fv(State { focus: true, focus_visible: true, ..State::default() }), w(2.0));
    assert_eq!(fv(State { disabled: true, ..State::default() }), w(1.0));
    assert_eq!(fv(State { disabled: true, control: true, ..State::default() }), w(3.0));
}

#[test]
fn unmapped_variants_are_reported_in_any_state() {
    let (_, unknown) = style_in("hover:bg-red-500 md:flex dark:hover:skew-x-3 group-hover:flex first:p-2 placeholder:text-sm -mt-2 p-4", &State::default());
    assert_eq!(unknown, ["dark:hover:skew-x-3", "group-hover:flex", "first:p-2", "placeholder:text-sm", "-mt-2"]);
}

/// A page of three: a box with `class` (id 1) after a focusable one (id 2) to Tab from, under an 800x600 viewport.
fn page(class: &str, kind: &str) -> Session {
    let mut s = Session::new(Page::default(), (800.0, 600.0));
    let batch = json!({"v": 1, "ops": [
        {"op": "create", "id": 2, "kind": "div", "props": {"tabIndex": 0, "style": {"width": 10, "height": 10}}},
        {"op": "create", "id": 1, "kind": kind, "props": {"className": class, "tabIndex": 0, "style": {"height": 20}}},
        {"op": "append", "parent": 0, "child": 2}, {"op": "append", "parent": 0, "child": 1}]});
    s.apply(&batch.to_string()).unwrap();
    s.take_out();
    s
}

fn width(s: &Session) -> Option<f32> {
    s.rect(1).filter(|r| r[2] > 0.0).map(|r| r[2])
}

#[test]
fn hover_restyles_through_runes_own_hover() {
    let mut s = page("w-10 hover:w-20", "div");
    assert_eq!(width(&s), Some(40.0));
    s.hover_el(1);
    assert_eq!(width(&s), Some(80.0));
    s.pointer_move(700.0, 500.0);
    s.frame();
    assert_eq!(width(&s), Some(40.0));
}

#[test]
fn a_press_focuses_without_showing_it_and_tab_shows_it() {
    let mut s = page("w-10 focus:w-16 focus-visible:w-20", "div");
    s.click_el(1);
    s.frame();
    assert_eq!(width(&s), Some(64.0), "focused by a press: focus, not focus-visible");
    s.click_el(2);
    s.key(rune::Key::Tab);
    s.frame();
    assert_eq!(s.rt.state.focus(), Some(1));
    assert_eq!(width(&s), Some(80.0), "focused by Tab: focus-visible");
    // a press on what already has the focus leaves it visible, as Chrome does
    s.click_el(1);
    s.frame();
    assert_eq!(width(&s), Some(80.0));
    // past the last, Tab leaves the page
    s.key(rune::Key::Tab);
    s.frame();
    assert_eq!(s.rt.state.focus(), None);
    assert_eq!(width(&s), Some(40.0));
}

#[test]
fn dark_and_the_breakpoints_follow_the_system_and_the_window() {
    let mut s = page("hidden w-10 md:flex dark:w-20", "div");
    assert_eq!(width(&s), Some(40.0), "md: holds at 800");
    s.set_dark(true);
    s.frame();
    assert_eq!(width(&s), Some(80.0));
    s.resize(600.0, 600.0);
    s.frame();
    assert_eq!(width(&s), None, "below md: hidden");
    s.set_dark(false);
    s.resize(1100.0, 600.0);
    s.frame();
    assert_eq!(width(&s), Some(40.0));
}

#[test]
fn disabled_is_a_controls() {
    let mut s = page("w-10 disabled:w-20", "button");
    assert_eq!(width(&s), Some(40.0));
    let off = json!({"v": 1, "ops": [{"op": "update", "id": 1, "set": {"disabled": true}}]});
    s.apply(&off.to_string()).unwrap();
    s.frame();
    assert_eq!(width(&s), Some(80.0));
}
