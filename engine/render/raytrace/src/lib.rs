//! raea-render's ray tracing (docs/raytracing.feature): Bevy's Solari for a scene whose camera asks for it (RT3,
//! solari.rs), and MetalFX's temporal denoiser on its lighting on the Mac (RT4, metalfx.rs). raea-render brings this
//! crate in under its `raytracing` feature and adds it to an engine only when the program asks.

use bevy::prelude::*;
use std::collections::HashMap;

mod solari;
#[cfg(target_os = "macos")]
mod metalfx;

pub use solari::add;

/// The program asks for Solari: a camera's SolariLighting or Pathtracer component (`prog` is raea-render's Program).
pub fn wanted(prog: &impl std::fmt::Debug) -> bool {
    format!("{prog:?}").contains("bevy_solari::")
}

/// After app.finish(): MetalFX's denoiser after Solari's lighting, on the Mac; nothing elsewhere.
pub fn add_denoiser(app: &mut App) {
    #[cfg(target_os = "macos")]
    metalfx::add(app);
    #[cfg(not(target_os = "macos"))]
    let _ = app;
}
