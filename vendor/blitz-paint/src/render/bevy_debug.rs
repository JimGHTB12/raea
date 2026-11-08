//! raea: bevy_ui's debug overlay (UiDebugOptions, GlobalUiDebugOptions; sdk/render's `ui_debug`). An element marked
//! `data-raea-bevy-debug="width r g b a boxes clipped"` has outlines drawn after the whole document, above every UI
//! node as bevy_ui_render's extract_debug_overlay keeps them: a ring `width` device px thick inside its border box
//! (`b`), padding box (`p`) and content box (`c`), following its corners' radii on the border and padding boxes, in
//! the sRGB colour given, cut by the clip its ancestors' overflow gives it unless `clipped` is 1 (show_clipped).

use kurbo::{Affine, Rect, RoundedRect, RoundedRectRadii, Shape};
use peniko::{Color, Fill};

use crate::kurbo_css::CssBox;
use anyrender::PaintScene;

/// One element's outlines, as recorded while the document was drawn.
pub(crate) struct DebugOutline {
    transform: Affine,
    clip: Option<Rect>,
    rings: Vec<(Rect, RoundedRectRadii)>,
    width: f64,
    color: Color,
}

/// The element's outlines from its `data-raea-bevy-debug` value, or None when it isn't one.
pub(crate) fn outline(attr: &str, frame: &CssBox, transform: Affine, clip: Rect, scale: f64) -> Option<DebugOutline> {
    let mut it = attr.split_whitespace();
    let mut num = || it.next()?.parse::<f64>().ok();
    let (width, r, g, b, a) = (num()? * scale, num()?, num()?, num()?, num()?);
    let boxes = it.next()?;
    let unclipped = it.next() == Some("1");
    let radii = &frame.border_radii;
    let outer = RoundedRectRadii::new(radii.top_left.x, radii.top_right.x, radii.bottom_right.x, radii.bottom_left.x);
    let bw = &frame.border_width;
    let inner = RoundedRectRadii::new(
        (radii.top_left.x - bw.x0.max(bw.y0)).max(0.0),
        (radii.top_right.x - bw.x1.max(bw.y0)).max(0.0),
        (radii.bottom_right.x - bw.x1.max(bw.y1)).max(0.0),
        (radii.bottom_left.x - bw.x0.max(bw.y1)).max(0.0),
    );
    let mut rings = Vec::new();
    for (k, rect, radii) in [('b', frame.border_box, outer), ('p', frame.padding_box, inner), ('c', frame.content_box, RoundedRectRadii::from_single_radius(0.0))] {
        if boxes.contains(k) && rect.width() > 0.0 && rect.height() > 0.0 {
            rings.push((rect, radii));
        }
    }
    let color = Color::new([r as f32, g as f32, b as f32, a as f32]);
    Some(DebugOutline { transform, clip: (!unclipped).then_some(clip), rings, width, color })
}

/// Draw the recorded outlines, in the order their elements were drawn, over everything.
pub(crate) fn draw(scene: &mut impl PaintScene, outlines: &[DebugOutline]) {
    for o in outlines {
        if let Some(clip) = o.clip {
            scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip);
        }
        for (rect, radii) in &o.rings {
            // a ring `width` thick inside the box: the box less the box inset by `width`, its radii shrunk as a border's
            let w = o.width.min(rect.width() / 2.0).min(rect.height() / 2.0);
            let shrink = |r: f64| (r - w).max(0.0);
            let inner_radii = RoundedRectRadii::new(shrink(radii.top_left), shrink(radii.top_right), shrink(radii.bottom_right), shrink(radii.bottom_left));
            let mut path = RoundedRect::from_rect(*rect, *radii).to_path(0.1);
            path.extend(RoundedRect::from_rect(rect.inset(-w), inner_radii).to_path(0.1));
            scene.fill(Fill::EvenOdd, o.transform, o.color, None, &path);
        }
        if o.clip.is_some() {
            scene.pop_layer();
        }
    }
}
