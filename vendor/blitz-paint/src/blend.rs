//! raea: CSS compositing (mix-blend-mode, background-blend-mode, isolation) as Vello's blend modes.

use peniko::{BlendMode, Compose, Mix};
use style::computed_values::{background_blend_mode::single_value::T as Bg, isolation::T as Isolation, mix_blend_mode::T as Css};
use style::properties::ComputedValues;

/// An element's own group's blend: its mix-blend-mode, normal for `isolation: isolate`, none when it needs no group.
pub(crate) fn element_blend(style: &ComputedValues) -> Option<BlendMode> {
    let mode = style.get_effects().mix_blend_mode;
    match (mode, style.get_box().isolation) {
        (Css::Normal, Isolation::Auto) => None,
        (Css::PlusLighter, _) => Some(BlendMode::new(Mix::Normal, Compose::Plus)),
        (mode, _) => Some(mix(mode).into()),
    }
}

fn mix(mode: Css) -> Mix {
    match mode {
        Css::Normal | Css::PlusLighter => Mix::Normal,
        Css::Multiply => Mix::Multiply,
        Css::Screen => Mix::Screen,
        Css::Overlay => Mix::Overlay,
        Css::Darken => Mix::Darken,
        Css::Lighten => Mix::Lighten,
        Css::ColorDodge => Mix::ColorDodge,
        Css::ColorBurn => Mix::ColorBurn,
        Css::HardLight => Mix::HardLight,
        Css::SoftLight => Mix::SoftLight,
        Css::Difference => Mix::Difference,
        Css::Exclusion => Mix::Exclusion,
        Css::Hue => Mix::Hue,
        Css::Saturation => Mix::Saturation,
        Css::Color => Mix::Color,
        Css::Luminosity => Mix::Luminosity,
    }
}

/// A background layer's blend with the layers under it and the background colour (background-blend-mode, its list
/// repeated to the number of layers); none for normal.
pub(crate) fn background_blend(style: &ComputedValues, layer: usize) -> Option<BlendMode> {
    let list = &style.get_background().background_blend_mode.0;
    let mode = match list.is_empty() {
        true => return None,
        false => list[layer % list.len()],
    };
    let m = match mode {
        Bg::Normal => return None,
        Bg::Multiply => Mix::Multiply,
        Bg::Screen => Mix::Screen,
        Bg::Overlay => Mix::Overlay,
        Bg::Darken => Mix::Darken,
        Bg::Lighten => Mix::Lighten,
        Bg::ColorDodge => Mix::ColorDodge,
        Bg::ColorBurn => Mix::ColorBurn,
        Bg::HardLight => Mix::HardLight,
        Bg::SoftLight => Mix::SoftLight,
        Bg::Difference => Mix::Difference,
        Bg::Exclusion => Mix::Exclusion,
        Bg::Hue => Mix::Hue,
        Bg::Saturation => Mix::Saturation,
        Bg::Color => Mix::Color,
        Bg::Luminosity => Mix::Luminosity,
    };
    Some(m.into())
}
