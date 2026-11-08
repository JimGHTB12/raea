//! The page handed to a render's encoder on the GPU (docs/blitz-bevy.feature B1d): the page's picture, alone or laid
//! over a 3D target, written into the packed RGBA8 buffer raea-render's fed render composites and encodes. Nothing comes
//! back to the CPU here; the encoder's NV12 is the render's one copy.
//!
//! Each texture is copied as stored (sRGB-encoded bytes) into a buffer the pass reads, so the bytes are the ones a
//! readback gave; the pass does on the GPU what raea did on the CPU (motion.rs bevy_over_3d), integer for
//! integer, so a render's frames are the same.

const WGSL: &str = r"
struct P { w: u32, h: u32, page_stride: u32, under_stride: u32, mode: u32 }
@group(0) @binding(0) var<storage, read> page: array<u32>;
@group(0) @binding(1) var<storage, read> under: array<u32>;
@group(0) @binding(2) var<storage, read_write> out: array<u32>;
@group(0) @binding(3) var<uniform> p: P;
fn ch(v: u32, i: u32) -> u32 { return (v >> (8u * i)) & 255u; }
@compute @workgroup_size(16, 16) fn main(@builtin(global_invocation_id) g: vec3u) {
    if g.x >= p.w || g.y >= p.h { return; }
    let s = page[g.y * p.page_stride + g.x];
    var o = s;
    if p.mode == 2u {
        // the page alone, opaque: over the black the encoder's buffer starts as
        o = s | (255u << 24u);
    } else if p.mode == 1u {
        // the page over the 3D, the 3D (premultiplied, as Bevy blends into it) over black: its own colour, a window's
        // picture, whatever alpha its fragments wrote; then Skia's SrcOver on the u8 grid, as bevy_over_3d does
        let d = under[g.y * p.under_stride + g.x];
        let k = 256u - ch(s, 3u);
        o = 255u << 24u;
        for (var i = 0u; i < 3u; i++) {
            let t = ch(d, i);
            o |= min(255u, ch(s, i) + ((t * k) >> 8u)) << (8u * i);
        }
    }
    out[g.y * p.w + g.x] = o;
}
";

/// The pass and the buffers it reuses (made again when a size changes).
pub struct Feed {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    staged: Vec<(wgpu::Buffer, u64)>,
}

impl Feed {
    pub fn new(device: &wgpu::Device) -> Feed {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("raea-page feed"), source: wgpu::ShaderSource::Wgsl(WGSL.into()) });
        let storage = |binding, read_only| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("raea-page feed"),
            entries: &[
                storage(0, true),
                storage(1, true),
                storage(2, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("raea-page feed"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("raea-page feed"),
            layout: Some(&pl),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Feed { pipeline, layout, staged: Vec::new() }
    }

    /// Staging buffer `i` of at least `size` bytes (a copy's destination the pass reads; never mapped).
    fn staged(&mut self, device: &wgpu::Device, i: usize, size: u64) -> wgpu::Buffer {
        while self.staged.len() <= i {
            self.staged.push((device.create_buffer(&wgpu::BufferDescriptor { label: Some("raea-page feed staging"), size: 4, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }), 4));
        }
        if self.staged[i].1 < size {
            self.staged[i] = (device.create_buffer(&wgpu::BufferDescriptor { label: Some("raea-page feed staging"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }), size);
        }
        self.staged[i].0.clone()
    }

    /// The top left `w` x `h` of `page` (as stored), over the top left of `under` when given, else made `opaque` or left
    /// as it is, into `out` (packed RGBA8, `w` x `h`); submitted on `queue`, after what was submitted before it.
    #[allow(clippy::too_many_arguments)]
    pub fn run(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, page: &wgpu::Texture, under: Option<&wgpu::Texture>, opaque: bool, w: u32, h: u32, out: &wgpu::Buffer) {
        let stride = (w * 4).div_ceil(256) * 256;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("raea-page feed") });
        let mut stage = |i: usize, tex: &wgpu::Texture, encoder: &mut wgpu::CommandEncoder| {
            let buf = self.staged(device, i, (stride * h) as u64);
            encoder.copy_texture_to_buffer(
                tex.as_image_copy(),
                wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(stride), rows_per_image: Some(h) } },
                wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            );
            buf
        };
        let page_buf = stage(0, page, &mut encoder);
        let under_buf = match under {
            Some(t) => stage(1, t, &mut encoder),
            None => page_buf.clone(),
        };
        let params = [w, h, stride / 4, stride / 4, if under.is_some() { 1 } else if opaque { 2 } else { 0 }, 0, 0, 0];
        let uniform = device.create_buffer(&wgpu::BufferDescriptor { label: Some("raea-page feed"), size: 32, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&uniform, 0, &params.iter().flat_map(|v: &u32| v.to_le_bytes()).collect::<Vec<u8>>());
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("raea-page feed"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: page_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: under_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: out.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: uniform.as_entire_binding() },
            ],
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("raea-page feed"), timestamp_writes: None });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        queue.submit([encoder.finish()]);
    }
}
