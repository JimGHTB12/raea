//! Your own draw in a camera's phase (docs/control.feature FC2d), as Bevy's shader_advanced/custom_phase_item queues
//! one: an entity with a PhaseItem (and an Aabb, so it's seen) is drawn in each view's Opaque3d phase by a pipeline of
//! its shader file as written, its `vertex` and `fragment` entry points, reading its own vertex buffer (laid out as
//! its `attributes` say) and index buffer. <PhaseItem> in raea's TSX gives an <Entity> the component; draw.rs queues,
//! prepares and draws it. A MeshPass on a Mesh3d entity draws that mesh again by its own shader in a phase after the
//! main opaque pass, as Bevy's shader_advanced/custom_render_phase draws its stencil (mesh_pass.rs), and
//! ShaderInstances draws it many times in one draw, a vertex buffer of yours stepped per instance
//! (shader_advanced/custom_shader_instancing, instances.rs).

use bevy::camera::visibility::{self, VisibilityClass};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::RenderApp;

mod draw;
mod instances;
mod mesh_pass;

pub use instances::ShaderInstances;
pub use mesh_pass::MeshPass;

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

/// A component that draws by a shader file of its own: the file, and where its registration is kept.
pub(crate) trait Shaded: Component<Mutability = bevy::ecs::component::Mutable> {
    const NAME: &'static str;
    fn shader(&self) -> &str;
    fn kept(&self) -> Option<&(String, u128)>;
    fn keep(&mut self, r: (String, u128));
}

// each one's `shader` path and `registered` field
macro_rules! shaded {
    ($($t:ty: $name:literal),*) => {$(
        impl Shaded for $t {
            const NAME: &'static str = $name;
            fn shader(&self) -> &str {
                &self.shader
            }
            fn kept(&self) -> Option<&(String, u128)> {
                self.registered.as_ref()
            }
            fn keep(&mut self, r: (String, u128)) {
                self.registered = Some(r);
            }
        }
    )*};
}
shaded!(PhaseItem: "PhaseItem", MeshPass: "MeshPass", ShaderInstances: "ShaderInstances");

/// Each new or changed `C`'s shader file read and registered as written (a file it can't read is logged).
pub(crate) fn register<C: Shaded>(world: &mut World) {
    let mut q = world.query_filtered::<(Entity, &C), Changed<C>>();
    let mut todo = Vec::new();
    for (e, c) in q.iter(world) {
        if c.kept().map(|r| r.0.as_str()) != Some(c.shader()) {
            todo.push((e, c.shader().to_owned()));
        }
    }
    for (e, path) in todo {
        match std::fs::read_to_string(&path) {
            Ok(src) => {
                let id = raea_sys_shader::register(world, src, &[], C::NAME, "raea_sys_phase");
                if let Some(mut c) = world.get_mut::<C>(e) {
                    c.keep((path, id));
                }
            }
            Err(err) => error!("{}: its shader {path}: {err}", C::NAME),
        }
    }
}

/// The mesh pass's and the instances' pipelines, each Bevy's mesh pipeline under its own specialize.
fn pipelines(mut commands: Commands, mesh: Res<bevy::pbr::MeshPipeline>) {
    commands.insert_resource(mesh_pass::PassPipeline { mesh: mesh.clone() });
    commands.insert_resource(instances::InstancesPipeline { mesh: mesh.clone() });
}

pub fn install(app: &mut App) {
    app.register_type::<PhaseItem>();
    app.add_systems(PostUpdate, register::<PhaseItem>);
    app.add_plugins(ExtractComponentPlugin::<PhaseItem>::default());
    draw::install(app);
    mesh_pass::install(app);
    instances::install(app);
    if let Some(r) = app.get_sub_app_mut(bevy::render::RenderApp) {
        r.add_systems(bevy::render::RenderStartup, pipelines.after(bevy::pbr::MeshPipelineSystems));
    }
}
