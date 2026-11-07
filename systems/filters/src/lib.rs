//! CSS `filter` on the GPU (docs/blitz-bevy.feature B4a). Vello draws no filters (anyrender_vello drops them), so a
//! filtered element's layer is painted into a scene of its own ([`Painter`]), drawn by Vello into a texture of its
//! device-space box, run through the filter's passes here (WGSL on Bevy's device: colour matrices, a separable
//! Gaussian blur, drop shadows, offsets, floods), and put back where the layer was as an image whose pixels Vello
//! copies from the result on the GPU (`override_image`, as H7's views). Nothing is read back.
//!
//! The passes work as Chrome's do: on premultiplied, sRGB-encoded values (no linearising), blur radii scaled by the
//! layer's transform; edges are transparent (CSS's `EdgeMode::None`).

mod painter;
mod passes;
mod premul;
mod primitives;

pub use painter::Painter;
pub use passes::Passes;
pub use premul::{PREMUL_WGSL, Premul, straight_target};

use std::sync::Arc;

use anyrender::filters::Filter;
use vello::kurbo::Affine;
use vello::peniko::{BlendMode, ImageData};


/// A filtered layer, painted apart: `scene` is its content in the box's own pixels (`origin` at 0,0), `image` the
/// placeholder its parent draws at `origin`, filled from the filtered result.
pub struct Filtered {
    pub scene: vello::Scene,
    pub filter: Arc<Filter>,
    /// device pixels per CSS pixel along the layer (its transform's scale): the blur radii and offsets in device px
    pub scale: f32,
    pub origin: (i32, i32),
    pub size: (u32, u32),
    pub image: ImageData,
    /// the filter's user space to the texture's pixels (an SVG filter's lights and noise are placed in user space)
    pub user: Affine,
    /// drawn by `Passes::linear`, not Vello: a pattern tile's linear gradient, exact
    pub linear: Option<Linear>,
    /// a layer a 3D transform's perspective places (B3 H5): `origin` and `size` are its content's, in its own space;
    /// the warp is where it lands on the page, and `image` that box's
    pub warp: Option<Warp>,
}

/// Where a warped layer lands: the page box `origin` and `size` (whole device px) and, for each of its pixels, the
/// homography back to the layer's texels (row-major 3x3, on x, y, w).
pub struct Warp {
    pub origin: (i32, i32),
    pub size: (u32, u32),
    pub to_layer: [f32; 9],
}

/// A linear gradient drawn per pixel as Chrome does (Skia): `t` at each pixel's centre, exact; a pixel on a hard stop
/// takes the earlier colour; premultiplied sRGB between stops. Vello's 512-entry ramp moves a hard stop by up to
/// 1/511 of the gradient's length and rounds ties up, which at a 4 px tile turns a checker into stripes.
pub struct Linear {
    /// from a texel's centre to the gradient's space
    pub to_gradient: Affine,
    pub start: vello::kurbo::Point,
    pub end: vello::kurbo::Point,
    pub repeat: bool,
    /// offsets and premultiplied sRGB colours, at most 8
    pub stops: Vec<(f32, [f32; 4])>,
}

impl Linear {
    /// `gradient` (its brush transform `brush`), drawn at `scale` texels per unit, when this can draw it exactly as
    /// Chrome: linear, sRGB, premultiplied, padded or repeated, at most 8 stops.
    fn of(gradient: &vello::peniko::Gradient, brush: Option<Affine>, scale: (f64, f64)) -> Option<Linear> {
        use vello::peniko::{Extend, GradientKind, InterpolationAlphaSpace, color::ColorSpaceTag, color::Srgb};
        let GradientKind::Linear(pos) = gradient.kind else { return None };
        let ok = gradient.interpolation_cs == ColorSpaceTag::Srgb
            && gradient.interpolation_alpha_space == InterpolationAlphaSpace::Premultiplied
            && matches!(gradient.extend, Extend::Pad | Extend::Repeat)
            && (1..=8).contains(&gradient.stops.len());
        if !ok {
            return None;
        }
        let stops = gradient.stops.iter().map(|s| (s.offset, s.color.to_alpha_color::<Srgb>().premultiply().components)).collect();
        let to_gradient = brush.unwrap_or(Affine::IDENTITY).inverse() * Affine::scale_non_uniform(1.0 / scale.0, 1.0 / scale.1);
        Some(Linear { to_gradient, start: pos.start, end: pos.end, repeat: gradient.extend == Extend::Repeat, stops })
    }
}

/// A filtered layer being painted: its scene, the plain layers opened inside it, and how it goes back in its parent.
struct Open {
    layer: Filtered,
    depth: u32,
    blend: BlendMode,
    alpha: f32,
}
