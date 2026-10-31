//! Tone.js's effects and mixing nodes on the core, as one Firewheel node type (`ToneFx`): stereo in, a stereo sidechain
//! in (a ducked channel's key), stereo out. Each kind is a deterministic DSP: Gain, Volume, Panner, Channel/PanVol,
//! Destination, Filter, AutoFilter, FeedbackDelay, PingPongDelay, Reverb, Freeverb, JCReverb, Compressor, Limiter, EQ3,
//! Distortion, Chorus, BitCrusher, Tremolo, Vibrato, Phaser, AutoPanner, Convolver, PitchShift. Parameters ramp on the
//! sample their event names (Tone's rampTo, linearRampTo, exponentialRampTo, setValueAtTime); LFOs (an effect's own,
//! and a Tone.LFO connected to a parameter) run on the timeline's time (from the origin the node is given), so a play
//! and a render of the same events are the same samples.

use firewheel::channel_config::{ChannelConfig, ChannelCount};
use firewheel::event::ProcEvents;
use firewheel::node::{AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, NodeError, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus};
use serde_json::Value;

use crate::dsp::*;

mod process;

use process::*;

/// An event for a ToneFx node.
pub enum FxEv {
    /// a parameter to `value` over `ramp` samples (0: at once), exponentially when `exp` (Tone's exponentialRampTo)
    Param { name: String, value: f64, ramp: u32, exp: bool },
    /// the clock sample of the timeline's 0 (live: where the timeline sits on the device's clock)
    Origin(i64),
}

#[derive(Clone)]
pub struct FxNode {
    pub kind: String,
    pub params: Value,
    pub origin: i64,
    pub muted: bool,
    /// a Convolver's impulse response
    pub ir: Option<std::sync::Arc<raea_io_media::Audio>>,
}

impl AudioNode for FxNode {
    type Configuration = ();

    fn info(&self, _: &()) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new().debug_name("tone fx").channel_config(ChannelConfig { num_inputs: ChannelCount::new(4).unwrap(), num_outputs: ChannelCount::STEREO }))
    }

    fn construct_processor(&self, _: &(), cx: ConstructProcessorContext) -> Result<impl AudioNodeProcessor, NodeError> {
        Ok(self.processor(cx.stream_info.sample_rate.get() as f64))
    }
}

impl FxNode {
    /// The node's processor at `rate` (for a node type that wraps this one: bevy_seedling's, raea-seedling).
    pub fn processor(&self, rate: f64) -> impl AudioNodeProcessor + use<> {
        let mut fx = Fx::new(&self.kind, &self.params, rate, self.origin, self.muted);
        if let Some(ir) = &self.ir {
            fx.conv = Some(Box::new(crate::conv::pair(ir, rate, self.params.get("normalize").and_then(|v| v.as_bool()).unwrap_or(true))));
        }
        fx
    }
}

#[derive(Clone, Copy)]
struct Ramp {
    v: f64,
    to: f64,
    step: f64,
    left: u32,
    /// the step is a ratio (an exponential ramp)
    mul: bool,
}

impl Ramp {
    fn new(v: f64) -> Ramp {
        Ramp { v, to: v, step: 0.0, left: 0, mul: false }
    }
    fn set(&mut self, to: f64, n: u32, exp: bool) {
        if n == 0 {
            self.v = to;
            self.to = to;
            self.left = 0;
        } else {
            self.to = to;
            self.left = n;
            // exponential: the same ratio each sample (Tone's exponentialRampTo; as Tone does, from or to 0 or across
            // a sign change it can't be, and it's linear)
            self.mul = exp && self.v * to > 0.0;
            self.step = if self.mul { (to / self.v).powf(1.0 / n as f64) } else { (to - self.v) / n as f64 };
        }
    }
    fn tick(&mut self) -> f64 {
        if self.left > 0 {
            if self.mul {
                self.v *= self.step;
            } else {
                self.v += self.step;
            }
            self.left -= 1;
            if self.left == 0 {
                self.v = self.to;
            }
        }
        self.v
    }
}

fn num(v: &Value, path: &[&str], d: f64) -> f64 {
    let mut x = v;
    for k in path {
        match x.get(*k) {
            Some(y) => x = y,
            None => return d,
        }
    }
    x.as_f64().or_else(|| x.as_bool().map(|b| if b { 1.0 } else { 0.0 })).unwrap_or(d)
}

