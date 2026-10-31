use style::color::mix::HueInterpolationMethod;
use crate::color::{Color, ToColorColor};
use color::DynamicColor;
use kurbo::{self, Affine, Point, Rect, Vec2};
use peniko::{self, ColorStop, Gradient, LinearGradientPosition, SweepGradientPosition};
use style::color::AbsoluteColor;
use style::{
    OwnedSlice,
    values::{
        computed::{
            Angle, AngleOrPercentage, CSSPixelLength, Gradient as StyloGradient, LengthPercentage,
            LineDirection, Percentage,
        },
        generics::{
            NonNegative,
            color::GenericColor,
            image::{
                EndingShape, GenericCircle, GenericEllipse, GenericEndingShape, GenericGradient,
                GenericGradientItem, GradientFlags, ShapeExtent,
            },
            position::GenericPosition,
        },
        specified::{
            percentage::ToPercentage,
            position::{HorizontalPositionKeyword, VerticalPositionKeyword},
        },
    },
};

type GradientItem<T> = GenericGradientItem<GenericColor<Percentage>, T>;
type LinearGradient<'a> = (
    &'a LineDirection,
    &'a [GradientItem<LengthPercentage>],
    GradientFlags,
);
type RadialGradient<'a> = (
    &'a EndingShape<NonNegative<CSSPixelLength>, NonNegative<LengthPercentage>>,
    &'a GenericPosition<LengthPercentage, LengthPercentage>,
    &'a OwnedSlice<GenericGradientItem<GenericColor<Percentage>, LengthPercentage>>,
    GradientFlags,
);
type ConicGradient<'a> = (
    &'a Angle,
    &'a GenericPosition<LengthPercentage, LengthPercentage>,
    &'a OwnedSlice<GenericGradientItem<GenericColor<Percentage>, AngleOrPercentage>>,
    GradientFlags,
);

pub(crate) fn to_peniko_gradient(
    gradient: &StyloGradient,
    origin_rect: Rect,
    bounding_box: Rect,
    scale: f64,
    current_color: &AbsoluteColor,
) -> (peniko::Gradient, Option<Affine>) {
    let (mut out, at) = match gradient {
        // https://developer.mozilla.org/en-US/docs/Web/CSS/gradient/linear-gradient
        GenericGradient::Linear {
            direction,
            items,
            flags,
            // compat_mode,
            ..
        } => linear_gradient(
            (direction, items, *flags),
            origin_rect,
            bounding_box,
            scale,
            current_color,
        ),
        GenericGradient::Radial {
            shape,
            position,
            items,
            flags,
            // compat_mode,
            ..
        } => radial_gradient((shape, position, items, *flags), origin_rect, current_color),
        GenericGradient::Conic {
            angle,
            position,
            items,
            flags,
            ..
        } => conic_gradient((angle, position, items, *flags), origin_rect, current_color),
    };
    // raea: the colour space and hue path the stops are mixed in (`in oklch`, `in hsl longer hue`); sRGB by default
    let method = match gradient {
        GenericGradient::Linear { color_interpolation_method: m, .. }
        | GenericGradient::Radial { color_interpolation_method: m, .. }
        | GenericGradient::Conic { color_interpolation_method: m, .. } => m,
    };
    out.interpolation_cs = interpolation_space(method.space);
    out.hue_direction = match method.hue {
        HueInterpolationMethod::Shorter | HueInterpolationMethod::Specified => peniko::color::HueDirection::Shorter,
        HueInterpolationMethod::Longer => peniko::color::HueDirection::Longer,
        HueInterpolationMethod::Increasing => peniko::color::HueDirection::Increasing,
        HueInterpolationMethod::Decreasing => peniko::color::HueDirection::Decreasing,
    };
    resample(&mut out);
    (out, at)
}

/// Vello mixes stops in sRGB whatever the gradient's space: a gradient mixed in another space (or along another hue
/// path) gets its stops resampled there, 16 a span, and is then mixed in sRGB between them.
fn resample(g: &mut peniko::Gradient) {
    use peniko::color::{ColorSpaceTag, HueDirection};
    if g.interpolation_cs == ColorSpaceTag::Srgb && g.hue_direction == HueDirection::Shorter {
        return;
    }
    const N: usize = 16;
    let old = std::mem::take(&mut g.stops);
    for (i, s) in old.iter().enumerate() {
        g.stops.push(peniko::ColorStop { offset: s.offset, color: s.color.convert(ColorSpaceTag::Srgb) });
        let Some(next) = old.get(i + 1) else { break };
        if next.offset <= s.offset {
            continue;
        }
        let mix = s.color.interpolate(next.color, g.interpolation_cs, g.hue_direction);
        for k in 1..N {
            let t = k as f32 / N as f32;
            let color = mix.eval(t).convert(ColorSpaceTag::Srgb);
            g.stops.push(peniko::ColorStop { offset: s.offset + (next.offset - s.offset) * t, color });
        }
    }
    g.interpolation_cs = ColorSpaceTag::Srgb;
    g.hue_direction = HueDirection::Shorter;
}

