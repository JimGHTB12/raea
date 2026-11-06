use super::ElementCx;
use crate::color::{Color, ToColorColor as _};
use anyrender::PaintScene;
use blitz_dom::local_name;
use kurbo::{BezPath, Cap, Circle, Join, Point, Stroke};
use peniko::Fill;
use style::dom::TElement as _;

// raea: Chrome's look on macOS (appearance: auto), as bevy-tsx's page draws it (engine/bevy-tsx/src/controls.rs
// `faces`): a 1px edge, grey or the accent blue, darker under the pointer; a checked checkbox filled with it and a white
// check, a checked radio white with a dot of it 60% across. Blitz drew smaller, darker controls in the text colour.
const ACCENT: Color = Color::from_rgba8(0, 117, 255, 255);
const ACCENT_HOVER: Color = Color::from_rgba8(0, 92, 200, 255);
const EDGE: Color = Color::from_rgba8(118, 118, 118, 255);
const EDGE_HOVER: Color = Color::from_rgba8(79, 79, 79, 255);

impl ElementCx<'_, '_> {
    pub(super) fn draw_input(&self, scene: &mut impl PaintScene) {
        if self.node.local_name() != "input" {
            return;
        }
        if self.node.attr(local_name!("type")) == Some("range") {
            return self.draw_range(scene);
        }
        let Some(on) = self.element.checkbox_input_checked() else {
            return;
        };
        let radio = match self.node.attr(local_name!("type")) {
            Some("checkbox") => false,
            Some("radio") => true,
            _ => return,
        };
        let hover = self.node.is_hovered() && self.node.attr(local_name!("disabled")).is_none();
        let edge = match (on, hover) {
            (true, false) => ACCENT,
            (true, true) => ACCENT_HOVER,
            (false, false) => EDGE,
            (false, true) => EDGE_HOVER,
        };
        let px = self.scale;
        let b = self.frame.border_box;
        let (w, h) = (b.width(), b.height());
        let s = w.min(h);
        let css_s = s / px;
        let fill = if on && !radio { edge } else { Color::WHITE };
        let radius = if radio { s / 2.0 } else { (css_s * 2.0 / 20.0).round().max(2.0) * px };
        let outer = b.to_rounded_rect(radius);
        let inner = b.inset(-px).to_rounded_rect((radius - px).max(0.0));
        let t = self.transform;
        scene.fill(Fill::NonZero, t, edge, None, &outer);
        scene.fill(Fill::NonZero, t, fill, None, &inner);
        if on && radio {
            let d = (css_s * 0.6).round() * px;
            scene.fill(Fill::NonZero, t, edge, None, &Circle::new(b.center(), d / 2.0));
        } else if on {
            // the check: a polyline at fractions of the box, 15% of its width wide (1.5px at least)
            let at = |x: f64, y: f64| Point::new(b.x0 + x * w, b.y0 + y * h);
            let mut path = BezPath::new();
            path.move_to(at(0.2, 0.52));
            path.line_to(at(0.41, 0.72));
            path.line_to(at(0.79, 0.19));
            let style = Stroke {
                width: (w / px * 0.15).max(1.5) * px,
                join: Join::Round,
                miter_limit: 10.0,
                start_cap: Cap::Butt,
                end_cap: Cap::Butt,
                dash_pattern: Default::default(),
                dash_offset: 0.0,
            };
            scene.stroke(&style, t, Color::WHITE, None, &path);
        }
    }
}

impl ElementCx<'_, '_> {
    /// raea: a range input's thumb at its value, as Chrome draws it: the page's own (::-webkit-slider-thumb with a
    /// width, its height, background and corner radius: Excalidraw's opacity slider) centred on the input, else
    /// Chrome's on macOS, a 4px track blue up to a 16px blue thumb. Blitz drew no thumb, only the input's own box.
    fn draw_range(&self, scene: &mut impl PaintScene) {
        let num = |v: Option<&str>| v.and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| v.is_finite());
        let min = num(self.node.attr(local_name!("min"))).unwrap_or(0.0);
        let max = num(self.node.attr(local_name!("max"))).unwrap_or(100.0).max(min);
        let value = num(self.node.attr(local_name!("value"))).unwrap_or((min + max) / 2.0).clamp(min, max);
        let at = if max > min { (value - min) / (max - min) } else { 0.0 };
        let (px, b, t) = (self.scale, self.frame.border_box, self.transform);
        let thumb = self.context.dom.slider_thumb_style(self.node.id);
        let length = |s: &style::values::computed::Size| match s {
            style::values::generics::length::GenericSize::LengthPercentage(lp) => lp.0.to_length().map(|l| f64::from(l.px())),
            _ => None,
        };
        let own = thumb.as_ref().and_then(|th| length(&th.get_position().width).map(|w| (th, w)));
        if let Some((th, w)) = own {
            let h = length(&th.get_position().height).unwrap_or(w);
            let (w, h) = (w * px, h * px);
            let r = th.get_border().border_top_left_radius.0.width.0.resolve(style::values::computed::Length::new((w / px) as f32));
            let color = th.get_background().background_color.resolve_to_absolute(&th.clone_color()).as_srgb_color();
            let cx = b.x0 + w / 2.0 + at * (b.width() - w);
            let rect = kurbo::Rect::from_center_size((cx, b.center().y), (w, h));
            scene.fill(Fill::NonZero, t, color, None, &rect.to_rounded_rect(f64::from(r.px()) * px));
            return;
        }
        let d = 16.0 * px;
        let cx = b.x0 + d / 2.0 + at * (b.width() - d);
        let track = kurbo::Rect::new(b.x0, b.center().y - 2.0 * px, b.x1, b.center().y + 2.0 * px);
        scene.fill(Fill::NonZero, t, Color::from_rgba8(239, 239, 239, 255), None, &track.to_rounded_rect(2.0 * px));
        scene.fill(Fill::NonZero, t, ACCENT, None, &track.with_size((cx - b.x0, track.height())).to_rounded_rect(2.0 * px));
        scene.fill(Fill::NonZero, t, ACCENT, None, &Circle::new((cx, b.center().y), d / 2.0));
    }
}
