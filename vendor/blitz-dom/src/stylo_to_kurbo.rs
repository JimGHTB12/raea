use euclid::default::Rect;
use kurbo::{Affine, Vec2};
use style::{
    properties::generated::style_structs::Box as BoxStyleStruct,
    values::{
        computed::{CSSPixelLength, Rotate},
        generics::transform::{Scale, Translate},
    },
};

// 6. Current Transformation Matrix
//
// The transformation matrix is computed from the transform, transform-origin, translate, rotate, scale, and offset properties as follows:
//
//   - Start with the identity matrix.
//   - Translate by the computed X, Y, and Z values of transform-origin.
//   - Translate by the computed X, Y, and Z values of translate.
//   - Rotate by the computed <angle> about the specified axis of rotate.
//   - Scale by the computed X, Y, and Z values of scale.
//   - Translate and rotate by the transform specified by offset.
//   - Multiply by each of the transform functions in transform from left to right.
//   - Translate by the negated computed X, Y and Z values of transform-origin.
//
// <https://drafts.csswg.org/css-transforms-2/#ctm>
pub fn resolve_2d_transform(
    box_styles: &BoxStyleStruct,
    reference_box: Rect<CSSPixelLength>,
) -> Option<Affine> {
    let translate = match &box_styles.translate {
        Translate::None => None,
        Translate::Translate(x, y, _z) => Some(Vec2 {
            x: x.resolve(reference_box.width()).px() as f64,
            y: y.resolve(reference_box.height()).px() as f64,
        }),
    };

    let rotate = match &box_styles.rotate {
        Rotate::None => None,
        Rotate::Rotate(angle) => Some(angle.radians64()),
        // A rotation about the z axis stays in the plane; the sign of the
        // axis component decides the direction, its magnitude cancels out.
        Rotate::Rotate3D(x, y, z, angle) if *x == 0.0 && *y == 0.0 && *z != 0.0 => {
            Some(angle.radians64() * z.signum() as f64)
        }
        // TODO: support 3D transforms
        Rotate::Rotate3D(_, _, _, _) => None,
    };

    let scale_transform = match &box_styles.scale {
        Scale::None => None,
        Scale::Scale(x, y, _z) => Some(Vec2 {
            x: *x as f64,
            y: *y as f64,
        }),
    };

    let transform = if box_styles.transform.0.is_empty() {
        None
    } else {
        box_styles
            .transform
            .to_transform_3d_matrix(Some(&reference_box))
            .ok()
            // The `has_3d` flag means a 3d function appeared in the list, not that the
            // matrix needs 3d: `translate3d(x, y, 0)` sets it while resolving to a
            // plain 2d translation.
            //
            // TODO: support 3D transforms. Outside a 3d rendering context the spec
            // still applies the transform as a painting effect -- its own example
            // paints `rotateY(50deg)` narrower -- so dropping one is wrong. `is_2d()`
            // is closer than the flag, not the rule.
            // See: https://drafts.csswg.org/css-transforms-2/#3d-transform-rendering
            .filter(|(t, _has_3d)| t.is_2d())
            .map(|(t, _)| {
                // See: https://drafts.csswg.org/css-transforms-2/#two-dimensional-subset
                // And https://docs.rs/kurbo/latest/kurbo/struct.Affine.html#method.new
                Affine::new([t.m11, t.m12, t.m21, t.m22, t.m41, t.m42].map(|v| v as f64))
            })
    };

    // TODO: support the "offset" property
    // <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/offset>

    if translate.is_none() && rotate.is_none() && scale_transform.is_none() && transform.is_none() {
        return None;
    }

    // Apply the transform origin by:
    //   - Translating by the origin offset
    //   - Applying our transform
    //   - Translating by the inverse of the origin offset
    let transform_origin = &box_styles.transform_origin;
    let origin_translation = Affine::translate(Vec2 {
        x: transform_origin
            .horizontal
            .resolve(reference_box.width())
            .px() as f64,
        y: transform_origin
            .vertical
            .resolve(reference_box.height())
            .px() as f64,
    });

    let mut resolved = Affine::IDENTITY;

    if let Some(translation) = translate {
        resolved *= Affine::translate(translation)
    }

    if let Some(rotation) = rotate {
        resolved *= Affine::rotate(rotation)
    }

    if let Some(scale_transform) = scale_transform {
        resolved *= Affine::scale_non_uniform(scale_transform.x, scale_transform.y)
    }

    if let Some(transform) = transform {
        resolved *= transform;
    }

    resolved = origin_translation * resolved * origin_translation.inverse();

    if resolved != Affine::IDENTITY {
        Some(resolved)
    } else {
        None
    }
}

