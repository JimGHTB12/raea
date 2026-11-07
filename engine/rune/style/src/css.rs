//! An element's `style` prop read into typed values (R2b): what the wire carries (React's style object: numbers are
//! px, strings are CSS) parsed once, each prop either understood or reported. Nothing is dropped silently: a prop this
//! doesn't know, or a value it can't read, goes to `unmapped` with why. `view` turns a `Css` into Rune builder calls.

use rune::Color;
use serde_json::{Map, Value};

/// A length: px, a share of the parent (`50%`), or auto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Len {
    Px(f32),
    Pct(f32),
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Display {
    #[default]
    Block,
    Flex,
    Inline,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Align {
    Start,
    End,
    Center,
    Stretch,
    Between,
    Around,
    Evenly,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}

/// A `box-shadow` layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowCss {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
    pub inset: bool,
}

/// What an element's style says, prop by prop (None: not set). Edges are [top, right, bottom, left]; corners
/// [top-left, top-right, bottom-right, bottom-left], CSS's orders.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Css {
    pub display: Option<Display>,
    pub column: Option<bool>,
    pub reverse: bool,
    pub wrap: bool,
    pub align_items: Option<Align>,
    pub align_self: Option<Align>,
    pub justify: Option<Align>,
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub basis: Option<Len>,
    pub width: Option<Len>,
    pub height: Option<Len>,
    pub min_width: Option<Len>,
    pub min_height: Option<Len>,
    pub max_width: Option<Len>,
    pub max_height: Option<Len>,
    pub padding: [Option<f32>; 4],
    pub margin: [Option<Len>; 4],
    pub row_gap: Option<f32>,
    pub column_gap: Option<f32>,
    pub absolute: bool,
    pub inset: [Option<f32>; 4],
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub background: Option<Color>,
    /// `linear-gradient(angle, stops)`: the angle in radians (CSS's: 0 up, PI/2 right), stops at 0..1
    pub gradient: Option<(f32, Vec<(f32, Color)>)>,
    pub border_width: [Option<f32>; 4],
    pub border_color: Option<Color>,
    pub radius: [Option<f32>; 4],
    pub shadows: Vec<ShadowCss>,
    pub opacity: Option<f32>,
    /// objectFit on an img or video (T3d): contain letterboxes, cover crops; `fill` (CSS's default) stretches
    pub fit: Option<Option<rune::SurfaceFit>>,
    // inherited by text
    pub color: Option<Color>,
    pub font_size: Option<f32>,
    pub font_weight: Option<f32>,
    pub font_family: Option<String>,
    pub line_height: Option<Len>,
    pub letter_spacing: Option<f32>,
    pub text_align: Option<Align>,
    pub italic: Option<bool>,
    /// transform, translate, scale, rotate, their origin and the transition (motion.rs, T3c)
    pub motion: crate::motion::Motion,
}

/// A style prop the host couldn't use: its name, its value as given, and why.
#[derive(Clone, Debug, PartialEq)]
pub struct Unmapped {
    pub prop: String,
    pub value: String,
    pub why: &'static str,
}

/// `style` read into a `Css`, and what it couldn't read.
pub fn parse(style: &Map<String, Value>) -> (Css, Vec<Unmapped>) {
    let mut c = Css::default();
    let mut bad = Vec::new();
    // shorthands first, so a longhand beside one wins whatever order the map iterates in (serde_json's is by name)
    let short = |k: &str| SHORTHANDS.contains(&k);
    for pass in [true, false] {
        for (k, v) in style.iter().filter(|(k, _)| short(k) == pass) {
            if let Err(why) = prop(&mut c, k, v) {
                bad.push(Unmapped { prop: k.clone(), value: show(v), why });
            }
        }
    }
    (c, bad)
}

/// The props that set several others, applied before those others.
const SHORTHANDS: [&str; 7] = ["padding", "margin", "gap", "borderWidth", "borderRadius", "overflow", "transition"];

fn show(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        v => v.to_string(),
    }
}

pub(crate) const NO_PROP: &str = "no Rune equivalent mapped";
pub(crate) const BAD_VALUE: &str = "a value the host can't read";

