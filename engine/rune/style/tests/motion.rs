//! T3c (docs/targets.feature): CSS transforms read as Rune's offset, scale and rotation, and transitions as their
//! properties and timing (Tailwind's utilities too). page/tests/motion.rs runs them on a page.

use raea_rune_style::css::parse;
use raea_rune_style::tailwind::style_in;
use raea_rune_style::variants::State;
use serde_json::{Map, Value, json};

fn css(v: Value) -> (raea_rune_style::css::Css, Vec<String>) {
    let (c, bad) = parse(&serde_json::from_value::<Map<String, Value>>(v).unwrap());
    (c, bad.into_iter().map(|b| b.prop).collect())
}

#[test]
fn transforms_read_as_offset_scale_and_rotation() {
    let (c, bad) = css(json!({"transform": "translateX(10px) translateY(-4px) scale(1.5) rotate(90deg)", "transformOrigin": "left top"}));
    assert!(bad.is_empty(), "{bad:?}");
    let m = c.motion;
    assert_eq!((m.translate, m.scale, m.origin), ([10.0, -4.0], Some(1.5), Some((0.0, 0.0))));
    assert!((m.rotate.unwrap() - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    // what Rune can't draw is reported: a translate along turned axes, skew, a scale per axis, a percentage
    for t in ["rotate(10deg) translateX(5px)", "skewX(10deg)", "scale(1, 2)", "translateX(50%)"] {
        assert_eq!(css(json!({"transform": t})).1, ["transform"], "{t}");
    }
    let (c, bad) = css(json!({"translate": "3px 4px", "scale": "110%", "rotate": "0.5turn"}));
    assert!(bad.is_empty(), "{bad:?}");
    assert_eq!((c.motion.translate, c.motion.scale), ([3.0, 4.0], Some(1.1)));
}

#[test]
fn transitions_read_their_properties_and_timing() {
    let (c, bad) = css(json!({"transition": "opacity 200ms ease-in 50ms, transform 200ms ease-in 50ms"}));
    assert!(bad.is_empty(), "{bad:?}");
    let t = c.motion.transition.unwrap();
    assert_eq!((t.duration, t.delay, t.easing), (0.2, 0.05, [0.42, 0.0, 1.0, 1.0]));
    assert!(t.props.opacity && t.props.transform && !t.props.colours);
    assert_eq!(css(json!({"transition": "opacity 1s, color 2s"})).1, ["transition"], "two timings on one element");
    // Tailwind: the utility's properties and its defaults, under duration-* and ease-* whatever their order
    let (s, unknown) = style_in("duration-300 transition-colors ease-out translate-x-4 scale-105 rotate-45 origin-top-left", &State::default());
    assert!(unknown.is_empty(), "{unknown:?}");
    let (c, bad) = parse(&s);
    assert!(bad.is_empty(), "{bad:?}");
    let t = c.motion.transition.unwrap();
    assert_eq!((t.duration, t.easing), (0.3, [0.0, 0.0, 0.2, 1.0]));
    assert!(t.props.colours && !t.props.opacity);
    assert_eq!((c.motion.tw_translate, c.motion.scale, c.motion.origin), ([16.0, 0.0], Some(1.05), Some((0.0, 0.0))));
    let (c, _) = parse(&style_in("transition", &State::default()).0);
    assert_eq!(c.motion.transition.unwrap().duration, 0.15, "Tailwind's default duration");
}
