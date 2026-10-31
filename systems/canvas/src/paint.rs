//! A call's state in tiny-skia's terms: its transform, path, colour or gradient, blend, stroke and clip, and a
//! shadow's blur.

use super::*;

pub(crate) fn f(v: &Value) -> f32 {
    v.as_f64().unwrap_or(0.0) as f32
}

pub(crate) fn ts_of(v: &Value) -> sk::Transform {
    let a = v.as_array().map(|a| a.iter().map(f).collect::<Vec<_>>()).unwrap_or_default();
    if a.len() < 6 {
        return sk::Transform::identity();
    }
    sk::Transform::from_row(a[0], a[1], a[2], a[3], a[4], a[5])
}

pub(crate) fn path_of(d: &Value) -> Option<sk::Path> {
    let d: Vec<f32> = d.as_array()?.iter().map(f).collect();
    let mut b = sk::PathBuilder::new();
    let mut i = 0;
    while i < d.len() {
        let op = d[i] as u32;
        i += 1;
        let need = match op {
            0 | 1 => 2,
            2 => 4,
            3 => 6,
            _ => 0,
        };
        if i + need > d.len() {
            break;
        }
        let a = &d[i..i + need];
        match op {
            0 => b.move_to(a[0], a[1]),
            1 => b.line_to(a[0], a[1]),
            2 => b.quad_to(a[0], a[1], a[2], a[3]),
            3 => b.cubic_to(a[0], a[1], a[2], a[3], a[4], a[5]),
            _ => b.close(),
        }
        i += need;
    }
    b.finish()
}

pub(crate) fn color_of(v: &Value, alpha: f32) -> Option<sk::Color> {
    let a = v.as_array()?;
    let g = |i: usize| a.get(i).and_then(Value::as_f64).unwrap_or(0.0) as f32;
    sk::Color::from_rgba((g(0) / 255.0).clamp(0.0, 1.0), (g(1) / 255.0).clamp(0.0, 1.0), (g(2) / 255.0).clamp(0.0, 1.0), (a.get(3).and_then(Value::as_f64).unwrap_or(1.0) as f32 * alpha).clamp(0.0, 1.0))
}

pub(crate) fn shader_of(v: &Value, alpha: f32) -> Option<sk::Shader<'static>> {
    if v.is_array() {
        return color_of(v, alpha).map(sk::Shader::SolidColor);
    }
    let stops: Vec<sk::GradientStop> = v["stops"].as_array()?.iter().filter_map(|s| Some(sk::GradientStop::new(f(s.get(0)?), color_of(s.get(1)?, alpha)?))).collect();
    if stops.is_empty() {
        return Some(sk::Shader::SolidColor(sk::Color::TRANSPARENT));
    }
    let ts = ts_of(&v["ts"]);
    let pt = |x: &str, y: &str| sk::Point::from_xy(f(&v[x]), f(&v[y]));
    match v["k"].as_str()? {
        "linear" => sk::LinearGradient::new(pt("x0", "y0"), pt("x1", "y1"), stops, sk::SpreadMode::Pad, ts),
        "radial" => sk::RadialGradient::new(pt("x0", "y0"), f(&v["r0"]), pt("x1", "y1"), f(&v["r1"]), stops, sk::SpreadMode::Pad, ts),
        "conic" => {
            let t = ts.pre_translate(f(&v["x"]), f(&v["y"])).pre_rotate(f(&v["a"]).to_degrees());
            sk::SweepGradient::new(sk::Point::zero(), 0.0, 360.0, stops, sk::SpreadMode::Pad, t)
        }
        _ => None,
    }
}

pub(crate) fn blend(s: &str) -> sk::BlendMode {
    use sk::BlendMode as B;
    match s {
        "source-in" => B::SourceIn,
        "source-out" => B::SourceOut,
        "source-atop" => B::SourceAtop,
        "destination-over" => B::DestinationOver,
        "destination-in" => B::DestinationIn,
        "destination-out" => B::DestinationOut,
        "destination-atop" => B::DestinationAtop,
        "lighter" => B::Plus,
        "copy" => B::Source,
        "xor" => B::Xor,
        "multiply" => B::Multiply,
        "screen" => B::Screen,
        "overlay" => B::Overlay,
        "darken" => B::Darken,
        "lighten" => B::Lighten,
        "color-dodge" => B::ColorDodge,
        "color-burn" => B::ColorBurn,
        "hard-light" => B::HardLight,
        "soft-light" => B::SoftLight,
        "difference" => B::Difference,
        "exclusion" => B::Exclusion,
        "hue" => B::Hue,
        "saturation" => B::Saturation,
        "color" => B::Color,
        "luminosity" => B::Luminosity,
        _ => B::SourceOver,
    }
}

