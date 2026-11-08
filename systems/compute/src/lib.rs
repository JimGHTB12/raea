//! A compute shader written in TypeScript (docs/shaders.feature SH10): <Compute wesl workgroups uniforms buffers images
//! defs> in a <Canvas3D>, more than one in the order written, each dispatched once a frame before the cameras draw (in
//! the RenderGraph schedule before camera_driver, as Bevy 0.20's compute example is). Its module, in WESL:
//!   0      `raea_compute`: the frame's time (`time()`, seconds) and the uniform slots (`u_NAME()`)
//!   1 + i  each buffer, `var<storage, read_write> NAME: array<f32>`: the compute's own, kept from frame to frame (a
//!          simulation's state), so a render that starts past frame 0 starts it fresh
//!   5 + i  each image, `var NAME: texture_storage_2d<rgba8unorm, write>`: a render:NAME image, which any shader's
//!          texture or a material's map reads by that name (it is made rgba8unorm, storage-capable)
//! The source has its one `@compute` entry point; `workgroups` is how many groups a frame dispatches.

use std::collections::HashMap;

use bevy::core_pipeline::schedule::camera_driver;
use bevy::prelude::*;
use bevy::render::{
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_asset::RenderAssets,
    render_resource::{
        binding_types::{storage_buffer_sized, texture_storage_2d, uniform_buffer_sized},
        *,
    },
    renderer::{RenderContext, RenderDevice, RenderGraph},
    texture::GpuImage,
    Render, RenderApp, RenderSystems,
};
use raea_sys_material::shader_material::{ident, SLOTS};
use raea_sys_shader::{accessors, register, Inputs};
use serde_json::Value;

mod slab;
pub mod written;

/// Buffers and images a compute may have.
pub const BUFFERS: usize = 4;
pub const IMAGES: usize = 4;

/// One compute, built.
#[derive(Clone, Debug)]
pub struct Compute {
    pub shader: u128,
    pub workgroups: [u32; 3],
    pub slots: [Vec4; SLOTS],
    /// each buffer's name and first contents (its length is its size, in f32s)
    pub buffers: Vec<(String, Vec<f32>)>,
    pub images: Vec<Handle<Image>>,
    /// a compute as written (a <Compute file>): its own bindings, entry and images (written.rs)
    pub written: Option<written::Written>,
    /// its row's id, whose keyed `uniforms` column it follows frame by frame; its uniforms' names, in slot order; the
    /// column last followed
    pub id: String,
    pub names: Vec<String>,
    pub given: String,
}

/// The frame's computes, in order, and the frame's time.
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
#[extract_app(bevy::render::RenderApp)]
pub struct Computes {
    pub list: Vec<Compute>,
    pub time: f32,
}

