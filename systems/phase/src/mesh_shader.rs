//! A camera's mesh-shader draw (docs/control.feature FC2d), as Bevy's shader_advanced/mesh_shader_intro draws its
//! cubes: a camera with a MeshShaderPass runs your task shader (when it has one) and mesh shader, which make their
//! primitives on the GPU and hand them to your fragment shader, straight into the view's colour and depth (compared
//! GreaterEqual, written) after the main opaque pass. Each is a shader file compiled as written, with the one entry point
//! of its stage, and `groups` is the draw's workgroup count (draw_mesh_tasks: the task shader's, or the mesh shader's
//! with no task shader). Bind group 0 is Globals (binding 0) and the view uniform (binding 1, dynamic), in every stage.
//! It needs a GPU with mesh shaders (wgpu's EXPERIMENTAL_MESH_SHADER, which raea's device takes where the adapter has
//! it); without them the pass is logged and draws nothing.

use std::collections::HashMap;

use bevy::camera::{MainPassResolutionOverride, Viewport};
use bevy::core_pipeline::core_3d::{main_opaque_pass_3d, CORE_3D_DEPTH_FORMAT};
use bevy::core_pipeline::{Core3d, Core3dSystems};
use bevy::material::descriptor::{MeshPipelineDescriptor, MeshState, TaskState};
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::globals::{GlobalsBuffer, GlobalsUniform};
use bevy::render::render_resource::binding_types::uniform_buffer;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice, ViewQuery};
use bevy::render::settings::WgpuFeatures;
use bevy::render::view::{ViewDepthStencilTexture, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};

/// A camera's mesh-shader draw: its task shader file ("" for none), mesh and fragment shader files, and its workgroup
/// count.
#[derive(Component, Clone, Debug, Default, Reflect, ExtractComponent)]
#[extract_app(RenderApp)]
#[reflect(Component, Default)]
#[type_path = "raea_sys_phase"]
pub struct MeshShaderPass {
    pub task: String,
    pub mesh: String,
    pub fragment: String,
    pub groups: [u32; 3],
    /// its files, registered as written (raea-sys-shader), and the ids register gave them (0 for no task shader)
    #[reflect(ignore)]
    pub registered: Option<([String; 3], [u128; 3])>,
}

impl MeshShaderPass {
    fn files(&self) -> [String; 3] {
        [self.task.clone(), self.mesh.clone(), self.fragment.clone()]
    }
}

pub fn install(app: &mut App) {
    app.register_type::<MeshShaderPass>();
    app.add_systems(PostUpdate, register);
    app.add_plugins(ExtractComponentPlugin::<MeshShaderPass>::default());
    let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
    r.add_systems(RenderStartup, init)
        .add_systems(Render, queue.in_set(RenderSystems::Queue))
        .add_systems(Core3d, draw.after(main_opaque_pass_3d).in_set(Core3dSystems::MainPass));
}

/// Each new or changed pass's files read and registered as written (a file it can't read is logged).
fn register(world: &mut World) {
    let mut q = world.query_filtered::<(Entity, &MeshShaderPass), Changed<MeshShaderPass>>();
    let todo: Vec<_> = q.iter(world).filter(|(_, p)| p.registered.as_ref().map(|r| &r.0) != Some(&p.files())).map(|(e, p)| (e, p.files())).collect();
    'pass: for (e, files) in todo {
        let mut ids = [0; 3];
        for (i, f) in files.iter().enumerate().filter(|(i, f)| *i > 0 || !f.is_empty()) {
            match crate::file(world, f, "MeshShaderPass") {
                Ok(id) => ids[i] = id,
                Err(err) => {
                    error!("MeshShaderPass: its shader {f}: {err}");
                    continue 'pass;
                }
            }
        }
        if let Some(mut p) = world.get_mut::<MeshShaderPass>(e) {
            p.registered = Some((files, ids));
        }
    }
}

/// The passes' bind group layout, and each pass's pipeline by its shaders and the view's target.
#[derive(Resource)]
struct Pipelines {
    layout: BindGroupLayoutDescriptor,
    made: HashMap<([u128; 3], TextureFormat, u32), CachedRenderPipelineId>,
}

