//! raea: a group's SVG filters as one anyrender graph for raea-page's GPU passes; the primitives anyrender's own
//! structs carry too little of go as [`SvgPrimitive`]s.

use anyrender::filters::svg::{Convolve, Light, Lighting, SvgPrimitive, Transfer, Turbulence};
use anyrender::filters::{Filter, FilterEffect, FilterInput, FilterInputs, FilterSource};
use crate::render::luminance_to_alpha;

/// A group's filters, or a CSS list's functions (the filters not in `elements`) in the colour space of the filter before them.
pub(crate) fn filter_in(g: &usvg::Group, elements: Option<&[String]>) -> Option<(Filter, kurbo::Rect)> {
    let first = g.filters().first()?;
    // the colour space the last primitive worked in
    let mut was_linear = false;
    let r = first.rect();
    let region = kurbo::Rect::new(r.left() as f64, r.top() as f64, r.right() as f64, r.bottom() as f64);
    let mut f = Filter::empty();
    let mut last: Option<anyrender::filters::FilterId> = None;
    for filter in g.filters() {
        let source = last;
        let mut named: std::collections::HashMap<&str, anyrender::filters::FilterId> = Default::default();
        // each result's subregion (what feTile repeats), the last one's
        let mut rects: std::collections::HashMap<&str, usvg::NonZeroRect> = Default::default();
        let mut last_rect = filter.rect();
        for p in filter.primitives() {
            let function = elements.is_some_and(|e| !e.iter().any(|id| id == filter.id()));
            let linear = if function { was_linear } else { p.color_interpolation() == usvg::filter::ColorInterpolation::LinearRGB };
            was_linear = linear;
            let input = |i: &usvg::filter::Input, named: &std::collections::HashMap<&str, anyrender::filters::FilterId>, prev: Option<anyrender::filters::FilterId>| match i {
                usvg::filter::Input::SourceGraphic => source.map(FilterInput::Result).unwrap_or(FilterInput::Source(FilterSource::SourceGraphic)),
                usvg::filter::Input::SourceAlpha => FilterInput::Source(FilterSource::SourceAlpha),
                usvg::filter::Input::Reference(n) => named.get(n.as_str()).or(prev.as_ref()).copied().map(FilterInput::Result).unwrap_or(FilterInput::Source(FilterSource::SourceGraphic)),
            };
            let prev = last;
            use usvg::filter::Kind;
            let id = match p.kind() {
                Kind::GaussianBlur(b) => f.add_in(
                    FilterEffect::GaussianBlur(anyrender::filters::blur::GaussianBlurFilter { std_deviation: b.std_dev_x().get(), std_deviation_y: Some(b.std_dev_y().get()), edge_mode: anyrender::filters::EdgeMode::None }),
                    FilterInputs::single(input(b.input(), &named, prev)),
                    linear,
                ),
                Kind::ColorMatrix(m) => {
                    let k = match m.kind() {
                        usvg::filter::ColorMatrixKind::Matrix(v) => v.as_slice().try_into().unwrap_or([0.0; 20]),
                        usvg::filter::ColorMatrixKind::Saturate(s) => saturate(s.get()),
                        usvg::filter::ColorMatrixKind::HueRotate(d) => hue_rotate(*d),
                        usvg::filter::ColorMatrixKind::LuminanceToAlpha => luminance_to_alpha(false),
                    };
                    f.add_in(FilterEffect::ColorMatrix(anyrender::filters::color_transformation::ColorMatrix(k)), FilterInputs::single(input(m.input(), &named, prev)), linear)
                }
                Kind::DropShadow(d) => {
                    let c = d.color();
                    let color = peniko::color::AlphaColor::from_rgba8(c.red, c.green, c.blue, (d.opacity().get() * 255.0).round() as u8);
                    f.add_in(FilterEffect::drop_shadow(d.dx(), d.dy(), d.std_dev_x().get(), color), FilterInputs::single(input(d.input(), &named, prev)), linear)
                }
                Kind::Offset(o) => f.add_in(FilterEffect::Offset(kurbo::Vec2::new(o.dx() as f64, o.dy() as f64)), FilterInputs::single(input(o.input(), &named, prev)), linear),
                Kind::Flood(fl) => {
                    let c = fl.color();
                    let color = peniko::color::AlphaColor::from_rgba8(c.red, c.green, c.blue, (fl.opacity().get() * 255.0).round() as u8);
                    f.add_in(FilterEffect::Flood(color), FilterInputs::NONE, linear)
                }
                Kind::Composite(c) => {
                    use anyrender::filters::composite::{ArithmeticCompositeOperator, CompositeOperator as Op};
                    let op = match c.operator() {
                        usvg::filter::CompositeOperator::Over => Op::Over,
                        usvg::filter::CompositeOperator::In => Op::In,
                        usvg::filter::CompositeOperator::Out => Op::Out,
                        usvg::filter::CompositeOperator::Atop => Op::Atop,
                        usvg::filter::CompositeOperator::Xor => Op::Xor,
                        usvg::filter::CompositeOperator::Arithmetic { k1, k2, k3, k4 } => Op::Arithmetic(ArithmeticCompositeOperator { k1, k2, k3, k4 }),
                    };
                    f.add_in(FilterEffect::Composite(op), FilterInputs::dual(input(c.input1(), &named, prev), input(c.input2(), &named, prev)), linear)
                }
                Kind::Merge(m) => {
                    // each input over the ones before it
                    let mut acc: Option<anyrender::filters::FilterId> = None;
                    for i in m.inputs() {
                        let this = input(i, &named, prev);
                        acc = Some(match acc {
                            None => f.add_in(FilterEffect::Offset(kurbo::Vec2::ZERO), FilterInputs::single(this), false),
                            Some(below) => f.add_in(FilterEffect::Composite(anyrender::filters::composite::CompositeOperator::Over), FilterInputs::dual(this, FilterInput::Result(below)), false),
                        });
                    }
                    acc?
                }
                // feImage isn't drawn: nothing, as Chrome shows while the image loads
                Kind::Image(_) => f.add_in(FilterEffect::Flood(peniko::color::AlphaColor::TRANSPARENT), FilterInputs::NONE, linear),
                Kind::Tile(t) => {
                    let r = match t.input() {
                        usvg::filter::Input::Reference(n) => rects.get(n.as_str()).copied().unwrap_or(last_rect),
                        usvg::filter::Input::SourceGraphic | usvg::filter::Input::SourceAlpha => filter.rect(),
                    };
                    let tile = SvgPrimitive::Tile { rect: [r.x(), r.y(), r.width(), r.height()] };
                    f.add_in(FilterEffect::Svg(Box::new(tile)), FilterInputs::single(input(t.input(), &named, prev)), linear)
                }
                k => match primitive(k, linear, p.rect()) {
                    Some((prim, ins)) => {
                        let ins = match ins.as_slice() {
                            [] => FilterInputs::NONE,
                            [a] => FilterInputs::single(input(a, &named, prev)),
                            [a, b, ..] => FilterInputs::dual(input(a, &named, prev), input(b, &named, prev)),
                        };
                        f.add_in(FilterEffect::Svg(Box::new(prim)), ins, linear)
                    }
                    // nothing else is left undrawn: the input as it is
                    None => f.add_in(FilterEffect::Offset(kurbo::Vec2::ZERO), FilterInputs::single(prev.map(FilterInput::Result).unwrap_or(FilterInput::Source(FilterSource::SourceGraphic))), false),
                },
            };
            if !p.result().is_empty() {
                rects.insert(p.result(), p.rect());
                named.insert(p.result(), id);
            }
            last = Some(id);
            last_rect = p.rect();
        }
    }
    f.set_output(last?);
    Some((f, region))
}

