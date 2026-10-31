//! raea's declared animations (forked from Animato 1.7.1, MIT OR Apache-2.0, Aarambh Dev Hub; see NOTICE): `track`,
//! keyframe tracks held or repeated past their keys, and `spec`, the wire format `<Animate>` sends once, compiled and
//! `sample`d per frame. Every value is a function of the frame: a seek gives exactly what a play-through gives.

mod sample;
pub mod spec;
pub mod track;

pub use spec::{Animation, Compiled, compile};
