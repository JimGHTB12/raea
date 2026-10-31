//! ctx.filter (sdk/ui/src/canvas/filter.ts parses it): a call's layer run through its CSS filter functions in order,
//! before its shadow and compositing, as the canvas spec orders them. The colour functions work on sRGB values, as
//! Chrome applies CSS filter functions (the Filter Effects spec's matrices, not in linearRGB); blur and drop-shadow's
//! lengths are bitmap pixels, untouched by the transform, as a shadow's are.

use serde_json::Value;
use tiny_skia as sk;

use crate::paint::{blur, color_of};

/// `pm` (a call's layer, premultiplied) through `list`, each function in turn.
pub(crate) fn apply(pm: &mut sk::Pixmap, list: &[Value]) {
    for f in list {
        let Some(a) = f.as_array() else { continue };
        let n = |i: usize| a.get(i).and_then(Value::as_f64).unwrap_or(0.0) as f32;
        match a.first().and_then(Value::as_str) {
            Some("blur") => blur(pm, n(1)),
            Some("drop-shadow") => drop_shadow(pm, n(1), n(2), n(3), a.get(4).unwrap_or(&Value::Null)),
            Some(name) => {
                let Some(m) = matrix(name, n(1)) else { continue };
                colours(pm, &m);
            }
            None => {}
        }
    }
}

/// A colour function as a 4x5 matrix on straight RGBA in 0-1 (the spec's feColorMatrix and feComponentTransfer
/// equivalents), rows R, G, B, A.
fn matrix(name: &str, v: f32) -> Option<[[f32; 5]; 4]> {
    let id_a = [0.0, 0.0, 0.0, 1.0, 0.0];
    let rgb = |m: [[f32; 3]; 3]| [[m[0][0], m[0][1], m[0][2], 0.0, 0.0], [m[1][0], m[1][1], m[1][2], 0.0, 0.0], [m[2][0], m[2][1], m[2][2], 0.0, 0.0], id_a];
    let linear = |slope: f32, icpt: f32| [[slope, 0.0, 0.0, 0.0, icpt], [0.0, slope, 0.0, 0.0, icpt], [0.0, 0.0, slope, 0.0, icpt], id_a];
    Some(match name {
        "brightness" => linear(v, 0.0),
        "contrast" => linear(v, 0.5 - 0.5 * v),
        "invert" => linear(1.0 - 2.0 * v, v),
        "opacity" => [[1.0, 0.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 0.0, v, 0.0]],
        "grayscale" => {
            let k = 1.0 - v;
            rgb([[0.2126 + 0.7874 * k, 0.7152 - 0.7152 * k, 0.0722 - 0.0722 * k], [0.2126 - 0.2126 * k, 0.7152 + 0.2848 * k, 0.0722 - 0.0722 * k], [0.2126 - 0.2126 * k, 0.7152 - 0.7152 * k, 0.0722 + 0.9278 * k]])
        }
        "sepia" => {
            let k = 1.0 - v;
            rgb([[0.393 + 0.607 * k, 0.769 - 0.769 * k, 0.189 - 0.189 * k], [0.349 - 0.349 * k, 0.686 + 0.314 * k, 0.168 - 0.168 * k], [0.272 - 0.272 * k, 0.534 - 0.534 * k, 0.131 + 0.869 * k]])
        }
        "saturate" => rgb([[0.213 + 0.787 * v, 0.715 - 0.715 * v, 0.072 - 0.072 * v], [0.213 - 0.213 * v, 0.715 + 0.285 * v, 0.072 - 0.072 * v], [0.213 - 0.213 * v, 0.715 - 0.715 * v, 0.072 + 0.928 * v]]),
        "hue-rotate" => {
            let (s, c) = v.to_radians().sin_cos();
            rgb([
                [0.213 + c * 0.787 - s * 0.213, 0.715 - c * 0.715 - s * 0.715, 0.072 - c * 0.072 + s * 0.928],
                [0.213 - c * 0.213 + s * 0.143, 0.715 + c * 0.285 + s * 0.140, 0.072 - c * 0.072 - s * 0.283],
                [0.213 - c * 0.213 - s * 0.787, 0.715 - c * 0.715 + s * 0.715, 0.072 + c * 0.928 + s * 0.072],
            ])
        }
        _ => return None,
    })
}

/// Each pixel's straight colour through `m`, clamped, and premultiplied again.
fn colours(pm: &mut sk::Pixmap, m: &[[f32; 5]; 4]) {
    for px in pm.pixels_mut() {
        let c = px.demultiply();
        let i = [c.red() as f32 / 255.0, c.green() as f32 / 255.0, c.blue() as f32 / 255.0, c.alpha() as f32 / 255.0];
        let o = |r: &[f32; 5]| ((r[0] * i[0] + r[1] * i[1] + r[2] * i[2] + r[3] * i[3] + r[4]).clamp(0.0, 1.0) * 255.0).round() as u8;
        let (r, g, b, a) = (o(&m[0]), o(&m[1]), o(&m[2]), o(&m[3]));
        *px = sk::ColorU8::from_rgba(r, g, b, a).premultiply();
    }
}

/// The layer's alpha in `colour`, offset and blurred, under the layer; `radius` is the blur radius, a Gaussian of
/// half its size (the spec's feGaussianBlur stdDeviation for drop-shadow, as box-shadow's).
fn drop_shadow(pm: &mut sk::Pixmap, x: f32, y: f32, radius: f32, colour: &Value) {
    let (w, h) = (pm.width(), pm.height());
    let (Some(mut s), Some(col)) = (sk::Pixmap::new(w, h), color_of(colour, 1.0)) else { return };
    let mask = sk::Mask::from_pixmap(pm.as_ref(), sk::MaskType::Alpha);
    let mut p = sk::Paint::default();
    p.set_color(col);
    s.fill_rect(sk::Rect::from_xywh(0.0, 0.0, w as f32, h as f32).unwrap(), &p, sk::Transform::identity(), Some(&mask));
    let Some(mut out) = sk::Pixmap::new(w, h) else { return };
    out.draw_pixmap(0, 0, s.as_ref(), &sk::PixmapPaint::default(), sk::Transform::from_translate(x, y), None);
    blur(&mut out, radius / 2.0);
    out.draw_pixmap(0, 0, pm.as_ref(), &sk::PixmapPaint::default(), sk::Transform::identity(), None);
    *pm = out;
}
