//! How a value moves between two keys, as a pure function of time (forked from Animato 1.7.1, MIT OR Apache-2.0,
//! Aarambh Dev Hub; see NOTICE, and rebuilt to be frame-pure): `easing`'s curves (with text names, `reverse()` and
//! stretched springs), `spring`'s damped oscillator in closed form, `color`'s CSS colours blended in Oklab, and
//! `interpolate`, what a keyframed value has to do.

pub mod color;
pub mod easing;
pub mod interpolate;
pub mod spring;
