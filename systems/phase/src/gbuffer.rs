//! A fullscreen draw into the deferred G-buffer and the shadow maps (docs/control.feature FC2d), as Bevy's
//! shader_advanced/deferred_raymarch draws its gyroid: a camera with a GBufferPass (and Bevy's DepthPrepass and
//! DeferredPrepass) runs your shader file's `fragment` over Bevy's fullscreen triangle after the late deferred prepass,
//! writing the G-buffer, the lighting pass id and depth (compared GreaterEqual), so Bevy's deferred lighting lights it
//! as any other surface; its `shadow` entry, when it has one, writes the depth of every shadow view the camera's lights
//! draw (and of each shared one), so it casts shadows too. Bind group 0 is the view uniform (dynamic) and Globals.

use std::collections::HashMap;

use bevy::camera::{MainPassResolutionOverride, Viewport};
use bevy::core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT;
use bevy::core_pipeline::deferred::copy_lighting_id::copy_deferred_lighting_id;
use bevy::core_pipeline::deferred::node::late_deferred_prepass;
use bevy::core_pipeline::deferred::{DEFERRED_LIGHTING_PASS_ID_FORMAT, DEFERRED_PREPASS_FORMAT};
use bevy::core_pipeline::prepass::ViewPrepassTextures;
use bevy::core_pipeline::{Core3d, Core3dSystems, FullscreenShader};
use bevy::pbr::{per_view_shadow_pass, shared_shadow_pass, ShadowView, ViewLightEntities, LATE_SHADOW_PASS};
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::globals::{GlobalsBuffer, GlobalsUniform};
use bevy::render::render_resource::binding_types::uniform_buffer;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, ViewQuery};
use bevy::render::view::{ViewDepthStencilTexture, ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};

/// A camera's fullscreen G-buffer draw: its shader file (as written), its G-buffer entry point and its shadow one
/// ("" for none).
#[derive(Component, Clone, Debug, Default, Reflect, ExtractComponent)]
#[extract_app(RenderApp)]
#[reflect(Component, Default)]
#[type_path = "raea_sys_phase"]
pub struct GBufferPass {
    pub shader: String,
    pub fragment: String,
    pub shadow: String,
    /// its shader file, registered as written (raea-sys-shader), and the id register gave it
    #[reflect(ignore)]
    pub registered: Option<(String, u128)>,
}

pub fn install(app: &mut App) {
    app.register_type::<GBufferPass>();
    app.add_systems(PostUpdate, crate::register::<GBufferPass>);
    app.add_plugins(ExtractComponentPlugin::<GBufferPass>::default());
    let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
    r.add_systems(RenderStartup, init).add_systems(Render, queue.in_set(RenderSystems::Queue)).add_systems(
        Core3d,
        (
            gbuffer.in_set(Core3dSystems::Prepass).after(late_deferred_prepass).before(copy_deferred_lighting_id),
            directional_shadows.after(per_view_shadow_pass::<LATE_SHADOW_PASS>).before(Core3dSystems::MainPass),
            shared_shadows.after(shared_shadow_pass::<LATE_SHADOW_PASS>).before(Core3dSystems::MainPass),
        ),
    );
}

/// The passes' bind group layout, the fullscreen vertex stage, and each pass's pipelines (its G-buffer one and its
/// shadow one) by what decides them.
#[derive(Resource)]
struct Pipelines {
    layout: BindGroupLayoutDescriptor,
    vertex: VertexState,
    made: HashMap<(u128, String, String), (CachedRenderPipelineId, Option<CachedRenderPipelineId>)>,
}

fn init(mut commands: Commands, fullscreen: Res<FullscreenShader>) {
    let layout = BindGroupLayoutDescriptor::new("raea_gbuffer_pass", &BindGroupLayoutEntries::sequential(ShaderStages::FRAGMENT, (uniform_buffer::<ViewUniform>(true), uniform_buffer::<GlobalsUniform>(false))));
    commands.insert_resource(Pipelines { layout, vertex: fullscreen.to_vertex_state(), made: HashMap::new() });
}

/// The key a pass's pipelines are kept by.
fn key(p: &GBufferPass) -> Option<(u128, String, String)> {
    Some((p.registered.as_ref()?.1, p.fragment.clone(), p.shadow.clone()))
}

/// Each pass's pipelines queued once its shader is registered.
fn queue(cache: Res<PipelineCache>, mut pipes: ResMut<Pipelines>, passes: Query<&GBufferPass>) {
    for p in &passes {
        let Some(k) = key(p) else { continue };
        if pipes.made.contains_key(&k) {
            continue;
        }
        let shader = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(k.0), core::marker::PhantomData);
        let depth = DepthStencilState { format: CORE_3D_DEPTH_FORMAT, depth_write_enabled: Some(true), depth_compare: Some(CompareFunction::GreaterEqual), stencil: default(), bias: default() };
        let target = |format| Some(ColorTargetState { format, blend: None, write_mask: ColorWrites::ALL });
        let pipeline = |entry: &str, targets: Vec<Option<ColorTargetState>>| {
            cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("raea_gbuffer_pass".into()),
                layout: vec![pipes.layout.clone()],
                vertex: pipes.vertex.clone(),
                fragment: Some(FragmentState { shader: shader.clone(), entry_point: Some(entry.to_owned().into()), targets, ..default() }),
                depth_stencil: Some(depth.clone()),
                ..default()
            })
        };
        let g = pipeline(&k.1, vec![target(DEFERRED_PREPASS_FORMAT), target(DEFERRED_LIGHTING_PASS_ID_FORMAT)]);
        let s = (!k.2.is_empty()).then(|| pipeline(&k.2, vec![]));
        pipes.made.insert(k, (g, s));
    }
}

