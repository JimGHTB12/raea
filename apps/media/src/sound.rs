//! `media audio` and `media loudness`: a file's sound, resampled, folded and measured.

use super::*;

pub(crate) fn slice(a: &Audio, from: Option<f64>, to: Option<f64>) -> Audio {
    let n = a.len();
    let at = |s: f64| ((s * a.rate as f64).round().max(0.0) as usize).min(n);
    let (lo, hi) = (from.map(at).unwrap_or(0), to.map(at).unwrap_or(n));
    Audio { rate: a.rate, channels: a.channels.iter().map(|c| c[lo..hi.max(lo)].to_vec()).collect() }
}

/// The sound at `rate`, folded to mono when asked. The fold follows ffmpeg's: to float, stereo at -3 dB a side
/// ((L + R) * 0.707); to 16-bit integers, whose downmix ffmpeg normalises against clipping, the average ((L + R) / 2).
pub fn audio_folded(path: &Path, rate: u32, mono: bool, from: Option<f64>, to: Option<f64>, integer: bool) -> R<Audio> {
    let a = slice(&soft()?.decode_audio(path)?, from, to);
    let a = if mono && a.channels.len() > 1 {
        // stereo folds down as ffmpeg's -ac 1 does, each side at -3 dB (L + R) * 0.707; more channels average
        let k = if a.channels.len() == 2 && !integer { std::f32::consts::SQRT_2 } else { a.channels.len() as f32 };
        let m = (0..a.len()).map(|i| a.channels.iter().map(|c| c[i]).sum::<f32>() / k).collect();
        Audio { rate: a.rate, channels: vec![m] }
    } else {
        a
    };
    Ok(raea_io_media::soft_resample(&a, rate))
}

pub(crate) fn write_f32le(a: &Audio, w: &mut dyn Write) -> R<()> {
    let mut buf = Vec::with_capacity(a.len() * a.channels.len() * 4);
    for i in 0..a.len() {
        for c in &a.channels {
            buf.extend_from_slice(&c[i].to_le_bytes());
        }
    }
    w.write_all(&buf).map_err(|e| e.to_string())
}

/// 16-bit PCM WAV, as ffmpeg writes by default.
pub(crate) fn write_wav(a: &Audio, out: &Path) -> R<()> {
    let ch = a.channels.len().max(1) as u16;
    let data = (a.len() * ch as usize * 2) as u32;
    let mut b = Vec::with_capacity(44 + data as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&ch.to_le_bytes());
    b.extend_from_slice(&a.rate.to_le_bytes());
    b.extend_from_slice(&(a.rate * ch as u32 * 2).to_le_bytes());
    b.extend_from_slice(&(ch * 2).to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data.to_le_bytes());
    for i in 0..a.len() {
        for c in &a.channels {
            b.extend_from_slice(&((c[i].clamp(-1.0, 1.0) * 32767.0).round() as i16).to_le_bytes());
        }
    }
    std::fs::write(out, b).map_err(|e| format!("{}: {e}", out.display()))
}

/// Mean and peak level (dBFS) over every sample of every channel, as volumedetect reports them.
pub(crate) fn levels(a: &Audio) -> (f64, f64) {
    let (mut sum, mut n, mut peak) = (0f64, 0usize, 0f32);
    for c in &a.channels {
        for x in c {
            sum += (*x as f64) * (*x as f64);
            peak = peak.max(x.abs());
            n += 1;
        }
    }
    let db = |p: f64| if p > 0.0 { 10.0 * p.log10() } else { -91.0 };
    (db(sum / n.max(1) as f64), db((peak as f64) * (peak as f64)))
}

pub fn loudness(path: &Path, from: Option<f64>, to: Option<f64>) -> R<Value> {
    let a = slice(&soft()?.decode_audio(path)?, from, to);
    if a.len() == 0 {
        return Err(format!("{}: no sound in that stretch", path.display()));
    }
    let (mean, max) = levels(&a);
    let r1 = |x: f64| (x * 10.0).round() / 10.0;
    Ok(json!({"file": path, "lufs": r1(raea_io_media::audio::lufs(&a)), "mean_db": r1(mean), "max_db": r1(max), "true_peak_db": r1(raea_io_media::audio::true_peak_db(&a)), "seconds": round3(a.duration())}))
}

// ---- measures over frames