/// The module a compute's source is compiled as: its bindings and accessors, then the source.
pub fn module(source: &str, uniforms: &[(String, &'static str)], buffers: &[String], images: &[String]) -> String {
    let mut m = String::from(
        "struct RaeaCompute { time: vec4<f32>, s: array<vec4<f32>, 16> };\n\
         @group(0) @binding(0) var<uniform> raea_compute: RaeaCompute;\n\
         fn time() -> f32 { return raea_compute.time.x; }\n",
    );
    for (i, b) in buffers.iter().enumerate() {
        m += &format!("@group(0) @binding({}) var<storage, read_write> {b}: array<f32>;\n", 1 + i);
    }
    for (i, t) in images.iter().enumerate() {
        m += &format!("@group(0) @binding({}) var {t}: texture_storage_2d<rgba8unorm, write>;\n", 1 + BUFFERS + i);
    }
    m += &accessors(uniforms, "raea_compute");
    m + "\n" + source
}

/// A compute from a <Compute>'s spec, {wesl | wgsl, workgroups, uniforms, buffers, images, defs} (or a file as written,
/// written.rs), its row's id `id`. `image` resolves a "render:NAME" as raea-runtime-plugins' fields::image does; the image is
/// made storage-capable here.
pub fn build(world: &mut World, v: &Value, id: &str, mut image: impl FnMut(&mut World, &Value) -> Result<Handle<Image>, String>) -> Result<Compute, String> {
    let o = v.as_object().ok_or_else(|| format!("a compute is {{wesl, workgroups, uniforms, buffers, images}}, got {v}"))?;
    let workgroups = workgroups(o)?;
    if o.contains_key("file") {
        let (shader, w) = written::build(world, o, id, &mut image)?;
        return Ok(Compute { shader, workgroups, slots: [Vec4::ZERO; SLOTS], buffers: vec![], images: vec![], written: Some(w), id: id.to_string(), names: vec![], given: String::new() });
    }
    let ins = Inputs::read(world, o, "a compute", &mut image)?;
    if !ins.textures.is_empty() || ins.array.is_some() {
        return Err("a compute has buffers and images, not textures".into());
    }
    if ins.src.matches("@compute").count() != 1 {
        return Err("a compute's source has one @compute entry point".into());
    }
    let named = |k: &str| -> Result<Vec<(&String, &Value)>, String> {
        let m = match o.get(k) {
            None => return Ok(vec![]),
            Some(x) => x.as_object().ok_or_else(|| format!("a compute's {k} are {{NAME: ...}}"))?,
        };
        if let Some(bad) = m.keys().find(|n| !ident(n)) {
            return Err(format!("a compute's {k}: {bad:?} isn't a WGSL name"));
        }
        Ok(m.iter().collect())
    };
    let mut buffers = vec![];
    for (k, x) in named("buffers")? {
        let data = match x {
            Value::Array(a) if !a.is_empty() => a.iter().map(|n| n.as_f64().map(|f| f as f32).ok_or_else(|| format!("buffer {k}: numbers, got {n}"))).collect::<Result<Vec<_>, _>>()?,
            Value::Object(s) => match s.get("size").and_then(Value::as_u64).filter(|n| (1..=1 << 24).contains(n)) {
                Some(n) => vec![0.0; n as usize],
                None => return Err(format!("buffer {k}: size is a number of f32s, 1 to {}", 1 << 24)),
            },
            _ => return Err(format!("buffer {k} is {{size}} or its first contents, [numbers], got {x}")),
        };
        buffers.push((k.clone(), data));
    }
    let mut images = vec![];
    let mut inames = vec![];
    for (k, x) in named("images")? {
        if !x.as_str().is_some_and(|s| s.starts_with("render:")) {
            return Err(format!("image {k} is a \"render:NAME\" (or \"render:NAME:WxH\"), got {x}"));
        }
        let h = image(world, x).map_err(|e| format!("image {k}: {e}"))?;
        if let Some(mut img) = world.resource_mut::<Assets<Image>>().get_mut(&h) {
            img.texture_descriptor.format = TextureFormat::Rgba8Unorm;
            img.texture_descriptor.usage |= TextureUsages::STORAGE_BINDING;
        }
        images.push(h);
        inames.push(k.clone());
    }
    if buffers.len() > BUFFERS || images.len() > IMAGES {
        return Err(format!("a compute has at most {BUFFERS} buffers and {IMAGES} images"));
    }
    let (names, _) = ins.names();
    let bnames: Vec<String> = buffers.iter().map(|b| b.0.clone()).collect();
    let shader = register(world, module(&ins.src, &names, &bnames, &inames), &ins.defs, "raea compute", "raea_sys_compute");
    let uniforms = names.iter().map(|(n, _)| n.clone()).collect();
    Ok(Compute { shader, workgroups, slots: ins.slots(), buffers, images, written: None, id: id.to_string(), names: uniforms, given: String::new() })
}

/// A spec's workgroups, [x(, y(, z))], each 1 to 65535.
fn workgroups(o: &serde_json::Map<String, Value>) -> Result<[u32; 3], String> {
    match o.get("workgroups").and_then(Value::as_array) {
        Some(a) if (1..=3).contains(&a.len()) && a.iter().all(|x| x.as_u64().is_some_and(|n| (1..=65535).contains(&n))) => {
            let n = |i: usize| a.get(i).and_then(Value::as_u64).unwrap_or(1) as u32;
            Ok([n(0), n(1), n(2)])
        }
        _ => Err(format!("a compute's workgroups are [x, y, z], each 1 to 65535, got {}", o.get("workgroups").unwrap_or(&Value::Null))),
    }
}

pub fn install(app: &mut App) {
    app.init_resource::<Computes>();
    app.add_plugins(ExtractResourcePlugin::<Computes>::default());
    app.add_systems(PostUpdate, (follow, stamp));
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
    render_app.init_resource::<State>();
    render_app.add_systems(Render, queue.in_set(RenderSystems::Queue));
    storage_slabs(render_app.world_mut());
    render_app.add_systems(RenderGraph, dispatch.before(camera_driver));
}

/// The mesh allocator's slabs storage buffers too, so a compute as written may bind one (as compute_mesh's plugin sets
/// MeshAllocatorSettings::extra_buffer_usages); before the allocator is made from them, in MeshAllocatorPlugin's finish.
fn storage_slabs(w: &mut World) {
    use bevy::render::mesh::allocator::MeshAllocatorSettings;
    match w.get_resource_mut::<MeshAllocatorSettings>() {
        Some(mut s) => s.extra_buffer_usages |= BufferUsages::STORAGE,
        None => w.insert_resource(MeshAllocatorSettings { extra_buffer_usages: BufferUsages::STORAGE, ..default() }),
    }
}

/// Each compute as written's entry and bindings from its row's `entry` and `bind` this frame (keyed strings) (a bad one is
/// logged, and it isn't dispatched until the columns change). Each mesh slab
/// it binds follows the mesh its named part holds.
fn follow(frame: Option<Res<raea_sys_frame::FrameRows>>, mut c: ResMut<Computes>, parts: Query<(&Name, &Mesh3d)>) {
    let Some(frame) = frame else { return };
    let cols = &frame.cols;
    for c in c.list.iter_mut() {
        let Some(r) = cols.id.iter().position(|i| *i == c.id) else { continue };
        let s = |k: &str| raea_sys_frame::keyed(|col| frame.at(r, col), k).unwrap_or_default();
        let Some(w) = c.written.as_mut() else {
            if let Err(e) = inline_uniforms(c, &s("uniforms")) {
                error!("Compute {}: {e}", c.id);
            }
            continue;
        };
        for b in w.buffers.values_mut() {
            if let written::Bound::Mesh { name, id, .. } = b {
                *id = parts.iter().find(|(n, _)| n.as_str() == name).map(|(_, m)| m.id());
            }
        }
        if let Err(e) = written::follow(w, &s("entry"), &s("bind")).and_then(|_| written::follow_uniforms(w, &s("uniforms"))) {
            error!("Compute {}: {e}", w.id);
            w.entry = None;
        }
    }
}

/// This frame's `uniforms` column (JSON, {NAME: value}) into an inline compute's slots, when it has changed; a name
/// it wasn't built with is refused (its slots are fixed at build).
fn inline_uniforms(c: &mut Compute, uniforms: &str) -> Result<(), String> {
    if uniforms.is_empty() || c.given == uniforms {
        return Ok(());
    }
    c.given = uniforms.to_string();
    let given: serde_json::Map<String, Value> = serde_json::from_str(uniforms).map_err(|e| format!("uniforms: {e}"))?;
    for (k, x) in &given {
        let i = c.names.iter().position(|n| n == k).ok_or_else(|| format!("uniform {k} isn't one it was built with ({:?})", c.names))?;
        c.slots[i] = raea_sys_material::shader_material::uniform(k, x)?.value;
    }
    Ok(())
}

/// The frame's time onto the computes (so they're extracted each frame).
fn stamp(time: Res<Time>, mut c: ResMut<Computes>) {
    if !c.list.is_empty() {
        c.time = time.elapsed_secs();
    }
}

const UNIFORM_BYTES: u64 = (16 * (1 + SLOTS)) as u64;

/// Pipelines by shader and shape, and each compute's buffers (kept by its place in the list, its shader and the
/// buffer's name).
#[derive(Resource, Default)]
struct State {
    pipelines: HashMap<(u128, usize, usize), (BindGroupLayoutDescriptor, CachedComputePipelineId)>,
    buffers: HashMap<(usize, u128, String), Buffer>,
    /// a compute as written's pipelines, by its shader and entry point
    written: HashMap<(u128, String), (BindGroupLayoutDescriptor, CachedComputePipelineId)>,
}

fn queue(c: Option<Res<Computes>>, mut s: ResMut<State>, cache: Res<PipelineCache>) {
    let Some(c) = c else { return };
    for k in &c.list {
        if let Some(w) = &k.written {
            let key = w.entry.as_ref().map(|e| (k.shader, e.clone()));
            if let Some(key) = key.filter(|key| !s.written.contains_key(key)) {
                let entry = key.1.clone();
                let layout = BindGroupLayoutDescriptor::new("raea_compute_written", &written::layout(&w.decls));
                let shader = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(k.shader), core::marker::PhantomData);
                let id = cache.queue_compute_pipeline(ComputePipelineDescriptor { label: Some("raea_compute_written".into()), layout: vec![layout.clone()], shader, entry_point: Some(entry.clone().into()), ..default() });
                s.written.insert(key, (layout, id));
            }
            continue;
        }
        let key = (k.shader, k.buffers.len(), k.images.len());
        if s.pipelines.contains_key(&key) {
            continue;
        }
        let mut entries = vec![uniform_buffer_sized(false, std::num::NonZeroU64::new(UNIFORM_BYTES)).build(0, ShaderStages::COMPUTE)];
        for i in 0..key.1 {
            entries.push(storage_buffer_sized(false, None).build(1 + i as u32, ShaderStages::COMPUTE));
        }
        for i in 0..key.2 {
            entries.push(texture_storage_2d(TextureFormat::Rgba8Unorm, StorageTextureAccess::WriteOnly).build((1 + BUFFERS + i) as u32, ShaderStages::COMPUTE));
        }
        let layout = BindGroupLayoutDescriptor::new("raea_compute", &entries);
        let id = cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("raea_compute".into()),
            layout: vec![layout.clone()],
            shader: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(k.shader), core::marker::PhantomData),
            ..default()
        });
        s.pipelines.insert(key, (layout, id));
    }
}

