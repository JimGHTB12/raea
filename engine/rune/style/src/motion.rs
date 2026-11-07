//! CSS transforms and transitions on Rune (docs/targets.feature T3c). `transform`'s translate, scale and rotate
//! functions, the individual `translate`, `scale` and `rotate` properties (Tailwind 4 writes those), and
//! `transformOrigin` become Rune's offset, scale, rotation and origin, which leave layout alone as CSS's do. A
//! transition becomes the element's curves: opacity and the transforms tween over the transition's duration on its
//! timing function (Rune's `Curve::tween`, CSS's own model) after its delay, and colours, corners and shadows take a
//! Rune spring of that duration (Rune moves those on springs only). Whatever Rune has no equivalent for (skew, matrix,
//! 3D, a scale that differs per axis, a translate after a rotate or scale, a percentage, `steps()`) is reported.

use rune::{AnimExt, Curve, El, Easing, Spring};
use serde_json::Value;

/// What `transform` and its siblings set: an offset in points, a uniform scale, a rotation in radians, the origin as
/// fractions of the box, and the transition.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Motion {
    /// `translate` (x, y), then `transform`'s translation, in points
    pub translate: [f32; 2],
    pub scale: Option<f32>,
    /// radians, clockwise
    pub rotate: Option<f32>,
    pub origin: Option<(f32, f32)>,
    pub transition: Option<Transition>,
    /// Tailwind's translate-x and translate-y (its --tw-translate-x/y), added to `translate`
    pub tw_translate: [f32; 2],
}

/// A CSS transition: which properties, for how long, on what curve, after what delay.
#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    pub props: Props,
    /// seconds
    pub duration: f32,
    /// cubic-bezier control points
    pub easing: [f32; 4],
    /// seconds
    pub delay: f32,
}

/// The groups of properties a transition can name.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Props {
    pub colours: bool,
    pub opacity: bool,
    pub transform: bool,
    pub shadow: bool,
}

/// Tailwind 4.3.3's defaults: `--default-transition-duration` and `--default-transition-timing-function`.
pub const DEFAULT_DURATION: f32 = 0.15;
pub const DEFAULT_EASING: [f32; 4] = [0.4, 0.0, 0.2, 1.0];

use crate::css::{BAD_VALUE as BAD, NO_PROP as NOT_RUNE};

/// One prop into `m`; Err(NOT_RUNE) when it isn't one of these.
pub fn prop(m: &mut Motion, k: &str, v: &Value) -> Result<(), &'static str> {
    let s = || v.as_str().ok_or(BAD);
    match k {
        "transform" => transform(m, s()?),
        "translate" => {
            let p: Vec<&str> = s()?.split_whitespace().collect();
            let x = px(p.first().ok_or(BAD)?)?;
            let y = p.get(1).map_or(Ok(0.0), |y| px(y))?;
            m.translate = [m.translate[0] + x, m.translate[1] + y];
            Ok(())
        }
        "--tw-translate-x" => Ok(m.tw_translate[0] = num_or_px(v)?),
        "--tw-translate-y" => Ok(m.tw_translate[1] = num_or_px(v)?),
        // a Tailwind transition-* utility: Tailwind's defaults, which duration-* and ease-* (read after, by name) override
        "--tw-transition" => {
            let t = tr(m);
            (t.duration, t.easing) = (DEFAULT_DURATION, DEFAULT_EASING);
            Ok(())
        }
        "scale" => {
            let k = match v {
                Value::Number(n) => n.as_f64().ok_or(BAD)? as f32,
                _ => uniform(s()?)?,
            };
            m.scale = Some(m.scale.unwrap_or(1.0) * k);
            Ok(())
        }
        "rotate" => {
            m.rotate = Some(m.rotate.unwrap_or(0.0) + angle(s()?)?);
            Ok(())
        }
        "transformOrigin" => Ok(m.origin = Some(origin(s()?)?)),
        "transition" => {
            // one transition, or several with the same timing (Tailwind's lists): their properties together
            let mut t: Option<Transition> = None;
            for part in s()?.split(',') {
                let words: Vec<&str> = part.split_whitespace().collect();
                let mut one = Transition { props: Props::default(), duration: 0.0, easing: EASE, delay: 0.0 };
                let mut times = 0;
                for w in words {
                    if let Some(d) = seconds(w) {
                        if times == 0 { one.duration = d } else { one.delay = d }
                        times += 1;
                    } else if let Some(e) = easing(w) {
                        one.easing = e;
                    } else {
                        one.props = union(one.props, group(w)?);
                    }
                }
                t = Some(match t {
                    None => one,
                    Some(a) if a.duration == one.duration && a.easing == one.easing && a.delay == one.delay => Transition { props: union(a.props, one.props), ..a },
                    Some(_) => return Err("transitions with different timings on one element aren't mapped"),
                });
            }
            Ok(m.transition = t)
        }
        "transitionProperty" => {
            let mut p = Props::default();
            for w in s()?.split(',') {
                p = union(p, group(w.trim())?);
            }
            Ok(tr(m).props = p)
        }
        "transitionDuration" => Ok(tr(m).duration = seconds(s()?.trim()).ok_or(BAD)?),
        "transitionDelay" => Ok(tr(m).delay = seconds(s()?.trim()).ok_or(BAD)?),
        "transitionTimingFunction" => Ok(tr(m).easing = easing(s()?.trim()).ok_or("only ease, linear, ease-in, ease-out, ease-in-out and cubic-bezier() are mapped")?),
        _ => Err(NOT_RUNE),
    }
}

