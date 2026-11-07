//! @raea/rune's props (R6a, sdk/rune/src/index.ts): an element's `data-rune` JSON mapped to Rune's builder. A field the
//! host can't read is reported with its element's path, never dropped quietly. Motion, a transition or lead and trail
//! mark the element as animated (view.rs lands every unmarked subtree at once, as the DOM does).

use rune::tok::motion as tok;
use rune::{Edge, El, Spring, SpringPair, SystemMaterial, Transition};
use serde_json::Value;

use crate::report::Report;

/// `e` with `json`'s props; whether it asked for motion.
pub fn apply(mut e: El, json: &str, path: &str, report: &mut Report) -> (El, bool) {
    let Ok(Value::Object(p)) = serde_json::from_str::<Value>(json) else {
        report.note(path, "data-rune", json, "not a JSON object of Rune props");
        return (e, false);
    };
    let mut animated = false;
    let mut bad = |what: &str, v: &Value| report.note(path, &format!("data-rune {what}"), &v.to_string(), "a Rune prop the host can't read");
    for (k, v) in &p {
        match k.as_str() {
            "motion" => match spring(v) {
                Some(s) => (e, animated) = (e.motion(s), true),
                None => bad(k, v),
            },
            "transition" => match transition(v) {
                Some(t) => (e, animated) = (e.transition(t), true),
                None => bad(k, v),
            },
            "lead" => match (spring(&v["lead"]), spring(&v["trail"])) {
                (Some(l), Some(t)) => (e, animated) = (e.lead(l, t), true),
                _ => bad(k, v),
            },
            "instant" => match v.as_bool() {
                Some(true) => e = e.instant_changes(),
                Some(false) => {}
                None => bad(k, v),
            },
            "blur" => match v.as_f64() {
                Some(s) => e = e.blur(s as f32),
                None => bad(k, v),
            },
            "backdropBlur" => match v.as_f64() {
                Some(s) => e = e.backdrop_blur(s as f32),
                None => bad(k, v),
            },
            "glass" => match v.as_bool() {
                Some(true) => e = e.system_material(SystemMaterial::Glass),
                Some(false) => {}
                None => bad(k, v),
            },
            _ => bad(k, v),
        }
    }
    (e, animated)
}

/// A preset's name or `{duration, bounce}`.
pub fn spring(v: &Value) -> Option<Spring> {
    match v {
        Value::String(s) => Some(match s.as_str() {
            "instant" => tok::INSTANT,
            "snap" => tok::SNAP,
            "settle" => tok::SETTLE,
            "reflow" => tok::REFLOW,
            "glide" => tok::GLIDE,
            "morph" => tok::MORPH,
            "pop" => tok::POP,
            "drift" => tok::DRIFT,
            _ => return None,
        }),
        Value::Object(o) => {
            let d = o.get("duration")?.as_f64()? as f32;
            let b = o.get("bounce").and_then(Value::as_f64).unwrap_or(0.0) as f32;
            (d > 0.0 && (0.0..=1.0).contains(&b)).then(|| Spring::new(d, b))
        }
        _ => None,
    }
}

fn edge(v: &Value) -> Option<Edge> {
    Some(match v.as_str()? {
        "leading" => Edge::Leading,
        "trailing" => Edge::Trailing,
        "top" => Edge::Top,
        "bottom" => Edge::Bottom,
        _ => return None,
    })
}

/// An effect, or `{effect, enter?, exit?}`.
fn transition(v: &Value) -> Option<Transition> {
    let (effect, enter, exit) = match v.get("effect") {
        Some(e) => (e, v.get("enter"), v.get("exit")),
        None => (v, None, None),
    };
    let t = match effect {
        Value::String(s) => match s.as_str() {
            "none" => Transition::none(),
            "opacity" => Transition::opacity(),
            "slide" => Transition::slide(),
            "blurReplace" => Transition::blur_replace(),
            _ => return None,
        },
        Value::Object(o) => {
            let (k, x) = o.iter().next()?;
            match k.as_str() {
                "scale" => Transition::scale(x.as_f64()? as f32),
                "offset" => Transition::offset(x.get(0)?.as_f64()? as f32, x.get(1)?.as_f64()? as f32),
                "move" => Transition::move_edge(edge(x)?),
                "push" => Transition::push(edge(x)?),
                "blur" => Transition::blur(x.as_f64()? as f32),
                _ => return None,
            }
        }
        _ => return None,
    };
    if enter.is_none() && exit.is_none() {
        return Some(t);
    }
    // a side left out keeps the token pair's own spring
    let enter = match enter {
        Some(s) => spring(s)?,
        None => tok::SETTLE,
    };
    let exit = match exit {
        Some(s) => spring(s)?,
        None => tok::SETTLE_EXIT,
    };
    Some(t.springs(SpringPair::new(enter, exit)))
}
