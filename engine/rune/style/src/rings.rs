//! Tailwind 4.3.3's box-shadow (T3e): `ring-*`, `ring-offset-*` and `shadow-*` each set a variable, and the element's
//! `box-shadow` is their layers in Tailwind's order (its inset shadow and inset ring, then the ring's offset, the ring,
//! the shadow). A ring is a spread the offset wider (`0 0 0 calc(W + O) colour`) over the offset's own layer
//! (`0 0 0 O colour`, white unless `ring-offset-<colour>`); `ring` alone is 1 px, and its colour is the text's. The
//! variables are taken out once composed: nothing downstream reads them.

use serde_json::{Map, Value, json};

pub(crate) const WIDTH: &str = "--tw-ring-width";
pub(crate) const COLOR: &str = "--tw-ring-color";
pub(crate) const OFFSET: &str = "--tw-ring-offset-width";
pub(crate) const OFFSET_COLOR: &str = "--tw-ring-offset-color";
pub(crate) const SHADOW: &str = "--tw-shadow";

/// A `ring-...` utility's variable: `ring` and `ring-N` its width, `ring-offset-N` and `ring-offset-<colour>` the
/// offset's, `ring-<colour>` its colour.
pub(crate) fn utility(rest: Option<&str>, colour: impl Fn(&str) -> Option<String>) -> Option<Vec<(&'static str, Value)>> {
    let Some(rest) = rest else { return Some(vec![(WIDTH, json!(1))]) };
    let width = |s: &str| s.parse::<f64>().ok().map(Value::from);
    Some(vec![match rest.strip_prefix("offset-") {
        Some(o) => match width(o) {
            Some(w) => (OFFSET, w),
            None => (OFFSET_COLOR, json!(colour(o)?)),
        },
        None => match width(rest) {
            Some(w) => (WIDTH, w),
            None => (COLOR, json!(colour(rest)?)),
        },
    }])
}

/// `out`'s ring and shadow variables as its `boxShadow`, when a ring or a shadow is set.
pub(crate) fn compose(out: &mut Map<String, Value>) {
    let mut take = |k: &str| out.remove(k);
    let (w, c, o, oc, shadow) = (take(WIDTH), take(COLOR), take(OFFSET), take(OFFSET_COLOR), take(SHADOW));
    let mut layers = Vec::new();
    if let Some(w) = w.and_then(|w| w.as_f64()).filter(|w| *w > 0.0) {
        let o = o.and_then(|o| o.as_f64()).unwrap_or(0.0);
        let s = |v: Option<Value>, or: &str| v.and_then(|v| v.as_str().map(str::to_string)).unwrap_or(or.into());
        if o > 0.0 {
            layers.push(format!("0 0 0 {o}px {}", s(oc, "#fff")));
        }
        layers.push(format!("0 0 0 {}px {}", w + o, s(c, "currentcolor")));
    }
    let shadow = shadow.and_then(|s| s.as_str().map(str::to_string));
    match shadow.as_deref() {
        Some("none") | None => {}
        Some(s) => layers.push(s.to_string()),
    }
    if !layers.is_empty() {
        out.insert("boxShadow".into(), json!(layers.join(", ")));
    } else if shadow.is_some() {
        out.insert("boxShadow".into(), json!("none"));
    }
}
