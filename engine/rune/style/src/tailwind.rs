//! Tailwind classes as style props (R2c): `className`'s utilities resolved with Tailwind's own theme (v4.3.3's
//! theme.css, MIT, in assets/) into the React style props css.rs reads, so a utility maps exactly as far as the prop
//! it sets does. Variants (`hover:`, `md:`, `dark:` and the rest variants.rs maps) hold or not for the element's state
//! (T3b). A utility this doesn't know (an unmapped variant, an important `!`, a negative, a name outside the table)
//! comes back in `unknown` to be reported, never dropped.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::{Map, Value, json};

use crate::rings;
use crate::variants::{self, State};

const THEME: &str = include_str!("../assets/tailwind-theme.css");

/// theme.css's custom properties, by name without the leading `--`.
fn theme() -> &'static HashMap<String, String> {
    static T: OnceLock<HashMap<String, String>> = OnceLock::new();
    T.get_or_init(|| {
        let mut m = HashMap::new();
        for decl in THEME.split(';') {
            let Some(i) = decl.find("--") else { continue };
            let Some((k, v)) = decl[i + 2..].split_once(':') else { continue };
            let v = v.split_whitespace().collect::<Vec<_>>().join(" ");
            m.insert(k.trim().to_string(), v);
        }
        m
    })
}

pub(crate) fn var(name: &str) -> Option<&'static str> {
    theme().get(name).map(String::as_str)
}

/// A theme length in px: `0.25rem`, `2px`, `0`, `calc(1.25 / 0.875)` (a plain number).
pub(crate) fn px(v: &str) -> Option<f64> {
    let v = v.trim();
    if let Some(r) = v.strip_suffix("rem") {
        return r.trim().parse::<f64>().ok().map(|x| x * 16.0);
    }
    if let Some(p) = v.strip_suffix("px") {
        return p.trim().parse().ok();
    }
    if let Some(body) = v.strip_prefix("calc(").and_then(|b| b.strip_suffix(')')) {
        let (a, b) = body.split_once('/')?;
        return Some(a.trim().parse::<f64>().ok()? / b.trim().parse::<f64>().ok()?);
    }
    v.parse().ok()
}

/// A spacing step (`4`, `0.5`, `px`), a fraction (`1/2`, as a %), or an arbitrary value (`[13px]`).
fn spacing(s: &str) -> Option<Value> {
    if let Some(a) = s.strip_prefix('[').and_then(|a| a.strip_suffix(']')) {
        return Some(match a.strip_suffix("px").and_then(|p| p.parse::<f64>().ok()) {
            Some(p) => json!(p),
            None => json!(a.replace('_', " ")),
        });
    }
    if s == "px" {
        return Some(json!(1.0));
    }
    if let Some((a, b)) = s.split_once('/') {
        let (a, b) = (a.parse::<f64>().ok()?, b.parse::<f64>().ok()?);
        return Some(json!(format!("{}%", a / b * 100.0)));
    }
    let n: f64 = s.parse().ok()?;
    Some(json!(n * px(var("spacing")?)?))
}

/// A size utility's value on an axis (`w` or `h`): spacing, full, auto, or the viewport's (`screen`, `dvh`, ...).
fn size_on(axis: &str, s: &str) -> Option<Value> {
    let unit = if axis == "w" { "vw" } else { "vh" };
    match s {
        "screen" => Some(json!(format!("100{unit}"))),
        "dvh" | "svh" | "lvh" | "dvw" | "svw" | "lvw" if s.ends_with(&unit[1..]) => Some(json!(format!("100{unit}"))),
        _ => size(s),
    }
}

/// A size utility's value: spacing, or full/auto.
fn size(s: &str) -> Option<Value> {
    match s {
        "full" => Some(json!("100%")),
        "auto" => Some(json!("auto")),
        _ => spacing(s),
    }
}

/// A colour (`blue-500`, `white`, `blue-500/50`, `[#123456]`) as CSS text css.rs reads.
fn colour(s: &str) -> Option<String> {
    let (name, alpha) = match s.split_once('/') {
        Some((n, a)) => (n, Some(a.parse::<f64>().ok()? / 100.0)),
        None => (s, None),
    };
    let base = match name {
        "transparent" => "transparent".to_string(),
        _ if name.starts_with('[') => name.trim_start_matches('[').trim_end_matches(']').to_string(),
        _ => var(&format!("color-{name}"))?.to_string(),
    };
    let Some(a) = alpha else { return Some(base) };
    if let Some(body) = base.strip_prefix("oklch(").and_then(|b| b.strip_suffix(')')) {
        return Some(format!("oklch({body} / {a})"));
    }
    let c = rune::color::hex(&base)?;
    let e = |x: f32| (rune::color::srgb_encode(x) * 255.0).round();
    Some(format!("rgb({} {} {} / {a})", e(c.r), e(c.g), e(c.b)))
}

