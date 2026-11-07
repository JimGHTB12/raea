//! T3c (docs/targets.feature): transitions become curves of their duration on a page, so a hover fades and slides as it
//! would in a browser; without a transition a change lands at once. style/tests/motion.rs checks the parsing.

use raea_rune_page::page::Page;
use raea_rune_page::session::Session;
use serde_json::json;

/// One 160x40 box with `class` at the top left (wide enough to stay under the pointer when it slides 40), and its drawn element after each of `frames` frames once it's hovered.
fn hovered(class: &str, frames: usize) -> Vec<rune::ElementInfo> {
    let mut s = Session::new(Page::default(), (400.0, 300.0));
    let batch = json!({"v": 1, "ops": [{"op": "create", "id": 1, "kind": "div", "props": {"className": format!("w-40 h-10 {class}")}}, {"op": "append", "parent": 0, "child": 1}]});
    s.apply(&batch.to_string()).unwrap();
    s.frame();
    s.pointer_move(100.0, 20.0);
    (0..frames)
        .map(|_| {
            s.frame();
            s.rt.elements().into_iter().find(|e| e.key.as_deref() == Some("w1")).unwrap()
        })
        .collect()
}

#[test]
fn a_hover_fades_and_slides_over_the_transitions_duration() {
    // 100 ms linear: six frames at 60 fps; the restyle lands a frame after the hover
    let f = hovered("opacity-100 hover:opacity-50 hover:translate-x-10 transition duration-100 ease-linear", 12);
    let (o, x): (Vec<f32>, Vec<f32>) = f.iter().map(|e| (e.opacity, e.rect[0])).unzip();
    assert!(o.windows(2).all(|w| w[1] <= w[0] + 1e-4) && x.windows(2).all(|w| w[1] >= w[0] - 1e-4), "monotonic: {o:?} {x:?}");
    // linear over 100 ms: an equal step each 1/60 s frame (a sixth of the way), so the whole change takes six frames
    // counted from where it began (the frames before the first recorded one included)
    let steps = |v: &[f32], from: f32, to: f32| -> f32 {
        let step = (to - from) / 6.0;
        let moving: Vec<f32> = v.iter().copied().take_while(|a| (a - to).abs() > 1e-3).collect();
        for w in moving.windows(2) {
            assert!((w[1] - w[0] - step).abs() < 1e-3 * (to - from).abs().max(1.0), "an even step each frame: {v:?}");
        }
        (moving[0] - from) / step + moving.len() as f32
    };
    assert!((steps(&o, 1.0, 0.5) - 6.0).abs() < 1e-2, "{o:?}");
    assert!((steps(&x, 0.0, 40.0) - 6.0).abs() < 1e-2, "{x:?}");
    assert_eq!((*o.last().unwrap(), *x.last().unwrap()), (0.5, 40.0));
}

#[test]
fn without_a_transition_a_hover_lands_at_once() {
    let f = hovered("bg-white hover:bg-black hover:translate-x-10", 3);
    let at = f.iter().position(|e| e.rect[0] > 0.0).expect("the hover lands");
    assert_eq!(f[at].rect[0], 40.0);
    assert_eq!(f[at].fill, rune::Color::BLACK);
}

#[test]
fn colours_spring_for_the_transitions_duration() {
    let f = hovered("bg-white hover:bg-black transition-colors duration-200", 20);
    let at = f.iter().position(|e| e.fill != rune::Color::WHITE).expect("it changes");
    assert_ne!(f[at].fill, rune::Color::BLACK, "on its way, not landed");
    assert_eq!(f.last().unwrap().fill, rune::Color::BLACK, "settled within 20 frames of a 200 ms spring");
}
