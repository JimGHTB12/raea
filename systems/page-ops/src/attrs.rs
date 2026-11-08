//! Names and values as the DOM wants them: SVG's case-kept attributes, attribute values and inline style CSS.

use super::*;

/// SVG's elements, whose attributes keep their camelCase names (viewBox) and whose namespace is SVG's.
pub(crate) fn in_svg(kind: &str) -> bool {
    matches!(
        kind,
        "svg" | "path" | "g" | "circle" | "ellipse" | "line" | "polyline" | "polygon" | "rect" | "defs" | "linearGradient" | "radialGradient"
            | "stop" | "clipPath" | "mask" | "use" | "symbol" | "tspan" | "textPath" | "pattern" | "filter" | "marker"
            | "desc" | "feGaussianBlur" | "feOffset" | "feBlend" | "feColorMatrix" | "feMerge" | "feMergeNode" | "feFlood"
            | "feComposite" | "feDropShadow" | "feComponentTransfer" | "feFuncR" | "feFuncG" | "feFuncB" | "feFuncA" | "feMorphology"
            | "feTurbulence" | "feDisplacementMap" | "feImage" | "feTile" | "feConvolveMatrix" | "feDiffuseLighting"
            | "feSpecularLighting" | "fePointLight" | "feSpotLight" | "feDistantLight" | "foreignObject"
    )
}

/// a multiline="shift" field's mark: its Enter without Shift breaks no line (host.rs)
pub const ENTER_IS_KEY: &str = "data-raea-enter-is-key";

pub(crate) fn name(kind: &str, svg: bool) -> QualName {
    let ns = if svg { ns!(svg) } else { ns!(html) };
    QualName { prefix: None, ns, local: LocalName::from(kind) }
}

pub fn qual(attr: &str) -> QualName {
    QualName { prefix: None, ns: Namespace::from(""), local: LocalName::from(attr) }
}

pub(crate) fn attrs(props: &Map<String, Value>, svg: bool) -> Vec<Attribute> {
    props
        .iter()
        .filter_map(|(k, v)| Some(Attribute { name: qual(&attr_name(k, svg)?), value: attr_value(k, v)? }))
        .collect()
}

/// The attribute a prop sets, as React names it in the DOM; None for props that aren't attributes (handlers, children,
/// keys, refs, objects).
pub(crate) fn attr_name(prop: &str, svg: bool) -> Option<String> {
    if matches!(prop, "children" | "key" | "ref" | "dangerouslySetInnerHTML" | "suppressHydrationWarning" | "multiline" | "scrollTop" | "scrollLeft" | "scrollEnd" | "$ns" | "$root")
        || (prop.len() > 2 && prop.starts_with("on") && prop.as_bytes()[2].is_ascii_uppercase())
    {
        return None;
    }
    Some(match prop {
        "className" => "class".into(),
        "htmlFor" => "for".into(),
        "tabIndex" => "tabindex".into(),
        "readOnly" => "readonly".into(),
        "autoFocus" => "autofocus".into(),
        // (an uncontrolled checkbox's or radio's first state, as React DOM sets it)
        "defaultChecked" => "checked".into(),
        "colSpan" => "colspan".into(),
        "rowSpan" => "rowspan".into(),
        "srcSet" => "srcset".into(),
        "crossOrigin" => "crossorigin".into(),
        "xlinkHref" => "href".into(),
        // SVG's presentation attributes are kebab-case; its element attributes (viewBox, gradientUnits) aren't
        p if svg && SVG_KEBAB.contains(&p) => kebab(p),
        p if svg => p.into(),
        // an HTML element's attribute names are lowercase, as setAttribute makes them (maxLength, spellCheck)
        p => p.to_ascii_lowercase(),
    })
}

pub(crate) const SVG_KEBAB: &[&str] = &[
    "strokeWidth", "strokeLinecap", "strokeLinejoin", "strokeDasharray", "strokeDashoffset", "strokeMiterlimit", "strokeOpacity",
    "fillOpacity", "fillRule", "clipRule", "clipPath", "stopColor", "stopOpacity", "fontSize", "fontFamily", "fontWeight", "textAnchor",
    "dominantBaseline", "alignmentBaseline", "letterSpacing", "vectorEffect", "colorInterpolationFilters", "floodColor", "floodOpacity",
    "markerEnd", "markerStart", "markerMid", "paintOrder", "shapeRendering", "pointerEvents",
];

