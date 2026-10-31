//! Declared animations: the wire format raea's `<Animate>` (and `<Mesh animate>`) sends once, when the element
//! mounts, and what Rust makes of it. An `Animation` is keyframe tracks on named properties in the element's local
//! frames; `compile` parses it once (curves, colours) and `Compiled::sample(frame)` evaluates every track at a
//! composition frame, a pure function of that frame. `css` and `patch_3d` turn the sample into the style keys or 3D
//! props the page and the scene draw.
//!
//! Properties: `opacity`, `x`, `y`, `z` (a page element's translate offset; a 3D node's position), `scale`, `scaleX`,
//! `scaleY`, `rotate` (degrees on the page), `rotateX`, `rotateY` (3D), `width`, `height` (with `unit`), `clipTop`,
//! `clipRight`, `clipBottom`, `clipLeft` (inset percentages: a wipe), `blur` (px), `text` (a number written as the
//! element's text, `digits` decimals), any colour property (`color`, `backgroundColor`, `borderColor`, `fill`,
//! `stroke`, 3D `color`/`emissive`: keys carry `color`), and any other name as a plain number on that style key or
//! 3D prop. Tracks on one property combine: x, y, z and rotations add, opacity and scales multiply (so an entrance
//! and an exit compose), anything else takes the latest track that has started.

#[cfg(feature = "bevy")]
use bevy::prelude::{ReflectComponent, ReflectDefault};
use serde::{Deserialize, Serialize};

use raea_core_ease::color::Color;
use raea_core_ease::easing::Easing;
use crate::track::{Repeat, Track};

pub use crate::sample::{Sampled, counter_text, num};

/// One element's declared animation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component, bevy::prelude::Reflect), reflect(Component, Default))]
#[serde(default)]
pub struct Animation {
    /// the composition frame the element's local frame 0 falls on (where its Sequence starts)
    pub origin: f64,
    /// a local frame to hold at whatever the composition's frame (inside a Freeze)
    pub freeze: Option<f64>,
    /// the frames per second its frames count; springs measure their settle time with it (0: the clock's)
    pub fps: f64,
    pub tracks: Vec<AnimTrack>,
    /// the local frame it's shown from, when it shows only for a window (mount="during"); before it, display none
    pub show_from: Option<f64>,
    /// the local frame it's hidden from again
    pub show_until: Option<f64>,
}

/// Keyframes on one property.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect), reflect(Default))]
#[serde(default)]
pub struct AnimTrack {
    pub prop: String,
    pub keys: Vec<AnimKey>,
    /// a length's unit: "px" (when empty) or "%"
    pub unit: String,
    /// past the last key: "" or "hold", "loop", "pingpong" (for ever), or "loop:N" / "pingpong:N"
    pub repeat: String,
    /// decimals a `text` number is written with
    pub digits: u32,
    /// written before and after a `text` number (a counter's "$" and " pts")
    pub prefix: String,
    pub suffix: String,
    /// between a `text` number's thousands ("," writes 12,345); empty: none
    pub separator: String,
}

/// One key: a value at a local frame, and the curve from it to the next key.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect), reflect(Default))]
#[serde(default)]
pub struct AnimKey {
    pub at: f64,
    pub value: f64,
    /// a colour property's value (CSS text); empty for numbers
    pub color: String,
    /// the curve to the next key (easing::Easing::parse); empty is linear
    pub ease: String,
}

#[derive(Clone, Debug, PartialEq)]
enum Prop {
    Opacity,
    X,
    Y,
    Z,
    Scale,
    ScaleX,
    ScaleY,
    Rotate,
    RotateX,
    RotateY,
    Width,
    Height,
    Clip(usize),
    Blur,
    Text,
    Color(String),
    Number(String),
}

const COLOR_PROPS: &[&str] = &["color", "backgroundColor", "borderColor", "fill", "stroke", "outlineColor", "emissive", "textDecorationColor"];

