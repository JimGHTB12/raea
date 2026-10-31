//! Tone.js's instruments and effects on raea's mixer core: deterministic DSP (oscillators, envelopes, filters,
//! convolution), the instruments a raea/tone graph names, its effects as one Firewheel node type, and the graph
//! built from its JSON. raea-sys-mix plays them, live and offline, sample for sample the same.

pub mod conv;
pub mod dsp;
pub mod fx;
pub mod tone;
pub mod tonegraph;
