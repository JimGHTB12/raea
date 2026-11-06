//! Generation flows run: route.rs sends each step of a flow to a backend (made.app, fal, Replicate, OpenRouter, a Modal
//! endpoint, Fish Audio, a ComfyUI server, or a model the host runs locally) and run.rs makes them, cached by content; cli.rs is
//! `raea gen`. The flow format and hosted backends are raea-io-gen-core, ComfyUI import and export raea-io-gen-comfy,
//! both re-exported here under their old paths.

pub mod cli;
mod hosted;
pub mod native;
pub mod route;
pub mod run;
#[cfg(test)]
mod tests;

pub use raea_io_gen_comfy as comfy;
pub use raea_io_gen_core::{direct, fal, fish, flow, http, made, modal, openrouter, replicate, setting, types};
