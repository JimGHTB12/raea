//! A surface that shows only the shadows falling on it (`<Mesh shadow="catch">`, three.js's ShadowMaterial): a floor
//! under a product shot over a picture or the page, so the product sits on what's behind it. Where nothing shades it,
//! it's clear; where a light's shadow falls, it's its material's colour at its material's opacity, scaled by how much of
//! the light is blocked (every shadow-casting light's shadow multiplied, as three's getShadowMask). It's blended and
//! left out of the depth prepass, so the fog and outlines see through it.
//!
//! The row is a mesh row with `shadow_catch` 1 (engine.rs spawns it with this material in place of its standard one).
//! Its colour and opacity are the spawn's: it isn't re-coloured frame by frame.

use super::*;
use bevy::pbr::{Material, MaterialPlugin};
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub fn install(app: &mut App) {
    bevy::asset::embedded_asset!(app, "shadow_catch.wesl");
    app.add_plugins(MaterialPlugin::<ShadowCatchMaterial>::default());
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct ShadowCatchMaterial {
    /// the shadow's colour, linear rgb, and its opacity where the light is wholly blocked
    #[uniform(0)]
    pub color: Vec4,
}

impl ShadowCatchMaterial {
    /// from the surface the row asked for: its base colour and opacity
    pub fn of(base: &StandardMaterial) -> Self {
        let c = base.base_color.to_linear();
        Self { color: Vec4::new(c.red, c.green, c.blue, c.alpha) }
    }
}

impl Material for ShadowCatchMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://raea_sys_material/shadow_catch.wesl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
    /// not in the camera's depth prepass: what's behind it stays seen by the passes that read that depth
    fn enable_prepass() -> bool {
        false
    }
}