fn init(mut commands: Commands) {
    let stages = ShaderStages::TASK | ShaderStages::MESH | ShaderStages::FRAGMENT;
    let layout = BindGroupLayoutDescriptor::new("raea_mesh_shader_pass", &BindGroupLayoutEntries::sequential(stages, (uniform_buffer::<GlobalsUniform>(false), uniform_buffer::<ViewUniform>(true))));
    commands.insert_resource(Pipelines { layout, made: HashMap::new() });
}

/// The key a view's pipeline is kept by: its shaders, its target's format and its sample count.
fn key(p: &MeshShaderPass, target: &ViewTarget, msaa: &Msaa) -> Option<([u128; 3], TextureFormat, u32)> {
    Some((p.registered.as_ref()?.1, target.main_texture_format(), msaa.samples()))
}

/// Each view's pipeline queued once its shaders are registered, where the device has mesh shaders.
fn queue(cache: Res<PipelineCache>, device: Res<RenderDevice>, mut pipes: ResMut<Pipelines>, views: Query<(&MeshShaderPass, &ViewTarget, &Msaa)>, mut told: Local<bool>) {
    for (p, target, msaa) in &views {
        let Some(k) = key(p, target, msaa) else { continue };
        if pipes.made.contains_key(&k) {
            continue;
        }
        if !device.features().contains(WgpuFeatures::EXPERIMENTAL_MESH_SHADER) {
            if !std::mem::replace(&mut *told, true) {
                error!("MeshShaderPass: this GPU has no mesh shaders (wgpu's EXPERIMENTAL_MESH_SHADER), so it draws nothing");
            }
            return;
        }
        let uuids = k.0.map(bevy::asset::uuid::Uuid::from_u128);
        let shader = |stage: usize| Handle::<Shader>::Uuid(uuids[stage], core::marker::PhantomData);
        let depth = DepthStencilState { format: CORE_3D_DEPTH_FORMAT, depth_write_enabled: Some(true), depth_compare: Some(CompareFunction::GreaterEqual), stencil: default(), bias: default() };
        let id = cache.queue_mesh_pipeline(MeshPipelineDescriptor {
            label: Some("raea_mesh_shader_pass".into()),
            layout: vec![pipes.layout.clone()],
            immediate_size: 0,
            task: (k.0[0] != 0).then(|| TaskState { shader: shader(0), ..default() }),
            mesh: MeshState { shader: shader(1), ..default() },
            primitive: default(),
            depth_stencil: Some(depth),
            multisample: MultisampleState { count: k.2, ..default() },
            fragment: Some(FragmentState { shader: shader(2), targets: vec![Some(ColorTargetState { format: k.1, blend: None, write_mask: ColorWrites::ALL })], ..default() }),
            zero_initialize_workgroup_memory: false,
        });
        pipes.made.insert(k, id);
    }
}

/// The draw into the view's colour and depth, in the camera's viewport.
#[allow(clippy::too_many_arguments)]
fn draw(view: ViewQuery<(&ExtractedCamera, Option<&MainPassResolutionOverride>, &ViewTarget, &Msaa, &ViewDepthStencilTexture, &ViewUniformOffset, &MeshShaderPass)>, pipes: Res<Pipelines>, cache: Res<PipelineCache>, views: Res<ViewUniforms>, globals: Res<GlobalsBuffer>, mut ctx: RenderContext) {
    let (camera, over, target, msaa, depth, offset, p) = view.into_inner();
    let Some(pipeline) = key(p, target, msaa).and_then(|k| pipes.made.get(&k)).and_then(|id| cache.get_render_pipeline(*id)) else { return };
    let (Some(g), Some(v)) = (globals.buffer.binding(), views.uniforms.binding()) else { return };
    let group = ctx.render_device().create_bind_group("raea_mesh_shader_pass", &cache.get_bind_group_layout(&pipes.layout), &BindGroupEntries::sequential((g, v)));
    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor { label: Some("raea_mesh_shader_pass"), color_attachments: &[Some(target.get_color_attachment())], depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)), timestamp_writes: None, occlusion_query_set: None, multiview_mask: None });
    pass.set_render_pipeline(pipeline);
    pass.set_bind_group(0, &group, &[offset.offset]);
    if let Some(v) = Viewport::from_viewport_and_override(camera.viewport.as_ref(), over) {
        pass.set_camera_viewport(&v);
    }
    let [x, y, z] = p.groups;
    pass.draw_mesh_tasks(x, y, z);
}
