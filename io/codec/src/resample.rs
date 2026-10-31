use crate::Audio;

/// Band-limited resampling: a Kaiser-windowed sinc (32 zero crossings each side, beta 9, cutoff just under the lower
/// Nyquist), the same family as ffmpeg's swresample, so mixes agree with ffmpeg's to well under -60 dB.
pub fn soft_resample(a: &Audio, to: u32) -> Audio {
    if a.rate == to || a.channels.is_empty() {
        return a.clone();
    }
    // a rate of 0 is no sound to speak of (and n would be infinite)
    if a.rate == 0 || to == 0 {
        return Audio { rate: to, channels: vec![Vec::new(); a.channels.len()] };
    }
    let (from, to_f) = (a.rate as f64, to as f64);
    let n = (a.len() as f64 * to_f / from).round() as usize;
    let cutoff = 0.97 * from.min(to_f) / from; // in cycles per input sample x2
    let zc = 32.0;
    let half = (zc / cutoff).ceil() as i64;
    let beta = 9.0f64;
    let i0 = |x: f64| {
        let (mut s, mut t, mut k) = (1.0, 1.0, 1.0);
        while t > 1e-12 * s {
            t *= (x / (2.0 * k)).powi(2);
            s += t;
            k += 1.0;
        }
        s
    };
    let i0b = i0(beta);
    // the kernel at 1024 phases between input samples
    const PH: usize = 1024;
    let taps = (2 * half + 1) as usize;
    let mut table = vec![0f32; (PH + 1) * taps];
    for p in 0..=PH {
        let frac = p as f64 / PH as f64;
        for k in 0..taps {
            let x = (k as i64 - half) as f64 - frac;
            let u = x * cutoff;
            let sinc = if u.abs() < 1e-12 {
                1.0
            } else {
                (std::f64::consts::PI * u).sin() / (std::f64::consts::PI * u)
            };
            let r = x / (half as f64 + 1.0);
            let w = if r.abs() >= 1.0 {
                0.0
            } else {
                i0(beta * (1.0 - r * r).sqrt()) / i0b
            };
            table[p * taps + k] = (cutoff * sinc * w) as f32;
        }
    }
    let step = from / to_f;
    // each output sample on its own (the same taps summed in the same order as one at a time, so the same bits), in
    // spans across threads; a sample whose taps all land inside the input skips the bounds test. A film's master
    // oversamples its 4.5 minutes 4x up to five times: 26.7 s a render on one thread (2026-10-20)
    let one = |c: &[f32], i: usize| -> f32 {
        let x = i as f64 * step;
        let j = x.floor() as i64;
        let p = (((x - j as f64) * PH as f64).round() as usize).min(PH);
        let row = &table[p * taps..(p + 1) * taps];
        let mut acc = 0f32;
        let lo = j - half;
        if lo >= 0 && ((lo + taps as i64) as usize) <= c.len() {
            let src = &c[lo as usize..lo as usize + taps];
            for (x, w) in src.iter().zip(row) {
                acc += x * w;
            }
        } else {
            for (k, w) in row.iter().enumerate() {
                let idx = lo + k as i64;
                if idx >= 0 && (idx as usize) < c.len() {
                    acc += c[idx as usize] * w;
                }
            }
        }
        acc
    };
    let threads = if n * taps < 1 << 22 { 1 } else { std::thread::available_parallelism().map(|t| t.get()).unwrap_or(4).clamp(1, 8) };
    let channels = a
        .channels
        .iter()
        .map(|c| {
            let mut out = vec![0f32; n];
            if threads == 1 {
                for (i, o) in out.iter_mut().enumerate() {
                    *o = one(c, i);
                }
            } else {
                let span = n.div_ceil(threads);
                std::thread::scope(|sc| {
                    for (t, part) in out.chunks_mut(span).enumerate() {
                        let one = &one;
                        sc.spawn(move || {
                            for (k, o) in part.iter_mut().enumerate() {
                                *o = one(c, t * span + k);
                            }
                        });
                    }
                });
            }
            out
        })
        .collect();
    Audio { rate: to, channels }
}


#[cfg(test)]
mod resample_tests {
    use super::*;

    /// soft_resample as it was (one sample at a time, a bounds test on every tap): the threaded one must give the
    /// same bits
    fn reference(a: &Audio, to: u32) -> Vec<Vec<f32>> {
        let (from, to_f) = (a.rate as f64, to as f64);
        let n = (a.len() as f64 * to_f / from).round() as usize;
        let cutoff = 0.97 * from.min(to_f) / from;
        let half = (32.0 / cutoff).ceil() as i64;
        let i0 = |x: f64| {
            let (mut s, mut t, mut k) = (1.0, 1.0, 1.0);
            while t > 1e-12 * s {
                t *= (x / (2.0 * k)).powi(2);
                s += t;
                k += 1.0;
            }
            s
        };
        let i0b = i0(9.0);
        const PH: usize = 1024;
        let taps = (2 * half + 1) as usize;
        let mut table = vec![0f32; (PH + 1) * taps];
        for p in 0..=PH {
            let frac = p as f64 / PH as f64;
            for k in 0..taps {
                let x = (k as i64 - half) as f64 - frac;
                let u = x * cutoff;
                let sinc = if u.abs() < 1e-12 { 1.0 } else { (std::f64::consts::PI * u).sin() / (std::f64::consts::PI * u) };
                let r = x / (half as f64 + 1.0);
                let w = if r.abs() >= 1.0 { 0.0 } else { i0(9.0 * (1.0 - r * r).sqrt()) / i0b };
                table[p * taps + k] = (cutoff * sinc * w) as f32;
            }
        }
        let step = from / to_f;
        a.channels
            .iter()
            .map(|c| {
                (0..n)
                    .map(|i| {
                        let x = i as f64 * step;
                        let j = x.floor() as i64;
                        let p = (((x - j as f64) * PH as f64).round() as usize).min(PH);
                        let row = &table[p * taps..(p + 1) * taps];
                        let mut acc = 0f32;
                        for (k, w) in row.iter().enumerate() {
                            let idx = j + k as i64 - half;
                            if idx >= 0 && (idx as usize) < c.len() {
                                acc += c[idx as usize] * w;
                            }
                        }
                        acc
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn threaded_resample_is_the_same_bits() {
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        let mut noise = |n: usize| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    (seed >> 40) as f32 / (1u64 << 23) as f32 - 1.0
                })
                .collect()
        };
        // long enough to be threaded (n * taps >= 2^22), and short ones done on one thread
        for (rate, to, len) in [(48000, 192000, 30000), (48000, 44100, 100000), (44100, 48000, 90000), (48000, 192000, 50), (22050, 48000, 7)] {
            let a = Audio { rate, channels: vec![noise(len), noise(len)] };
            let got = soft_resample(&a, to);
            let want = reference(&a, to);
            for (g, w) in got.channels.iter().zip(&want) {
                assert_eq!(g.len(), w.len());
                assert!(g.iter().zip(w).all(|(x, y)| x.to_bits() == y.to_bits()), "{rate} -> {to}, {len} samples");
            }
        }
    }
}
