//! Springs (forked from animato-spring 1.7.1). Animato steps a spring with semi-implicit Euler each update(dt), so its
//! value depends on the frame rate and on how it was played. Here a spring is the damped harmonic oscillator's closed
//! form: m x'' + c x' + k (x - to) = 0 with x(0) = from, x'(0) = 0, solved for each damping regime and sampled at any
//! time, so a seek, a play-through and a different fps all agree. The semantics match raea's spring()
//! (sdk/raea/src/spring.ts): the same settle measure, delay, reverse, stretch to a duration and overshoot clamping.

/// |u| below this counts as settled (raea's durationRestThreshold default).
pub const DEFAULT_REST: f64 = 0.005;
const CRITICAL_TOLERANCE: f64 = 1e-9;
const SETTLE_SEARCH_LIMIT: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Regime {
    Critical { w0: f64 },
    Under { decay: f64, wd: f64, amp: f64 },
    Over { decay: f64, wh: f64 },
}

/// The oscillator's unit response u(t) = (x(t) - to) / (from - to): u(0) = 1, u -> 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oscillator {
    regime: Regime,
    /// the first time u reaches 0 (x crosses `to`), infinite when it never does
    pub first_crossing: f64,
}

impl Oscillator {
    pub fn new(mass: f64, damping: f64, stiffness: f64) -> Result<Oscillator, String> {
        if !(mass > 0.0 && mass.is_finite()) {
            return Err(format!("mass must be a positive number, got {mass}"));
        }
        if !(stiffness > 0.0 && stiffness.is_finite()) {
            return Err(format!("stiffness must be a positive number, got {stiffness}"));
        }
        if !(damping >= 0.0 && damping.is_finite()) {
            return Err(format!("damping must not be negative, got {damping}"));
        }
        let w0 = (stiffness / mass).sqrt();
        let zeta = damping / (2.0 * (stiffness * mass).sqrt());
        if (zeta - 1.0).abs() < CRITICAL_TOLERANCE {
            return Ok(Oscillator { regime: Regime::Critical { w0 }, first_crossing: f64::INFINITY });
        }
        if zeta < 1.0 {
            let decay = zeta * w0;
            let wd = w0 * (1.0 - zeta * zeta).sqrt();
            let amp = 1.0 / (1.0 - zeta * zeta).sqrt();
            let first_crossing = (std::f64::consts::PI - wd.atan2(decay)) / wd;
            return Ok(Oscillator { regime: Regime::Under { decay, wd, amp }, first_crossing });
        }
        let decay = zeta * w0;
        let wh = w0 * (zeta * zeta - 1.0).sqrt();
        Ok(Oscillator { regime: Regime::Over { decay, wh }, first_crossing: f64::INFINITY })
    }

    /// u(t), t in seconds.
    pub fn unit(&self, t: f64) -> f64 {
        match self.regime {
            Regime::Critical { w0 } => (-w0 * t).exp() * (1.0 + w0 * t),
            Regime::Under { decay, wd, .. } => (-decay * t).exp() * ((wd * t).cos() + (decay / wd) * (wd * t).sin()),
            Regime::Over { decay, wh } => (-decay * t).exp() * ((wh * t).cosh() + (decay / wh) * (wh * t).sinh()),
        }
    }

    /// An upper bound on |u| from t on (the decay envelope).
    pub fn envelope(&self, t: f64) -> f64 {
        match self.regime {
            Regime::Under { decay, amp, .. } => amp * (-decay * t).exp(),
            _ => self.unit(t),
        }
    }

    /// The settle time in frames at `fps`: the first frame after which |u| stays below `threshold`.
    pub fn natural_frames(&self, fps: f64, threshold: f64) -> u64 {
        let mut last_above: i64 = -1;
        for f in 0..SETTLE_SEARCH_LIMIT {
            let t = f as f64 / fps;
            if self.envelope(t) < threshold {
                break;
            }
            if self.unit(t).abs() >= threshold {
                last_above = f as i64;
            }
        }
        (last_above + 1) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overshoots_then_settles_at_any_seek() {
        // mass 1, damping 10, stiffness 100: zeta 0.5, so it overshoots
        let o = Oscillator::new(1.0, 10.0, 100.0).unwrap();
        // u is what's left to go (1 at rest at the start, 0 at the target): an overshoot takes it below 0
        let peak = (0..60).map(|f| 1.0 - o.unit(f as f64 / 30.0)).fold(0.0, f64::max);
        assert!(peak > 1.1 && peak < 1.2, "peak {peak}");
        let n = o.natural_frames(30.0, DEFAULT_REST);
        assert!((25..40).contains(&n), "settles in {n} frames");
        for f in n..n + 300 {
            assert!(o.unit(f as f64 / 30.0).abs() < DEFAULT_REST);
        }
        // critical damping never overshoots
        let c = Oscillator::new(1.0, 20.0, 100.0).unwrap();
        assert!((0..120).all(|f| c.unit(f as f64 / 30.0) >= -1e-12));
    }
}
