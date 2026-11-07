//! T3e: what the T3 set's run against Chrome needed mapped, each at the value Chrome computes.

use raea_rune_style::css::{Len, color, length, parse, set_viewport};
use raea_rune_style::outline::drawn;
use raea_rune_style::tailwind::style_in;
use raea_rune_style::variants::State;
use serde_json::{Value, json};

fn style(class: &str, s: &State) -> serde_json::Map<String, Value> {
    let (m, unknown) = style_in(class, s);
    assert!(unknown.is_empty(), "{unknown:?}");
    m
}

fn rgb8(c: rune::Color) -> [u8; 4] {
    let e = |x: f32| (rune::color::srgb_encode(x) * 255.0).round() as u8;
    [e(c.r), e(c.g), e(c.b), (c.a * 255.0).round() as u8]
}

#[test]
fn a_none_component_is_zero() {
    // Tailwind's greys have no hue: zinc-50, as Chrome computes it
    assert_eq!(rgb8(color("oklch(98.5% 0 none)").expect("read")), [250, 250, 250, 255]);
}

#[test]
fn screen_sizes_are_the_viewport() {
    let s = style("h-screen min-h-dvh w-screen", &State::default());
    assert_eq!((s["height"].clone(), s["minHeight"].clone(), s["width"].clone()), (json!("100vh"), json!("100vh"), json!("100vw")));
    set_viewport((640.0, 480.0));
    assert_eq!((length(&json!("100vh")), length(&json!("50vw"))), (Some(Len::Px(480.0)), Some(Len::Px(320.0))));
}

#[test]
fn leading_steps_are_lengths() {
    let (c, bad) = parse(&style("text-sm leading-6", &State::default()));
    assert!(bad.is_empty(), "{bad:?}");
    assert_eq!(c.line_height, Some(Len::Px(24.0)), "24px, not 24 times the font size");
}

#[test]
fn rings_are_shadow_layers_in_tailwinds_order() {
    let on = State { focus_visible: true, focus: true, ..State::default() };
    let s = style("shadow-sm focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2", &on);
    let (c, bad) = parse(&s);
    assert!(bad.is_empty(), "{bad:?}");
    // the offset's white, the ring a spread the offset wider, then shadow-sm's two layers
    let spreads: Vec<f32> = c.shadows.iter().map(|l| l.spread).collect();
    assert_eq!(spreads, [2.0, 4.0, 0.0, -1.0]);
    assert_eq!(rgb8(c.shadows[0].color), [255, 255, 255, 255]);
    assert!(!s.keys().any(|k| k.starts_with("--tw-")), "the variables are composed away: {s:?}");
    // not focused: only the shadow
    assert_eq!(parse(&style("shadow-sm focus-visible:ring-2", &State::default())).0.shadows.len(), 2);
    // an alpha ring
    let c = parse(&style("ring-2 ring-blue-500/30", &State::default())).0;
    assert_eq!((c.shadows.len(), c.shadows[0].spread, (c.shadows[0].color.a * 100.0).round()), (1, 2.0, 30.0));
}

#[test]
fn outlines_are_drawn_as_chrome_draws_them() {
    let fg = rune::Color::BLACK;
    let c = |class: &str| parse(&style(class, &State::default())).0;
    // no outline of its own: Chrome's focus-visible ring, 1px of -webkit-focus-ring-color
    let (w, off, col) = drawn(&c("p-1").outline, fg, true).expect("a ring");
    assert_eq!((w, off, rgb8(col)), (1.0, 0.0, [0, 95, 204, 255]));
    assert_eq!(drawn(&c("p-1").outline, fg, false), None, "only while the focus shows");
    assert_eq!(drawn(&c("outline-none").outline, fg, true), None);
    let (o, bad) = parse(json!({"outline": "2px solid red", "outlineOffset": 3}).as_object().unwrap());
    assert!(bad.is_empty(), "{bad:?}");
    let (w, off, col) = drawn(&o.outline, fg, false).expect("its own outline");
    assert_eq!((w, off, rgb8(col)), (2.0, 3.0, [255, 0, 0, 255]));
    assert_eq!(parse(json!({"outlineStyle": "dashed"}).as_object().unwrap()).1.len(), 1, "a style Rune doesn't draw is reported");
}
