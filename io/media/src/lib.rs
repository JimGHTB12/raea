//! raea's media plane: one seam (`Backend`) and several implementations, so the edit's clips, voice and music
//! never depend on one tool. A decoder answers "the frame showing at t" as an as-of lookup on presentation times
//! (the latest frame with pts <= t), so any order of requests gives the same frames. Every backend hands back
//! elementary streams; muxing is pure Rust (muxide) everywhere.
//!
//!   ffmpeg  the ffmpeg CLI (the baseline)
//!   soft    portable software: openh264 (H.264, the h264 half), re_mp4 (demux), muxide (mux), Symphonia (audio), Opus (soft-audio)
//!   apple   platform hardware on Apple: AVFoundation / VideoToolbox (decode HEVC and H.264, encode H.264/HEVC + AAC)
//!   av1     royalty-free AV1: dav1d decode, rav1e encode
//!
//! Other platforms (Android MediaCodec, Linux VA-API / NVDEC) slot in behind the same trait.
//!
//! This crate is that one name: the backends compiled in (by feature), each wrapped by a signer when one is
//! installed (sign.rs), and the seam's types re-exported, so callers say `raea_io_media::backend("apple")`.

pub use raea_io_codec::*;
#[cfg(all(feature = "apple", target_vendor = "apple"))]
pub use raea_io_apple as apple;
#[cfg(feature = "av1")]
pub use raea_io_av1 as av1;
pub use raea_io_ffmpeg as ffmpeg;
#[cfg(feature = "soft-audio")]
pub use raea_io_soft as soft;
/// a source's sound in 4 s blocks, decoded just ahead of whoever reads it (the mixer's voices)
#[cfg(feature = "soft-audio")]
pub mod pcm;
/// optimized copies of long-GOP sources: where they are and how they're made
pub mod mezz;

#[cfg(feature = "soft-audio")]
use std::path::Path;

/// `secs` seconds of a file's sound from `start` (decode_audio's time): a seek and a short decode, for streaming a long
/// take as it plays.
#[cfg(feature = "soft-audio")]
pub fn decode_audio_window(path: &Path, start: f64, secs: f64) -> R<Audio> {
    soft::decode_audio_window_symphonia(path, start, secs)
}

/// How long a file's sound is, from its container, without decoding it.
#[cfg(feature = "soft-audio")]
pub fn probe_seconds(path: &Path) -> Option<f64> {
    soft::probe_seconds_symphonia(path)
}

/// The backends compiled into this build, by name.
pub fn backends() -> Vec<Box<dyn Backend>> {
    #[allow(unused_mut)] // with no backend features, nothing is pushed
    let mut v: Vec<Box<dyn Backend>> = vec![Box::new(ffmpeg::Ffmpeg)];
    #[cfg(feature = "soft-audio")]
    v.push(Box::new(soft::Soft));
    #[cfg(all(feature = "apple", target_vendor = "apple"))]
    v.push(Box::new(apple::Apple));
    #[cfg(feature = "av1")]
    v.push(Box::new(av1::Av1));
    // an installed signer wraps each backend, so its encoders sign and mark what they write (sign.rs)
    v.into_iter().map(sign::wrap).collect()
}

/// Signing what raea writes (Content Credentials, the watermark): a hook a signer installs from outside this crate
pub mod sign;
pub use sign::signs;

/// The backends to try for a file, in order: RAEA_MEDIA's, the platform's, the portable one, then the ffmpeg CLI
/// (which reads what the others don't: VP9 WebM).
pub fn preferred() -> Vec<Box<dyn Backend>> {
    let mut names: Vec<String> = std::env::var("RAEA_MEDIA").into_iter().collect();
    names.extend(["apple", "soft", "ffmpeg"].map(String::from));
    let mut all = backends();
    let mut out = vec![];
    for n in names {
        if let Some(i) = all.iter().position(|b| b.name() == n) {
            out.push(all.remove(i));
        }
    }
    out
}

pub fn backend(name: &str) -> R<Box<dyn Backend>> {
    backends()
        .into_iter()
        .find(|b| b.name() == name)
        .ok_or(format!("no backend {name:?} in this build"))
}

