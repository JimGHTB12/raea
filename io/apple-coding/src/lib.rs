//! Coding on Apple below a whole file: VideoToolbox one packet at a time, AAC through AudioToolbox, and the
//! intra-only mezzanine copy a long-GOP source is read through.
#![cfg(target_vendor = "apple")]

/// packet-level AAC encoding (AudioToolbox), for coders handing PCM in and taking packets out
pub mod aenc;
pub mod mezz;
/// packet-level VideoToolbox coding (one compressed packet at a time), for coders handed packets
pub mod packets;
