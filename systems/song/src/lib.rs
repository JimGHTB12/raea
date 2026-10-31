//! Sound through ONNX models: a song's beats, bars and energy map (Beat This!), stems (HTDemucs), speech (Silero VAD),
//! shot cuts (TransNetV2), and DeepFilterNet3 denoising.

pub mod onnx;
pub mod beatthis;
pub mod stems;
pub mod song;
pub mod transnet;
#[cfg(target_vendor = "apple")]
pub mod vad;
#[cfg(target_vendor = "apple")]
#[cfg(feature = "denoise")]
pub mod denoise;