/// Each compute whose pipeline and images are ready, dispatched in order in one compute pass.
fn dispatch(c: Option<Res<Computes>>, mut s: ResMut<State>, cache: Res<PipelineCache>, images: Res<RenderAssets<GpuImage>>, slabs: Slabs, mut ctx: RenderContext) {
    let Some(c) = c else { return };
    let device = ctx.render_device().clone();
    let mut groups = vec![];
    'computes: for (n, k) in c.list.iter().enumerate() {
        if let Some(w) = &k.written {
            let Some((layout, id)) = w.entry.as_ref().and_then(|e| s.written.get(&(k.shader, e.clone()))) else { continue };
            let (Some(pipeline), layout) = (cache.get_compute_pipeline(*id), cache.get_bind_group_layout(layout)) else { continue };
            // its own buffers made once and kept; a mesh's slab once the allocator has placed it
            let (mut bufs, mut ranges) = (std::collections::BTreeMap::new(), HashMap::new());
            for (name, b) in &w.buffers {
                let buf = match b {
                    written::Bound::Own(bytes) => s.buffers.entry((n, k.shader, name.clone())).or_insert_with(|| storage(&device, bytes)).clone(),
                    written::Bound::Mesh { id: Some(id), indices, .. } => {
                        let Some((buf, r)) = slab::slab_of(&slabs.0, &slabs.1, *id, *indices) else { continue 'computes };
                        ranges.insert(name.as_str(), r);
                        buf
                    }
                    written::Bound::Mesh { id: None, .. } => continue 'computes,
                };
                bufs.insert(name.clone(), buf);
            }
            let uniforms = match w.ranged {
                true => match w.packer.pack(&|b, end| ranges.get(b).map(|r| r[end as usize])) {
                    Ok((u, _)) => u,
                    Err(e) => {
                        error!("Compute {}: {e}", w.id);
                        continue;
                    }
                },
                false => w.uniforms.clone(),
            };
            if let Some(group) = written::group(&device, &layout, w, &images, &bufs, &uniforms) {
                groups.push((pipeline, group, k.workgroups));
            }
            continue;
        }
        let Some((layout, id)) = s.pipelines.get(&(k.shader, k.buffers.len(), k.images.len())) else { continue };
        let Some(pipeline) = cache.get_compute_pipeline(*id) else { continue };
        let Some(views) = k.images.iter().map(|h| images.get(h.id())).collect::<Option<Vec<_>>>() else { continue };
        let layout = cache.get_bind_group_layout(layout);
        for (name, data) in &k.buffers {
            s.buffers.entry((n, k.shader, name.clone())).or_insert_with(|| storage(&device, &data.iter().flat_map(|f| f.to_ne_bytes()).collect::<Vec<_>>()));
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(UNIFORM_BYTES as usize);
        for v in std::iter::once(Vec4::new(c.time, 0.0, 0.0, 0.0)).chain(k.slots.iter().copied()) {
            for f in v.to_array() {
                bytes.extend_from_slice(&f.to_ne_bytes());
            }
        }
        let uniform = device.create_buffer_with_data(&BufferInitDescriptor { label: Some("raea_compute"), contents: &bytes, usage: BufferUsages::UNIFORM });
        let mut entries = vec![BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() }];
        for (i, (name, _)) in k.buffers.iter().enumerate() {
            entries.push(BindGroupEntry { binding: 1 + i as u32, resource: s.buffers[&(n, k.shader, name.clone())].as_entire_binding() });
        }
        for (i, v) in views.iter().enumerate() {
            entries.push(BindGroupEntry { binding: (1 + BUFFERS + i) as u32, resource: BindingResource::TextureView(&v.texture_view) });
        }
        groups.push((pipeline, device.create_bind_group("raea_compute", &layout, &entries), k.workgroups));
    }
    if groups.is_empty() {
        return;
    }
    let mut pass = ctx.command_encoder().begin_compute_pass(&ComputePassDescriptor { label: Some("raea_compute"), timestamp_writes: None });
    for (pipeline, group, [x, y, z]) in &groups {
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.dispatch_workgroups(*x, *y, *z);
    }
}

/// The mesh allocator and the meshes on the GPU, for a compute that binds a mesh's slab.
type Slabs<'w> = (Res<'w, bevy::render::mesh::allocator::MeshAllocator>, Res<'w, RenderAssets<bevy::render::mesh::RenderMesh>>);

fn storage(device: &RenderDevice, bytes: &[u8]) -> Buffer {
    device.create_buffer_with_data(&BufferInitDescriptor { label: Some("raea_compute_buffer"), contents: bytes, usage: BufferUsages::STORAGE | BufferUsages::COPY_DST })
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_module_binds_buffers_and_images() {
        let m = super::module("@compute @workgroup_size(1) fn main() { cells[0] = u_k(); }", &[("k".into(), "f32")], &["cells".into()], &["out".into()]);
        assert!(m.contains("@binding(1) var<storage, read_write> cells: array<f32>;"));
        assert!(m.contains("@binding(5) var out: texture_storage_2d<rgba8unorm, write>;"));
        assert!(m.contains("fn u_k() -> f32 { return raea_compute.s[0].x; }") && m.contains("fn time() -> f32"));
    }
}
