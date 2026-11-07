//! raea-page's last pass, with the other fullscreen passes a page runs: Vello's straight, sRGB-encoded storage
//! texture made into a page's premultiplied image, and a premultiplied picture (a camera's) made straight for Vello's atlas.

use wgpu::{TextureDimension, TextureFormat, TextureUsages};

/// Vello stores straight alpha in an `Rgba8Unorm` storage texture (sRGB-encoded values); a page's image is
/// `Rgba8UnormSrgb` and premultiplied. One fullscreen triangle reads the one and writes the other.
pub struct Premul {
    pub pipeline: wgpu::RenderPipeline,
    /// the same pass for a StraightAlpha page: the colour decoded, its alpha kept apart
    pub straight: wgpu::RenderPipeline,
    /// a premultiplied picture (a camera's) made straight for Vello, which takes an element's picture so
    pub unpremul: wgpu::RenderPipeline,
    pub layout: wgpu::BindGroupLayout,
}

pub const PREMUL_WGSL: &str = include_str!("premul.wgsl");

impl Premul {
    pub fn new(device: &wgpu::Device) -> Premul {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("raea-page premul"), source: wgpu::ShaderSource::Wgsl(PREMUL_WGSL.into()) });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("raea-page premul"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                count: None,
            }],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("raea-page premul"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let make = |entry: &str| device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("raea-page premul"),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: TextureFormat::Rgba8UnormSrgb, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let (pipeline, straight, unpremul) = (make("fs"), make("fs_straight"), make("fs_unpremul"));
        Premul { pipeline, straight, unpremul, layout }
    }
}

/// A straight copy's texture: sRGB-encoded, as Vello's atlas takes a picture's texels.
pub fn straight_target(device: &wgpu::Device, size: wgpu::Extent3d) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("raea-page view made straight"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}
