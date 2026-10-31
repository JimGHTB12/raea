//! The filter passes: WGSL (passes.wgsl) on Bevy's device, run over a filtered layer's texture.

use super::*;

use std::collections::HashMap;

use anyrender::filters::composite::CompositeOperator;
use anyrender::filters::{FilterEffect, FilterInput, FilterSource};
use wgpu::util::DeviceExt;

/// The filter passes' pipelines on Bevy's device, and the textures they work in (kept by size between frames).
pub struct Passes {
    layout: wgpu::BindGroupLayout,
    module: wgpu::ShaderModule,
    pipelines: HashMap<(&'static str, wgpu::TextureFormat), wgpu::RenderPipeline>,
    pool: HashMap<(u32, u32, wgpu::TextureFormat, u32), Vec<wgpu::Texture>>,
    taken: Vec<((u32, u32, wgpu::TextureFormat, u32), wgpu::Texture)>,
    /// feTurbulence's lattices, by seed
    pub(crate) lattices: HashMap<i32, wgpu::Texture>,
}

/// Premultiplied, sRGB-encoded working values: half floats, so a blur's many small weights don't band.
const WORK: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
/// What Vello draws into (straight alpha, storage) and what it copies an image from (straight alpha, copy source).
const VELLO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

const WGSL: &str = concat!(include_str!("passes.wgsl"), include_str!("primitives.wgsl"));

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct Params {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub m: [[f32; 4]; 5],
    pub o: [[f32; 4]; 2],
    pub c: [[f32; 4]; 8],
    /// a convolution's kernel or the transfer functions' tables
    pub t: [[f32; 4]; 64],
}

impl Default for Params {
    fn default() -> Params {
        Params { a: [0.0; 4], b: [0.0; 4], m: [[0.0; 4]; 5], o: [[0.0; 4]; 2], c: [[0.0; 4]; 8], t: [[0.0; 4]; 64] }
    }
}

impl Params {
    fn bytes(&self) -> &[u8] {
        // SAFETY: plain f32s, no padding (324 of them, 1296 bytes)
        unsafe { std::slice::from_raw_parts(self as *const Params as *const u8, std::mem::size_of::<Params>()) }
    }
}

impl Passes {
    pub fn new(device: &wgpu::Device) -> Passes {
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("raea-page filters"),
            entries: &[
                texture(0),
                texture(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("raea-page filters"), source: wgpu::ShaderSource::Wgsl(WGSL.into()) });
        Passes { layout, module, pipelines: HashMap::new(), pool: HashMap::new(), taken: vec![], lattices: HashMap::new() }
    }