fn interpolation_space(space: style::color::ColorSpace) -> peniko::color::ColorSpaceTag {
    use peniko::color::ColorSpaceTag as T;
    use style::color::ColorSpace as S;
    match space {
        S::Srgb => T::Srgb,
        S::Hsl => T::Hsl,
        S::Hwb => T::Hwb,
        S::Lab => T::Lab,
        S::Lch => T::Lch,
        S::Oklab => T::Oklab,
        S::Oklch => T::Oklch,
        S::SrgbLinear => T::LinearSrgb,
        // Vello has no linear Display P3: P3's own curve is sRGB's, so its gradients mix as P3 does
        S::DisplayP3 | S::DisplayP3Linear => T::DisplayP3,
        S::A98Rgb => T::A98Rgb,
        S::ProphotoRgb => T::ProphotoRgb,
        S::Rec2020 => T::Rec2020,
        S::XyzD50 => T::XyzD50,
        S::XyzD65 => T::XyzD65,
    }
}

fn linear_gradient(
    gradient: LinearGradient,
    rect: Rect,
    bounding_box: Rect,
    scale: f64,
    current_color: &AbsoluteColor,
) -> (peniko::Gradient, Option<Affine>) {
    let (direction, items, flags) = gradient;

    // raea: the gradient box's centre (the tile's, with background-size), not the border box's: an angled gradient
    // in a tile, or in a padding box inside borders, was laid out around the wrong point
    let _ = bounding_box;
    let center = rect.center();
    let (start, end) = match direction {
        LineDirection::Angle(angle) => {
            let angle = -angle.radians64() + std::f64::consts::PI;
            let offset_length =
                rect.width() / 2.0 * angle.sin().abs() + rect.height() / 2.0 * angle.cos().abs();
            let offset_vec = Vec2::new(angle.sin(), angle.cos()) * offset_length;
            (center - offset_vec, center + offset_vec)
        }
        LineDirection::Horizontal(horizontal) => {
            let start = Point::new(rect.x0, rect.y0 + rect.height() / 2.0);
            let end = Point::new(rect.x1, rect.y0 + rect.height() / 2.0);
            match horizontal {
                HorizontalPositionKeyword::Right => (start, end),
                HorizontalPositionKeyword::Left => (end, start),
            }
        }
        LineDirection::Vertical(vertical) => {
            let start = Point::new(rect.x0 + rect.width() / 2.0, rect.y0);
            let end = Point::new(rect.x0 + rect.width() / 2.0, rect.y1);
            match vertical {
                VerticalPositionKeyword::Top => (end, start),
                VerticalPositionKeyword::Bottom => (start, end),
            }
        }
        LineDirection::Corner(horizontal, vertical) => {
            let (start_x, end_x) = match horizontal {
                HorizontalPositionKeyword::Right => (rect.x0, rect.x1),
                HorizontalPositionKeyword::Left => (rect.x1, rect.x0),
            };
            let (start_y, end_y) = match vertical {
                VerticalPositionKeyword::Top => (rect.y1, rect.y0),
                VerticalPositionKeyword::Bottom => (rect.y0, rect.y1),
            };
            (Point::new(start_x, start_y), Point::new(end_x, end_y))
        }
    };

    let gradient_length = CSSPixelLength::new((start.distance(end) / scale) as f32);
    let repeating = flags.contains(GradientFlags::REPEATING);

    let mut gradient = peniko::Gradient::new_linear(start, end).with_extend(if repeating {
        peniko::Extend::Repeat
    } else {
        peniko::Extend::Pad
    });

    let (first_offset, last_offset) = resolve_length_color_stops(
        current_color,
        items,
        gradient_length,
        &mut gradient,
        repeating,
    );
    if repeating && gradient.stops.len() > 1 {
        gradient.kind = peniko::GradientKind::Linear(LinearGradientPosition {
            start: start + (end - start) * first_offset as f64,
            end: end + (start - end) * (1.0 - last_offset) as f64,
        });
    }

    (gradient, None)
}

