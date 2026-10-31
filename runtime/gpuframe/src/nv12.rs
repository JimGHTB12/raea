//! The NV12 pass: a frame packed to NV12 on the GPU (the K kernel) for the encoder, read back once.

use super::*;

/// The NV12 readback (G8): after the cameras have drawn, a K kernel (core/k/kernels/nv12.k) reads the target
/// through a Unorm view and writes NV12 plus the frame tag into `out`, which is what comes back to the CPU (3.1 MB a
/// frame at 1080x1920, not 8.4 MB of padded RGBA).
#[derive(Resource, Clone, ExtractResource)]
#[extract_app(bevy::render::RenderApp)]
pub struct Nv12Job {
    pub target: Handle<Image>,
    pub out: Handle<ShaderBuffer>,
    pub w: u32,
    pub h: u32,
    /// stills: the composite's RGBA words and the tag texel come back as they are, not converted to NV12
    pub rgba: bool,
}

/// The compiled kernel, made once in the render world.
#[derive(Resource, Default)]
pub struct Nv12Gpu(pub Option<wgpu::ComputePipeline>);

pub const NV12_K: &str = include_str!("../../../core/k/kernels/nv12.k");

/// Where a packed (stills) readback's tag word sits: after the w x h words, 256-aligned for the texel copy.
pub fn packed_tag(w: u32, h: u32) -> usize {
    (w as usize * h as usize * 4).div_ceil(256) * 256
}

pub fn nv12_words(w: u32, h: u32) -> usize {
    (w * h / 4 + w * h / 8 + 1) as usize
}

pub fn nv12_pass(world: &mut World) {
    let Some(job) = world.get_resource::<Nv12Job>().cloned() else { return };
    let device = world.resource::<bevy::render::renderer::RenderDevice>().wgpu_device().clone();
    if world.resource::<Nv12Gpu>().0.is_none() {
        let k = raea_core_k::parse(NV12_K).expect("nv12.k parses");
        let sc: std::collections::HashMap<&str, i32> = [("w", job.w as i32), ("h", job.h as i32), ("T_w", job.w as i32)].into();
        let src = raea_sys_k::wgsl_tex(&k, &[], &["T"], &sc, &["px"]).expect("nv12.k compiles");
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("nv12.k"), source: wgpu::ShaderSource::Wgsl(src.into()) });
        let p = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("nv12.k"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        world.resource_mut::<Nv12Gpu>().0 = Some(p);
    }
    let (Some(img), Some(out)) = (world.resource::<RenderAssets<GpuImage>>().get(&job.target), world.resource::<RenderAssets<GpuShaderBuffer>>().get(&job.out)) else {
        return;
    };
    let view = img.texture.create_view(&wgpu::TextureViewDescriptor { format: Some(wgpu::TextureFormat::Rgba8Unorm), ..Default::default() });
    let n = nv12_words(job.w, job.h) as u32;
    let groups = n.div_ceil(256);
    let (gx, gy) = (groups.min(65535), groups.div_ceil(groups.min(65535)));
    let shape = wgpu::util::DeviceExt::create_buffer_init(&device, &wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &[n.to_le_bytes(), (gx * 256).to_le_bytes()].concat(),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let pipeline = world.resource::<Nv12Gpu>().0.clone().unwrap();
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
            wgpu::BindGroupEntry { binding: 1, resource: out.buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: shape.as_entire_binding() },
        ],
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("raea-render engine") });
    {
        let mut pass = enc.begin_compute_pass(&Default::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(gx, gy, 1);
    }
    // an encoder, not a finished buffer: pending encoders are submitted after the cameras' work
    world.resource_mut::<bevy::render::renderer::PendingCommandBuffers>().push_encoder(enc, "raea nv12 composite");
}