/// feColorMatrix's saturate.
fn saturate(s: f32) -> [f32; 20] {
    [
        0.213 + 0.787 * s, 0.715 - 0.715 * s, 0.072 - 0.072 * s, 0.0, 0.0,
        0.213 - 0.213 * s, 0.715 + 0.285 * s, 0.072 - 0.072 * s, 0.0, 0.0,
        0.213 - 0.213 * s, 0.715 - 0.715 * s, 0.072 + 0.928 * s, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

/// feColorMatrix's hueRotate, `deg` degrees.
fn hue_rotate(deg: f32) -> [f32; 20] {
    let (sn, cs) = deg.to_radians().sin_cos();
    [
        0.213 + cs * 0.787 - sn * 0.213, 0.715 - cs * 0.715 - sn * 0.715, 0.072 - cs * 0.072 + sn * 0.928, 0.0, 0.0,
        0.213 - cs * 0.213 + sn * 0.143, 0.715 + cs * 0.285 + sn * 0.140, 0.072 - cs * 0.072 - sn * 0.283, 0.0, 0.0,
        0.213 - cs * 0.213 - sn * 0.787, 0.715 - cs * 0.715 + sn * 0.715, 0.072 + cs * 0.928 + sn * 0.072, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

/// The primitives anyrender's structs can't carry, in full, with their inputs; `linear`: the primitive works in
/// linearRGB (its lighting colour converted), `rect` its subregion (a stitched turbulence's tile).
fn primitive(k: &usvg::filter::Kind, linear: bool, rect: usvg::NonZeroRect) -> Option<(SvgPrimitive, Vec<&usvg::filter::Input>)> {
    use usvg::filter::{EdgeMode, Kind, LightSource};
    let channel = |c: usvg::filter::ColorChannel| c as u8;
    let transfer = |t: &usvg::filter::TransferFunction| match t {
        usvg::filter::TransferFunction::Identity => Transfer::Identity,
        usvg::filter::TransferFunction::Table(v) => Transfer::Table(v.clone()),
        usvg::filter::TransferFunction::Discrete(v) => Transfer::Discrete(v.clone()),
        usvg::filter::TransferFunction::Linear { slope, intercept } => Transfer::Linear { slope: *slope, intercept: *intercept },
        usvg::filter::TransferFunction::Gamma { amplitude, exponent, offset } => Transfer::Gamma { amplitude: *amplitude, exponent: *exponent, offset: *offset },
    };
    let lighting = |specular, surface_scale, constant, exponent, c: usvg::Color, light: LightSource| {
        let lin = |v: u8| {
            let s = v as f32 / 255.0;
            if !linear { s } else if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
        };
        let light = match light {
            LightSource::DistantLight(d) => Light::Distant { azimuth: d.azimuth, elevation: d.elevation },
            LightSource::PointLight(p) => Light::Point { at: [p.x, p.y, p.z] },
            LightSource::SpotLight(s) => Light::Spot {
                at: [s.x, s.y, s.z],
                points_at: [s.points_at_x, s.points_at_y, s.points_at_z],
                exponent: s.specular_exponent.get(),
                cone: s.limiting_cone_angle,
            },
        };
        SvgPrimitive::Lighting(Lighting { specular, surface_scale, constant, exponent, color: [lin(c.red), lin(c.green), lin(c.blue)], light })
    };
    Some(match k {
        Kind::Blend(b) => (SvgPrimitive::Blend(blend(b.mode())), vec![b.input1(), b.input2()]),
        Kind::Morphology(m) => (
            SvgPrimitive::Morphology { dilate: m.operator() == usvg::filter::MorphologyOperator::Dilate, rx: m.radius_x().get(), ry: m.radius_y().get() },
            vec![m.input()],
        ),
        Kind::ConvolveMatrix(c) => {
            let m = c.matrix();
            let edge = match c.edge_mode() {
                EdgeMode::None => 0,
                EdgeMode::Duplicate => 1,
                EdgeMode::Wrap => 2,
            };
            let kernel = m.data().to_vec();
            let convolve = Convolve { columns: m.columns(), rows: m.rows(), kernel, divisor: c.divisor().get(), bias: c.bias(), target: (m.target_x(), m.target_y()), edge, preserve_alpha: c.preserve_alpha() };
            (SvgPrimitive::Convolve(convolve), vec![c.input()])
        }
        Kind::Turbulence(t) => {
            let stitch = t.stitch_tiles().then(|| [rect.x(), rect.y(), rect.width(), rect.height()]);
            let turbulence = Turbulence {
                frequency: (t.base_frequency_x().get(), t.base_frequency_y().get()),
                octaves: t.num_octaves(),
                seed: t.seed(),
                fractal: t.kind() == usvg::filter::TurbulenceKind::FractalNoise,
                stitch,
            };
            (SvgPrimitive::Turbulence(turbulence), vec![])
        }
        Kind::DisplacementMap(d) => (
            SvgPrimitive::Displacement { scale: d.scale(), x: channel(d.x_channel_selector()), y: channel(d.y_channel_selector()) },
            vec![d.input1(), d.input2()],
        ),
        Kind::ComponentTransfer(c) => (SvgPrimitive::Transfer([transfer(c.func_r()), transfer(c.func_g()), transfer(c.func_b()), transfer(c.func_a())]), vec![c.input()]),
        Kind::DiffuseLighting(d) => (lighting(false, d.surface_scale(), d.diffuse_constant(), 1.0, d.lighting_color(), d.light_source()), vec![d.input()]),
        Kind::SpecularLighting(s) => (lighting(true, s.surface_scale(), s.specular_constant(), s.specular_exponent(), s.lighting_color(), s.light_source()), vec![s.input()]),
        _ => return None,
    })
}

fn blend(m: usvg::BlendMode) -> peniko::Mix {
    use peniko::Mix;
    use usvg::BlendMode as B;
    match m {
        B::Normal => Mix::Normal,
        B::Multiply => Mix::Multiply,
        B::Screen => Mix::Screen,
        B::Overlay => Mix::Overlay,
        B::Darken => Mix::Darken,
        B::Lighten => Mix::Lighten,
        B::ColorDodge => Mix::ColorDodge,
        B::ColorBurn => Mix::ColorBurn,
        B::HardLight => Mix::HardLight,
        B::SoftLight => Mix::SoftLight,
        B::Difference => Mix::Difference,
        B::Exclusion => Mix::Exclusion,
        B::Hue => Mix::Hue,
        B::Saturation => Mix::Saturation,
        B::Color => Mix::Color,
        B::Luminosity => Mix::Luminosity,
    }
}