fn radial_gradient(
    gradient: RadialGradient,
    rect: Rect,
    current_color: &AbsoluteColor,
) -> (peniko::Gradient, Option<Affine>) {
    let (shape, position, items, flags) = gradient;
    let repeating = flags.contains(GradientFlags::REPEATING);

    let mut gradient = peniko::Gradient::new_radial((0.0, 0.0), 1.0).with_extend(if repeating {
        peniko::Extend::Repeat
    } else {
        peniko::Extend::Pad
    });

    let (width_px, height_px) = (
        resolve_gradient_length(&position.horizontal, rect.width()),
        resolve_gradient_length(&position.vertical, rect.height()),
    );

    let gradient_scale: Option<Vec2> = match shape {
        GenericEndingShape::Circle(circle) => {
            let scale = match circle {
                GenericCircle::Extent(extent) => match extent {
                    ShapeExtent::FarthestSide => width_px
                        .max(rect.width() - width_px)
                        .max(height_px.max(rect.height() - height_px)),
                    ShapeExtent::ClosestSide => width_px
                        .min(rect.width() - width_px)
                        .min(height_px.min(rect.height() - height_px)),
                    ShapeExtent::FarthestCorner => {
                        (width_px.max(rect.width() - width_px)
                            + height_px.max(rect.height() - height_px))
                            * 0.5_f64.sqrt()
                    }
                    ShapeExtent::ClosestCorner => {
                        (width_px.min(rect.width() - width_px)
                            + height_px.min(rect.height() - height_px))
                            * 0.5_f64.sqrt()
                    }
                    _ => 0.0,
                },
                GenericCircle::Radius(radius) => radius.0.px() as f64,
            };
            Some(Vec2::new(scale, scale))
        }
        GenericEndingShape::Ellipse(ellipse) => match ellipse {
            GenericEllipse::Extent(extent) => match extent {
                ShapeExtent::FarthestCorner | ShapeExtent::FarthestSide => {
                    let mut scale = Vec2::new(
                        width_px.max(rect.width() - width_px),
                        height_px.max(rect.height() - height_px),
                    );
                    if *extent == ShapeExtent::FarthestCorner {
                        scale *= 2.0_f64.sqrt();
                    }
                    Some(scale)
                }
                ShapeExtent::ClosestCorner | ShapeExtent::ClosestSide => {
                    let mut scale = Vec2::new(
                        width_px.min(rect.width() - width_px),
                        height_px.min(rect.height() - height_px),
                    );
                    if *extent == ShapeExtent::ClosestCorner {
                        scale *= 2.0_f64.sqrt();
                    }
                    Some(scale)
                }
                _ => None,
            },
            GenericEllipse::Radii(x, y) => Some(Vec2::new(
                resolve_gradient_length(&x.0, rect.width()),
                resolve_gradient_length(&y.0, rect.height()),
            )),
        },
    };

    // raea: a degenerate ending shape as css-images-3 §3.2.2 draws it (and Chrome does): a zero circle or ellipse is a
    // very small one, a zero width or height a very thin and very long one, so the box is the last stop's colour
    // (`closest-side at left top` was the first stop's, from a singular transform)
    let is_circle = matches!(shape, GenericEndingShape::Circle(_));
    let gradient_scale = gradient_scale.map(|s| {
        const SMALL: f64 = 1.0e-3;
        const LARGE: f64 = 1.0e6;
        match (s.x <= 0.0, s.y <= 0.0) {
            (true, true) => Vec2::new(SMALL, SMALL),
            (true, false) if !is_circle => Vec2::new(SMALL, LARGE),
            (false, true) if !is_circle => Vec2::new(LARGE, SMALL),
            _ => s,
        }
    });

    let gradient_transform = {
        // If the gradient has no valid scale, we don't need to calculate the color stops
        if let Some(gradient_scale) = gradient_scale {
            let (first_offset, last_offset) = resolve_length_color_stops(
                current_color,
                items,
                CSSPixelLength::new(gradient_scale.x as f32),
                &mut gradient,
                repeating,
            );
            let scale = if repeating && gradient.stops.len() >= 2 {
                (last_offset - first_offset) as f64
            } else {
                1.0
            };
            Some(
                Affine::scale_non_uniform(gradient_scale.x * scale, gradient_scale.y * scale)
                    .then_translate(get_translation(position, rect)),
            )
        } else {
            None
        }
    };

    (gradient, gradient_transform)
}

