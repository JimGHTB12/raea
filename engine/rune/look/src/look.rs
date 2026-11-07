//! An element's look as Chrome's computed style gives it (T3e, the T3 set's bar in projects/rune's README): its
//! background as Rune drew it, its text colour, its top border (width and colour) and outline when it has one, its
//! shadow layers, all colours in 8-bit sRGB with 8-bit alpha (Rune's are linear). The style ones are what the view
//! mapped (Page::looks); the background is the drawn fill, so a transition's in-between frame shows as drawn.

use rune::Color;
use serde_json::{Value, json};

use raea_rune_page::css::Css;

/// A colour as Chrome's canvas reads it back: 8-bit sRGB channels and alpha.
pub fn rgba(c: Color) -> [u8; 4] {
    let enc = |x: f32| (rune::color::srgb_encode(x.clamp(0.0, 1.0)) * 255.0).round() as u8;
    [enc(c.r), enc(c.g), enc(c.b), (c.a.clamp(0.0, 1.0) * 255.0).round() as u8]
}

/// Element `e`'s look, from its mapped style and text colour (`mapped`); `focus_visible` is whether it has the page's
/// focus and shows it, which draws Chrome's ring when it has no outline of its own (outline.rs, as the view draws it).
pub fn of(e: &rune::ElementInfo, mapped: Option<&(Css, Color)>, focus_visible: bool) -> Value {
    let mut v = json!({"bg": rgba(e.fill)});
    let Some((c, fg)) = mapped else { return v };
    v["fg"] = json!(rgba(*fg));
    if let Some(w) = c.border_width[0].filter(|w| *w > 0.0) {
        v["border"] = json!([w, rgba(c.border_color.unwrap_or(*fg))]);
    }
    if let Some((w, _, col)) = raea_rune_page::outline::drawn(&c.outline, *fg, focus_visible) {
        v["outline"] = json!([w, rgba(col)]);
    }
    if !c.shadows.is_empty() {
        v["shadow"] = c.shadows.iter().map(|s| json!([rgba(s.color), s.x, s.y, s.blur, s.spread, s.inset])).collect();
    }
    v
}