/// One prop into `c`, or why not.
fn prop(c: &mut Css, k: &str, v: &Value) -> Result<(), &'static str> {
    let len = || length(v).ok_or(BAD_VALUE);
    let px = || match length(v) {
        Some(Len::Px(p)) => Ok(p),
        _ => Err(BAD_VALUE),
    };
    let edges4 = |c: &mut [Option<f32>; 4], i: Option<usize>| -> Result<(), &'static str> {
        let p = px()?;
        match i {
            Some(i) => c[i] = Some(p),
            None => *c = [Some(p); 4],
        }
        Ok(())
    };
    match k {
        "display" => {
            c.display = Some(match text(v)? {
                "flex" => Display::Flex,
                "block" => Display::Block,
                "inline" | "inline-block" => Display::Inline,
                "none" => Display::None,
                _ => return Err(BAD_VALUE),
            })
        }
        "flexDirection" => {
            let (col, rev) = match text(v)? {
                "row" => (false, false),
                "column" => (true, false),
                "row-reverse" => (false, true),
                "column-reverse" => (true, true),
                _ => return Err(BAD_VALUE),
            };
            (c.column, c.reverse) = (Some(col), rev);
            if rev {
                return Err("reversed flex directions aren't mapped");
            }
        }
        "flexWrap" => c.wrap = text(v)? == "wrap",
        "alignItems" => c.align_items = Some(align(v)?),
        "alignSelf" => c.align_self = Some(align(v)?),
        "justifyContent" => c.justify = Some(align(v)?),
        "flexGrow" => c.grow = Some(num(v)?),
        "flexShrink" => c.shrink = Some(num(v)?),
        "flexBasis" => c.basis = Some(len()?),
        "width" => c.width = Some(len()?),
        "height" => c.height = Some(len()?),
        "minWidth" => c.min_width = Some(len()?),
        "minHeight" => c.min_height = Some(len()?),
        "maxWidth" => c.max_width = Some(len()?),
        "maxHeight" => c.max_height = Some(len()?),
        "padding" => edges4(&mut c.padding, None)?,
        "paddingTop" => edges4(&mut c.padding, Some(0))?,
        "paddingRight" => edges4(&mut c.padding, Some(1))?,
        "paddingBottom" => edges4(&mut c.padding, Some(2))?,
        "paddingLeft" => edges4(&mut c.padding, Some(3))?,
        "margin" => c.margin = [Some(len()?); 4],
        "marginTop" => c.margin[0] = Some(len()?),
        "marginRight" => c.margin[1] = Some(len()?),
        "marginBottom" => c.margin[2] = Some(len()?),
        "marginLeft" => c.margin[3] = Some(len()?),
        "gap" => (c.row_gap, c.column_gap) = (Some(px()?), Some(px()?)),
        "rowGap" => c.row_gap = Some(px()?),
        "columnGap" => c.column_gap = Some(px()?),
        "position" => match text(v)? {
            "absolute" => c.absolute = true,
            "relative" | "static" => c.absolute = false,
            _ => return Err("only static, relative and absolute positions are mapped"),
        },
        "top" => c.inset[0] = Some(px()?),
        "right" => c.inset[1] = Some(px()?),
        "bottom" => c.inset[2] = Some(px()?),
        "left" => c.inset[3] = Some(px()?),
        "overflow" => (c.overflow_x, c.overflow_y) = (overflow(v)?, overflow(v)?),
        "overflowX" => c.overflow_x = overflow(v)?,
        "overflowY" => c.overflow_y = overflow(v)?,
        "backgroundColor" => c.background = Some(color(text(v)?).ok_or(BAD_VALUE)?),
        "backgroundImage" => c.gradient = Some(gradient(text(v)?).ok_or("only linear-gradient(angle, colour stops) is mapped")?),
        "borderWidth" => edges4(&mut c.border_width, None)?,
        "borderTopWidth" => edges4(&mut c.border_width, Some(0))?,
        "borderRightWidth" => edges4(&mut c.border_width, Some(1))?,
        "borderBottomWidth" => edges4(&mut c.border_width, Some(2))?,
        "borderLeftWidth" => edges4(&mut c.border_width, Some(3))?,
        "borderColor" => c.border_color = Some(color(text(v)?).ok_or(BAD_VALUE)?),
        "borderRadius" => edges4(&mut c.radius, None)?,
        "borderTopLeftRadius" => edges4(&mut c.radius, Some(0))?,
        "borderTopRightRadius" => edges4(&mut c.radius, Some(1))?,
        "borderBottomRightRadius" => edges4(&mut c.radius, Some(2))?,
        "borderBottomLeftRadius" => edges4(&mut c.radius, Some(3))?,
        "boxShadow" => c.shadows = shadows(text(v)?).ok_or(BAD_VALUE)?,
        "opacity" => c.opacity = Some(num(v)?),
        "objectFit" => c.fit = Some(match text(v)? { "contain" => Some(rune::SurfaceFit::Fit), "cover" => Some(rune::SurfaceFit::Fill), "fill" => None, _ => return Err(BAD_VALUE) }),
        "color" => c.color = Some(color(text(v)?).ok_or(BAD_VALUE)?),
        "fontSize" => c.font_size = Some(px()?),
        "fontWeight" => c.font_weight = Some(weight(v)?),
        "fontFamily" => c.font_family = Some(text(v)?.split(',').next().unwrap_or("").trim().trim_matches(['"', '\'']).to_string()),
        "fontStyle" => c.italic = Some(text(v)? == "italic"),
        "lineHeight" => {
            c.line_height = Some(match v {
                // a bare number is a multiple of the font size
                Value::Number(n) => Len::Pct(n.as_f64().ok_or(BAD_VALUE)? as f32 * 100.0),
                _ => len()?,
            })
        }
        "letterSpacing" => c.letter_spacing = Some(px()?),
        "textAlign" => {
            c.text_align = Some(match text(v)? {
                "left" | "start" => Align::Start,
                "right" | "end" => Align::End,
                "center" => Align::Center,
                _ => return Err(BAD_VALUE),
            })
        }
        _ => return crate::motion::prop(&mut c.motion, k, v),
    }
    Ok(())
}

