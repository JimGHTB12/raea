//! raea: bevy_ui's drop shadow (bevy_ui_render box_shadow.wesl, after Evan Wallace's rounded-box shadow) on the CPU, for a
//! node marked data-raea-bevy-shadow="SAMPLES". Bevy draws a quad the shadow's size plus 6 blur radii, and each pixel's
//! coverage is a Gaussian (standard deviation the blur radius) of the rounded box: across, the exact erf integral of a
//! row of it; down, the Gaussian summed in SAMPLES steps (BoxShadowSamples) over the 3 deviations each side, clamped to
//! the box. Vello's analytic shadow is a different approximation of that blur, so the shadow is drawn from this bitmap,
//! in f32 as the shader computes it, each pixel at its centre.
use blitz_dom::node::RasterImageData;
use std::sync::Arc;

const PI: f32 = std::f32::consts::PI;

fn gaussian(x: f32, sigma: f32) -> f32 {
    (-(x * x) / (2. * sigma * sigma)).exp() / ((2. * PI).sqrt() * sigma)
}

/// box_shadow.wesl's erf approximation (Abramowitz and Stegun 7.1.27)
fn erf(p: f32) -> f32 {
    let (s, a) = (p.signum(), p.abs());
    let mut r = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    r *= r;
    s - s / (r * r)
}

/// the Gaussian blur of one row of the rounded box, across: `radius` the corner's (x, y), `half` the box's half size
fn horizontal(x: f32, y: f32, blur: f32, radius: (f32, f32), half: (f32, f32)) -> f32 {
    let mut c = half.0;
    if 0.0 < radius.0.min(radius.1) {
        let d = (half.1 - radius.1 - y.abs()).min(0.);
        c = half.0 - radius.0 + radius.0 * (1. - d * d / (radius.1 * radius.1)).max(0.).sqrt();
    }
    let k = 0.5f32.sqrt() / blur;
    let (lo, hi) = (0.5 + 0.5 * erf((x - c) * k), 0.5 + 0.5 * erf((x + c) * k));
    hi - lo
}

/// select_corner_radius: the corner of `p`'s quadrant (radii top left, top right, bottom right, bottom left)
fn corner(p: (f32, f32), radii: [f32; 4]) -> (f32, f32) {
    let r = match (0.0 < p.0, 0.0 < p.1) {
        (false, false) => radii[0],
        (true, false) => radii[1],
        (true, true) => radii[2],
        (false, true) => radii[3],
    };
    (r, r)
}

/// roundedBoxShadow: the shadow's coverage at `p`, relative to the box's centre
pub(crate) fn coverage(p: (f32, f32), half: (f32, f32), blur: f32, radii: [f32; 4], samples: u32) -> f32 {
    let (low, high) = (p.1 - half.1, p.1 + half.1);
    let start = (-3. * blur).clamp(low, high);
    let end = (3. * blur).clamp(low, high);
    let step = (end - start) / samples as f32;
    let mut y = start + step * 0.5;
    let mut value = 0.0;
    let r = corner(p, radii);
    for _ in 0..samples {
        value += horizontal(p.0, p.1 - y, blur, r, half) * gaussian(y, blur) * step;
        y += step;
    }
    value
}

/// The shadow of a box `size` (device px) centred at `center`, its corners `radii`, as an RGBA bitmap of the colour
/// `rgb` at `alpha` times the coverage, on whole pixels, and where its top left pixel goes; None when nothing shows.
pub(crate) fn bitmap(center: (f64, f64), size: (f64, f64), radii: [f64; 4], blur: f64, samples: u32, rgb: [u8; 3], alpha: f32) -> Option<((f64, f64), RasterImageData)> {
    if size.0 <= 0.0 || size.1 <= 0.0 || alpha <= 0.0 || samples == 0 {
        return None;
    }
    let blur = (blur as f32).max(0.01);
    let bounds = (size.0 + 6. * blur as f64, size.1 + 6. * blur as f64);
    let (x0, y0) = ((center.0 - bounds.0 / 2.).floor(), (center.1 - bounds.1 / 2.).floor());
    let (x1, y1) = ((center.0 + bounds.0 / 2.).ceil(), (center.1 + bounds.1 / 2.).ceil());
    let (w, h) = ((x1 - x0) as u32, (y1 - y0) as u32);
    if w == 0 || h == 0 || u64::from(w) * u64::from(h) > 64 << 20 {
        return None;
    }
    let half = ((size.0 / 2.) as f32, (size.1 / 2.) as f32);
    let radii = radii.map(|r| r as f32);
    let mut out = vec![0u8; w as usize * h as usize * 4];
    for j in 0..h {
        let py = y0 + f64::from(j) + 0.5;
        // the quad covers the bounds only: a pixel centre outside them draws nothing
        if (py - center.1).abs() > bounds.1 / 2. {
            continue;
        }
        for i in 0..w {
            let px = x0 + f64::from(i) + 0.5;
            if (px - center.0).abs() > bounds.0 / 2. {
                continue;
            }
            let g = alpha * coverage(((px - center.0) as f32, (py - center.1) as f32), half, blur, radii, samples);
            let a = (g.clamp(0., 1.) * 255.).round() as u8;
            let k = (j as usize * w as usize + i as usize) * 4;
            out[k..k + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], a]);
        }
    }
    Some(((x0, y0), RasterImageData::new(w, h, Arc::new(out))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_box_is_half_covered_at_its_edge_and_whole_at_its_centre() {
        // far from the corners, a box much bigger than its blur: 0.5 on an edge, about 1 inside, about 0 well outside
        let (half, r) = ((200., 200.), [0.; 4]);
        assert!((coverage((200., 0.), half, 10., r, 15) - 0.5).abs() < 0.02);
        assert!((coverage((0., 0.), half, 10., r, 15) - 1.0).abs() < 0.02);
        assert!(coverage((260., 0.), half, 10., r, 15) < 0.01);
    }

    #[test]
    fn its_bitmap_is_the_shadow_quad_on_whole_pixels() {
        let ((x, y), img) = bitmap((50.5, 50.0), (20., 10.), [0.; 4], 2., 4, [0, 0, 0], 0.8).unwrap();
        // 20 + 12 by 10 + 12, centred on (50.5, 50): x 34.5..66.5 -> 34..67, y 39..61
        assert_eq!((x, y, img.width, img.height), (34., 39., 33, 22));
    }
}