fn conic_gradient(
    gradient: ConicGradient,
    rect: Rect,
    current_color: &AbsoluteColor,
) -> (peniko::Gradient, Option<Affine>) {
    let (angle, position, items, flags) = gradient;

    let repeating = flags.contains(GradientFlags::REPEATING);
    let mut gradient = peniko::Gradient::new_sweep((0.0, 0.0), 0.0, std::f32::consts::PI * 2.0)
        .with_extend(if repeating {
            peniko::Extend::Repeat
        } else {
            peniko::Extend::Pad
        });

    let (first_offset, last_offset) = resolve_angle_color_stops(
        current_color,
        items,
        CSSPixelLength::new(1.0),
        &mut gradient,
        repeating,
    );
    if repeating && gradient.stops.len() >= 2 {
        gradient.kind = peniko::GradientKind::Sweep(SweepGradientPosition {
            center: Point::new(0.0, 0.0),
            start_angle: std::f32::consts::PI * 2.0 * first_offset,
            end_angle: std::f32::consts::PI * 2.0 * last_offset,
        });
    }

    let gradient_transform = Some(
        Affine::rotate(angle.radians() as f64 - std::f64::consts::PI / 2.0)
            .then_translate(get_translation(position, rect)),
    );

    (gradient, gradient_transform)
}

#[inline]
fn resolve_length_color_stops(
    current_color: &AbsoluteColor,
    items: &[GradientItem<LengthPercentage>],
    gradient_length: CSSPixelLength,
    gradient: &mut Gradient,
    repeating: bool,
) -> (f32, f32) {
    resolve_color_stops(
        current_color,
        items,
        gradient_length,
        gradient,
        repeating,
        |gradient_length: CSSPixelLength, position: &LengthPercentage| -> Option<f32> {
            position
                .to_percentage_of(gradient_length)
                .map(|percentage| percentage.to_percentage().unwrap_or(0.0))
        },
    )
}

#[inline]
fn resolve_color_stops<T>(
    current_color: &AbsoluteColor,
    items: &[GradientItem<T>],
    gradient_length: CSSPixelLength,
    gradient: &mut Gradient,
    repeating: bool,
    item_resolver: impl Fn(CSSPixelLength, &T) -> Option<f32>,
) -> (f32, f32) {
    let mut hint: Option<f32> = None;

    for (idx, item) in items.iter().enumerate() {
        let (color, offset) = match item {
            GenericGradientItem::SimpleColorStop(color) => {
                // raea: one stop is that colour throughout (1 / 0 made its offset NaN, and Vello drew nothing)
                let step = if items.len() > 1 { 1.0 / (items.len() as f32 - 1.0) } else { 0.0 };
                (
                    color.resolve_to_absolute(current_color).as_dynamic_color(),
                    step * idx as f32,
                )
            }
            GenericGradientItem::ComplexColorStop { color, position } => {
                let offset = item_resolver(gradient_length, position);
                if let Some(offset) = offset {
                    (
                        color.resolve_to_absolute(current_color).as_dynamic_color(),
                        offset,
                    )
                } else {
                    continue;
                }
            }
            GenericGradientItem::InterpolationHint(position) => {
                hint = item_resolver(gradient_length, position);
                continue;
            }
        };

        if idx == 0 && !repeating && offset != 0.0 {
            gradient
                .stops
                .push(peniko::ColorStop { color, offset: 0.0 });
        }

        match hint {
            None => gradient.stops.push(peniko::ColorStop { color, offset }),
            Some(hint) => {
                let &last_stop = gradient.stops.last().unwrap();

                if hint <= last_stop.offset {
                    // Upstream code has a bug here, so we're going to do something different
                    match gradient.stops.len() {
                        0 => (),
                        1 => {
                            gradient.stops.pop();
                        }
                        _ => {
                            let prev_stop = gradient.stops[gradient.stops.len() - 2];
                            if prev_stop.offset == hint {
                                gradient.stops.pop();
                            }
                        }
                    }
                    gradient.stops.push(peniko::ColorStop {
                        color,
                        offset: hint,
                    });
                } else if hint >= offset {
                    gradient.stops.push(peniko::ColorStop {
                        color: last_stop.color,
                        offset: hint,
                    });
                    gradient.stops.push(peniko::ColorStop {
                        color,
                        offset: last_stop.offset,
                    });
                } else if hint == (last_stop.offset + offset) / 2.0 {
                    gradient.stops.push(peniko::ColorStop { color, offset });
                } else {
                    let mid_point = (hint - last_stop.offset) / (offset - last_stop.offset);
                    let mut interpolate_stop = |cur_offset: f32| {
                        let relative_offset =
                            (cur_offset - last_stop.offset) / (offset - last_stop.offset);
                        let multiplier = relative_offset.powf(0.5f32.log(mid_point));
                        let [last_r, last_g, last_b, last_a] = last_stop.color.components;
                        let [r, g, b, a] = color.components;

                        let color = Color::new([
                            (last_r + multiplier * (r - last_r)),
                            (last_g + multiplier * (g - last_g)),
                            (last_b + multiplier * (b - last_b)),
                            (last_a + multiplier * (a - last_a)),
                        ]);
                        gradient.stops.push(peniko::ColorStop {
                            color: DynamicColor::from_alpha_color(color),
                            offset: cur_offset,
                        });
                    };
                    if mid_point > 0.5 {
                        for i in 0..7 {
                            interpolate_stop(
                                last_stop.offset
                                    + (hint - last_stop.offset) * (7.0 + i as f32) / 13.0,
                            );
                        }
                        interpolate_stop(hint + (offset - hint) / 3.0);
                        interpolate_stop(hint + (offset - hint) * 2.0 / 3.0);
                    } else {
                        interpolate_stop(last_stop.offset + (hint - last_stop.offset) / 3.0);
                        interpolate_stop(last_stop.offset + (hint - last_stop.offset) * 2.0 / 3.0);
                        for i in 0..7 {
                            interpolate_stop(hint + (offset - hint) * (i as f32) / 13.0);
                        }
                    }
                    gradient.stops.push(peniko::ColorStop { color, offset });
                }
            }
        }
    }

    // Post-process the stops for repeating gradients
    if repeating && gradient.stops.len() > 1 {
        let first_offset = gradient.stops.first().unwrap().offset;
        let last_offset = gradient.stops.last().unwrap().offset;
        if first_offset != 0.0 || last_offset != 1.0 {
            let scale_inv = 1e-7_f32.max(1.0 / (last_offset - first_offset));
            for stop in &mut *gradient.stops {
                stop.offset = (stop.offset - first_offset) * scale_inv;
            }
        }
        (first_offset, last_offset)
    } else {
        // Ensure that the gradient ends at offset 1.0
        if !gradient.stops.is_empty() {
            let last_stop = &gradient.stops.last().unwrap();
            if last_stop.offset < 1.0 {
                let last_stop = ColorStop {
                    offset: 1.0,
                    ..(**last_stop)
                };
                gradient.stops.push(last_stop);
            }
        }
        (0.0, 1.0)
    }
}