impl Prop {
    fn of(name: &str, has_color: bool) -> Prop {
        match name {
            "opacity" => Prop::Opacity,
            "x" => Prop::X,
            "y" => Prop::Y,
            "z" => Prop::Z,
            "scale" => Prop::Scale,
            "scaleX" => Prop::ScaleX,
            "scaleY" => Prop::ScaleY,
            "rotate" | "rotateZ" => Prop::Rotate,
            "rotateX" => Prop::RotateX,
            "rotateY" => Prop::RotateY,
            "width" => Prop::Width,
            "height" => Prop::Height,
            "clipTop" => Prop::Clip(0),
            "clipRight" => Prop::Clip(1),
            "clipBottom" => Prop::Clip(2),
            "clipLeft" => Prop::Clip(3),
            "blur" => Prop::Blur,
            "text" => Prop::Text,
            n if has_color || COLOR_PROPS.contains(&n) => Prop::Color(n.to_string()),
            n => Prop::Number(n.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ColorKey {
    Parsed(Color),
    /// anything Rust can't read (a CSS variable): blended by the page's own color-mix()
    Raw(String),
}

#[derive(Clone, Debug, PartialEq)]
enum Values {
    Num(Track<f64>),
    Color(Track<ColorKey>),
}

#[derive(Clone, Debug, PartialEq)]
struct CTrack {
    prop: Prop,
    values: Values,
    unit: String,
    digits: u32,
    /// a counter's prefix, suffix and thousands separator
    prefix: String,
    suffix: String,
    separator: String,
    start: f64,
    /// the first track on the same property (override properties pick one of them each frame)
    group: usize,
}

/// An animation ready to sample.
#[derive(Clone, Debug, PartialEq)]
pub struct Compiled {
    pub origin: f64,
    pub freeze: Option<f64>,
    pub show_from: Option<f64>,
    pub show_until: Option<f64>,
    tracks: Vec<CTrack>,
    /// when its last key lands (local frames), repeats aside
    pub end: f64,
}

fn repeat_of(s: &str) -> Result<Repeat, String> {
    let (k, n) = match s.split_once(':') {
        Some((k, n)) => (k, Some(n.trim().parse::<u32>().map_err(|_| format!("repeat {s:?}: {n:?} isn't a count"))?)),
        None => (s, None),
    };
    Ok(match k.trim() {
        "" | "hold" | "once" => Repeat::Once,
        "loop" => Repeat::Loop(n),
        "pingpong" => Repeat::PingPong(n),
        other => return Err(format!("no repeat named {other:?} (hold, loop, pingpong)")),
    })
}

/// The animation parsed once: its curves and colours. An error names the track and what's wrong with it.
pub fn compile(a: &Animation) -> Result<Compiled, String> {
    let fps = if a.fps > 0.0 { a.fps } else { 30.0 };
    let mut tracks = Vec::with_capacity(a.tracks.len());
    let mut end: f64 = 0.0;
    for t in &a.tracks {
        if t.keys.is_empty() {
            continue;
        }
        let has_color = t.keys.iter().any(|k| !k.color.is_empty());
        let prop = Prop::of(&t.prop, has_color);
        let repeat = repeat_of(&t.repeat).map_err(|e| format!("{}: {e}", t.prop))?;
        let mut eases = Vec::with_capacity(t.keys.len());
        for k in &t.keys {
            if !k.at.is_finite() {
                return Err(format!("{}: a key's frame isn't a finite number", t.prop));
            }
            eases.push(Easing::parse(&k.ease, fps).map_err(|e| format!("{}: {e}", t.prop))?);
        }
        let values = if matches!(prop, Prop::Color(_)) {
            let mut tr = Track::new().repeat(repeat);
            for (k, e) in t.keys.iter().zip(eases) {
                let c = Color::parse(&k.color).map(ColorKey::Parsed).unwrap_or_else(|| ColorKey::Raw(k.color.clone()));
                tr = tr.key_eased(k.at, c, e);
            }
            Values::Color(tr)
        } else {
            let mut tr = Track::new().repeat(repeat);
            for (k, e) in t.keys.iter().zip(eases) {
                if !k.value.is_finite() {
                    return Err(format!("{}: a key's value isn't a finite number", t.prop));
                }
                tr = tr.key_eased(k.at, k.value, e);
            }
            Values::Num(tr)
        };
        let start = t.keys.iter().map(|k| k.at).fold(f64::INFINITY, f64::min);
        end = end.max(t.keys.iter().map(|k| k.at).fold(f64::NEG_INFINITY, f64::max));
        let group = tracks.iter().position(|o: &CTrack| o.prop == prop).unwrap_or(tracks.len());
        tracks.push(CTrack { prop, values, unit: if t.unit.is_empty() { "px".into() } else { t.unit.clone() }, digits: t.digits, prefix: t.prefix.clone(), suffix: t.suffix.clone(), separator: t.separator.clone(), start, group });
    }
    Ok(Compiled { origin: a.origin, freeze: a.freeze, show_from: a.show_from, show_until: a.show_until, tracks, end })
}

impl Compiled {
    /// The element's local frame at composition frame `frame`.
    pub fn local(&self, frame: f64) -> f64 {
        self.freeze.unwrap_or(frame - self.origin)
    }

    /// Every track at composition frame `frame`.
    pub fn sample(&self, frame: f64) -> Sampled {
        let t = self.local(frame);
        let mut s = Sampled { hidden: self.show_from.is_some_and(|a| t < a) || self.show_until.is_some_and(|b| t >= b), ..Default::default() };
        // override properties: the latest track that has started (else the earliest) on that property
        let wins = |i: usize| {
            let g = self.tracks[i].group;
            let mut best = i;
            for (j, o) in self.tracks.iter().enumerate() {
                if o.group != g || j == best {
                    continue;
                }
                let cur = &self.tracks[best];
                let better = if o.start <= t { cur.start > t || o.start >= cur.start } else { cur.start > t && o.start < cur.start };
                if better {
                    best = j;
                }
            }
            best == i
        };
        for (i, tr) in self.tracks.iter().enumerate() {
            match &tr.values {
                Values::Num(track) => {
                    let Some(v) = track.value_at(t) else { continue };
                    let add = |o: &mut Option<f64>| *o = Some(o.unwrap_or(0.0) + v);
                    let mul = |o: &mut Option<f64>| *o = Some(o.unwrap_or(1.0) * v);
                    match &tr.prop {
                        Prop::Opacity => mul(&mut s.opacity),
                        Prop::X => s.x = Some((s.x.as_ref().map_or(0.0, |x| x.0) + v, tr.unit.clone())),
                        Prop::Y => s.y = Some((s.y.as_ref().map_or(0.0, |y| y.0) + v, tr.unit.clone())),
                        Prop::Z => add(&mut s.z),
                        Prop::Scale => {
                            mul(&mut s.scale_x);
                            mul(&mut s.scale_y);
                        }
                        Prop::ScaleX => mul(&mut s.scale_x),
                        Prop::ScaleY => mul(&mut s.scale_y),
                        Prop::Rotate => add(&mut s.rotate),
                        Prop::RotateX => add(&mut s.rotate_x),
                        Prop::RotateY => add(&mut s.rotate_y),
                        _ if !wins(i) => {}
                        Prop::Width => s.width = Some((v, tr.unit.clone())),
                        Prop::Height => s.height = Some((v, tr.unit.clone())),
                        Prop::Clip(k) => s.clip[*k] = Some(v),
                        Prop::Blur => s.blur = Some(v.max(0.0)),
                        Prop::Text => s.text = Some(counter_text(v, tr.digits, &tr.prefix, &tr.suffix, &tr.separator)),
                        Prop::Number(n) => s.numbers.push((n.clone(), v)),
                        Prop::Color(_) => {}
                    }
                }
                Values::Color(track) => {
                    if !wins(i) {
                        continue;
                    }
                    let Prop::Color(name) = &tr.prop else { continue };
                    let Some((k, p)) = track.locate(t) else { continue };
                    let keys = track.keys();
                    let a = &keys[k].value;
                    let (css, hex) = if p == 0.0 || k + 1 >= keys.len() {
                        match a {
                            ColorKey::Parsed(c) => (c.css(false), Some(c.css(true))),
                            ColorKey::Raw(r) => (r.clone(), None),
                        }
                    } else {
                        match (a, &keys[k + 1].value) {
                            (ColorKey::Parsed(x), ColorKey::Parsed(y)) => {
                                let c = x.mix(y, p);
                                (c.css(false), Some(c.css(true)))
                            }
                            (x, y) => {
                                let txt = |c: &ColorKey| match c {
                                    ColorKey::Parsed(c) => c.css(false),
                                    ColorKey::Raw(r) => r.clone(),
                                };
                                (format!("color-mix(in oklab, {} {}%, {})", txt(x), num((1.0 - p) * 100.0), txt(y)), None)
                            }
                        }
                    };
                    s.colors.push((name.clone(), css, hex));
                }
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value};

    fn key(at: f64, value: f64, ease: &str) -> AnimKey {
        AnimKey { at, value, color: String::new(), ease: ease.into() }
    }
    fn track(prop: &str, keys: Vec<AnimKey>) -> AnimTrack {
        AnimTrack { prop: prop.into(), keys, ..Default::default() }
    }

    #[test]
    fn a_rise_sampled_at_composition_frames() {
        let a = Animation { origin: 100.0, fps: 30.0, tracks: vec![track("y", vec![key(0.0, 30.0, "out-cubic"), key(6.0, 0.0, "")]), track("opacity", vec![key(0.0, 0.0, "out-cubic"), key(5.0, 1.0, "")])], ..Default::default() };
        let c = compile(&a).unwrap();
        let css = c.sample(100.0).css();
        assert_eq!(css["translate"], "0px 30px");
        assert_eq!(css["opacity"], 0.0);
        let css = c.sample(106.0).css();
        assert_eq!(css["translate"], "0px 0px");
        assert_eq!(css["opacity"], 1.0);
        // between keys, out-cubic: at frame 3 of 6, 30 * (1 - 0.875)
        assert_eq!(c.sample(103.0).css()["translate"], "0px 3.75px");
    }

    #[test]
    fn enter_and_exit_compose() {
        let a = Animation {
            fps: 30.0,
            tracks: vec![
                track("opacity", vec![key(0.0, 0.0, "out-cubic"), key(8.0, 1.0, "")]),
                track("opacity", vec![key(40.0, 1.0, "in-cubic"), key(48.0, 0.0, "")]),
                track("x", vec![key(0.0, -64.0, "out-cubic"), key(8.0, 0.0, "")]),
                track("x", vec![key(40.0, 0.0, "in-cubic"), key(48.0, -64.0, "")]),
            ],
            ..Default::default()
        };
        let c = compile(&a).unwrap();
        assert_eq!(c.sample(20.0).css()["opacity"], 1.0);
        assert_eq!(c.sample(48.0).css()["opacity"], 0.0);
        assert_eq!(c.sample(20.0).css()["translate"], "0px 0px");
        assert_eq!(c.sample(48.0).css()["translate"], "-64px 0px");
    }

    #[test]
    fn colours_blend_or_fall_back_to_color_mix() {
        let col = |at: f64, c: &str| AnimKey { at, value: 0.0, color: c.into(), ease: String::new() };
        let a = Animation { fps: 30.0, tracks: vec![AnimTrack { prop: "backgroundColor".into(), keys: vec![col(0.0, "#000000"), col(10.0, "#ffffff")], ..Default::default() }, AnimTrack { prop: "color".into(), keys: vec![col(0.0, "var(--a)"), col(10.0, "#ffffff")], ..Default::default() }], ..Default::default() };
        let c = compile(&a).unwrap();
        let s = c.sample(0.0).css();
        assert_eq!(s["backgroundColor"], "rgba(0, 0, 0, 1)");
        assert_eq!(s["color"], "var(--a)");
        let s = c.sample(5.0).css();
        assert!(s["color"].as_str().unwrap().starts_with("color-mix(in oklab, var(--a) 50%"));
        assert_eq!(c.sample(10.0).css()["backgroundColor"], "rgba(255, 255, 255, 1)");
    }

    #[test]
    fn text_counts_and_wipes_clip() {
        let a = Animation { fps: 30.0, tracks: vec![AnimTrack { prop: "text".into(), keys: vec![key(0.0, 0.0, "out-cubic"), key(24.0, 20.0, "")], ..Default::default() }, track("clipRight", vec![key(0.0, 100.0, "in-out-cubic"), key(8.0, 0.0, "")])], ..Default::default() };
        let c = compile(&a).unwrap();
        assert_eq!(c.sample(0.0).text.as_deref(), Some("0"));
        assert_eq!(c.sample(24.0).text.as_deref(), Some("20"));
        assert_eq!(counter_text(1234567.891, 2, "$", " pts", ","), "$1,234,567.89 pts");
        assert_eq!(counter_text(-1234.0, 0, "", "", ","), "-1,234");
        assert_eq!(counter_text(-0.001, 2, "", "x", ","), "0.00x");
        assert_eq!(counter_text(999.0, 0, "", "", ","), "999");
        assert_eq!(c.sample(4.0).css()["clipPath"], "inset(0% 50% 0% 0%)");
    }

    #[test]
    fn freeze_holds_and_3d_patches() {
        let a = Animation { origin: 10.0, freeze: Some(3.0), fps: 30.0, tracks: vec![track("x", vec![key(0.0, 0.0, ""), key(6.0, 6.0, "")]), track("scale", vec![key(0.0, 0.5, ""), key(6.0, 1.0, "")])], ..Default::default() };
        let c = compile(&a).unwrap();
        assert_eq!(c.sample(500.0).x.unwrap().0, 3.0);
        let mut p: Map<String, Value> = serde_json::from_str(r#"{"position":[1,2,3],"scale":1}"#).unwrap();
        assert!(c.sample(0.0).patch_3d(&mut p));
        assert_eq!(p["position"], serde_json::json!([3.0, 2.0, 3.0]));
        assert_eq!(p["scale"], serde_json::json!(0.75));
    }

    #[test]
    fn a_show_window_hides_outside_it() {
        let a = Animation { origin: 100.0, fps: 30.0, show_from: Some(5.0), show_until: Some(20.0), tracks: vec![track("opacity", vec![key(5.0, 0.0, ""), key(10.0, 1.0, "")])], ..Default::default() };
        let c = compile(&a).unwrap();
        assert!(c.sample(104.0).hidden && !c.sample(105.0).hidden && !c.sample(119.0).hidden && c.sample(120.0).hidden);
    }

    #[test]
    fn bad_curves_are_named() {
        let a = Animation { fps: 30.0, tracks: vec![track("y", vec![key(0.0, 1.0, "outCubic"), key(1.0, 0.0, "")])], ..Default::default() };
        assert!(compile(&a).unwrap_err().starts_with("y: no easing named"));
    }

    #[test]
    fn spring_keys_overshoot_and_land() {
        let a = Animation { fps: 30.0, tracks: vec![track("scale", vec![key(0.0, 0.6, "spring(1,10,100)"), key(30.0, 1.0, "")])], ..Default::default() };
        let c = compile(&a).unwrap();
        let peak = (0..30).map(|f| c.sample(f as f64).scale_x.unwrap()).fold(0.0, f64::max);
        assert!(peak > 1.0, "peak {peak}");
        assert_eq!(c.sample(30.0).scale_x, Some(1.0));
    }
}