/// `className` as style props for an element in state `s` (variants.rs; `State::default()` is light at no width, where
/// no variant holds): the plain utilities, then those whose variants
/// hold in Tailwind's order, a later one winning; and the utilities, variants included, that it couldn't place. A
/// variant's utility that doesn't hold now is still checked, so what can't map is reported in any state.
pub fn style_in(class: &str, s: &State) -> (Map<String, Value>, Vec<String>) {
    let mut hits: Vec<(u64, Props)> = Vec::new();
    let mut unknown = Vec::new();
    for u in class.split_whitespace() {
        match variants::split(u, s).and_then(|(util, key, holds)| Some((utility(util)?, key, holds))) {
            Some((props, key, true)) => hits.push((key, props)),
            Some(_) => {}
            None => unknown.push(u.to_string()),
        }
    }
    // stable: within one place in the cascade, the class's own order
    hits.sort_by_key(|h| h.0);
    let mut out = Map::new();
    for (k, v) in hits.into_iter().flat_map(|h| h.1) {
        out.insert(k.to_string(), v);
    }
    crate::rings::compose(&mut out);
    (out, unknown)
}

type Props = Vec<(&'static str, Value)>;

fn one(k: &'static str, v: Value) -> Option<Props> {
    Some(vec![(k, v)])
}

fn utility(u: &str) -> Option<Props> {
    if u.contains(':') || u.starts_with('!') || u.starts_with('-') {
        return None;
    }
    let s = |v: &str| json!(v);
    match u {
        "flex" | "inline-flex" => return one("display", s("flex")),
        "block" => return one("display", s("block")),
        "inline" | "inline-block" => return one("display", s("inline")),
        "hidden" => return one("display", s("none")),
        "flex-row" => return one("flexDirection", s("row")),
        "flex-col" => return one("flexDirection", s("column")),
        "flex-wrap" => return one("flexWrap", s("wrap")),
        "flex-nowrap" => return one("flexWrap", s("nowrap")),
        "flex-1" => return Some(vec![("flexGrow", json!(1)), ("flexShrink", json!(1)), ("flexBasis", s("0%"))]),
        "flex-auto" => return Some(vec![("flexGrow", json!(1)), ("flexShrink", json!(1)), ("flexBasis", s("auto"))]),
        "flex-none" => return Some(vec![("flexGrow", json!(0)), ("flexShrink", json!(0)), ("flexBasis", s("auto"))]),
        "grow" => return one("flexGrow", json!(1)),
        "grow-0" => return one("flexGrow", json!(0)),
        "shrink" => return one("flexShrink", json!(1)),
        "shrink-0" => return one("flexShrink", json!(0)),
        "absolute" | "relative" | "static" => return one("position", s(u)),
        "italic" => return one("fontStyle", s("italic")),
        "object-contain" | "object-cover" | "object-fill" => return one("objectFit", s(&u[7..])),
        "not-italic" => return one("fontStyle", s("normal")),
        "text-left" | "text-center" | "text-right" => return one("textAlign", s(&u[5..])),
        "border" => return one("borderWidth", json!(1)),
        "rounded" => return one("borderRadius", json!(px(var("radius")?)?)),
        "rounded-none" => return one("borderRadius", json!(0)),
        "rounded-full" => return one("borderRadius", json!(9999)),
        "shadow" => return one(rings::SHADOW, s(var("shadow")?)),
        "shadow-none" => return one(rings::SHADOW, s("none")),
        "ring" => return rings::utility(None, colour),
        "outline" => return Some(vec![("outlineStyle", s("solid")), ("outlineWidth", json!(1))]),
        "outline-none" => return one("outlineStyle", s("none")),
        // transitions (T3c): the property list Tailwind 4.3.3 writes, its default duration and curve under any
        // duration-* or ease-* (motion.rs reads --tw-transition first)
        "transition" => return transition("color, background-color, border-color, outline-color, opacity, box-shadow, transform, translate, scale, rotate"),
        "transition-all" => return transition("all"),
        "transition-colors" => return transition("color, background-color, border-color, outline-color, fill, stroke"),
        "transition-opacity" => return transition("opacity"),
        "transition-shadow" => return transition("box-shadow"),
        "transition-transform" => return transition("transform, translate, scale, rotate"),
        "transition-none" => return one("transitionProperty", s("none")),
        "ease-linear" => return one("transitionTimingFunction", s("linear")),
        "ease-in" => return one("transitionTimingFunction", s(var("ease-in")?)),
        "ease-out" => return one("transitionTimingFunction", s(var("ease-out")?)),
        "ease-in-out" => return one("transitionTimingFunction", s(var("ease-in-out")?)),
        "origin-center" => return one("transformOrigin", s("center")),
        "origin-top" | "origin-bottom" | "origin-left" | "origin-right" => return one("transformOrigin", s(&u[7..])),
        "origin-top-left" | "origin-top-right" | "origin-bottom-left" | "origin-bottom-right" => return one("transformOrigin", s(&u[7..].replace('-', " "))),
        "inset-0" => return Some(["top", "right", "bottom", "left"].map(|k| (k, json!(0))).to_vec()),
        _ => {}
    }
    let (head, rest) = u.split_once('-')?;
    let pad = |keys: &[&'static str]| -> Option<Props> {
        let v = spacing(rest.split_once('-').map_or(rest, |(_, r)| r))?;
        Some(keys.iter().map(|k| (*k, v.clone())).collect())
    };
    match head {
        "p" => one("padding", spacing(rest)?),
        "px" => one_two("paddingLeft", "paddingRight", spacing(rest)?),
        "py" => one_two("paddingTop", "paddingBottom", spacing(rest)?),
        "pt" => one("paddingTop", spacing(rest)?),
        "pr" => one("paddingRight", spacing(rest)?),
        "pb" => one("paddingBottom", spacing(rest)?),
        "pl" => one("paddingLeft", spacing(rest)?),
        "m" => one("margin", size(rest)?),
        "mx" => one_two("marginLeft", "marginRight", size(rest)?),
        "my" => one_two("marginTop", "marginBottom", size(rest)?),
        "mt" => one("marginTop", size(rest)?),
        "mr" => one("marginRight", size(rest)?),
        "mb" => one("marginBottom", size(rest)?),
        "ml" => one("marginLeft", size(rest)?),
        "gap" => match rest.split_once('-') {
            Some(("x", _)) => pad(&["columnGap"]),
            Some(("y", _)) => pad(&["rowGap"]),
            _ => one("gap", spacing(rest)?),
        },
        "w" => one("width", size_on("w", rest)?),
        "h" => one("height", size_on("h", rest)?),
        "size" => one_two("width", "height", size(rest)?),
        "min" | "max" => {
            let (axis, v) = rest.split_once('-')?;
            let k = match (head, axis) {
                ("min", "w") => "minWidth",
                ("min", "h") => "minHeight",
                ("max", "w") => "maxWidth",
                ("max", "h") => "maxHeight",
                _ => return None,
            };
            let v = match var(&format!("container-{v}")) {
                Some(c) if axis == "w" => json!(px(c)?),
                _ => size_on(axis, v)?,
            };
            one(k, v)
        }
        "top" | "right" | "bottom" | "left" => one(["top", "right", "bottom", "left"].into_iter().find(|k| *k == head)?, spacing(rest)?),
        "inset" => match rest.split_once('-') {
            Some(("x", _)) => pad(&["left", "right"]),
            Some(("y", _)) => pad(&["top", "bottom"]),
            _ => Some(["top", "right", "bottom", "left"].map(|k| (k, spacing(rest).unwrap_or(json!(0)))).to_vec()),
        },
        "items" => one("alignItems", s(flexword(rest)?)),
        "self" => one("alignSelf", s(flexword(rest)?)),
        "justify" => one("justifyContent", s(flexword(rest)?)),
        "basis" => one("flexBasis", size(rest)?),
        "overflow" => match rest.split_once('-') {
            Some(("x", v)) => one("overflowX", s(v)),
            Some(("y", v)) => one("overflowY", s(v)),
            _ => one("overflow", s(rest)),
        },
        "opacity" => one("opacity", json!(rest.parse::<f64>().ok()? / 100.0)),
        "bg" => one("backgroundColor", s(&colour(rest)?)),
        "text" => match var(&format!("text-{rest}")) {
            Some(sz) => {
                let lh = var(&format!("text-{rest}--line-height")).and_then(px)?;
                Some(vec![("fontSize", json!(px(sz)?)), ("lineHeight", json!(lh))])
            }
            None => one("color", s(&colour(rest)?)),
        },
        "font" => match var(&format!("font-weight-{rest}")) {
            Some(w) => one("fontWeight", json!(w.parse::<f64>().ok()?)),
            None => one("fontFamily", s(match rest {
                "sans" => "Inter",
                "mono" => "monospace",
                _ => return None,
            })),
        },
        "leading" => one("lineHeight", match var(&format!("leading-{rest}")) {
            Some(l) => json!(l.parse::<f64>().ok()?),
            // a spacing step is a length (leading-6 is 24px), never a bare number, which lineHeight reads as a ratio
            None => match spacing(rest)? {
                Value::Number(n) => json!(format!("{}px", n.as_f64()?)),
                v => v,
            },
        }),
        "rounded" => radius(rest),
        "border" => border(rest),
        "shadow" => one(rings::SHADOW, s(var(&format!("shadow-{rest}"))?)),
        "ring" => rings::utility(Some(rest), colour),
        "duration" => one("transitionDuration", s(&format!("{}ms", rest.parse::<f64>().ok()?))),
        "delay" => one("transitionDelay", s(&format!("{}ms", rest.parse::<f64>().ok()?))),
        "translate" => match rest.split_once('-') {
            Some(("x", v)) => one("--tw-translate-x", spacing(v)?),
            Some(("y", v)) => one("--tw-translate-y", spacing(v)?),
            _ => None,
        },
        // scale-105 is 1.05 (both axes); scale-x-* and scale-y-* differ per axis, which Rune can't
        "scale" => one("scale", json!(rest.parse::<f64>().ok()? / 100.0)),
        "rotate" => one("rotate", s(&format!("{}deg", rest.parse::<f64>().ok()?))),
        _ => None,
    }
}

fn one_two(a: &'static str, b: &'static str, v: Value) -> Option<Props> {
    Some(vec![(a, v.clone()), (b, v)])
}

fn flexword(w: &str) -> Option<&'static str> {
    Some(match w {
        "start" => "flex-start",
        "end" => "flex-end",
        "center" => "center",
        "stretch" => "stretch",
        "between" => "space-between",
        "around" => "space-around",
        "evenly" => "space-evenly",
        _ => return None,
    })
}

