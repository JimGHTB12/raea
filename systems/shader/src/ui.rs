//! raea's UiMaterial (docs/control.feature FC2a, Bevy's ui/ui_material and bevy_ui_render's MaterialNode): a page
//! node's own fragment shader, run by Bevy's UiMaterial pipeline as written (its `@group(1)` bindings renumbered into
//! one Rust type's slots, ui_bind.rs) on a bevy_ui node of the page node's border box, its border widths and corner
//! radii, under a Camera2d drawing into an image that size, cleared to nothing. The page shows the image in the node's
//! box (premultiplied, as Bevy's alpha blend over a clear target leaves it), so the material sits in the page's paint
//! order: over what's under the node, under its children and whatever is drawn after it.
//!
//! Each frame the host hands `set` the page's material nodes (their ids, specs and box sizes) before the engine draws,
//! and after it shows `textures` (or `read`'s pictures) in those nodes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::asset::uuid_handle;
use bevy::camera::{ClearColorConfig, RenderTarget};
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, RenderPipelineDescriptor, TextureFormat, TextureUsages};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::shader::{Shader, ShaderRef};
use bevy::ui::UiTargetCamera;
use bevy::ui_render::prelude::{MaterialNode, UiMaterial, UiMaterialKey, UiMaterialPlugin};
use serde_json::Value;

use crate::layout;
use crate::ui_bind::{rebind, Rebound};

/// The fragment shader every pipeline starts from, swapped in `specialize` for the material's own (its uuid).
const PLACEHOLDER: Handle<Shader> = uuid_handle!("8c3e0f1a-4b6d-4c2e-9f7a-1d5b3e9c7a20");
const STUB: &str = "import bevy_ui_render::ui_vertex_output::UiVertexOutput;\n@fragment\nfn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> { return vec4(0.0); }\n";

/// One Rust type for every author's UiMaterial: eight uniform buffers of 256 bytes, four textures with their samplers,
/// and the shader that reads them (ui_bind.rs's slots).
#[derive(Asset, AsBindGroup, TypePath, Clone, Debug)]
#[bind_group_data(UiKey)]
pub struct RaeaUiMaterial {
    #[uniform(0)]
    u0: [Vec4; 16],
    #[uniform(1)]
    u1: [Vec4; 16],
    #[uniform(2)]
    u2: [Vec4; 16],
    #[uniform(3)]
    u3: [Vec4; 16],
    #[uniform(4)]
    u4: [Vec4; 16],
    #[uniform(5)]
    u5: [Vec4; 16],
    #[uniform(6)]
    u6: [Vec4; 16],
    #[uniform(7)]
    u7: [Vec4; 16],
    #[texture(8)]
    #[sampler(9)]
    t0: Option<Handle<Image>>,
    #[texture(10)]
    #[sampler(11)]
    t1: Option<Handle<Image>>,
    #[texture(12)]
    #[sampler(13)]
    t2: Option<Handle<Image>>,
    #[texture(14)]
    #[sampler(15)]
    t3: Option<Handle<Image>>,
    shader: u128,
}

/// What picks a pipeline: the material's shader.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UiKey {
    shader: u128,
}

impl From<&RaeaUiMaterial> for UiKey {
    fn from(m: &RaeaUiMaterial) -> Self {
        UiKey { shader: m.shader }
    }
}

impl UiMaterial for RaeaUiMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(PLACEHOLDER)
    }
    fn specialize(descriptor: &mut RenderPipelineDescriptor, key: UiMaterialKey<Self>) {
        if let Some(f) = descriptor.fragment.as_mut() {
            if f.shader.id() == PLACEHOLDER.id() {
                f.shader = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(key.bind_group_data.shader), core::marker::PhantomData);
            }
        }
    }
}

/// A material node as the page laid it out this frame: its page id, its spec ({fragment, uniforms, textures, defs,
/// border: [l, t, r, b], radius: [tl, tr, br, bl]}, px) and its border box's size in whole px.
pub struct UiMat {
    pub id: u64,
    pub spec: Value,
    pub size: (u32, u32),
}

struct Entry {
    camera: Entity,
    node: Entity,
    image: Handle<Image>,
    material: Handle<RaeaUiMaterial>,
    size: (u32, u32),
}

