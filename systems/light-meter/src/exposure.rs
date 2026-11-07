//! A rendered frame's exposure, as a reflected meter and a histogram read it: the mean of its display luma, the 1st and
//! 99th percentiles, how much clips to white or crushes to black, and how many stops the frame's mean linear light is
//! from mid grey (0.18). `raea render` prints it for every still and frame folder (the 11-06 research: agents loop on
//! "too dark, add light" for want of a number; three-live and the scene reviewers report exactly these).
//! Transparent pixels (an --alpha render's empty background) are left out.

use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Exposure {
    /// mean display (sRGB-encoded) luma, 0-1
    pub mean_luma: f64,
    pub p01: f64,
    pub p99: f64,
    /// the share of pixels with a channel at 254 or more
    pub clipped_high: f64,
    /// the share with every channel at 1 or less
    pub clipped_low: f64,
    /// log2(mean linear luminance / 0.18): 0 is a frame metered to mid grey
    pub over_stops: f64,
    pub pixels: usize,
}

fn linear(c: u8) -> f64 {
    let c = c as f64 / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// The stats of RGBA8 pixels (straight alpha, sRGB).
pub fn stats(rgba: &[u8]) -> Option<Exposure> {
    let lin: Vec<f64> = (0..=255u8).map(linear).collect();
    let mut hist = [0usize; 256];
    let (mut n, mut sum_luma, mut sum_lin, mut hi, mut lo) = (0usize, 0.0, 0.0, 0usize, 0usize);
    for p in rgba.chunks_exact(4) {
        if p[3] == 0 {
            continue;
        }
        n += 1;
        let y = 0.2126 * lin[p[0] as usize] + 0.7152 * lin[p[1] as usize] + 0.0722 * lin[p[2] as usize];
        sum_lin += y;
        // display luma: the linear luminance encoded back (what a histogram shows)
        let enc = if y <= 0.0031308 { 12.92 * y } else { 1.055 * y.powf(1.0 / 2.4) - 0.055 };
        sum_luma += enc;
        hist[(enc * 255.0).round().clamp(0.0, 255.0) as usize] += 1;
        if p[0].max(p[1]).max(p[2]) >= 254 {
            hi += 1;
        }
        if p[0].max(p[1]).max(p[2]) <= 1 {
            lo += 1;
        }
    }
    if n == 0 {
        return None;
    }
    let pct = |q: f64| {
        let want = (q * n as f64).ceil().max(1.0) as usize;
        let mut acc = 0;
        for (i, c) in hist.iter().enumerate() {
            acc += c;
            if acc >= want {
                return i as f64 / 255.0;
            }
        }
        1.0
    };
    let mean_lin = (sum_lin / n as f64).max(1e-6);
    Some(Exposure { mean_luma: sum_luma / n as f64, p01: pct(0.01), p99: pct(0.99), clipped_high: hi as f64 / n as f64, clipped_low: lo as f64 / n as f64, over_stops: (mean_lin / 0.18).log2(), pixels: n })
}

fn r(x: f64, k: i32) -> f64 {
    let m = 10f64.powi(k);
    (x * m).round() / m
}

impl Exposure {
    pub fn json(&self) -> Value {
        json!({"meanLuma": r(self.mean_luma, 3), "p01": r(self.p01, 3), "p99": r(self.p99, 3), "clippedHigh": r(self.clipped_high, 4), "clippedLow": r(self.clipped_low, 4), "overStops": r(self.over_stops, 2)})
    }

    /// What's wrong with it, in scene lint's levels: (level, says). Empty when nothing is.
    pub fn found(&self, what: &str) -> Vec<(&'static str, String)> {
        let mut out = Vec::new();
        if self.clipped_high > 0.02 {
            out.push(("warning", format!("{what}: {:.1}% of the pixels clip to white (lower the exposure or the brightest light, or tone map)", self.clipped_high * 100.0)));
        }
        if self.clipped_low > 0.05 {
            out.push(("warning", format!("{what}: {:.1}% of the pixels crush to black (a fill, an ambient or the sky's light reaches them nowhere)", self.clipped_low * 100.0)));
        }
        if self.over_stops > 1.5 {
            out.push(("warning", format!("{what}: {:+.1} stops over mid grey: the frame reads washed out unless it is meant high-key", self.over_stops)));
        } else if self.over_stops < -2.5 {
            out.push(("warning", format!("{what}: {:+.1} stops under mid grey: the frame reads dark unless it is meant low-key (`raea scene light` meters the subject)", self.over_stops)));
        }
        if self.p99 - self.p01 < 0.25 {
            out.push(("note", format!("{what}: flat, the 1st to 99th percentile spans only {:.2} of the range (little contrast)", self.p99 - self.p01)));
        }
        out
    }

    /// One line for a render's message.
    pub fn line(&self) -> String {
        format!("mean luma {:.2}, p01 {:.2}, p99 {:.2}, {:.1}% clip white, {:.1}% crush black, {:+.1} stops from mid grey", self.mean_luma, self.p01, self.p99, self.clipped_high * 100.0, self.clipped_low * 100.0, self.over_stops)
    }
}

/// The exposure of PNG files, each read and measured, and all of them pooled (as one frame of all their pixels):
/// `{"frames": [{file, exposure}], "exposure": pooled, "found": [{check, level, says}]}`.
pub fn of_pngs(files: &[std::path::PathBuf]) -> Result<(Exposure, Value), String> {
    let mut per = Vec::new();
    let mut pooled: Vec<u8> = Vec::new();
    let mut found = Vec::new();
    // pooling every frame of a long folder is a lot of pixels: every k-th, at most 24 frames
    let k = files.len().div_ceil(24).max(1);
    for f in files.iter().step_by(k) {
        let img = image::open(f).map_err(|e| format!("{}: {e}", f.display()))?.to_rgba8();
        let raw = img.as_raw();
        let name = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if let Some(e) = stats(raw) {
            per.push(json!({"file": name, "exposure": e.json()}));
            if files.len() > 1 {
                for (level, says) in e.found(&name) {
                    if level == "warning" {
                        found.push(json!({"check": "exposure", "level": level, "says": says}));
                    }
                }
            }
        }
        // a quarter of each frame's pixels into the pool (every 4th), so a folder is measured in bounded memory
        pooled.extend(raw.chunks_exact(4).step_by(4).flatten());
    }
    let all = stats(&pooled).ok_or("exposure: every pixel is transparent")?;
    if files.len() == 1 {
        for (level, says) in all.found("the frame") {
            found.push(json!({"check": "exposure", "level": level, "says": says}));
        }
    }
    let sampled = if k > 1 { json!(per.len()) } else { Value::Null };
    Ok((all, json!({"exposure": all.json(), "frames": per, "sampled": sampled, "found": found})))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(v: u8, n: usize) -> Vec<u8> {
        (0..n).flat_map(|_| [v, v, v, 255]).collect()
    }

    #[test]
    fn mid_grey_is_zero_stops_and_white_clips() {
        // sRGB 118 is linear 0.18
        let e = stats(&flat(118, 100)).unwrap();
        assert!(e.over_stops.abs() < 0.03, "{}", e.over_stops);
        assert!((e.mean_luma - 118.0 / 255.0).abs() < 0.01);
        assert!(e.found("f").iter().any(|(l, s)| *l == "note" && s.contains("flat")));
        let w = stats(&flat(255, 100)).unwrap();
        assert_eq!(w.clipped_high, 1.0);
        assert!(w.over_stops > 2.4);
        assert!(w.found("f").iter().any(|(_, s)| s.contains("clip to white")));
        let b = stats(&flat(0, 100)).unwrap();
        assert_eq!(b.clipped_low, 1.0);
        assert!(b.found("f").iter().any(|(_, s)| s.contains("crush to black")));
    }

    #[test]
    fn percentiles_and_transparency() {
        // half black, half white: p01 black, p99 white; transparent pixels ignored
        let mut px = flat(0, 50);
        px.extend(flat(255, 50));
        px.extend((0..100).flat_map(|_| [128u8, 128, 128, 0]));
        let e = stats(&px).unwrap();
        assert_eq!(e.pixels, 100);
        assert_eq!((e.p01, e.p99), (0.0, 1.0));
        assert!(stats(&[9, 9, 9, 0]).is_none());
    }
}