/// `rounded-{size}`, `rounded-{t|r|b|l|tl|tr|br|bl}[-{size}]`.
fn radius(rest: &str) -> Option<Props> {
    let size = |s: &str| -> Option<Value> {
        match s {
            "none" => Some(json!(0)),
            "full" => Some(json!(9999)),
            "" => Some(json!(px(var("radius")?)?)),
            _ => Some(json!(px(var(&format!("radius-{s}"))?)?)),
        }
    };
    if let Some(v) = size(rest) {
        return one("borderRadius", v);
    }
    let (side, s) = rest.split_once('-').unwrap_or((rest, ""));
    let corners: &[&'static str] = match side {
        "t" => &["borderTopLeftRadius", "borderTopRightRadius"],
        "r" => &["borderTopRightRadius", "borderBottomRightRadius"],
        "b" => &["borderBottomRightRadius", "borderBottomLeftRadius"],
        "l" => &["borderTopLeftRadius", "borderBottomLeftRadius"],
        "tl" => &["borderTopLeftRadius"],
        "tr" => &["borderTopRightRadius"],
        "br" => &["borderBottomRightRadius"],
        "bl" => &["borderBottomLeftRadius"],
        _ => return None,
    };
    let v = size(s)?;
    Some(corners.iter().map(|k| (*k, v.clone())).collect())
}

/// `border-{n}`, `border-{t|r|b|l|x|y}[-{n}]`, `border-{colour}`.
fn border(rest: &str) -> Option<Props> {
    if let Ok(n) = rest.parse::<f64>() {
        return one("borderWidth", json!(n));
    }
    let (side, n) = rest.split_once('-').unwrap_or((rest, "1"));
    let sides: &[&'static str] = match side {
        "t" => &["borderTopWidth"],
        "r" => &["borderRightWidth"],
        "b" => &["borderBottomWidth"],
        "l" => &["borderLeftWidth"],
        "x" => &["borderLeftWidth", "borderRightWidth"],
        "y" => &["borderTopWidth", "borderBottomWidth"],
        _ => return one("borderColor", json!(colour(rest)?)),
    };
    let w = n.parse::<f64>().ok()?;
    Some(sides.iter().map(|k| (*k, json!(w))).collect())
}

/// A `transition-*` utility: its properties, and Tailwind's default duration and curve under any duration-* or ease-*.
fn transition(props: &str) -> Option<Props> {
    Some(vec![("transitionProperty", json!(props)), ("--tw-transition", json!(true))])
}