/// CSS's initial timing function, `ease`.
const EASE: [f32; 4] = [0.25, 0.1, 0.25, 1.0];

fn tr(m: &mut Motion) -> &mut Transition {
    m.transition.get_or_insert(Transition { props: Props { colours: true, opacity: true, transform: true, shadow: true }, duration: 0.0, easing: EASE, delay: 0.0 })
}

fn union(a: Props, b: Props) -> Props {
    Props { colours: a.colours || b.colours, opacity: a.opacity || b.opacity, transform: a.transform || b.transform, shadow: a.shadow || b.shadow }
}

/// A transition-property name as the group it moves.
fn group(w: &str) -> Result<Props, &'static str> {
    let none = Props::default();
    Ok(match w {
        "all" => Props { colours: true, opacity: true, transform: true, shadow: true },
        "none" => none,
        "opacity" => Props { opacity: true, ..none },
        "transform" | "translate" | "scale" | "rotate" => Props { transform: true, ..none },
        "box-shadow" | "boxShadow" => Props { shadow: true, ..none },
        _ if w.ends_with("color") || w.ends_with("Color") || matches!(w, "fill" | "stroke" | "background") || w.starts_with("--tw-gradient") => Props { colours: true, ..none },
        // what a transition names that a Rune element doesn't move (filter, display, visibility...): nothing to animate
        _ if matches!(w, "filter" | "-webkit-backdrop-filter" | "backdrop-filter" | "display" | "visibility" | "content-visibility" | "overlay" | "pointer-events") => none,
        _ => return Err("a transition property the host doesn't animate"),
    })
}

fn seconds(w: &str) -> Option<f32> {
    if let Some(ms) = w.strip_suffix("ms") {
        return ms.parse::<f32>().ok().map(|x| x / 1000.0);
    }
    w.strip_suffix('s')?.parse().ok()
}

fn easing(w: &str) -> Option<[f32; 4]> {
    Some(match w {
        "ease" => EASE,
        "linear" => [0.0, 0.0, 1.0, 1.0],
        "ease-in" => [0.42, 0.0, 1.0, 1.0],
        "ease-out" => [0.0, 0.0, 0.58, 1.0],
        "ease-in-out" => [0.42, 0.0, 0.58, 1.0],
        _ => {
            let body = w.strip_prefix("cubic-bezier(")?.strip_suffix(')')?;
            let n: Vec<f32> = body.split(',').map(|x| x.trim().parse().ok()).collect::<Option<_>>()?;
            <[f32; 4]>::try_from(n).ok()?
        }
    })
}

fn px(w: &str) -> Result<f32, &'static str> {
    let w = w.trim();
    if w == "0" {
        return Ok(0.0);
    }
    w.strip_suffix("px").and_then(|p| p.trim().parse().ok()).ok_or("only px (or 0) translations are mapped")
}

fn num_or_px(v: &Value) -> Result<f32, &'static str> {
    match v {
        Value::Number(n) => n.as_f64().map(|x| x as f32).ok_or(BAD),
        Value::String(s) => px(s),
        _ => Err(BAD),
    }
}

/// A scale: `1.1`, `110%`, or two equal values.
fn uniform(w: &str) -> Result<f32, &'static str> {
    let one = |x: &str| -> Result<f32, &'static str> {
        match x.strip_suffix('%') {
            Some(p) => p.parse::<f32>().map(|p| p / 100.0).map_err(|_| BAD),
            None => x.parse().map_err(|_| BAD),
        }
    };
    let vals = w.split([' ', ',']).filter(|x| !x.is_empty()).map(one).collect::<Result<Vec<_>, _>>()?;
    match vals.as_slice() {
        [k] => Ok(*k),
        [x, y] | [x, y, _] if x == y => Ok(*x),
        _ => Err("a scale that differs per axis has no Rune equivalent"),
    }
}

