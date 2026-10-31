//! raea: SVG filter primitives carried in full: the upstream structs above keep one radius, one frequency and no
//! light colour, too little to draw feMorphology, feTurbulence or the lighting filters as Chrome does.
//! Coordinates are in the filtered element's user space; colours straight, in the primitive's colour space.

use peniko::Mix;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SvgPrimitive {
    /// feBlend: the primary input over the secondary, mixed by `Mix`
    Blend(Mix),
    /// feMorphology: the max (dilate) or min (erode) over a box `rx` by `ry` a side
    Morphology { dilate: bool, rx: f32, ry: f32 },
    Convolve(Convolve),
    Turbulence(Turbulence),
    /// feDisplacementMap: the primary input displaced by the secondary's channels (0 r, 1 g, 2 b, 3 a)
    Displacement { scale: f32, x: u8, y: u8 },
    /// feComponentTransfer: r, g, b, a
    Transfer([Transfer; 4]),
    Lighting(Lighting),
    /// feTile: the input's subregion `rect` (x, y, width, height) repeated
    Tile { rect: [f32; 4] },
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Convolve {
    pub columns: u32,
    pub rows: u32,
    /// row-major, as written (the pass flips it)
    pub kernel: Vec<f32>,
    pub divisor: f32,
    pub bias: f32,
    pub target: (u32, u32),
    /// 0 none, 1 duplicate, 2 wrap
    pub edge: u8,
    pub preserve_alpha: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Turbulence {
    pub frequency: (f32, f32),
    pub octaves: u32,
    pub seed: i32,
    pub fractal: bool,
    /// the tile stitched to (the primitive's region), when stitchTiles="stitch"
    pub stitch: Option<[f32; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Transfer {
    Identity,
    Table(Vec<f32>),
    Discrete(Vec<f32>),
    Linear { slope: f32, intercept: f32 },
    Gamma { amplitude: f32, exponent: f32, offset: f32 },
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lighting {
    pub specular: bool,
    pub surface_scale: f32,
    /// diffuseConstant or specularConstant
    pub constant: f32,
    pub exponent: f32,
    /// straight rgb, in the primitive's colour space
    pub color: [f32; 3],
    pub light: Light,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Light {
    /// degrees
    Distant { azimuth: f32, elevation: f32 },
    Point { at: [f32; 3] },
    Spot { at: [f32; 3], points_at: [f32; 3], exponent: f32, cone: Option<f32> },
}

impl SvgPrimitive {
    /// How far past its input the primitive draws, in user units.
    pub fn reach(&self) -> f32 {
        match self {
            SvgPrimitive::Morphology { dilate: true, rx, ry } => rx.max(*ry),
            SvgPrimitive::Displacement { scale, .. } => scale.abs() / 2.0,
            _ => 0.0,
        }
    }
}