/// A prop's value as its attribute's text; None for no attribute (false, null, a function or an object).
pub(crate) fn attr_value(prop: &str, v: &Value) -> Option<String> {
    match (prop, v) {
        ("style", Value::Object(o)) => Some(style_css(o)),
        (_, Value::String(s)) => Some(s.clone()),
        (_, Value::Number(n)) => Some(number(n.as_f64().unwrap_or(0.0))),
        (_, Value::Bool(true)) => Some(if prop.starts_with("aria-") || prop.starts_with("data-") { "true".into() } else { String::new() }),
        (_, Value::Bool(false)) if prop.starts_with("aria-") || prop.starts_with("data-") => Some("false".into()),
        _ => None,
    }
}

/// React's style object as CSS: camelCase names in kebab-case (custom properties as written), numbers in px except
/// for the properties React leaves unitless.
pub fn style_css(style: &Map<String, Value>) -> String {
    let mut css = String::new();
    for (k, v) in style {
        let value = match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => {
                let x = n.as_f64().unwrap_or(0.0);
                if x == 0.0 || UNITLESS.contains(&k.as_str()) || k.starts_with("--") { number(x) } else { format!("{}px", number(x)) }
            }
            _ => continue,
        };
        let name = if k.starts_with("--") { k.clone() } else { kebab(k) };
        css.push_str(&name);
        css.push(':');
        css.push_str(&value);
        css.push(';');
    }
    css
}

/// React's unitless properties (react-dom's isUnitlessNumber).
pub(crate) const UNITLESS: &[&str] = &[
    "animationIterationCount", "aspectRatio", "borderImageOutset", "borderImageSlice", "borderImageWidth", "boxFlex", "boxFlexGroup",
    "boxOrdinalGroup", "columnCount", "columns", "flex", "flexGrow", "flexPositive", "flexShrink", "flexNegative", "flexOrder", "gridArea",
    "gridRow", "gridRowEnd", "gridRowSpan", "gridRowStart", "gridColumn", "gridColumnEnd", "gridColumnSpan", "gridColumnStart",
    "fontWeight", "lineClamp", "lineHeight", "opacity", "order", "orphans", "scale", "tabSize", "widows", "zIndex", "zoom", "fillOpacity",
    "floodOpacity", "stopOpacity", "strokeDasharray", "strokeDashoffset", "strokeMiterlimit", "strokeOpacity", "strokeWidth",
    "WebkitLineClamp",
];

pub(crate) fn number(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 1e15 { format!("{}", x as i64) } else { format!("{x}") }
}

/// camelCase as kebab-case; a vendor prefix gets its leading dash, keyed WebkitX, MozX, or as Chrome's style object and
/// the HTML runtime key it, webkitX, mozX, msX (webkitX came out webkit-x, which stylo drops: #34).
pub(crate) fn kebab(s: &str) -> String {
    let vendor = ["ms", "webkit", "moz", "Webkit", "Moz"].iter().any(|p| s.strip_prefix(p).is_some_and(|r| r.starts_with(|c: char| c.is_ascii_uppercase())));
    let mut out = String::from(if vendor { "-" } else { "" });
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('-');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// The mutator's document id for an op id, for callers that work with nodes (the bridge's twins, boxes).
pub fn node(nodes: &Nodes, id: u64) -> Option<NodeId> {
    nodes.by_id.get(&id).copied()
}

#[test]
fn a_vendor_prefix_keeps_its_dash_however_it_is_keyed() {
    let ks = ["WebkitLineClamp", "webkitLineClamp", "webkitTextStroke", "MozTabSize", "mozTabSize", "msTransform", "backgroundColor"];
    assert_eq!(ks.map(kebab), ["-webkit-line-clamp", "-webkit-line-clamp", "-webkit-text-stroke", "-moz-tab-size", "-moz-tab-size", "-ms-transform", "background-color"]);
}
