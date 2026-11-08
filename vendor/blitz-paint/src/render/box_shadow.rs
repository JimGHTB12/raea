use super::ElementCx;
use crate::color::{Color, ToColorColor as _};
use anyrender::PaintScene;
use kurbo::{Rect, Vec2};
use peniko::{Compose, Fill, Mix};

impl ElementCx<'_, '_> {
    pub(super) fn draw_outset_box_shadow(&self, scene: &mut impl PaintScene) {
        let box_shadow = &self.style.get_effects().box_shadow.0;
        let has_outset_shadow = box_shadow.iter().any(|s| !s.inset);
        if !has_outset_shadow {
            return;
        }

        let current_color = self.style.clone_color();
        let opacity = self.style.get_effects().opacity;
        let bg_color = self
            .style
            .get_background()
            .background_color
            .resolve_to_absolute(&current_color)
            .as_srgb_color();
        let bg_is_opaque = bg_color.components[3] >= 1.0;
        let needs_clip = opacity < 1.0 || !bg_is_opaque;
        // raea: a bevy_ui node's shadows (data-raea-bevy-shadow="BoxShadowSamples") are drawn as bevy_ui draws them
        let bevy = self.node.attr(blitz_dom::LocalName::from("data-raea-bevy-shadow")).and_then(|a| a.parse::<u32>().ok());

        let max_shadow_rect = box_shadow.iter().fold(Rect::ZERO, |prev, shadow| {
            let x = shadow.base.horizontal.px() as f64 * self.scale;
            let y = shadow.base.vertical.px() as f64 * self.scale;
            let blur = shadow.base.blur.px() as f64 * self.scale;
            let spread = shadow.spread.px() as f64 * self.scale;
            let offset = spread + blur * 2.5;

            // raea: a bevy_ui node's shadow spreads by bevy_ui's rule, its height by the width's ratio
            let rect = match bevy.is_some().then(|| bevy_spread(self.frame.border_box, 0.0, spread)).flatten() {
                Some((r, _)) => r.inflate(blur * 2.5, blur * 2.5) + Vec2::new(x, y),
                None => self.frame.border_box.inflate(offset, offset) + Vec2::new(x, y),
            };

            prev.union(rect)
        });

        self.context.layer_manager.maybe_with_layer(
            scene,
            needs_clip,
            1.0,
            self.transform,
            &self.frame.shadow_clip(max_shadow_rect),
            None,
            None,
            |scene| {
                for shadow in box_shadow.iter().filter(|s| !s.inset).rev() {
                    let shadow_color = shadow
                        .base
                        .color
                        .resolve_to_absolute(&current_color)
                        .as_srgb_color();

                    let alpha = shadow_color.components[3];
                    if alpha != 0.0 {
                        // raea: the offset in the element's axes, so it turns with a rotated element (#18)
                        let transform = self.transform.pre_translate(Vec2 {
                            x: shadow.base.horizontal.px() as f64 * self.scale,
                            y: shadow.base.vertical.px() as f64 * self.scale,
                        });

                        let spread = shadow.spread.px() as f64 * self.scale;
                        // raea: a blur radius is twice the Gaussian's standard deviation (CSS Backgrounds 3,
                        // "shadow blur"), which draw_box_shadow takes; and at the page's scale, as the spread is
                        let blur = shadow.base.blur.px() as f64 * self.scale / 2.0;

                        // raea: a bevy_ui node's shadow, spread by bevy_ui's rule and blurred as box_shadow.wesl blurs it
                        if let Some(samples) = bevy {
                            let Some((rect, ratio)) = bevy_spread(self.frame.border_box, 1.0, spread) else { continue };
                            let r = &self.frame.border_radii;
                            let radii = [r.top_left.x, r.top_right.x, r.bottom_right.x, r.bottom_left.x].map(|v| v * ratio);
                            let [cr, cg, cb, _] = shadow_color.components;
                            let rgb = [cr, cg, cb].map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
                            let c = rect.center();
                            if let Some(((x0, y0), image)) = super::bevy_shadow::bitmap((c.x, c.y), (rect.width(), rect.height()), radii, blur, samples, rgb, alpha) {
                                let brush = super::to_peniko_image(&image, peniko::ImageQuality::Low);
                                scene.draw_image(brush.as_ref(), transform.pre_translate(Vec2::new(x0, y0)));
                            }
                            continue;
                        }

                        // Without blur or spread the shadow is exactly the element's shape,
                        // so fill the border box path directly (preserving individual radii)
                        if blur == 0.0 && spread == 0.0 {
                            scene.fill(
                                Fill::NonZero,
                                transform,
                                shadow_color,
                                None,
                                &self.frame.border_box_path(),
                            );
                            continue;
                        }

                        // TODO draw shadows with matching individual radii instead of averaging
                        // raea: the shadow's corners grow and shrink with its spread, as its box does
                        let radius = self.frame.border_radii.average();
                        let radius = if radius > 0.0 { (radius + spread).max(0.0) } else { 0.0 };
                        let rect = self.frame.border_box.inflate(spread, spread);

                        // Fill the color
                        scene.draw_box_shadow(transform, rect, shadow_color, radius, blur);
                    }
                }
            },
        )
    }