#[derive(Resource, Default)]
struct UiMats {
    by_id: HashMap<u64, Entry>,
    pictures: HashMap<String, Handle<Image>>,
}

/// Each material node's target image, for the render world to share.
#[derive(Resource, Clone, Default, ExtractResource)]
#[extract_app(RenderApp)]
struct Targets(Vec<(u64, AssetId<Image>, u32, u32)>);

/// The targets' textures on the render device by page id, with the device and queue to read them back.
#[derive(Resource, Clone, Default)]
struct Shared(Arc<Mutex<SharedInner>>);

#[derive(Default)]
struct SharedInner {
    textures: HashMap<u64, (wgpu::Texture, u32, u32)>,
    device: Option<(RenderDevice, RenderQueue)>,
}

/// marks a UiMaterial's camera, so nothing else takes it for the scene's
#[derive(Component)]
pub struct UiMaterialCamera;

pub fn install(app: &mut App) {
    let _ = app.world_mut().resource_mut::<Assets<Shader>>().insert(&PLACEHOLDER, Shader::from_wesl(STUB, "embedded://raea_sys_shader/ui_placeholder.wesl"));
    let shared = Shared::default();
    app.add_plugins((UiMaterialPlugin::<RaeaUiMaterial>::default(), ExtractResourcePlugin::<Targets>::default()))
        .init_resource::<UiMats>()
        .init_resource::<Targets>()
        .insert_resource(shared.clone());
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.insert_resource(shared).add_systems(Render, share.in_set(RenderSystems::PrepareResources));
    }
}

/// The render world's side: each target's texture shared, once its GpuImage is made.
fn share(targets: Option<Res<Targets>>, images: Res<RenderAssets<GpuImage>>, shared: Res<Shared>, device: Res<RenderDevice>, queue: Res<RenderQueue>) {
    let mut s = shared.0.lock().unwrap();
    s.textures.clear();
    for &(id, image, w, h) in targets.iter().flat_map(|t| t.0.iter()) {
        if let Some(gi) = images.get(image) {
            s.textures.insert(id, ((*gi.texture).clone(), w, h));
        }
    }
    if s.device.is_none() {
        s.device = Some((device.clone(), queue.clone()));
    }
}

/// This frame's material nodes: each made, changed or kept; the rest removed.
pub fn set(world: &mut World, mats: &[UiMat]) -> Result<(), String> {
    let scale = world.get_resource::<bevy::ui::UiScale>().map_or(1.0, |s| s.0);
    let mut state = world.remove_resource::<UiMats>().unwrap_or_default();
    let keep: Vec<u64> = mats.iter().map(|m| m.id).collect();
    for (id, e) in state.by_id.extract_if(|id, _| !keep.contains(id)) {
        let _ = id;
        world.despawn(e.camera);
        world.despawn(e.node);
    }
    let mut result = Ok(());
    for m in mats {
        if let Err(e) = put(world, &mut state, m, scale) {
            result = Err(e);
        }
    }
    let targets = Targets(state.by_id.iter().map(|(&id, e)| (id, e.image.id(), e.size.0, e.size.1)).collect());
    world.insert_resource(targets);
    world.insert_resource(state);
    result
}