fn text(v: &Value) -> Result<&str, &'static str> {
    v.as_str().map(str::trim).ok_or(BAD_VALUE)
}

fn num(v: &Value) -> Result<f32, &'static str> {
    match v {
        Value::Number(n) => n.as_f64().map(|f| f as f32).ok_or(BAD_VALUE),
        Value::String(s) => s.trim().parse().map_err(|_| BAD_VALUE),
        _ => Err(BAD_VALUE),
    }
}

/// A React style length: a number is px; a string is `Npx`, `N%`, `0` or `auto`.
pub fn length(v: &Value) -> Option<Len> {
    match v {
        Value::Number(n) => Some(Len::Px(n.as_f64()? as f32)),
        Value::String(s) => {
            let s = s.trim();
            if s == "auto" {
                Some(Len::Auto)
            } else if let Some(p) = s.strip_suffix('%') {
                p.trim().parse().ok().map(Len::Pct)
            } else {
                s.strip_suffix("px").unwrap_or(s).trim().parse().ok().map(Len::Px)
            }
        }
        _ => None,
    }
}

fn align(v: &Value) -> Result<Align, &'static str> {
    Ok(match text(v)? {
        "flex-start" | "start" | "normal" => Align::Start,
        "flex-end" | "end" => Align::End,
        "center" => Align::Center,
        "stretch" => Align::Stretch,
        "space-between" => Align::Between,
        "space-around" => Align::Around,
        "space-evenly" => Align::Evenly,
        _ => return Err(BAD_VALUE),
    })
}

fn overflow(v: &Value) -> Result<Overflow, &'static str> {
    Ok(match text(v)? {
        "visible" => Overflow::Visible,
        "hidden" | "clip" => Overflow::Hidden,
        "scroll" | "auto" => Overflow::Scroll,
        _ => return Err(BAD_VALUE),
    })
}

fn weight(v: &Value) -> Result<f32, &'static str> {
    match v.as_str().map(str::trim) {
        Some("normal") => Ok(400.0),
        Some("bold") => Ok(700.0),
        _ => num(v),
    }
}