    pub(super) fn draw_inset_box_shadow(&self, scene: &mut impl PaintScene) {
        let current_color = self.style.clone_color();
        let box_shadow = &self.style.get_effects().box_shadow.0;
        let has_inset_shadow = box_shadow.iter().any(|s| s.inset);
        if !has_inset_shadow {
            return;
        }

        let padding_box = self.frame.padding_box_path();

        for shadow in box_shadow.iter().filter(|s| s.inset) {
            let shadow_color = shadow
                .base
                .color
                .resolve_to_absolute(&current_color)
                .as_srgb_color();
            if shadow_color == Color::TRANSPARENT {
                return;
            }

            // TODO draw shadows with matching individual radii instead of averaging
            let radius = self.frame.border_radii.average();
            // raea: in the element's axes (#18), and at the page's scale, as an outer shadow's offset is
            let transform = self.transform.pre_translate(Vec2 {
                x: shadow.base.horizontal.px() as f64 * self.scale,
                y: shadow.base.vertical.px() as f64 * self.scale,
            });

            scene.push_layer(
                Fill::NonZero,
                Mix::Normal,
                1.0,
                self.transform,
                &padding_box,
                None,
                None,
            );
            scene.fill(
                Fill::NonZero,
                self.transform,
                shadow_color,
                None,
                &padding_box,
            );

            scene.push_layer(
                Fill::NonZero,
                Compose::DestOut,
                1.0,
                self.transform,
                &padding_box,
                None,
                None,
            );
            // The unshadowed area is the padding box shrunk by the spread radius
            let spread = shadow.spread.px() as f64 * self.scale;
            let hole = self.frame.padding_box.inflate(-spread, -spread);
            scene.draw_box_shadow(
                transform,
                hole,
                Color::WHITE,
                radius,
                // raea: the blur radius is twice the standard deviation draw_box_shadow takes
                shadow.base.blur.px() as f64 * self.scale / 2.0,
            );

            scene.pop_layer();
            scene.pop_layer();
        }
    }
}

/// bevy_ui's spread (box_shadow.rs extract_shadows): the shadow `spread` wider than the node, half each side, its height
/// and corners scaled by the same ratio, about the node's centre; none when that leaves it no size.
pub(crate) fn bevy_spread(border_box: Rect, radius: f64, spread: f64) -> Option<(Rect, f64)> {
    let (w, h) = (border_box.width(), border_box.height());
    if w <= 0.0 {
        return None;
    }
    let ratio = (spread + w) / w;
    let (sw, sh) = (w + spread, h * ratio);
    if sw <= 0.0 || sh <= 0.0 {
        return None;
    }
    let c = border_box.center();
    Some((Rect::from_center_size(c, (sw, sh)), radius * ratio))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bevy_spread_grows_width_by_spread_and_height_by_its_ratio() {
        // a 200 x 100 node spread 20: 220 wide, 110 high (ratio 1.1), corners 4 -> 4.4, about the same centre
        let (r, radius) = bevy_spread(Rect::new(10.0, 10.0, 210.0, 110.0), 4.0, 20.0).unwrap();
        for (got, want) in [(r.x0, 0.0), (r.y0, 5.0), (r.x1, 220.0), (r.y1, 115.0)] {
            assert!((got - want).abs() < 1e-9, "{r:?}");
        }
        assert!((radius - 4.4).abs() < 1e-12);
        // a spread that takes the whole width leaves no shadow
        assert!(bevy_spread(Rect::new(0.0, 0.0, 10.0, 10.0), 0.0, -10.0).is_none());
    }
}
