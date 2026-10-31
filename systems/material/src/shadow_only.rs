//! A mesh that casts shadows without being drawn (`<Mesh shadow="only">`): an off-screen wall or roof whose shadow falls
//! into the frame, a blind's slats cutting the sun's shafts in the fog, a stand-in shape for something drawn another
//! way. Its surface is this material, opaque, so the lights' shadow passes draw it depth only as they do any opaque
//! mesh (and the volumetric fog, walking the same maps, sees it block the light); its colour pass discards every pixel
//! and it's left out of the depth prepass, so the camera sees nothing of it.
//!
//! The row is a mesh row with `shadow_only` 1 (engine.rs spawns it with this material in place of its standard one).

use super::*;
use bevy::pbr::{Material, MaterialPlugin};
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

pub fn install(app: &mut App) {
    bevy::asset::embedded_asset!(app, "shadow_only.wesl");
    app.add_plugins(MaterialPlugin::<ShadowOnlyMaterial>::default());
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
pub struct ShadowOnlyMaterial {
    /// unused: a bind group has to hold something
    #[uniform(0)]
    unused: Vec4,
}

impl Material for ShadowOnlyMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://raea_sys_material/shadow_only.wesl".into()
    }
    /// not in the camera's depth prepass (the fog and the outlines read that depth): only the shadow passes see it
    fn enable_prepass() -> bool {
        false
    }
}
