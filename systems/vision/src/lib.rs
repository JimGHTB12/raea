//! Apple Vision and Core ML models on frames: faces, hands, hair, depth, objects, optical flow, text, the camera's
//! move, and sound classification. `vision::Vision` is the trait the portable models implement too.

#[cfg(target_vendor = "apple")]
pub mod coreml;
#[cfg(target_vendor = "apple")]
pub mod picture;
#[cfg(target_vendor = "apple")]
pub mod face;
#[cfg(target_vendor = "apple")]
pub mod hands;
#[cfg(target_vendor = "apple")]
pub mod hair;
pub mod depth;
pub mod object;
#[cfg(target_vendor = "apple")]
pub mod flow;
pub mod vision;
pub mod textread;
pub mod camera;
#[cfg(target_vendor = "apple")]
pub mod sound;
