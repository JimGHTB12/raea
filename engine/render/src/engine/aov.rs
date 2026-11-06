//! RT1 (docs/raytracing.feature): each frame's motion vectors and depth, the guide data a neural or denoising pass
//! after the render needs (MLX-DLSS's run-sequence, MetalFX). Asked for with `want_aov`, before the first draw: the 3D
//! cameras then draw Bevy's depth and motion-vector prepasses (MSAA off: a multisampled prepass can't be read), and
//! each frame's first draw reads both back (`take_aov`). A redraw of the same frame keeps the first draw's: drawn
//! again, every mesh's last pose is its pose now, and its motion would read zero.

use super::*;
use bevy::core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, ViewPrepassTextures};

/// One frame's guide data at the prepass's size (the frame's, times RAEA_SS).
pub struct Aov {
    pub w: u32,
    pub h: u32,
    /// w x h x 2, Bevy's motion vectors: this frame's UV minus the last frame's (x right, y down)
    pub motion: Vec<f32>,
    /// w x h, reverse Z (1 at the near plane, 0 at infinity)
    pub depth: Vec<f32>,
}

/// The guide data is wanted; `frame` is the frame last read, `last` what it read.
#[derive(Resource, Default)]
pub(crate) struct AovWanted {
    frame: Option<i64>,
    last: Option<Aov>,
}

fn aov_prepasses(mut cmd: Commands, q: Query<Entity, (With<Camera3d>, Without<MotionVectorPrepass>)>) {
    for e in &q {
        cmd.entity(e).insert((DepthPrepass, MotionVectorPrepass, Msaa::Off));
    }
}

// the motion texture is made without COPY_SRC (bevy_core_pipeline's prepare_core_3d_depth_textures), so it's read in
// a shader, not copied: both textures into one buffer, three floats a pixel
const AOV_WGSL: &str = r#"
@group(0) @binding(0) var depth_t: texture_depth_2d;
@group(0) @binding(1) var motion_t: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> out: array<f32>;
@group(0) @binding(3) var<uniform> dims: vec4<u32>;
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= dims.x || id.y >= dims.y) { return; }
    let p = vec2<i32>(id.xy);
    let m = textureLoad(motion_t, p, 0).xy;
    let i = (id.y * dims.x + id.x) * 3u;
    out[i] = m.x;
    out[i + 1u] = m.y;
    out[i + 2u] = textureLoad(depth_t, p, 0);
}
"#;

impl Engine {
    /// Draw the guide data from now on (before the first draw: a camera's prepass is set up as it's extracted).
    pub fn want_aov(&mut self) {
        if self.app.world().contains_resource::<AovWanted>() {
            return;
        }
        self.app.init_resource::<AovWanted>();
        self.app.add_systems(Update, aov_prepasses);
    }

    /// Frame f's guide data, read after its first draw (Engine::draw calls this).
    pub(crate) fn capture_aov(&mut self, f: i64) {
        let Some(w) = self.app.world().get_resource::<AovWanted>() else { return };
        if w.frame == Some(f) {
            return;
        }
        let got = self.read_aov();
        let mut w = self.app.world_mut().resource_mut::<AovWanted>();
        w.frame = Some(f);
        match got {
            Ok(a) => w.last = Some(a),
            Err(e) => {
                w.last = None;
                eprintln!("raea: frame {f}: no motion vectors or depth: {e}");
            }
        }
    }

    /// The guide data the last frame drawn read, taken (None when it wasn't asked for or couldn't be read).
    pub fn take_aov(&mut self) -> Option<Aov> {
        self.app.world_mut().get_resource_mut::<AovWanted>()?.last.take()
    }

    fn read_aov(&mut self) -> Result<Aov, String> {
        let rw = self.app.get_sub_app_mut(bevy::render::RenderApp).ok_or("no render app")?.world_mut();
        // the biggest view with both (the frame's camera; a screen's or a mirror's is smaller)
        let mut q = rw.query::<&ViewPrepassTextures>();
        let pick = q
            .iter(rw)
            .filter(|t| t.depth.is_some() && t.motion_vectors.is_some())
            .max_by_key(|t| t.size.width * t.size.height)
            .ok_or("no 3D camera drew a depth and motion-vector prepass")?;
        // the target is a row taller than the frame (the frame tag's row, open.rs), at the frame's scale (RAEA_SS)
        let (w, h) = (pick.size.width, pick.size.height.min(self.h * (pick.size.width / self.w.max(1)).max(1)));
        let depth = pick.depth_only_view().ok_or("the prepass has no depth view")?.clone();
        let motion = pick.motion_vectors.as_ref().unwrap().texture.default_view.clone();
        let device = rw.resource::<bevy::render::renderer::RenderDevice>().wgpu_device().clone();
        let queue = rw.resource::<bevy::render::renderer::RenderQueue>().clone();
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("aov"), source: wgpu::ShaderSource::Wgsl(AOV_WGSL.into()) });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("aov"), layout: None, module: &module, entry_point: Some("main"), compilation_options: Default::default(), cache: None });
        let size = w as u64 * h as u64 * 12;
        let out = device.create_buffer(&wgpu::BufferDescriptor { label: Some("aov"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let back = device.create_buffer(&wgpu::BufferDescriptor { label: Some("aov readback"), size, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
        let dims = wgpu::util::DeviceExt::create_buffer_init(&device, &wgpu::util::BufferInitDescriptor { label: Some("aov dims"), contents: bytemuck::cast_slice(&[w, h, 0u32, 0u32]), usage: wgpu::BufferUsages::UNIFORM });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("aov"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&*depth) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&*motion) },
                wgpu::BindGroupEntry { binding: 2, resource: out.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: dims.as_entire_binding() },
            ],
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("aov") });
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        }
        enc.copy_buffer_to_buffer(&out, 0, &back, 0, size);
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        back.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::PollType::Wait { submission_index: None, timeout: Some(std::time::Duration::from_secs(60)) }).map_err(|e| format!("the guide data's readback: {e}"))?;
        rx.recv().map_err(|e| e.to_string())?.map_err(|e| format!("the guide data's readback: {e}"))?;
        let data = back.slice(..).get_mapped_range().map_err(|e| format!("the guide data's readback: {e:?}"))?;
        let px: &[f32] = bytemuck::cast_slice(&data);
        let n = (w * h) as usize;
        let (mut motion, mut depth) = (Vec::with_capacity(n * 2), Vec::with_capacity(n));
        for p in px.chunks_exact(3) {
            motion.extend_from_slice(&p[..2]);
            depth.push(p[2]);
        }
        drop(data);
        back.unmap();
        Ok(Aov { w, h, motion, depth })
    }
}