/// An angle in radians: deg, rad, turn, grad.
fn angle(w: &str) -> Result<f32, &'static str> {
    let w = w.trim();
    let (n, k) = if let Some(n) = w.strip_suffix("deg") {
        (n, std::f32::consts::PI / 180.0)
    } else if let Some(n) = w.strip_suffix("grad") {
        (n, std::f32::consts::PI / 200.0)
    } else if let Some(n) = w.strip_suffix("rad") {
        (n, 1.0)
    } else if let Some(n) = w.strip_suffix("turn") {
        (n, std::f32::consts::TAU)
    } else if w == "0" {
        ("0", 1.0)
    } else {
        return Err(BAD);
    };
    n.trim().parse::<f32>().map(|x| x * k).map_err(|_| BAD)
}

/// `transformOrigin` as fractions of the box: keywords or percentages, x then y.
fn origin(w: &str) -> Result<(f32, f32), &'static str> {
    let mut xy = (0.5, 0.5);
    let words: Vec<&str> = w.split_whitespace().collect();
    for (i, t) in words.iter().enumerate() {
        match *t {
            "left" => xy.0 = 0.0,
            "right" => xy.0 = 1.0,
            "top" => xy.1 = 0.0,
            "bottom" => xy.1 = 1.0,
            "center" => {}
            _ => {
                let f = t.strip_suffix('%').and_then(|p| p.parse::<f32>().ok()).ok_or("only keyword and % origins are mapped")? / 100.0;
                if i == 0 { xy.0 = f } else { xy.1 = f }
            }
        }
    }
    Ok(xy)
}

/// `transform`'s functions, left to right: translations first, then rotations and uniform scales.
fn transform(m: &mut Motion, s: &str) -> Result<(), &'static str> {
    if s.trim() == "none" {
        return Ok(());
    }
    let mut turned = false;
    for f in s.split(')').map(str::trim).filter(|f| !f.is_empty()) {
        let (name, args) = f.split_once('(').ok_or(BAD)?;
        let a: Vec<&str> = args.split(',').map(str::trim).collect();
        let shift = |m: &mut Motion, x: f32, y: f32| -> Result<(), &'static str> {
            // a translation after a rotation or scale moves along the turned axes: Rune's offset doesn't
            if turned && (x != 0.0 || y != 0.0) {
                return Err("a translate after a rotate or scale has no Rune equivalent");
            }
            m.translate = [m.translate[0] + x, m.translate[1] + y];
            Ok(())
        };
        match name.trim() {
            "translate" => shift(m, px(a[0])?, a.get(1).map_or(Ok(0.0), |y| px(y))?)?,
            "translateX" => shift(m, px(a[0])?, 0.0)?,
            "translateY" => shift(m, 0.0, px(a[0])?)?,
            "translate3d" if a.get(2).is_some_and(|z| px(z) == Ok(0.0)) => shift(m, px(a[0])?, px(a[1])?)?,
            "scale" | "scale3d" => {
                m.scale = Some(m.scale.unwrap_or(1.0) * uniform(&a.join(" "))?);
                turned = true;
            }
            "rotate" | "rotateZ" => {
                m.rotate = Some(m.rotate.unwrap_or(0.0) + angle(a[0])?);
                turned = true;
            }
            _ => return Err("skew, matrix, perspective and 3D transforms have no Rune equivalent"),
        }
    }
    Ok(())
}

/// `e` with `m`'s transforms and transition, and whether it animates (a transition with a duration): a subtree with
/// one isn't landed at once.
pub fn apply(mut e: El, m: &Motion) -> (El, bool) {
    let [x, y] = [m.translate[0] + m.tw_translate[0], m.translate[1] + m.tw_translate[1]];
    if x != 0.0 || y != 0.0 {
        e = e.offset(x, y);
    }
    if let Some(k) = m.scale {
        e = e.scale(k);
    }
    if let Some(r) = m.rotate {
        e = e.rotate(r);
    }
    if let Some((ox, oy)) = m.origin {
        e = e.origin(ox, oy);
    }
    let Some(t) = m.transition.as_ref().filter(|t| t.duration > 0.0) else { return (e, false) };
    let [a, b, c, d] = t.easing;
    let tween = Curve::tween(t.duration, Easing::cubic(a, b, c, d)).delay(t.delay);
    if t.props.opacity {
        e = e.opacity_motion(tween);
    }
    if t.props.transform {
        e = e.offset_motion(tween).scale_motion(tween).rotate_motion(tween);
    }
    if t.props.colours || t.props.shadow {
        e = e.motion(Spring::new(t.duration, 0.0));
    }
    (e, true)
}
