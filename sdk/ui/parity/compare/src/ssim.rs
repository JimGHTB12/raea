//! SSIM (Wang et al. 2004) per colour channel (R, G and B, averaged: luma alone can't tell a rose box from a blue one
//! of the same lightness, which cases/calib/off-pixels proves), a 7x7 uniform window clipped at the image's edges, as a map with one value a
//! pixel: the frame's score is the map's mean, an element's the mean over its box. Built on summed-area tables, so a
//! 1440x900 pair costs a few milliseconds.

use image::RgbaImage;

pub struct Map {
    pub w: usize,
    pub h: usize,
    v: Vec<f32>,
}

const R: i64 = 3; // the window's radius: 7x7
const C1: f64 = (0.01 * 255.0) * (0.01 * 255.0);
const C2: f64 = (0.03 * 255.0) * (0.03 * 255.0);

/// One channel, composited over black (both renderers' targets are opaque).
fn channel(img: &RgbaImage, c: usize) -> Vec<f64> {
    img.pixels().map(|p| p[c] as f64 * p[3] as f64 / 255.0).collect()
}

/// A summed-area table (w+1)x(h+1) of f(a, b).
fn table(w: usize, h: usize, a: &[f64], b: &[f64], f: impl Fn(f64, f64) -> f64) -> Vec<f64> {
    let mut t = vec![0.0; (w + 1) * (h + 1)];
    for y in 0..h {
        let mut row = 0.0;
        for x in 0..w {
            row += f(a[y * w + x], b[y * w + x]);
            t[(y + 1) * (w + 1) + x + 1] = t[y * (w + 1) + x + 1] + row;
        }
    }
    t
}

/// The SSIM map of two images of the same size (Err: the sizes differ).
pub fn map(a: &RgbaImage, b: &RgbaImage) -> Result<Map, String> {
    if a.dimensions() != b.dimensions() {
        return Err(format!("the frames differ in size: {:?} and {:?}", a.dimensions(), b.dimensions()));
    }
    let (w, h) = (a.width() as usize, a.height() as usize);
    let mut v = vec![0f32; w * h];
    for c in 0..3 {
        one(w, h, &channel(a, c), &channel(b, c), &mut v);
    }
    for x in &mut v {
        *x /= 3.0;
    }
    Ok(Map { w, h, v })
}

/// One channel's SSIM map, added into `v`.
fn one(w: usize, h: usize, la: &[f64], lb: &[f64], v: &mut [f32]) {
    let (la, lb) = (la.to_vec(), lb.to_vec());
    let sa = table(w, h, &la, &lb, |x, _| x);
    let sb = table(w, h, &la, &lb, |_, y| y);
    let saa = table(w, h, &la, &lb, |x, _| x * x);
    let sbb = table(w, h, &la, &lb, |_, y| y * y);
    let sab = table(w, h, &la, &lb, |x, y| x * y);
    let at = |t: &[f64], x0: usize, y0: usize, x1: usize, y1: usize| t[y1 * (w + 1) + x1] - t[y0 * (w + 1) + x1] - t[y1 * (w + 1) + x0] + t[y0 * (w + 1) + x0];
    for y in 0..h {
        let (y0, y1) = ((y as i64 - R).max(0) as usize, ((y as i64 + R + 1).min(h as i64)) as usize);
        for x in 0..w {
            let (x0, x1) = ((x as i64 - R).max(0) as usize, ((x as i64 + R + 1).min(w as i64)) as usize);
            let n = ((x1 - x0) * (y1 - y0)) as f64;
            let (ma, mb) = (at(&sa, x0, y0, x1, y1) / n, at(&sb, x0, y0, x1, y1) / n);
            let va = (at(&saa, x0, y0, x1, y1) / n - ma * ma).max(0.0);
            let vb = (at(&sbb, x0, y0, x1, y1) / n - mb * mb).max(0.0);
            let cov = at(&sab, x0, y0, x1, y1) / n - ma * mb;
            v[y * w + x] += (((2.0 * ma * mb + C1) * (2.0 * cov + C2)) / ((ma * ma + mb * mb + C1) * (va + vb + C2))) as f32;
        }
    }
}

impl Map {
    /// The mean over the whole frame.
    pub fn mean(&self) -> f64 {
        self.v.iter().map(|&x| x as f64).sum::<f64>() / self.v.len().max(1) as f64
    }

    /// The mean over a box in pixels (clipped to the frame; None when nothing of it is inside).
    pub fn mean_in(&self, x: f64, y: f64, w: f64, h: f64) -> Option<f64> {
        let x0 = x.floor().max(0.0) as usize;
        let y0 = y.floor().max(0.0) as usize;
        let x1 = ((x + w).ceil().max(0.0) as usize).min(self.w);
        let y1 = ((y + h).ceil().max(0.0) as usize).min(self.h);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        let mut s = 0.0;
        for yy in y0..y1 {
            for xx in x0..x1 {
                s += self.v[yy * self.w + xx] as f64;
            }
        }
        Some(s / ((x1 - x0) * (y1 - y0)) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_one_and_different_is_less() {
        let mut a = RgbaImage::from_pixel(64, 48, image::Rgba([255, 255, 255, 255]));
        for x in 10..30 {
            for y in 10..20 {
                a.put_pixel(x, y, image::Rgba([200, 20, 60, 255]));
            }
        }
        let m = map(&a, &a).unwrap();
        assert!((m.mean() - 1.0).abs() < 1e-9);
        let mut b = a.clone();
        for x in 30..34 {
            for y in 10..20 {
                b.put_pixel(x, y, image::Rgba([200, 20, 60, 255]));
            }
        }
        let m = map(&a, &b).unwrap();
        assert!(m.mean() < 1.0);
        assert!(m.mean_in(28.0, 8.0, 8.0, 14.0).unwrap() < 0.9);
        assert!((m.mean_in(0.0, 30.0, 64.0, 18.0).unwrap() - 1.0).abs() < 1e-9);
    }
}
