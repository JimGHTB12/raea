//! Generation flows' ground. A flow (flow.rs) names generation steps by what they make (image.generate,
//! video.generate, audio.sfx, mesh.generate, …), its ops in one table (types.rs writes it as raea/gen's TypeScript);
//! the hosted backends a step runs on are here too: made.app (one key for 118 image, video and audio models at the
//! provider's price), fal, Replicate, OpenRouter, Modal endpoints, Fish Audio's speech and Tripo 3D (its own CLI, the founder's key). ComfyUI is raea-io-gen-comfy; routing and running
//! are raea-io-gen.

pub mod direct;
pub mod fal;
pub mod fish;
pub mod flow;
pub mod http;
pub mod made;
pub mod modal;
pub mod openrouter;
pub mod replicate;
pub mod tripo;
pub mod types;

/// A setting: the environment, then raea's settings file (raea_io_home::setting).
pub use raea_io_home::setting;
