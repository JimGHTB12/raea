//! Your own draw in a camera's phase (docs/control.feature FC2d), as Bevy's shader_advanced/custom_phase_item queues
//! one: an entity with a PhaseItem (and an Aabb, so it's seen) is drawn in each view's Opaque3d phase by a pipeline of
//! its shader file as written, its `vertex` and `fragment` entry points, reading its own vertex buffer (laid out as
//! its `attributes` say) and index buffer. <PhaseItem> in raea's TSX gives an <Entity> the component; draw.rs queues,
//! prepares and draws it.

use bevy::camera::visibility::{self, VisibilityClass};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::RenderApp;

mod draw;

/// A vertex attribute: its format (Bevy's VertexFormat by name, "Float32x3"), byte offset and shader location.
#[derive(Clone, Debug, Default, Reflect)]
pub struct PhaseAttribute {
    pub format: String,
    pub offset: u64,
    pub location: u32,
}

/// What a phase item draws: its shader file (as written) and entry points, its vertex buffer's words (32-bit each,
/// each `stride` bytes a vertex) and attributes, its indices (u32), and its depth state: whether it writes depth and
/// how it compares (Bevy's CompareFunction by name).
#[derive(Component, Clone, Debug, Default, Reflect, ExtractComponent)]
#[extract_app(RenderApp)]
#[reflect(Component, Default)]
#[require(VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<PhaseItem>)]
pub struct PhaseItem {
    pub shader: String,
    pub vertex: String,
    pub fragment: String,
    pub stride: u64,
    pub attributes: Vec<PhaseAttribute>,
    pub words: Vec<f32>,
    pub indices: Vec<u32>,
    pub depth_write: bool,
    pub depth_compare: String,
    /// its shader file, registered as written (raea-sys-shader), and the id register gave it
    #[reflect(ignore)]
    pub registered: Option<(String, u128)>,
}

/// Each new or changed PhaseItem's shader file read and registered (a file it can't read is logged).
fn register(world: &mut World) {
    let mut q = world.query_filtered::<(Entity, &PhaseItem), Changed<PhaseItem>>();
    let items: Vec<(Entity, String)> = q.iter(world).filter(|(_, p)| p.registered.as_ref().map(|r| &r.0) != Some(&p.shader)).map(|(e, p)| (e, p.shader.clone())).collect();
    for (e, path) in items {
        match std::fs::read_to_string(&path) {
            Ok(src) => {
                let id = raea_sys_shader::register(world, src, &[], "raea phase item", "raea_sys_phase");
                if let Some(mut p) = world.get_mut::<PhaseItem>(e) {
                    p.registered = Some((path, id));
                }
            }
            Err(err) => error!("PhaseItem: its shader {path}: {err}"),
        }
    }
}

pub fn install(app: &mut App) {
    app.register_type::<PhaseItem>();
    app.add_systems(PostUpdate, register);
    app.add_plugins(ExtractComponentPlugin::<PhaseItem>::default());
    draw::install(app);
}