/// A CSS colour: `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb()`/`rgba()` with commas or spaces, `transparent`, or one of the
/// basic named colours.
pub fn color(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.starts_with('#') {
        return rune::color::hex(s);
    }
    if let Some(body) = s.strip_prefix("rgba(").or_else(|| s.strip_prefix("rgb(")) {
        let body = body.strip_suffix(')')?.replace('/', " ");
        let parts: Vec<f32> = body.split([',', ' ']).filter(|p| !p.is_empty()).map(|p| p.trim().parse().ok()).collect::<Option<_>>()?;
        let (r, g, b) = (*parts.first()?, *parts.get(1)?, *parts.get(2)?);
        let a = parts.get(3).copied().unwrap_or(1.0);
        let d = |x: f32| rune::color::srgb_decode(x / 255.0);
        return Some(Color { r: d(r), g: d(g), b: d(b), a });
    }
    if let Some(body) = s.strip_prefix("oklch(").and_then(|b| b.strip_suffix(')')) {
        // Tailwind v4's palette: `oklch(62.3% 0.214 259.815 [/ a])`
        let (lch, a) = body.split_once('/').map_or((body, None), |(l, a)| (l, Some(a)));
        let p: Vec<&str> = lch.split_whitespace().collect();
        let l = match p.first()?.strip_suffix('%') {
            Some(pct) => pct.parse::<f32>().ok()? / 100.0,
            None => p.first()?.parse().ok()?,
        };
        let a = a.map_or(Some(1.0), |a| a.trim().parse().ok())?;
        return Some(rune::oklcha(l, p.get(1)?.parse().ok()?, p.get(2)?.parse().ok()?, a));
    }
    let hex = match s {
        "transparent" => return Some(Color::TRANSPARENT),
        "black" => "#000000",
        "white" => "#ffffff",
        "red" => "#ff0000",
        "green" => "#008000",
        "blue" => "#0000ff",
        "gray" | "grey" => "#808080",
        _ => return None,
    };
    rune::color::hex(hex)
}

/// `linear-gradient(<angle>?, <colour> <stop%>?, ...)`, stops without a position spread evenly.
pub fn gradient(s: &str) -> Option<(f32, Vec<(f32, Color)>)> {
    let body = s.trim().strip_prefix("linear-gradient(")?.strip_suffix(')')?;
    let parts = split_top(body);
    let (angle, stops) = match parts.first()?.trim() {
        a if a.ends_with("deg") => (a.strip_suffix("deg")?.trim().parse::<f32>().ok()?.to_radians(), &parts[1..]),
        "to bottom" => (std::f32::consts::PI, &parts[1..]),
        "to top" => (0.0, &parts[1..]),
        "to right" => (std::f32::consts::FRAC_PI_2, &parts[1..]),
        "to left" => (3.0 * std::f32::consts::FRAC_PI_2, &parts[1..]),
        _ => (std::f32::consts::PI, &parts[..]),
    };
    if stops.len() < 2 || stops.len() > 8 {
        return None;
    }
    let n = (stops.len() - 1) as f32;
    let out = stops
        .iter()
        .enumerate()
        .map(|(i, st)| {
            let st = st.trim();
            let (col, at) = match st.rsplit_once(' ') {
                Some((c, p)) if p.ends_with('%') && !c.ends_with(',') && color(c).is_some() => (c, p.strip_suffix('%')?.parse::<f32>().ok()? / 100.0),
                _ => (st, i as f32 / n),
            };
            Some((at, color(col)?))
        })
        .collect::<Option<Vec<_>>>()?;
    Some((angle, out))
}

/// `box-shadow` layers: `[inset] x y [blur [spread]] colour`, comma-separated.
pub fn shadows(s: &str) -> Option<Vec<ShadowCss>> {
    if s.trim() == "none" {
        return Some(Vec::new());
    }
    split_top(s)
        .into_iter()
        .map(|layer| {
            let mut inset = false;
            let mut nums = Vec::new();
            let mut col = None;
            for tok in split_space(layer) {
                if tok == "inset" {
                    inset = true;
                } else if let Some(Len::Px(p)) = length(&Value::String(tok.to_string())) {
                    nums.push(p);
                } else {
                    col = Some(color(tok)?);
                }
            }
            if nums.len() < 2 {
                return None;
            }
            let g = |i: usize| nums.get(i).copied().unwrap_or(0.0);
            Some(ShadowCss { x: g(0), y: g(1), blur: g(2), spread: g(3), color: col.unwrap_or(Color::BLACK), inset })
        })
        .collect()
}

/// `s` split at commas outside parentheses.
fn split_top(s: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0, 0);
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// `s` split at spaces outside parentheses.
fn split_space(s: &str) -> Vec<&str> {
    let (mut out, mut depth, mut start) = (Vec::new(), 0, 0);
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ' ' if depth == 0 => {
                if i > start {
                    out.push(&s[start..i]);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}
