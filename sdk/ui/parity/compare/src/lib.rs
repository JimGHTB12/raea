//! ui-parity-compare: the oracle's side of ui-parity and the comparison. Chrome driven over the DevTools protocol
//! (chrome.rs), what each side's tree and frame read as (compare.rs: boxes, values, states, SSIM from ssim.rs) and the
//! accessibility records (a11y.rs), with the input both sides take.

pub mod a11y;
pub mod chrome;
pub mod compare;
pub mod ssim;

use std::sync::atomic::{AtomicU64, Ordering};

/// One input event, in viewport points, as both sides take it.
#[derive(Clone, Debug)]
pub enum Input {
    Move(f64, f64),
    /// x, y, click count, right button
    Down(f64, f64, u32, bool),
    Up(f64, f64, u32, bool),
    Leave,
    /// x, y, dx, dy (the DOM's sign: positive scrolls toward the end)
    Wheel(f64, f64, f64, f64),
    Key(String, Vec<String>),
    Text(String),
    Resize(f64, f64),
    /// at least this many frames before the next read
    Wait(u32),
}

/// A number no other call in this process gets (a fresh Chrome profile's name).
pub fn uniq() -> u64 {
    static N: AtomicU64 = AtomicU64::new(0);
    N.fetch_add(1, Ordering::SeqCst)
}
