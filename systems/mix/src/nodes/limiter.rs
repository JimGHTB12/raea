//! The master's look-ahead true-peak limiter.

use super::*;

/// The look-ahead true-peak limiter, streaming the render master's rule (io/codec/src/audio.rs `limit`): each sample's
/// need is the gain that keeps its true peak under the ceiling; the gain falls linearly, by at most 1/lookahead a sample,
/// to the least need in the look-ahead window (so it is there before the peak leaves the window) and is let go over 80
/// ms; each sample also takes no more than its own need. It never overshoots, so nothing is clamped (a clamp made a
/// corner at every peak of a limited kick: a click each cycle). The true peak is read 4x oversampled through a 48-tap
/// windowed sinc, as BS.1770 meters read it.
pub struct Limiter {
    delay: Vec<[f32; 2]>,
    need: Vec<f32>,
    at: usize,
    gain: f32,
    pub pre: f32,
    pub ceiling: f32,
    /// each channel's last TAPS samples in
    hist: [[f32; TAPS]; 2],
    taps: [[f32; TAPS]; 3],
}

const TAPS: usize = 12;

impl Default for Limiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Limiter {
    pub fn new() -> Limiter {
        // the samples between hist[5] and hist[6] at a quarter, half and three quarters
        let mut taps = [[0f32; TAPS]; 3];
        for (k, p) in [0.25f64, 0.5, 0.75].iter().enumerate() {
            for (j, t) in taps[k].iter_mut().enumerate() {
                let x = 5.0 + p - j as f64;
                let sinc = if x.abs() < 1e-12 { 1.0 } else { (std::f64::consts::PI * x).sin() / (std::f64::consts::PI * x) };
                // a Hann window over +-6 samples
                let w = 0.5 * (1.0 + (std::f64::consts::PI * x / 6.0).cos());
                *t = (sinc * w) as f32;
            }
        }
        Limiter { delay: Vec::new(), need: Vec::new(), at: 0, gain: 1.0, pre: 1.0, ceiling: CEILING, hist: [[0.0; TAPS]; 2], taps }
    }

    pub fn run(&mut self, l: &mut [f32], r: &mut [f32], rate: f64) {
        let n = ((LOOKAHEAD * rate).round() as usize).max(8);
        if self.need.len() != n {
            self.need = vec![1.0; n];
            self.delay = vec![[0.0; 2]; n];
            self.at = 0;
        }
        let release = (-1.0 / (0.080 * rate)).exp() as f32;
        let attack = 1.0 / n as f32;
        // aimed 0.26 dB under: the 12-tap meter reads a peak between samples up to ~1% low
        let (pre, ceiling) = (self.pre, self.ceiling * 0.97);
        let need = |v: f32| if v > ceiling { ceiling / v } else { 1.0 };
        for i in 0..l.len() {
            let frame = [l[i] * pre, r[i] * pre];
            // the sample leaving the window: its own need, settled
            let own = self.need[self.at];
            let late = std::mem::replace(&mut self.delay[self.at], frame);
            self.need[self.at] = need(frame[0].abs().max(frame[1].abs()));
            // the true peak between the samples 6 and 5 back (the oversampler's delay), on both their slots
            let mut peak = 0f32;
            for c in 0..2 {
                let h = &mut self.hist[c];
                h.copy_within(1.., 0);
                h[TAPS - 1] = frame[c];
                for tp in &self.taps {
                    let v: f32 = tp.iter().zip(h.iter()).map(|(a, b)| a * b).sum();
                    peak = peak.max(v.abs());
                }
            }
            let g = need(peak);
            for back in [6usize, 5] {
                let k = (self.at + n - back) % n;
                self.need[k] = self.need[k].min(g);
            }
            let want = self.need.iter().cloned().fold(1.0f32, f32::min).min(own);
            self.gain = if want < self.gain { (self.gain - attack).max(want) } else { want + (self.gain - want) * release };
            let k = self.gain.min(own);
            l[i] = late[0] * k;
            r[i] = late[1] * k;
            self.at = (self.at + 1) % n;
        }
    }
}

#[cfg(test)]
mod limiter_tests {
    use super::*;

    #[test]
    fn a_loud_wave_stays_under_the_ceiling_with_no_clamp_and_quiet_sound_passes() {
        let rate = 48_000.0;
        // a 2.0 sine at a quarter of the rate, 45 degrees off the samples: the true peak sits 3 dB over the samples
        let n = 48_000;
        let wave = |amp: f32| -> Vec<f32> { (0..n).map(|i| amp * (std::f32::consts::FRAC_PI_2 * i as f32 + std::f32::consts::FRAC_PI_4).sin()).collect() };
        let mut lim = Limiter::new();
        let (mut l, mut r) = (wave(2.0), wave(2.0));
        for (a, b) in l.chunks_mut(256).zip(r.chunks_mut(256)) {
            lim.run(a, b, rate);
        }
        // the sample peak under the ceiling less the 3 dB the true peak adds
        let sp = l.iter().skip(4800).fold(0f32, |m, v| m.max(v.abs()));
        assert!(sp <= CEILING * 0.7072 + 1e-3, "sample peak {sp} lets the true peak over {CEILING}");
        // quiet: through unchanged, only later by the look-ahead
        let mut q = Limiter::new();
        let (mut a, mut b) = (wave(0.3), wave(0.3));
        let input = a.clone();
        q.run(&mut a, &mut b, rate);
        let d = ((LOOKAHEAD * rate).round() as usize).max(8);
        assert!((d..n).all(|i| (a[i] - input[i - d]).abs() < 1e-6), "quiet sound changed");
    }
}