// raea: 3D transforms (docs/blitz-bevy.feature B3 H5). The current transformation matrix in full, as a 4x4 on column
// vectors (`m[row][col]`, CSS px), and the containing block's perspective; the painter flattens each element's onto
// the page (its 3x3 on the z = 0 plane): an affine where the result is one, a projective layer where it isn't.

/// raea: an element whose transform, flattened onto the page, is more than an affine (a perspective's), or which faces
/// away: `h` is its 3x3 (row-major, on x, y, w) from its own device px to its place in its parent's, the box's
/// position taken out as an affine transform's is; the painter draws it as a layer warped by `h`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    pub h: Option<[f64; 9]>,
    /// its back is toward the viewer (css-transforms-2 §11: the inverse's m33 below zero)
    pub back_facing: bool,
    /// in a 3D rendering context (its parent `transform-style: preserve-3d`): `h` is from the context root's parent's
    /// space, the box's position included, and `context` the same as an affine where it is one
    pub in_context: bool,
    pub context: Option<Affine>,
    /// in a 3D rendering context: how near the viewer its centre is (z over w from the context root's parent), which
    /// the painter sorts the context's planes by
    pub depth: f64,
    /// in a 3D rendering context: its box's corners (top left, top right, bottom right, bottom left) in the context root's
    /// parent's space, CSS px, x, y and z over w; None when one is at or behind the viewer. The painter splits planes
    /// that cross by them (css-transforms-2's BSP)
    pub quad: Option<[[f64; 3]; 4]>,
}

/// raea: whether an element's children share its 3D rendering context: `transform-style: preserve-3d` with none of
/// the grouping properties that force it flat (css-transforms-2 §3.1: overflow other than visible or clip, opacity,
/// filter).
pub fn preserves_3d(s: &style::properties::ComputedValues) -> bool {
    use style::values::computed::Overflow;
    let b = s.get_box();
    let shown = |o: Overflow| matches!(o, Overflow::Visible | Overflow::Clip);
    let fx = s.get_effects();
    b.transform_style == style::values::computed::TransformStyle::Preserve3d
        && shown(b.overflow_x)
        && shown(b.overflow_y)
        && fx.opacity >= 1.0
        && fx.filter.0.is_empty()
}

/// A 4x4 on column vectors, `m[row][col]`.
pub type Mat4 = [[f64; 4]; 4];

pub const IDENTITY4: Mat4 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

