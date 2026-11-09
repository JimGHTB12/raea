//! The 3D's half of the layer pass: triangles in clip space drawn straight into the mask (one sample a mask pixel, at
//! frame px (4x + 2, 4y + 2) as the mask's own), nearest first, each triangle's value where it is in front. A
//! triangle crossing the camera's near side is cut there. Exact: no antialiasing, no colours to read back.

use crate::file::SCALE;

/// A triangle's corners in clip space (x, y, z, w) and the value it draws (0: in no layer, though it still hides
/// what's behind it).
pub type Tri = ([[f32; 4]; 3], u16);

/// The mask (`frame` is the frame's size in px, the mask its quarter, row-major) of `tris`: each sample the value of
/// the nearest triangle over it, 0 where none is.
pub fn raster(frame: (u32, u32), tris: impl IntoIterator<Item = Tri>) -> Vec<u16> {
    let (w, h) = frame;
    let (mw, mh) = (w.div_ceil(SCALE) as usize, h.div_ceil(SCALE) as usize);
    let mut ids = vec![0u16; mw * mh];
    // 1/w at each sample: nearer is bigger
    let mut near = vec![0f32; mw * mh];
    const EPS: f32 = 1e-5;
    let (shift, farthest) = (if crate::mutant("shift") { 2.0 } else { 0.0 }, crate::mutant("occlusion"));
    for (corners, v) in tris {
        for tri in cut(corners, EPS) {
            // frame px (pixel centers at +0.5), then mask px: frame px 4x + 2.5 is mask px x + 0.5
            let p: Vec<[f32; 3]> = tri.iter().map(|c| {
                let (fx, fy) = ((c[0] / c[3] + 1.0) * 0.5 * w as f32 + shift, (1.0 - c[1] / c[3]) * 0.5 * h as f32);
                [(fx - 2.5) / SCALE as f32 + 0.5, (fy - 2.5) / SCALE as f32 + 0.5, 1.0 / c[3]]
            }).collect();
            let area = (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
            if area.abs() < 1e-12 {
                continue;
            }
            let x0 = p.iter().map(|q| q[0]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
            let x1 = (p.iter().map(|q| q[0]).fold(f32::MIN, f32::max).ceil().max(0.0) as usize).min(mw);
            let y0 = p.iter().map(|q| q[1]).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
            let y1 = (p.iter().map(|q| q[1]).fold(f32::MIN, f32::max).ceil().max(0.0) as usize).min(mh);
            for y in y0..y1 {
                for x in x0..x1 {
                    let (sx, sy) = (x as f32 + 0.5, y as f32 + 0.5);
                    let e = |a: &[f32; 3], b: &[f32; 3]| ((b[0] - a[0]) * (sy - a[1]) - (sx - a[0]) * (b[1] - a[1])) / area;
                    let (b0, b1, b2) = (e(&p[1], &p[2]), e(&p[2], &p[0]), e(&p[0], &p[1]));
                    // inside, its edges' samples to one side only (a sample on an edge two triangles share is one's)
                    let inside = |b: f32, a: &[f32; 3], c: &[f32; 3]| b > 0.0 || (b == 0.0 && ((c[1] - a[1]) * area.signum() < 0.0 || (c[1] == a[1] && (c[0] - a[0]) * area.signum() > 0.0)));
                    if !(inside(b0, &p[1], &p[2]) && inside(b1, &p[2], &p[0]) && inside(b2, &p[0], &p[1])) {
                        continue;
                    }
                    let inv_w = b0 * p[0][2] + b1 * p[1][2] + b2 * p[2][2];
                    let i = y * mw + x;
                    if (inv_w > near[i]) != (farthest && near[i] > 0.0) {
                        near[i] = inv_w;
                        ids[i] = v;
                    }
                }
            }
        }
    }
    ids
}

/// The triangle's part in front of the camera (w > eps): none, one or two triangles.
fn cut(t: [[f32; 4]; 3], eps: f32) -> Vec<[[f32; 4]; 3]> {
    if t.iter().all(|c| c[3] > eps) {
        return vec![t];
    }
    let mut poly: Vec<[f32; 4]> = Vec::new();
    for i in 0..3 {
        let (a, b) = (t[i], t[(i + 1) % 3]);
        if a[3] > eps {
            poly.push(a);
        }
        if (a[3] > eps) != (b[3] > eps) {
            let s = (eps - a[3]) / (b[3] - a[3]);
            poly.push(std::array::from_fn(|k| a[k] + (b[k] - a[k]) * s));
        }
    }
    (1..poly.len().saturating_sub(1)).map(|i| [poly[0], poly[i], poly[i + 1]]).collect()
}
