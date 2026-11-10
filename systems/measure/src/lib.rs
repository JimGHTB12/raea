//! What a render's output measures (docs/direction.feature DR2), read with ffmpeg as `raea check` reads an export:
//! each frame's light and how much it changed from the frame before, a text's contrast against the pixels behind it,
//! the mix's integrated loudness and true peak, and when a sound starts in the mix. Each is a number a check compares
//! with its own limit; nothing here decides.

use std::path::Path;
use std::process::{Command, Output};

/// Frames are read at this size: enough to see light and movement, small enough that a two-minute video reads in
/// seconds. Mean light and mean change don't depend on the frame's shape.
const SIDE: usize = 64;
/// The sound envelope's step, in seconds.
pub const HOP: f64 = 0.01;
/// How far after a sound's start its onset is looked for, and how far before (a render rounds a start to a frame).
const AFTER: f64 = 0.2;
const BEFORE: f64 = 0.05;
/// A rise this many dB over three steps (30 ms) is an onset.
const RISE: f32 = 6.0;
/// ... once it is within this many dB of the loudest step in the next tenth of a second: a rise out of digital
/// silence to a level no one hears (an encoder's pre-echo, 20 ms before the sound and 80 dB under it) isn't one.
const NEAR: f32 = 10.0;

/// One frame's light and movement: its mean luma (0 black, 1 white) and the mean change of each pixel's luma from the
/// frame before (0 to 1; None for the first).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Picture {
    pub luma: f64,
    pub motion: Option<f64>,
}

fn ffmpeg(args: &[&str]) -> Result<Output, String> {
    let out = Command::new("ffmpeg").args(["-v", "error", "-nostdin"]).args(args).output().map_err(|e| format!("ffmpeg: {e}"))?;
    if out.status.success() { Ok(out) } else { Err(format!("ffmpeg: {}", String::from_utf8_lossy(&out.stderr).trim())) }
}

fn name(file: &Path) -> Result<&str, String> {
    file.to_str().ok_or_else(|| format!("{}: its name isn't text", file.display()))
}

/// Each frame of a video, or the one of an image, as its light and movement, in order.
pub fn pictures(file: &Path) -> Result<Vec<Picture>, String> {
    let scale = format!("scale={SIDE}:{SIDE}:flags=area:out_range=full,format=gray");
    let out = ffmpeg(&["-i", name(file)?, "-an", "-vf", &scale, "-f", "rawvideo", "-"])?;
    Ok(of_gray(&out.stdout))
}

/// Frames of SIDE x SIDE grey bytes as pictures.
pub fn of_gray(bytes: &[u8]) -> Vec<Picture> {
    let n = (SIDE * SIDE) as f64;
    let frames: Vec<&[u8]> = bytes.chunks_exact(SIDE * SIDE).collect();
    let mut out = Vec::with_capacity(frames.len());
    for (i, f) in frames.iter().enumerate() {
        let luma = f.iter().map(|&v| v as f64).sum::<f64>() / n / 255.0;
        let motion = i.checked_sub(1).map(|b| f.iter().zip(frames[b]).map(|(a, b)| (*a as f64 - *b as f64).abs()).sum::<f64>() / n / 255.0);
        out.push(Picture { luma, motion });
    }
    out
}

/// A CSS colour's red, green and blue, 0-255 (`rgb(..)`, `rgba(..)`; white where it says none).
pub fn rgb_of(css: &str) -> [f64; 3] {
    let n: Vec<f64> = css.split(|c: char| !(c.is_ascii_digit() || c == '.')).filter(|s| !s.is_empty()).filter_map(|s| s.parse().ok()).collect();
    [n.first().copied().unwrap_or(255.0), n.get(1).copied().unwrap_or(255.0), n.get(2).copied().unwrap_or(255.0)]
}