fn put(world: &mut World, state: &mut UiMats, m: &UiMat, scale: f32) -> Result<(), String> {
    let (w, h) = (m.size.0.max(1), m.size.1.max(1));
    let o = m.spec.as_object().ok_or("a UiMaterial is {fragment, uniforms, textures}")?;
    let material = build(world, state, o)?;
    let px4 = |k: &str| -> Result<[f32; 4], String> {
        match o.get(k) {
            None => Ok([0.0; 4]),
            Some(v) => {
                let a = v.as_array().filter(|a| a.len() == 4).ok_or_else(|| format!("a UiMaterial's {k} is four px, got {v}"))?;
                let f: Vec<f32> = a.iter().map(|x| x.as_f64().map(|f| f as f32 / scale).ok_or_else(|| format!("a UiMaterial's {k} is four px, got {v}"))).collect::<Result<_, _>>()?;
                Ok([f[0], f[1], f[2], f[3]])
            }
        }
    };
    let b = px4("border")?;
    let r = radii(o.get("radius"), scale)?;
    let node = Node {
        width: percent(100),
        height: percent(100),
        border: UiRect { left: Val::Px(b[0]), top: Val::Px(b[1]), right: Val::Px(b[2]), bottom: Val::Px(b[3]) },
        border_radius: BorderRadius { top_left: r[0], top_right: r[1], bottom_right: r[2], bottom_left: r[3] },
        ..default()
    };
    if let Some(e) = state.by_id.get_mut(&m.id).filter(|e| e.size == (w, h)) {
        if let Some(mut cur) = world.resource_mut::<Assets<RaeaUiMaterial>>().get_mut(&e.material) {
            *cur = material;
        }
        world.entity_mut(e.node).insert(node);
        return Ok(());
    }
    if let Some(old) = state.by_id.remove(&m.id) {
        world.despawn(old.camera);
        world.despawn(old.node);
    }
    let mut image = Image::new_target_texture(w, h, TextureFormat::Rgba8UnormSrgb, None);
    // the page copies it on the GPU into its atlas (place_texture_at)
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let image = world.resource_mut::<Assets<Image>>().add(image);
    let material = world.resource_mut::<Assets<RaeaUiMaterial>>().add(material);
    let camera = world.spawn((Camera2d, Camera { order: -100, clear_color: ClearColorConfig::Custom(Color::NONE), ..default() }, RenderTarget::Image(image.clone().into()), UiMaterialCamera)).id();
    let node = world.spawn((node, MaterialNode(material.clone()), UiTargetCamera(camera))).id();
    state.by_id.insert(m.id, Entry { camera, node, image, material, size: (w, h) });
    Ok(())
}

/// The corners a spec gives, each [x, y] as Vals (y "auto": circular), resolved by Bevy against the node itself; px
/// divided by the UI's scale, as the border's are.
fn radii(v: Option<&Value>, scale: f32) -> Result<[CornerRadius; 4], String> {
    let Some(v) = v else { return Ok(Default::default()) };
    let bad = || format!("a UiMaterial's radius is four [x, y] Vals, got {v}");
    let a = v.as_array().filter(|a| a.len() == 4).ok_or_else(bad)?;
    let mut out: [CornerRadius; 4] = Default::default();
    for (i, c) in a.iter().enumerate() {
        let c = c.as_array().filter(|c| c.len() == 2).ok_or_else(bad)?;
        let x = val(c[0].as_str().ok_or_else(bad)?, scale)?;
        out[i] = match c[1].as_str().ok_or_else(bad)? {
            "auto" => x.into(),
            y => (x, val(y, scale)?).into(),
        };
    }
    Ok(out)
}

/// A CSS length as the page writes a Val: px, %, vw, vh, vmin, vmax, auto.
fn val(s: &str, scale: f32) -> Result<Val, String> {
    let n = |u: &str| s.strip_suffix(u).and_then(|n| n.trim().parse::<f32>().ok());
    Ok(match s.trim() {
        "auto" => Val::Auto,
        "0" => Val::Px(0.0),
        _ => {
            if let Some(v) = n("px") {
                Val::Px(v / scale)
            } else if let Some(v) = n("%") {
                Val::Percent(v)
            } else if let Some(v) = n("vmin") {
                Val::VMin(v)
            } else if let Some(v) = n("vmax") {
                Val::VMax(v)
            } else if let Some(v) = n("vw") {
                Val::Vw(v)
            } else if let Some(v) = n("vh") {
                Val::Vh(v)
            } else {
                return Err(format!("a UiMaterial's corner {s:?} isn't a Val"));
            }
        }
    })
}