    fn pipeline(&mut self, device: &wgpu::Device, entry: &'static str, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let (layout, module) = (&self.layout, &self.module);
        self.pipelines
            .entry((entry, format))
            .or_insert_with(|| {
                let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("raea-page filters"), bind_group_layouts: &[Some(layout)], immediate_size: 0 });
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(entry),
                    layout: Some(&pl),
                    vertex: wgpu::VertexState { module, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
                    }),
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    multiview_mask: None,
                    cache: None,
                })
            })
            .clone()
    }

    /// A texture of the pool's, `w` x `h`, held until `recycle`.
    pub fn texture(&mut self, device: &wgpu::Device, (w, h): (u32, u32), format: wgpu::TextureFormat, usage: wgpu::TextureUsages) -> wgpu::Texture {
        let key = (w, h, format, usage.bits());
        let t = self.pool.get_mut(&key).and_then(Vec::pop).unwrap_or_else(|| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("raea-page filters"),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        });
        self.taken.push((key, t.clone()));
        t
    }

    /// Every texture handed out this frame back in the pool (after the frame's work is submitted); sizes not used
    /// this frame let go.
    pub fn recycle(&mut self) {
        let used: std::collections::HashSet<_> = self.taken.iter().map(|(k, _)| *k).collect();
        self.pool.retain(|k, _| used.contains(k));
        for (k, t) in self.taken.drain(..) {
            self.pool.entry(k).or_default().push(t);
        }
    }

    pub(crate) fn pass(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, entry: &'static str, src: &wgpu::Texture, other: Option<&wgpu::Texture>, p: Params, to: &wgpu::Texture) {
        let pipeline = self.pipeline(device, entry, to.format());
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(entry), contents: p.bytes(), usage: wgpu::BufferUsages::UNIFORM });
        let (a, b) = (src.create_view(&Default::default()), other.unwrap_or(src).create_view(&Default::default()));
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(entry),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&a) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&b) },
                wgpu::BindGroupEntry { binding: 2, resource: buffer.as_entire_binding() },
            ],
        });
        let view = to.create_view(&Default::default());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(entry),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
    }

    pub(crate) fn work(&mut self, device: &wgpu::Device, size: (u32, u32)) -> wgpu::Texture {
        self.texture(device, size, WORK, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
    }

    /// A Gaussian blur of `sigma` device px: two passes, on a copy 2, 4 or 8 times smaller when sigma is large (as
    /// Skia blurs), a kernel of at most about 3 x 4 texels a side then.
    fn blur(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, src: &wgpu::Texture, sigma: f32, size: (u32, u32)) -> wgpu::Texture {
        if sigma < 0.25 {
            return src.clone();
        }
        let n = if sigma > 32.0 { 8 } else if sigma > 16.0 { 4 } else if sigma > 8.0 { 2 } else { 1 };
        if n > 1 {
            let small = (size.0.div_ceil(n), size.1.div_ceil(n));
            let down = self.work(device, small);
            self.pass(device, encoder, "down", src, None, Params { a: [n as f32, 0.0, 0.0, 0.0], ..Default::default() }, &down);
            // the averaging is a blur of its own (a box n wide: variance (n^2 - 1) / 12): the rest is the kernel's
            let rest = (sigma * sigma - ((n * n - 1) as f32) / 12.0).max(0.0).sqrt() / n as f32;
            let blurred = self.blur_at(device, encoder, &down, rest, small);
            let out = self.work(device, size);
            self.pass(device, encoder, "up", &blurred, None, Params { a: [n as f32, 0.0, 0.0, 0.0], ..Default::default() }, &out);
            return out;
        }
        self.blur_at(device, encoder, src, sigma, size)
    }

    /// Two passes of a Gaussian blur of `sigma` texels, at the texture's own size.
    /// A blur with its own deviation on each axis (SVG's `stdDeviation="6 0"`), at full size.
    fn blur_xy(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, src: &wgpu::Texture, sx: f32, sy: f32, size: (u32, u32)) -> wgpu::Texture {
        let mut t = src.clone();
        for (dir, sigma) in [([1.0, 0.0], sx), ([0.0, 1.0], sy)] {
            if sigma >= 0.25 {
                let out = self.work(device, size);
                let r = (sigma * 3.0).ceil().min(512.0);
                self.pass(device, encoder, "blur", &t, None, Params { a: [dir[0], dir[1], sigma, r], ..Default::default() }, &out);
                t = out;
            }
        }
        t
    }

    /// `src` through a one-input pass `name` (sRGB to linear light and back).
    fn convert(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, name: &'static str, src: &wgpu::Texture, size: (u32, u32)) -> wgpu::Texture {
        let out = self.work(device, size);
        self.pass(device, encoder, name, src, None, Params::default(), &out);
        out
    }

    fn blur_at(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, src: &wgpu::Texture, sigma: f32, size: (u32, u32)) -> wgpu::Texture {
        if sigma < 0.25 {
            return src.clone();
        }
        let r = (sigma * 3.0).ceil().min(512.0);
        let (mid, out) = (self.work(device, size), self.work(device, size));
        self.pass(device, encoder, "blur", src, None, Params { a: [1.0, 0.0, sigma, r], ..Default::default() }, &mid);
        self.pass(device, encoder, "blur", &mid, None, Params { a: [0.0, 1.0, sigma, r], ..Default::default() }, &out);
        out
    }

    /// `l` drawn into `to` (Rgba8Unorm, straight alpha: what Vello would have drawn).
    pub fn linear(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, l: &Linear, to: &wgpu::Texture) {
        let k = l.to_gradient.as_coeffs();
        let mut p = Params {
            a: [l.start.x as f32, l.start.y as f32, l.end.x as f32, l.end.y as f32],
            b: [l.stops.len() as f32, if l.repeat { 1.0 } else { 0.0 }, 0.0, 0.0],
            ..Default::default()
        };
        p.m[0] = [k[0] as f32, k[1] as f32, k[2] as f32, k[3] as f32];
        p.m[1] = [k[4] as f32, k[5] as f32, 0.0, 0.0];
        for (i, (o, c)) in l.stops.iter().enumerate() {
            p.o[i / 4][i % 4] = *o;
            p.c[i] = *c;
        }
        // the pass reads nothing: any texture binds
        let none = self.work(device, (1, 1));
        self.pass(device, encoder, "linear", &none, None, p, to);
    }

    /// `layer`'s filter run over `drawn` (Vello's picture of it): the result, straight alpha in `Rgba8Unorm`, ready
    /// for Vello to copy as the layer's image.
    pub fn run(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, layer: &Filtered, drawn: &wgpu::Texture) -> wgpu::Texture {
        let size = layer.size;
        let source = self.work(device, size);
        self.pass(device, encoder, "load", drawn, None, Params::default(), &source);
        let mut results: Vec<wgpu::Texture> = vec![];
        let mut alpha: Option<wgpu::Texture> = None;
        for node in layer.filter.nodes() {
            // an input: a result, the layer (SVG's SourceGraphic), its alpha in black (SourceAlpha), or the last result
            let mut get = |this: &mut Self, i: Option<FilterInput>| match i {
                Some(FilterInput::Result(id)) => results.get(id.0 as usize).cloned().unwrap_or_else(|| source.clone()),
                Some(FilterInput::Source(FilterSource::SourceAlpha | FilterSource::BackgroundAlpha)) => alpha
                    .get_or_insert_with(|| {
                        let out = this.work(device, size);
                        this.pass(device, encoder, "tint", &source, None, Params { b: [0.0, 0.0, 0.0, 1.0], ..Default::default() }, &out);
                        out
                    })
                    .clone(),
                Some(FilterInput::Source(_)) => source.clone(),
                None => results.last().cloned().unwrap_or_else(|| source.clone()),
            };
            let (mut input, mut second) = (get(self, node.inputs.primary.clone()), node.inputs.secondary.clone().map(|i| get(self, Some(i))));
            // SVG's linearRGB: the inputs out of sRGB, the result back after
            if node.linear_rgb {
                input = self.convert(device, encoder, "tolin", &input, size);
                second = second.map(|t| self.convert(device, encoder, "tolin", &t, size));
            }
            // a colour named in the primitive is sRGB: into the space the primitive works in
            let color = |c: &vello::peniko::color::AlphaColor<vello::peniko::color::Srgb>| match node.linear_rgb {
                true => c.convert::<vello::peniko::color::LinearSrgb>().premultiply().components,
                false => c.premultiply().components,
            };
            let out = match &node.effect {
                FilterEffect::ColorMatrix(m) => {
                    let out = self.work(device, size);
                    let m = m.0;
                    let row = |i: usize| [m[i * 5], m[i * 5 + 1], m[i * 5 + 2], m[i * 5 + 3]];
                    let p = Params { m: [row(0), row(1), row(2), row(3), [m[4], m[9], m[14], m[19]]], ..Default::default() };
                    self.pass(device, encoder, "matrix", &input, None, p, &out);
                    out
                }
                FilterEffect::GaussianBlur(b) => match b.std_deviation_y.filter(|y| *y != b.std_deviation) {
                    Some(y) => self.blur_xy(device, encoder, &input, b.std_deviation * layer.scale, y * layer.scale, size),
                    None => self.blur(device, encoder, &input, b.std_deviation * layer.scale, size),
                },
                FilterEffect::Composite(op) => {
                    let out = self.work(device, size);
                    let (mode, k) = match op {
                        CompositeOperator::Over => (0.0, [0.0; 4]),
                        CompositeOperator::In => (1.0, [0.0; 4]),
                        CompositeOperator::Out => (2.0, [0.0; 4]),
                        CompositeOperator::Atop => (3.0, [0.0; 4]),
                        CompositeOperator::Xor => (4.0, [0.0; 4]),
                        CompositeOperator::Arithmetic(a) => (5.0, [a.k1, a.k2, a.k3, a.k4]),
                    };
                    let other = second.clone().unwrap_or_else(|| self.work(device, size));
                    self.pass(device, encoder, "composite", &input, Some(&other), Params { a: [mode, 0.0, 0.0, 0.0], b: k, ..Default::default() }, &out);
                    out
                }
                FilterEffect::DropShadow(s) => {
                    let tinted = self.work(device, size);
                    let c = color(&s.color);
                    let p = Params { a: [(s.dx * layer.scale).round(), (s.dy * layer.scale).round(), 0.0, 0.0], b: c, ..Default::default() };
                    self.pass(device, encoder, "tint", &input, None, p, &tinted);
                    let shadow = self.blur(device, encoder, &tinted, s.std_deviation * layer.scale, size);
                    let out = self.work(device, size);
                    self.pass(device, encoder, "over", &input, Some(&shadow), Params::default(), &out);
                    out
                }
                FilterEffect::Offset(v) => {
                    let out = self.work(device, size);
                    let p = Params { a: [(v.x as f32 * layer.scale).round(), (v.y as f32 * layer.scale).round(), 0.0, 0.0], ..Default::default() };
                    self.pass(device, encoder, "offset", &input, None, p, &out);
                    out
                }
                FilterEffect::Flood(c) => {
                    let out = self.work(device, size);
                    self.pass(device, encoder, "flood", &input, None, Params { b: color(c), ..Default::default() }, &out);
                    out
                }
                FilterEffect::Svg(prim) => self.svg(device, encoder, prim, &input, second.as_ref(), layer),
                other => {
                    static ONCE: std::sync::Once = std::sync::Once::new();
                    ONCE.call_once(|| eprintln!("raea: the filter primitive {:?} isn't drawn (docs/blitz-bevy.feature B4a)", std::mem::discriminant(other)));
                    input
                }
            };
            let out = if node.linear_rgb { self.convert(device, encoder, "tosrgb", &out, size) } else { out };
            results.push(out);
        }
        let mut last = results.get(layer.filter.output().0 as usize).cloned().unwrap_or(source);
        // a warped layer: onto the page box it lands in
        let mut size = size;
        if let Some(w) = layer.warp.as_ref().filter(|w| w.size.0 > 0 && w.size.1 > 0) {
            let out = self.work(device, w.size);
            let m = w.to_layer;
            let p = Params { m: [[m[0], m[1], m[2], 0.0], [m[3], m[4], m[5], 0.0], [m[6], m[7], m[8], 0.0], [0.0; 4], [0.0; 4]], ..Default::default() };
            self.pass(device, encoder, "warp", &last, None, p, &out);
            last = out;
            size = w.size;
        }
        let out = self.texture(device, size, VELLO, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
        self.pass(device, encoder, "store", &last, None, Params::default(), &out);
        out
    }
}
