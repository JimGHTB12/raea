//! `media motion`, `media cuts` and `media freezes`: how the picture changes, frame to frame.

use super::*;

/// The decoder's own planes (NV12: luma, then Cb Cr interleaved at half size), as ffmpeg's filters see a 4:2:0 frame.
pub(crate) fn yuv(f: Frame) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let n = (f.width * f.height) as usize;
    let mut buf = f.rgba;
    let uv = buf.split_off(n.min(buf.len()));
    let (u, v) = (uv.iter().step_by(2).copied().collect(), uv.iter().skip(1).step_by(2).copied().collect());
    (buf, u, v)
}

pub(crate) fn sad(a: &[u8], b: &[u8]) -> u64 {
    a.iter().zip(b).map(|(x, y)| (*x as i32 - *y as i32).unsigned_abs() as u64).sum()
}

/// Samples from `from` to `to` at `fps` (the source's frames when `fps` is its rate): each tick's time, and the time
/// to read, the last frame before the middle of the tick, as ffmpeg's fps filter picks it (at 6 fps from 30, tick 0 is
/// source frame 2 and tick 1 frame 7).
pub(crate) fn times(duration: f64, from: Option<f64>, to: Option<f64>, fps: f64) -> Vec<(f64, f64)> {
    let (a, b) = (from.unwrap_or(0.0).max(0.0), to.unwrap_or(duration).min(duration));
    let n = ((b - a) * fps).round().max(0.0) as usize;
    (0..n).map(|i| (a + i as f64 / fps, a + (i as f64 + 0.5) / fps - 1e-6)).collect()
}

/// The video's own duration: its track's (what ffmpeg's filters run to), else the decoder's.
pub(crate) fn video_span(path: &Path, info: &Info) -> f64 {
    video_track(path).map(|t| t.duration).unwrap_or(info.duration)
}

/// How much the picture changes: the mean absolute luma difference between samples `fps` apart (signalstats YDIF).
pub fn motion(path: &Path, from: Option<f64>, to: Option<f64>, fps: f64) -> R<(Value, Vec<(f64, f64)>)> {
    let mut d = backend()?.open_video(path)?;
    let ts = times(video_span(path, d.info()), from, to, fps);
    let mut prev: Option<Vec<u8>> = None;
    let mut out = Vec::new();
    for (t, at) in ts {
        let (y, ..) = yuv(d.frame_nv12_at(at)?);
        let ydif = prev.as_ref().map(|p| sad(p, &y) as f64 / y.len() as f64).unwrap_or(0.0);
        out.push((t, ydif));
        prev = Some(y);
    }
    let mean = if out.is_empty() { 0.0 } else { out.iter().map(|(_, v)| v).sum::<f64>() / out.len() as f64 };
    Ok((json!({"file": path, "ydif_mean": round3(mean), "samples": out.len(), "fps": fps}), out))
}

/// Shot changes: ffmpeg's scene score (select's get_scene_score: the mean absolute luma difference from the frame before,
/// against the change in it since the frame before that, over 100) above `threshold`, frame by frame.
pub fn cuts(path: &Path, threshold: f64, from: Option<f64>, to: Option<f64>) -> R<(Value, Vec<(usize, f64, f64)>)> {
    let mut d = backend()?.open_video(path)?;
    let info = d.info().clone();
    // every frame the file holds, at its own time (a variable-rate take has no fixed step)
    let ts = frames_of(path, &info, from, to);
    let mut prev: Option<(Vec<u8>, Vec<u8>, Vec<u8>)> = None;
    let mut prev_mafd = 0.0;
    let mut hits = Vec::new();
    for (n, (_, at)) in ts.into_iter().enumerate() {
        let f = d.frame_nv12_at(at)?;
        let pts = f.pts;
        let (y, u, v) = yuv(f);
        if let Some((py, ..)) = &prev {
            // a YUV frame is scored on its luma alone (f_select.c: nb_planes = is_yuv ? 1 : ...)
            let mafd = sad(py, &y) as f64 / y.len() as f64;
            let score = (mafd.min((mafd - prev_mafd).abs()) / 100.0).clamp(0.0, 1.0);
            prev_mafd = mafd;
            if score > threshold {
                hits.push((n, pts, score));
            }
        }
        prev = Some((y, u, v));
    }
    let v = json!({"file": path, "threshold": threshold, "cuts": hits.iter().map(|(_, t, s)| json!({"at": round3(*t), "score": round3(*s)})).collect::<Vec<_>>()});
    Ok((v, hits))
}

/// Every frame the file holds as (tick, time to read): its own times when the file states them, else a fixed step.
pub(crate) fn frames_of(path: &Path, info: &Info, from: Option<f64>, to: Option<f64>) -> Vec<(f64, f64)> {
    match video_track(path) {
        Some(t) => t.pts.iter().filter(|p| from.is_none_or(|a| **p >= a - 1e-6) && to.is_none_or(|b| **p < b)).map(|p| (*p, *p + 1e-4)).collect(),
        None => times(info.duration, from, to, info.fps),
    }
}

/// Held pictures: ffmpeg's freezedetect. Each frame is compared with the last one that moved (the mean absolute difference
/// over every plane, as a fraction of full scale); a stretch at or under `noise` lasting `min` seconds or more is a
/// freeze, reported as (start, Some((duration, end))) when it ends, or (start, None) when the file ends frozen.
pub fn freezes(path: &Path, noise: f64, min: f64) -> R<Vec<(f64, Option<(f64, f64)>)>> {
    let mut d = backend()?.open_video(path)?;
    let info = d.info().clone();
    let mut reference: Option<(f64, Vec<u8>, Vec<u8>, Vec<u8>)> = None;
    let mut frozen_now = false;
    let mut out: Vec<(f64, Option<(f64, f64)>)> = Vec::new();
    for (pts, at) in frames_of(path, &info, None, None) {
        let (y, u, v) = yuv(d.frame_nv12_at(at)?);
        let mut frozen = false;
        if let Some((rp, ry, ru, rv)) = &reference {
            let mafd = (sad(ry, &y) + sad(ru, &u) + sad(rv, &v)) as f64 / (y.len() + u.len() + v.len()) as f64 / 256.0;
            frozen = mafd <= noise;
            if pts - rp >= min - 1e-9 {
                if !frozen_now {
                    out.push((*rp, None));
                }
                if !frozen {
                    if let Some(last) = out.last_mut() {
                        last.1 = Some((pts - last.0, pts));
                    }
                }
                frozen_now = frozen;
            }
        }
        if !frozen {
            reference = Some((pts, y, u, v));
        }
    }
    Ok(out)
}

// ---- encoding (proxy, trim, concat)
