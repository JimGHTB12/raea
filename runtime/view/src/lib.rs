//! The direct render's views: an edit's frames without the Bevy frame loop, on the GPU (gpuview.rs: the GPU pool,
//! the Blitz overlay and one composite.k dispatch a frame) or on the CPU (cpuview.rs, the same frames by the kernels'
//! native twins), and what they share with the engine's overlay: the 2D rows (rows.rs) drawn by Blitz (rows2d.rs),
//! clips' masks (masks.rs), and the paint device as Bevy's renderer with the frame pool's budget (device.rs).

pub mod cpuview;
mod device;
pub use device::*;
pub mod gpuview;
pub mod masks;
pub mod rows;
pub mod rows2d;
