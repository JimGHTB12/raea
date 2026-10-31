//! Ink outlines (an <Outline> in the scene: outline on the camera row): after the scene's main pass, before bloom and
//! tone mapping, a full-frame pass draws a line in `outline` where the depth steps (a silhouette against what's behind
//! it) or the surface turns (a crease), read from the camera's depth and normal prepasses. With a frame look the
//! camera draws a cell per texel, so the line is a cell wide: Return of the Obra Dinn's line.
//!
//! Columns on the camera row: outline (the line's colour), outline_depth (how far behind, as a fraction of the nearer
//! surface's distance, a neighbour must be for a silhouette: 0.04 by default), outline_normal (how far the surface
//! must turn between neighbours for a crease, 1 - cos of the angle: 0.3 by default; 0 for none), outline_width (the
//! line's width in pixels, 1 by default: a cartoon's even, thick line), outline_far (how far from the camera lines
//! reach, in metres: they fade out over its last third; none by default), outline_skip (the composition passes with no
//! line: compose.rs). Needs Msaa::Off (set where the camera is made).

use bevy::{
    core_pipeline::{prepass::ViewPrepassTextures, schedule::Core3d, Core3dSystems, FullscreenShader},
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin, UniformComponentPlugin},
        render_resource::{
            binding_types::{sampler, texture_2d, texture_depth_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, ViewQuery},
        view::ViewTarget,
        RenderApp, RenderStartup,
    },
};

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType)]
#[extract_app(bevy::render::RenderApp)]
pub struct InkOutline {
    pub color: Vec4,
    pub depth: f32,
    pub normal: f32,
    /// the line's width in pixels of the camera's target (1: one pixel, as it always was)
    pub width: f32,
    /// how far lines reach, in the depth's units (distance / the camera's near plane): they fade out over the last
    /// third, so far ground meeting the sky draws no line (0: no limit)
    pub far: f32,
    /// the composition passes that draw no line (bits, 1 << pass: compose.rs): none on their pixels, and no crease
    /// against them; a scene object's silhouette against one still draws, on the object
    pub skip: u32,
}

pub fn install(app: &mut App) {
    bevy::asset::embedded_asset!(app, "outline.wesl");
    app.add_plugins((ExtractComponentPlugin::<InkOutline>::default(), UniformComponentPlugin::<InkOutline>::default()));
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
    render_app.add_systems(RenderStartup, init_pipeline);
    render_app.add_systems(Core3d, draw.in_set(Core3dSystems::EarlyPostProcess));
}

#[derive(Resource)]
struct OutlinePipeline {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    /// one pipeline a view target format: an HDR camera's (Rgba16Float) and an LDR one's
    hdr: CachedRenderPipelineId,
    ldr: CachedRenderPipelineId,
}

fn init_pipeline(mut commands: Commands, device: Res<RenderDevice>, assets: Res<AssetServer>, fullscreen: Res<FullscreenShader>, cache: Res<PipelineCache>) {
    let layout = BindGroupLayoutDescriptor::new(
        "ink_outline_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<InkOutline>(true),
                texture_depth_2d(),
                texture_2d(TextureSampleType::Float { filterable: false }),
            ),
        ),
    );
    let shader = assets.load("embedded://raea_sys_post/outline.wesl");
    let pipeline = |format: TextureFormat| {
        cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("ink_outline".into()),
            layout: vec![layout.clone()],
            vertex: fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: shader.clone(),
                targets: vec![Some(ColorTargetState { format, blend: None, write_mask: ColorWrites::ALL })],
                ..default()
            }),
            ..default()
        })
    };
    let (hdr, ldr) = (pipeline(super::bevy_api::HDR_FORMAT), pipeline(TextureFormat::Rgba8UnormSrgb));
    commands.insert_resource(OutlinePipeline { layout: layout.clone(), sampler: device.create_sampler(&SamplerDescriptor::default()), hdr, ldr });
}

fn draw(
    // the outline itself as well as its index: a camera whose <Outline> went (a cartoon scene, then a plain one) keeps
    // a stale index into a buffer that no longer has its entry, which drew the plain scene's surfaces solid ink
    view: ViewQuery<(&ViewTarget, &ViewPrepassTextures, &DynamicUniformIndex<InkOutline>, &InkOutline)>,
    pipeline: Option<Res<OutlinePipeline>>,
    cache: Res<PipelineCache>,
    uniforms: Res<ComponentUniforms<InkOutline>>,
    mut ctx: RenderContext,
) {
    let Some(p) = pipeline else { return };
    let (target, prepass, index, _) = view.into_inner();
    let id = if target.main_texture_format() == super::bevy_api::HDR_FORMAT { p.hdr } else { p.ldr };
    let (Some(rp), Some(binding), Some(depth), Some(normal)) = (cache.get_render_pipeline(id), uniforms.uniforms().binding(), prepass.depth_only_view(), prepass.normal_view()) else {
        return;
    };
    let post = target.post_process_write();
    let group = ctx.render_device().create_bind_group(
        "ink_outline_group",
        &cache.get_bind_group_layout(&p.layout),
        &BindGroupEntries::sequential((post.source, &p.sampler, binding.clone(), depth, normal)),
    );
    let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
        label: Some("ink_outline"),
        color_attachments: &[Some(RenderPassColorAttachment { view: post.destination, depth_slice: None, resolve_target: None, ops: Operations::default() })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(rp);
    pass.set_bind_group(0, &group, &[index.index()]);
    pass.draw(0..3, 0..1);
}