#[inline]
fn resolve_angle_color_stops(
    current_color: &AbsoluteColor,
    items: &[GradientItem<AngleOrPercentage>],
    gradient_length: CSSPixelLength,
    gradient: &mut Gradient,
    repeating: bool,
) -> (f32, f32) {
    resolve_color_stops(
        current_color,
        items,
        gradient_length,
        gradient,
        repeating,
        |_gradient_length: CSSPixelLength, position: &AngleOrPercentage| -> Option<f32> {
            match position {
                AngleOrPercentage::Angle(angle) => {
                    Some(angle.radians() / (std::f64::consts::PI * 2.0) as f32)
                }
                AngleOrPercentage::Percentage(percentage) => {
                    Some(percentage.to_percentage().unwrap_or(0.0))
                }
            }
        },
    )
}

#[inline]
fn resolve_gradient_length(value: &LengthPercentage, basis: f64) -> f64 {
    // Percentage resolution can overflow even though computed lengths are finite.
    value
        .resolve(CSSPixelLength::new(basis as f32))
        .px()
        .clamp(f32::MIN, f32::MAX) as f64
}

#[inline]
fn get_translation(
    position: &GenericPosition<LengthPercentage, LengthPercentage>,
    rect: Rect,
) -> Vec2 {
    Vec2::new(
        rect.x0 + resolve_gradient_length(&position.horizontal, rect.width()),
        rect.y0 + resolve_gradient_length(&position.vertical, rect.height()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_percentage_overflow_is_clamped() {
        for percentage in [f32::MAX, f32::MIN] {
            let value = LengthPercentage::new_percent(Percentage(percentage));
            assert_eq!(resolve_gradient_length(&value, 300.0), percentage as f64);
        }
    }

    #[test]
    fn finite_gradient_lengths_are_preserved() {
        let length = LengthPercentage::new_length(CSSPixelLength::new(-12.0));
        let percentage = LengthPercentage::new_percent(Percentage(-0.5));
        assert_eq!(resolve_gradient_length(&length, 300.0), -12.0);
        assert_eq!(resolve_gradient_length(&percentage, 300.0), -150.0);
    }
}