/// Its bind group: the view uniform and Globals.
fn bind_group(ctx: &RenderContext, cache: &PipelineCache, pipes: &Pipelines, views: &ViewUniforms, globals: &GlobalsBuffer) -> Option<BindGroup> {
    let entries = BindGroupEntries::sequential((views.uniforms.binding()?, globals.buffer.binding()?));
    Some(ctx.render_device().create_bind_group("raea_gbuffer_pass", &cache.get_bind_group_layout(&pipes.layout), &entries))
}

/// The G-buffer draw in the camera's viewport (as Bevy's own passes draw: a target bigger than the view, as raea's
/// is by its tag row, would otherwise stretch the fullscreen triangle's uv over it), then the view's depth copied to its
/// prepass depth, as the example does.
#[allow(clippy::too_many_arguments)]
fn gbuffer(view: ViewQuery<(&ExtractedCamera, Option<&MainPassResolutionOverride>, &ViewUniformOffset, &ViewDepthStencilTexture, &ViewPrepassTextures, &GBufferPass)>, pipes: Res<Pipelines>, cache: Res<PipelineCache>, views: Res<ViewUniforms>, globals: Res<GlobalsBuffer>, mut ctx: RenderContext) {
    let (camera, over, offset, depth, prepass, p) = view.into_inner();
    let Some(&(id, _)) = key(p).and_then(|k| pipes.made.get(&k)) else { return };
    let (Some(pipeline), Some(deferred), Some(pass_id)) = (cache.get_render_pipeline(id), &prepass.deferred, &prepass.deferred_lighting_pass_id) else { return };
    let Some(group) = bind_group(&ctx, &cache, &pipes, &views, &globals) else { return };
    {
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor { label: Some("raea_gbuffer_pass"), color_attachments: &[Some(deferred.get_attachment()), Some(pass_id.get_attachment())], depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)), timestamp_writes: None, occlusion_query_set: None, multiview_mask: None });
        if let Some(v) = Viewport::from_viewport_and_override(camera.viewport.as_ref(), over) {
            pass.set_camera_viewport(&v);
        }
        pass.set_render_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[offset.offset]);
        pass.draw(0..3, 0..1);
    }
    if let Some(d) = &prepass.depth {
        ctx.command_encoder().copy_texture_to_texture(depth.texture().as_image_copy(), d.texture.texture.as_image_copy(), prepass.size);
    }
}

/// The shadow entry drawn into one shadow view's depth.
fn shadow(ctx: &mut RenderContext, cache: &PipelineCache, pipes: &Pipelines, views: &ViewUniforms, globals: &GlobalsBuffer, id: CachedRenderPipelineId, view: &ShadowView, offset: &ViewUniformOffset) {
    let (Some(pipeline), Some(group)) = (cache.get_render_pipeline(id), bind_group(ctx, cache, pipes, views, globals)) else { return };
    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor { label: Some("raea_gbuffer_pass_shadow"), color_attachments: &[], depth_stencil_attachment: Some(view.depth_attachment.get_attachment(StoreOp::Store)), timestamp_writes: None, occlusion_query_set: None, multiview_mask: None });
    pass.set_render_pipeline(pipeline);
    pass.set_bind_group(0, &group, &[offset.offset]);
    pass.draw(0..3, 0..1);
}

/// The camera's lights' own shadow views (a directional light's cascades).
#[allow(clippy::too_many_arguments)]
fn directional_shadows(view: ViewQuery<(&ViewLightEntities, &GBufferPass)>, shadow_views: Query<(&ShadowView, &ViewUniformOffset)>, pipes: Res<Pipelines>, cache: Res<PipelineCache>, views: Res<ViewUniforms>, globals: Res<GlobalsBuffer>, mut ctx: RenderContext) {
    let (lights, p) = view.into_inner();
    let Some(&(_, Some(id))) = key(p).and_then(|k| pipes.made.get(&k)) else { return };
    for light in lights.lights.iter().copied() {
        if let Ok((v, offset)) = shadow_views.get(light) {
            shadow(&mut ctx, &cache, &pipes, &views, &globals, id, v, offset);
        }
    }
}

/// A shared shadow view (a point or spot light's), drawn by every pass with a shadow entry.
#[allow(clippy::too_many_arguments)]
fn shared_shadows(view: ViewQuery<(&ShadowView, &ViewUniformOffset)>, passes: Query<&GBufferPass>, pipes: Res<Pipelines>, cache: Res<PipelineCache>, views: Res<ViewUniforms>, globals: Res<GlobalsBuffer>, mut ctx: RenderContext) {
    let (v, offset) = view.into_inner();
    for p in &passes {
        if let Some(&(_, Some(id))) = key(p).and_then(|k| pipes.made.get(&k)) {
            shadow(&mut ctx, &cache, &pipes, &views, &globals, id, v, offset);
        }
    }
}