/// The relative luminance (WCAG) of an sRGB colour 0-255.
pub fn lum([r, g, b]: [f64; 3]) -> f64 {
    let c = |v: f64| {
        let v = v / 255.0;
        if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * c(r) + 0.7152 * c(g) + 0.0722 * c(b)
}

/// WCAG's contrast ratio (1 to 21) of a text of luminance `text` against RGB pixels: the median of the half least
/// like the text is what is behind it (the other half is mostly the letters themselves).
pub fn ratio_behind(rgb: &[u8], text: f64) -> Option<f64> {
    let mut ls: Vec<f64> = rgb.chunks_exact(3).map(|p| lum([p[0] as f64, p[1] as f64, p[2] as f64])).collect();
    if ls.len() < 2 {
        return None;
    }
    ls.sort_by(|a, b| (b - text).abs().total_cmp(&(a - text).abs()));
    let mut behind = ls[..ls.len() / 2].to_vec();
    behind.sort_by(f64::total_cmp);
    let bg = behind[behind.len() / 2];
    Some((text.max(bg) + 0.05) / (text.min(bg) + 0.05))
}

/// The contrast of the text of colour `color` drawn in `crop` (x, y, w, h in the file's pixels) at `at` seconds of a
/// video (an image has one frame), against what is behind it: `raea check`'s caption contrast. None when the crop
/// falls outside the frame or ffmpeg can't read it.
pub fn contrast(file: &Path, at: f64, crop: [u32; 4], color: &str) -> Option<f64> {
    // even sizes and offsets: a 4:2:0 frame crops to them
    let [x, y, w, h] = [crop[0] & !1, crop[1] & !1, crop[2].max(2) & !1, crop[3].max(2) & !1];
    let vf = format!("crop={w}:{h}:{x}:{y}");
    let out = ffmpeg(&["-ss", &format!("{at:.3}"), "-i", name(file).ok()?, "-frames:v", "1", "-vf", &vf, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"]).ok()?;
    (out.stdout.len() >= (w * h * 3) as usize).then(|| ratio_behind(&out.stdout, lum(rgb_of(color)))).flatten()
}

/// The mix's integrated loudness in LUFS and its true peak in dBTP (EBU R128, ffmpeg's ebur128). None when the file
/// has no sound.
pub fn loudness(file: &Path) -> Option<(f64, Option<f64>)> {
    let out = Command::new("ffmpeg").args(["-hide_banner", "-nostats", "-i", name(file).ok()?, "-vn", "-af", "ebur128=peak=true", "-f", "null", "-"]).output().ok()?;
    of_ebur128(&String::from_utf8_lossy(&out.stderr))
}

/// ebur128's summary read: its integrated loudness and true peak.
pub fn of_ebur128(log: &str) -> Option<(f64, Option<f64>)> {
    let summary = log.rsplit_once("Summary:")?.1;
    let num = |key: &str| summary.lines().find(|l| l.trim_start().starts_with(key)).and_then(|l| l.split_whitespace().nth(1)).and_then(|x| x.parse::<f64>().ok());
    Some((num("I:")?, num("Peak:")))
}

/// The mix's level in dB every HOP seconds (its RMS, mono, -120 at silence). Empty when the file has no sound.
pub fn envelope(file: &Path) -> Result<Vec<f32>, String> {
    let out = ffmpeg(&["-i", name(file)?, "-vn", "-ac", "1", "-ar", "16000", "-f", "f32le", "-"])?;
    let samples: Vec<f32> = out.stdout.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
    Ok(levels(&samples, (16000.0 * HOP) as usize))
}

/// Samples as dB a step of `hop` samples.
pub fn levels(samples: &[f32], hop: usize) -> Vec<f32> {
    samples.chunks(hop.max(1)).map(|c| 10.0 * ((c.iter().map(|s| s * s).sum::<f32>() / c.len() as f32).max(1e-12)).log10()).collect()
}

/// When the sound placed at `at` seconds is heard: the start of the first step, from BEFORE before it to AFTER after
/// it, where the mix has risen RISE dB over the three steps before and come within NEAR dB of where it goes (within a
/// step, 10 ms). None when nothing starts there: a sound under a louder bed, or one that never plays.
pub fn onset(env: &[f32], at: f64) -> Option<f64> {
    let (a, b) = (((at - BEFORE) / HOP).floor().max(3.0) as usize, (((at + AFTER) / HOP).ceil().max(0.0) as usize).min(env.len()));
    let loudest = |i: usize| env[i..(i + 10).min(env.len())].iter().copied().fold(f32::MIN, f32::max);
    (a..b).find(|&i| env[i] - env[i - 3] >= RISE && env[i] >= loudest(i) - NEAR).map(|i| i as f64 * HOP)
}

#[cfg(test)]
mod tests;
