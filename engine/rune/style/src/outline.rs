//! CSS `outline` (T3e): its style, width, colour and offset as the style says them, and the outline the host draws
//! from them with Rune's `El::outline` (a ring `offset` points outside the border box). An element draws its own
//! outline when its style is one Rune draws (solid, or Chrome's `auto`, drawn solid); with no style set, the focused
//! element shows Chrome's `:focus-visible` default (`outline: auto 1px -webkit-focus-ring-color`, which Chrome computes
//! as 1 px of rgb(0, 95, 204)) when its focus is visible. `outline-style: none` (Tailwind's `outline-none`) draws none.

use rune::Color;
use serde_json::Value;

use crate::css::{BAD_VALUE, Len, color, length};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutlineCss {
    /// true for a style Rune draws, false for `none` or `hidden`
    pub drawn: Option<bool>,
    pub width: Option<f32>,
    pub color: Option<Color>,
    pub offset: Option<f32>,
}

/// Chrome's focus ring colour, `-webkit-focus-ring-color` on the Mac
const RING: [u8; 3] = [0, 95, 204];

fn style(s: &str) -> Option<bool> {
    match s {
        "none" | "hidden" => Some(false),
        "solid" | "auto" => Some(true),
        _ => None,
    }
}

fn width(v: &Value) -> Option<f32> {
    match (v.as_str().map(str::trim), length(v)) {
        (Some("thin"), _) => Some(1.0),
        (Some("medium"), _) => Some(3.0),
        (Some("thick"), _) => Some(5.0),
        (_, Some(Len::Px(p))) => Some(p),
        _ => None,
    }
}

/// An `outline*` prop into `o`, or why not: a style other than none, hidden, solid or auto isn't read.
pub fn prop(o: &mut OutlineCss, k: &str, v: &Value) -> Result<(), &'static str> {
    let text = || v.as_str().map(str::trim).ok_or(BAD_VALUE);
    match k {
        "outlineStyle" => o.drawn = Some(style(text()?).ok_or(BAD_VALUE)?),
        "outlineWidth" => o.width = Some(width(v).ok_or(BAD_VALUE)?),
        "outlineColor" => o.color = Some(color(text()?).ok_or(BAD_VALUE)?),
        "outlineOffset" => o.offset = Some(width(v).ok_or(BAD_VALUE)?),
        // the shorthand: a width, a style and a colour in any order, each optional
        _ => {
            for t in text()?.split_whitespace() {
                let tv = Value::String(t.into());
                if let Some(s) = style(t) {
                    o.drawn = Some(s);
                } else if let Some(w) = width(&tv) {
                    o.width = Some(w);
                } else {
                    o.color = Some(color(t).ok_or(BAD_VALUE)?);
                }
            }
        }
    }
    Ok(())
}

/// The outline drawn: (width, offset, colour), the colour defaulting to the text's (`fg`); `focus_visible` is whether
/// this element has the page's focus and shows it.
pub fn drawn(o: &OutlineCss, fg: Color, focus_visible: bool) -> Option<(f32, f32, Color)> {
    let (w, c) = match o.drawn {
        Some(false) => return None,
        Some(true) => (o.width.unwrap_or(3.0), o.color.unwrap_or(fg)),
        None if focus_visible => {
            let d = |x: u8| rune::color::srgb_decode(x as f32 / 255.0);
            (o.width.unwrap_or(1.0), o.color.unwrap_or(Color { r: d(RING[0]), g: d(RING[1]), b: d(RING[2]), a: 1.0 }))
        }
        None => return None,
    };
    (w > 0.0).then_some((w, o.offset.unwrap_or(0.0), c))
}