/// The material a spec asks for: its shader registered (as written, its bindings renumbered), its uniforms packed in
/// their declared types, its textures loaded.
fn build(world: &mut World, state: &mut UiMats, o: &serde_json::Map<String, Value>) -> Result<RaeaUiMaterial, String> {
    let path = o.get("fragment").and_then(Value::as_str).ok_or("a UiMaterial's fragment is its shader file")?;
    let src = std::fs::read_to_string(path).map_err(|e| format!("a UiMaterial's fragment {path}: {e}"))?;
    let defs = raea_sys_material::shader_material::defs(o, "a UiMaterial")?;
    let Rebound { src, uniforms, textures, .. } = rebind(&src).map_err(|e| format!("{path}: {e}"))?;
    let given = o.get("uniforms").and_then(Value::as_object).cloned().unwrap_or_default();
    let slots = layout::slots(&src, &uniforms, &given, &defs).map_err(|e| format!("{path}: {e}"))?;
    let given = o.get("textures").and_then(Value::as_object).cloned().unwrap_or_default();
    if let Some(k) = given.keys().find(|k| !textures.contains(k)) {
        return Err(format!("{path} has no texture {k}"));
    }
    let mut t: [Option<Handle<Image>>; 4] = Default::default();
    for (i, name) in textures.iter().enumerate() {
        let p = given.get(name).and_then(Value::as_str).ok_or_else(|| format!("{path}'s texture {name} has no picture (a file)"))?;
        t[i] = Some(picture(world, state, p)?);
    }
    let shader = crate::register(world, src, &defs, "raea ui material", "raea_sys_shader");
    let [t0, t1, t2, t3] = t;
    let [u0, u1, u2, u3, u4, u5, u6, u7] = slots;
    Ok(RaeaUiMaterial { u0, u1, u2, u3, u4, u5, u6, u7, t0, t1, t2, t3, shader })
}

/// A picture file as an Image, decoded once (sRGB, as asset_server.load reads a colour texture).
fn picture(world: &mut World, state: &mut UiMats, path: &str) -> Result<Handle<Image>, String> {
    if let Some(h) = state.pictures.get(path) {
        return Ok(h.clone());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("a UiMaterial texture {path}: {e}"))?;
    let ext = std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("png");
    let image = Image::from_buffer(&bytes, ImageType::Extension(ext), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::default()).map_err(|e| format!("a UiMaterial texture {path}: {e}"))?;
    let h = world.resource_mut::<Assets<Image>>().add(image);
    state.pictures.insert(path.to_string(), h.clone());
    Ok(h)
}

/// Each material node's picture on the render device (premultiplied, `Rgba8UnormSrgb`), w x h, by page id: drawn by
/// the engine's last draw.
pub fn textures(world: &World) -> Vec<(u64, wgpu::Texture, u32, u32)> {
    let Some(s) = world.get_resource::<Shared>() else { return vec![] };
    s.0.lock().unwrap().textures.iter().map(|(&id, (t, w, h))| (id, t.clone(), *w, *h)).collect()
}

/// Each material node's picture read back, straight RGBA, by page id (the hosts that read frames back).
pub fn read(world: &World) -> Result<Vec<(u64, u32, u32, Vec<u8>)>, String> {
    let Some(s) = world.get_resource::<Shared>() else { return Ok(vec![]) };
    let s = s.0.lock().unwrap();
    let Some((device, queue)) = s.device.as_ref() else { return Ok(vec![]) };
    let mut out = Vec::new();
    for (&id, (t, w, h)) in &s.textures {
        let mut px = readback(device.wgpu_device(), queue, t, *w, *h)?;
        for p in px.chunks_exact_mut(4) {
            let a = p[3] as u32;
            if a != 0 && a != 255 {
                for c in &mut p[..3] {
                    *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
        out.push((id, *w, *h, px));
    }
    Ok(out)
}

fn readback(device: &wgpu::Device, queue: &wgpu::Queue, t: &wgpu::Texture, w: u32, h: u32) -> Result<Vec<u8>, String> {
    let row = (w * 4).div_ceil(256) * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("UiMaterial readback"), size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("UiMaterial readback") });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: t, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    buf.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: Some(std::time::Duration::from_secs(60)) }).map_err(|e| format!("a UiMaterial's readback: {e}"))?;
    rx.recv().map_err(|e| e.to_string())?.map_err(|e| format!("a UiMaterial's readback: {e}"))?;
    let data = buf.slice(..).get_mapped_range().map_err(|e| format!("a UiMaterial's readback: {e:?}"))?;
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h as usize {
        out.extend_from_slice(&data[y * row as usize..y * row as usize + w as usize * 4]);
    }
    drop(data);
    buf.unmap();
    Ok(out)
}