/// Freeverb's tunings at 44.1 kHz, scaled to the rate.
const COMBS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASSES: [usize; 4] = [556, 441, 341, 225];
const STEREO_SPREAD: usize = 23;

struct Verb {
    combs: Vec<(Delay, usize, OnePole)>,
    alls: Vec<(Delay, usize)>,
}

impl Verb {
    fn new(rate: f64, spread: usize) -> Verb {
        let k = rate / 44100.0;
        Verb {
            combs: COMBS.iter().map(|n| {
                let l = ((n + spread) as f64 * k) as usize;
                (Delay::new(l + 2), l, OnePole::default())
            }).collect(),
            alls: ALLPASSES.iter().map(|n| {
                let l = ((n + spread) as f64 * k) as usize;
                (Delay::new(l + 2), l)
            }).collect(),
        }
    }
    /// `fb(len)`: each comb's feedback by its length; `damp` the damping lowpass coefficient
    fn run(&mut self, x: f32, fb: &dyn Fn(usize) -> f32, damp: f32) -> f32 {
        let mut out = 0f32;
        for (d, l, lp) in self.combs.iter_mut() {
            let y = d.read(*l as f64);
            let f = lp.run(y, 1.0 - damp);
            d.push(x + f * fb(*l));
            out += y;
        }
        for (d, l) in self.alls.iter_mut() {
            let b = d.read(*l as f64);
            let y = -out + b;
            d.push(out + b * 0.5);
            out = y;
        }
        out
    }
}

/// The node's kind, picked once (a string matched per sample cost the reverb song a fifth of a core).
#[derive(Clone, Copy, PartialEq, Debug)]
enum K {
    Gain,
    Volume,
    Panner,
    Channel,
    Filter,
    AutoFilter,
    Delay,
    PingPong,
    Reverb,
    Freeverb,
    JcReverb,
    Comp,
    Limiter,
    Eq3,
    Dist,
    Chorus,
    Vibrato,
    Crusher,
    Tremolo,
    AutoPanner,
    Phaser,
    Convolver,
    PitchShift,
    Pass,
}

/// A Tone.LFO connected to a parameter: its value is the parameter's from `started` on (before, Tone's stopped value:
/// where the wave starts).
struct Mod {
    p: usize,
    freq: f64,
    min: f64,
    max: f64,
    amp: f64,
    wave: u8,
    phase: f64,
    started: Option<f64>,
}

impl Mod {
    fn value(&self, t: f64) -> f64 {
        let lt = self.started.map(|s| (t - s).max(0.0)).unwrap_or(0.0);
        let ph = (lt * self.freq + self.phase / 360.0).rem_euclid(1.0);
        let w = match self.wave {
            1 => if ph < 0.5 { 1.0 } else { -1.0 },
            2 => if ph < 0.25 { 4.0 * ph } else if ph < 0.75 { 2.0 - 4.0 * ph } else { 4.0 * ph - 4.0 },
            3 => if ph < 0.5 { 2.0 * ph } else { 2.0 * ph - 2.0 },
            _ => (ph * std::f64::consts::TAU).sin(),
        } * self.amp;
        self.min + (self.max - self.min) * (0.5 + 0.5 * w)
    }
}

fn kind_of(s: &str) -> K {
    match s {
        "Gain" => K::Gain,
        "Volume" => K::Volume,
        "Destination" => K::Volume,
        "Panner" => K::Panner,
        "Channel" => K::Channel,
        "PanVol" => K::Channel,
        "Filter" => K::Filter,
        "AutoFilter" => K::AutoFilter,
        "FeedbackDelay" => K::Delay,
        "PingPongDelay" => K::PingPong,
        "Reverb" => K::Reverb,
        "Freeverb" => K::Freeverb,
        "JCReverb" => K::JcReverb,
        "Compressor" => K::Comp,
        "Limiter" => K::Limiter,
        "EQ3" => K::Eq3,
        "Distortion" => K::Dist,
        "Chorus" => K::Chorus,
        "Vibrato" => K::Vibrato,
        "BitCrusher" => K::Crusher,
        "Tremolo" => K::Tremolo,
        "AutoPanner" => K::AutoPanner,
        "Phaser" => K::Phaser,
        "Convolver" => K::Convolver,
        "PitchShift" => K::PitchShift,
        _ => K::Pass,
    }
}
