//! A page laid over a camera's picture as raea lays it (docs/blitz-bevy.feature B3 H10), so a plain App's live
//! frame and a render agree. Bevy's UI blends an image node in linear light; raea's feed blends the page's
//! premultiplied sRGB-encoded bytes over the 3D's (Skia's SrcOver on the u8 grid: the 3D over black, then
//! `dst * (256 - srcA) >> 8` plus the source, sdk/react motion.rs). [`PageOverlay`] on a camera does the latter: one
//! fullscreen pass after the camera's UI pass and before upscaling, reading the view's picture and the page's
//! image and writing the same bytes the feed would.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::renderer::{RenderContext, RenderDevice, ViewQuery};
use bevy::render::sync_world::RenderEntity;
use bevy::render::texture::GpuImage;
use bevy::render::view::ViewTarget;
use bevy::render::{Extract, ExtractSchedule, RenderApp};
use wgpu::TextureFormat;

use raea_sys_page::PageImage;

/// On a camera: `page`'s image laid over what it drew, as raea's feed lays a page over the 3D.
#[derive(Component, Clone, Copy, Debug)]
pub struct PageOverlay {
    pub page: Entity,
}

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
        r.init_resource::<Overlays>().add_systems(ExtractSchedule, extract);
        r.add_systems(bevy::core_pipeline::Core3d, overlay.after(bevy::ui_render::render_pass::ui_pass).before(bevy::core_pipeline::upscaling::upscaling))
            .add_systems(bevy::core_pipeline::Core2d, overlay.after(bevy::ui_render::render_pass::ui_pass).before(bevy::core_pipeline::upscaling::upscaling));
    }
}

/// Each overlaid camera's page image, by the camera's render entity.
#[derive(Resource, Default)]
struct Overlays(HashMap<Entity, AssetId<Image>>);

fn extract(cameras: Extract<Query<(RenderEntity, &PageOverlay)>>, pages: Extract<Query<&PageImage>>, mut overlays: ResMut<Overlays>) {
    overlays.0 = cameras.iter().filter_map(|(r, o)| Some((r, pages.get(o.page).ok()?.0.id()))).collect();
}

const WGSL: &str = r"
@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var page: texture_2d<f32>;
@group(0) @binding(2) var at: sampler;
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let p = vec2f(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4f(p * 2.0 - 1.0, 0.0, 1.0);
}
fn linear(c: vec3f) -> vec3f {
    return select(pow((c + 0.055) / 1.055, vec3f(2.4)), c / 12.92, c <= vec3f(0.04045));
}
fn encode(c: vec3f) -> vec3f {
    return select(1.055 * pow(c, vec3f(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3f(0.0031308));
}
// the stored byte of a value in [0, 1]
fn byte(c: vec3f) -> vec3f {
    return round(clamp(c, vec3f(0.0), vec3f(1.0)) * 255.0);
}
@fragment fn fs(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let d = textureLoad(scene, vec2i(p.xy), 0);
    let uv = p.xy / vec2f(textureDimensions(scene));
    let s = textureSampleLevel(page, at, uv, 0.0);
    // the 3D over black, then the page: SrcOver on the u8 grid, as the feed composites (sdk/react motion.rs)
    let da = round(clamp(d.a, 0.0, 1.0) * 255.0);
    let t = floor((byte(encode(d.rgb)) * da + 127.0) / 255.0);
    let k = 256.0 - round(clamp(s.a, 0.0, 1.0) * 255.0);
    let o = min(byte(encode(s.rgb)) + floor(t * k / 256.0), vec3f(255.0));
    return vec4f(linear(o / 255.0), 1.0);
}
";

/// The pass's pipelines, one per view format, and what they share.
#[derive(Default)]
struct Pipes {
    made: Option<(wgpu::BindGroupLayout, wgpu::PipelineLayout, wgpu::ShaderModule, wgpu::Sampler)>,
    by_format: HashMap<TextureFormat, wgpu::RenderPipeline>,
}

fn overlay(view: ViewQuery<(Entity, &ViewTarget)>, overlays: Res<Overlays>, images: Res<RenderAssets<GpuImage>>, device: Res<RenderDevice>, mut pipes: Local<Pipes>, mut ctx: RenderContext) {
    let (entity, target) = view.into_inner();
    let Some(page) = overlays.0.get(&entity).and_then(|id| images.get(*id)) else { return };
    let device = device.wgpu_device();
    let pipes = &mut *pipes;
    let (layout, pipeline_layout, module, sampler) = pipes.made.get_or_insert_with(|| {
        let tex = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("raea-page overlay"),
            entries: &[tex(0), tex(1), wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("raea-page overlay"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("raea-page overlay"), source: wgpu::ShaderSource::Wgsl(WGSL.into()) });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("raea-page overlay"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..default() });
        (layout, pipeline_layout, module, sampler)
    });
    let format = target.main_texture_format();
    let pipeline = pipes.by_format.entry(format).or_insert_with(|| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("raea-page overlay"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState { module, entry_point: Some("vs"), compilation_options: default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState { module, entry_point: Some("fs"), compilation_options: default(), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: default(),
            depth_stencil: None,
            multisample: default(),
            multiview_mask: None,
            cache: None,
        })
    });
    let post = target.post_process_write();
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("raea-page overlay"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(post.source) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&page.texture_view) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(sampler) },
        ],
    });
    let mut pass = ctx.command_encoder().begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("raea-page overlay"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: post.destination, depth_slice: None, resolve_target: None, ops: wgpu::Operations::default() })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &bind, &[]);
    pass.draw(0..3, 0..1);
}