pub(crate) fn stroke_of(v: &Value) -> sk::Stroke {
    let dash: Vec<f32> = v["dash"].as_array().map(|a| a.iter().map(f).collect()).unwrap_or_default();
    sk::Stroke {
        width: f(&v["width"]).max(0.0),
        miter_limit: v["miter"].as_f64().unwrap_or(10.0) as f32,
        line_cap: match v["cap"].as_str() {
            Some("round") => sk::LineCap::Round,
            Some("square") => sk::LineCap::Square,
            _ => sk::LineCap::Butt,
        },
        line_join: match v["join"].as_str() {
            Some("round") => sk::LineJoin::Round,
            Some("bevel") => sk::LineJoin::Bevel,
            _ => sk::LineJoin::Miter,
        },
        dash: if dash.is_empty() { None } else { sk::StrokeDash::new(dash, f(&v["offset"])) },
    }
}

/// The clip a call is drawn through: its paths intersected (None: no clip).
pub(crate) fn clip_of(st: &Value, w: u32, h: u32) -> Option<sk::Mask> {
    let clips = st["clip"].as_array()?;
    if clips.is_empty() {
        return None;
    }
    let mut mask = sk::Mask::new(w, h)?;
    for (i, c) in clips.iter().enumerate() {
        let rule = if c[1].as_str() == Some("evenodd") { sk::FillRule::EvenOdd } else { sk::FillRule::Winding };
        let Some(p) = path_of(&c[0]) else {
            mask.clear();
            return Some(mask);
        };
        if i == 0 {
            mask.fill_path(&p, rule, true, sk::Transform::identity());
        } else {
            mask.intersect_path(&p, rule, true, sk::Transform::identity());
        }
    }
    Some(mask)
}

/// A box blur run three times (close to a Gaussian of sigma `s`), on a pixmap's premultiplied channels.
pub(crate) fn blur(pm: &mut sk::Pixmap, sigma: f32) {
    if sigma < 0.5 {
        return;
    }
    // the three box sizes for a Gaussian (W3C's filter effects: d = floor(s * 3 * sqrt(2 * pi) / 4 + 0.5))
    let d = (sigma * 3.0 * (2.0 * std::f32::consts::PI).sqrt() / 4.0 + 0.5).floor() as i32;
    if d < 1 {
        return;
    }
    let (w, h) = (pm.width() as i32, pm.height() as i32);
    let data = pm.data_mut();
    let mut buf = vec![0f32; data.len()];
    for (i, v) in data.iter().enumerate() {
        buf[i] = *v as f32;
    }
    let pass = |buf: &mut Vec<f32>, horizontal: bool, r_left: i32, r_right: i32| {
        let (len, lines) = if horizontal { (w, h) } else { (h, w) };
        let mut line = vec![0f32; (len * 4) as usize];
        for l in 0..lines {
            let idx = |i: i32| -> usize { if horizontal { ((l * w + i) * 4) as usize } else { ((i * w + l) * 4) as usize } };
            for c in 0..4 {
                let mut sum = 0f32;
                let n = (r_left + r_right + 1) as f32;
                for i in -r_left..=r_right {
                    if (0..len).contains(&i) {
                        sum += buf[idx(i) + c];
                    }
                }
                for i in 0..len {
                    line[(i * 4) as usize + c as usize] = sum / n;
                    let out = i - r_left;
                    let inn = i + r_right + 1;
                    if (0..len).contains(&out) {
                        sum -= buf[idx(out) + c];
                    }
                    if (0..len).contains(&inn) {
                        sum += buf[idx(inn) + c];
                    }
                }
            }
            for i in 0..len {
                for c in 0..4 {
                    buf[idx(i) + c] = line[(i * 4) as usize + c];
                }
            }
        }
    };
    let (a, b) = if d % 2 == 1 { ((d / 2, d / 2), (d / 2, d / 2)) } else { ((d / 2, d / 2 - 1), (d / 2 - 1, d / 2)) };
    for horizontal in [true, false] {
        let boxes = if d % 2 == 1 { [a, a, a] } else { [a, b, (d / 2, d / 2)] };
        for (l, r) in boxes {
            pass(&mut buf, horizontal, l, r);
        }
    }
    for (i, v) in data.iter_mut().enumerate() {
        *v = buf[i].round().clamp(0.0, 255.0) as u8;
    }
    // keep each pixel premultiplied (colour no more than alpha)
    for px in data.chunks_mut(4) {
        let a = px[3];
        for c in &mut px[..3] {
            *c = (*c).min(a);
        }
    }
}