pub fn mul4(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut m = [[0.0; 4]; 4];
    for (r, row) in m.iter_mut().enumerate() {
        for (c, v) in row.iter_mut().enumerate() {
            *v = (0..4).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    m
}

pub fn translate4(x: f64, y: f64, z: f64) -> Mat4 {
    let mut m = IDENTITY4;
    m[0][3] = x;
    m[1][3] = y;
    m[2][3] = z;
    m
}

fn scale4(x: f64, y: f64, z: f64) -> Mat4 {
    let mut m = IDENTITY4;
    m[0][0] = x;
    m[1][1] = y;
    m[2][2] = z;
    m
}

/// `rotate3d(x, y, z, a)` as css-transforms-2 gives it (the axis normalised; a zero axis is no rotation)
fn rotate4(x: f64, y: f64, z: f64, a: f64) -> Mat4 {
    let len = (x * x + y * y + z * z).sqrt();
    if len == 0.0 {
        return IDENTITY4;
    }
    let (x, y, z) = (x / len, y / len, z / len);
    let (s, c) = a.sin_cos();
    let t = 1.0 - c;
    [
        [c + x * x * t, x * y * t - z * s, x * z * t + y * s, 0.0],
        [y * x * t + z * s, c + y * y * t, y * z * t - x * s, 0.0],
        [z * x * t - y * s, z * y * t + x * s, c + z * z * t, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// A stylo (euclid, row-vector) matrix on column vectors.
fn from_euclid(t: &euclid::default::Transform3D<f64>) -> Mat4 {
    [[t.m11, t.m21, t.m31, t.m41], [t.m12, t.m22, t.m32, t.m42], [t.m13, t.m23, t.m33, t.m43], [t.m14, t.m24, t.m34, t.m44]]
}

/// The element's current transformation matrix in full (css-transforms-2 §6, the order above
/// `resolve_2d_transform`), CSS px; None when it has no transform.
pub fn resolve_3d_transform(box_styles: &BoxStyleStruct, reference_box: Rect<CSSPixelLength>) -> Option<Mat4> {
    let mut m = IDENTITY4;
    let mut any = false;
    if let Translate::Translate(x, y, z) = &box_styles.translate {
        m = mul4(&m, &translate4(x.resolve(reference_box.width()).px() as f64, y.resolve(reference_box.height()).px() as f64, z.px() as f64));
        any = true;
    }
    match &box_styles.rotate {
        Rotate::None => {}
        Rotate::Rotate(angle) => {
            m = mul4(&m, &rotate4(0.0, 0.0, 1.0, angle.radians64()));
            any = true;
        }
        Rotate::Rotate3D(x, y, z, angle) => {
            m = mul4(&m, &rotate4(*x as f64, *y as f64, *z as f64, angle.radians64()));
            any = true;
        }
    }
    if let Scale::Scale(x, y, z) = &box_styles.scale {
        m = mul4(&m, &scale4(*x as f64, *y as f64, *z as f64));
        any = true;
    }
    if !box_styles.transform.0.is_empty() {
        let (t, _) = box_styles.transform.to_transform_3d_matrix(Some(&reference_box)).ok()?;
        m = mul4(&m, &from_euclid(&t.cast()));
        any = true;
    }
    if !any {
        return None;
    }
    let o = &box_styles.transform_origin;
    let (ox, oy, oz) = (o.horizontal.resolve(reference_box.width()).px() as f64, o.vertical.resolve(reference_box.height()).px() as f64, o.depth.px() as f64);
    Some(mul4(&mul4(&translate4(ox, oy, oz), &m), &translate4(-ox, -oy, -oz)))
}

/// raea: whether any of `translate`, `rotate`, `scale` or `transform` is set (resolve_3d_transform's matrix isn't None).
pub fn has_transform(box_styles: &BoxStyleStruct) -> bool {
    !matches!(box_styles.translate, Translate::None) || !matches!(box_styles.rotate, Rotate::None) || !matches!(box_styles.scale, Scale::None) || !box_styles.transform.0.is_empty()
}

/// An element's `perspective` as the matrix its children's transforms are seen through: about its perspective-origin
/// in its border box (`size`, CSS px). None for `none`. raea: `scroll` is the element's scroll offset: its children sit
/// in its content, which scrolls under the box, and the origin stays put in the box (perspective-origin-scroll).
pub fn resolve_perspective(box_styles: &BoxStyleStruct, size: (f32, f32), scroll: (f64, f64)) -> Option<Mat4> {
    use style::values::generics::box_::GenericPerspective;
    let GenericPerspective::Length(d) = &box_styles.perspective else { return None };
    // a perspective under 1px is 1px (css-transforms-2 §8.2)
    let d = (d.0.px() as f64).max(1.0);
    let o = &box_styles.perspective_origin;
    let (ox, oy) = (
        o.horizontal.resolve(CSSPixelLength::new(size.0)).px() as f64 + scroll.0,
        o.vertical.resolve(CSSPixelLength::new(size.1)).px() as f64 + scroll.1,
    );
    let mut p = IDENTITY4;
    p[3][2] = -1.0 / d;
    Some(mul4(&mul4(&translate4(ox, oy, 0.0), &p), &translate4(-ox, -oy, 0.0)))
}

/// `m` flattened onto the page: its 3x3 on the z = 0 plane (rows and columns x, y, w), row-major.
pub fn project(m: &Mat4) -> [f64; 9] {
    [m[0][0], m[0][1], m[0][3], m[1][0], m[1][1], m[1][3], m[3][0], m[3][1], m[3][3]]
}

pub fn det3(h: &[f64; 9]) -> f64 {
    h[0] * (h[4] * h[8] - h[5] * h[7]) - h[1] * (h[3] * h[8] - h[5] * h[6]) + h[2] * (h[3] * h[7] - h[4] * h[6])
}

pub fn det4(m: &Mat4) -> f64 {
    let minor = |c: usize| {
        let cols: Vec<usize> = (0..4).filter(|&k| k != c).collect();
        let h: [f64; 9] = std::array::from_fn(|i| m[1 + i / 3][cols[i % 3]]);
        det3(&h)
    };
    (0..4).map(|c| if c % 2 == 0 { 1.0 } else { -1.0 } * m[0][c] * minor(c)).sum()
}
