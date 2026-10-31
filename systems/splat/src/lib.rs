//! Gaussian splats (`<Splat>` in raea/3d): a scene or an object reconstructed as millions of 3D gaussians
//! (a 3DGS `.ply` with spherical harmonics up to degree 3, an antimatter15 `.splat`, or a Niantic `.spz`), drawn by
//! splat.wesl as screen-space ellipses (EWA projection of each covariance), back to front, blended over the scene.
//!
//! The splats are one mesh entity with this module's material, so they draw in Bevy's transparent pass: they test
//! against the depth the opaque meshes wrote (a mesh in front hides them), sit on layer 0 (a frame `<Look>`'s camera
//! draws them into its cells, so pixel, ink and paint apply), and are placed, scaled and hidden by sync_meshes as any
//! mesh is. They write no depth and cast no shadow.
//!
//! The order is this module's, per frame: each splat's centre where the row's effects put it (anim_center, the
//! shader's own steps), its depth along the scene camera's view, a stable radix sort far to near, ties by index.
//! It's a function of the frame's rows and camera alone, so frame N sorts and draws the same in a render and in play.
//!
//! Columns of a splat row (all but src optional; every number keyed per frame):
//!   src (.ply / .splat / .spz; a `x.ply.json` beside it with `transform`, a column-major 4x4, places the file in the
//!   world: its rotation, uniform scale and offset), x y z rot_x rot_y rot_z scale_x scale_y scale_z visible,
//!   morph_to (a second splat file, the first frame's), morph_crop (only its splats this near its centre) and morph
//!   (0..1), spread (the dust cloud's radius),
//!   twist (radians a metre about the vertical through the centre), wave_amp wave_len wave_phase (a ripple from the
//!   centre), explode (0..1), disperse (0..1) disperse_mode ('wind' or 'dust') wind_x wind_y wind_z curl,
//!   reveal (0..1) reveal_mode ('radius' or 'plane') reveal_x reveal_y reveal_z (the radius's origin) reveal_ax
//!   reveal_ay reveal_az (the plane's axis) reveal_soft reveal_glow, pulse pulse_scale pulse_glow, splat_scale,
//!   opacity, brightness, saturation, contrast, tint (hex) | tint_r tint_g tint_b, glow (hex), seed, crop (metres
//!   from the centre past which nothing draws), bands (flat light bands; a paint or cartoon `<Look>` sets them),
//!   paint (0..1: the splats themselves painted: flat bands, grown into strokes with hard edges)

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use raea_core_program::eval::Tup;
use raea_core_program::ir::V;
use raea_sys_frame::*;
use bevy::light::NotShadowCaster;
use bevy::pbr::{Material, MaterialPlugin, MeshMaterial3d};
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;
use std::sync::{Arc, Mutex};

mod read;
pub(crate) use read::*;
mod order;
pub(crate) use order::*;
#[cfg(test)]
mod tests;

pub fn install(app: &mut App) {
    bevy::asset::embedded_asset!(app, "splat.wesl");
    app.add_plugins((MaterialPlugin::<SplatMaterial>::default(), MaterialPlugin::<SplatMask>::default()))
        .add_systems(PostUpdate, sync_splats.after(bevy::transform::TransformSystems::Propagate));
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct SplatMaterial {
    #[uniform(0)]
    p: SplatParams,
    #[storage(1, read_only)]
    splats: Handle<ShaderBuffer>,
    #[storage(2, read_only)]
    sh: Handle<ShaderBuffer>,
    #[storage(3, read_only)]
    target: Handle<ShaderBuffer>,
    #[storage(4, read_only)]
    order: Handle<ShaderBuffer>,
}

#[derive(ShaderType, Clone, Copy, PartialEq, Debug)]
pub struct SplatParams {
    sh_rot: Mat3,
    center: Vec3,
    radius: f32,
    count: u32,
    sh_deg: u32,
    sh_words: u32,
    morph: f32,
    seed: f32,
    twist: f32,
    wave_amp: f32,
    wave_len: f32,
    wave_phase: f32,
    explode: f32,
    disperse: f32,
    disperse_mode: u32,
    wind: Vec3,
    curl: f32,
    reveal: f32,
    reveal_mode: u32,
    reveal_soft: f32,
    reveal_glow: f32,
    reveal_origin: Vec3,
    crop: f32,
    reveal_axis: Vec3,
    bands: f32,
    pulse: f32,
    pulse_scale: f32,
    pulse_glow: f32,
    splat_scale: f32,
    opacity: f32,
    brightness: f32,
    saturation: f32,
    contrast: f32,
    tint: Vec4,
    glow_color: Vec4,
    paint: f32,
    spread: f32,
}

impl Material for SplatMaterial {
    fn vertex_shader() -> ShaderRef {
        "embedded://raea_sys_splat/splat.wesl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "embedded://raea_sys_splat/splat.wesl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Premultiplied
    }
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        // both faces of every quad (a splat's corners may come out either way round), depth tested, never written
        descriptor.primitive.cull_mode = None;
        if let Some(d) = descriptor.depth_stencil.as_mut() {
            d.depth_write_enabled = Some(false);
        }
        Ok(())
    }
}

/// The same splats on the looked models' layers (1 to 30), drawn by each look's camera into its model's cells as an
/// eraser: where a splat is in front of the model (the model's own depth test decides), the model's cells are cut by
/// the splat's alpha (dst * (1 - a), so the order doesn't matter), and a model standing inside a splat scene is hidden
/// by what's in front of it, as a mesh would be.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct SplatMask {
    #[uniform(0)]
    p: SplatParams,
    #[storage(1, read_only)]
    splats: Handle<ShaderBuffer>,
    #[storage(2, read_only)]
    sh: Handle<ShaderBuffer>,
    #[storage(3, read_only)]
    target: Handle<ShaderBuffer>,
    #[storage(4, read_only)]
    order: Handle<ShaderBuffer>,
}

impl Material for SplatMask {
    fn vertex_shader() -> ShaderRef {
        "embedded://raea_sys_splat/splat.wesl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "embedded://raea_sys_splat/splat.wesl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Premultiplied
    }
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        use bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState};
        descriptor.primitive.cull_mode = None;
        if let Some(d) = descriptor.depth_stencil.as_mut() {
            d.depth_write_enabled = Some(false);
        }
        if let Some(f) = descriptor.fragment.as_mut() {
            f.shader_defs.push("SPLAT_MASK".into());
            let erase = BlendComponent { src_factor: BlendFactor::Zero, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add };
            for t in f.targets.iter_mut().flatten() {
                t.blend = Some(BlendState { color: erase, alpha: erase });
            }
        }
        Ok(())
    }
}

// ---- clouds: a file read once, its splats in the world's frame

/// A splat row's entity: its material, cloud, morph target, and the last sort (the camera and effects it was for).
#[derive(Component)]
pub struct Splats {
    mat: Handle<SplatMaterial>,
    mask: Handle<SplatMask>,
    cloud: Arc<Cloud>,
    target: Option<Arc<Vec<[f32; 16]>>>,
    order: Handle<ShaderBuffer>,
    last: Option<(Mat4, Vec3, Vec3, SplatParams)>,
    keys: Vec<u32>,
}

fn s(v: &BTreeMap<String, V>, k: &str) -> Option<String> {
    match v.get(k) {
        Some(V::S(s)) => Some(s.clone()),
        _ => None,
    }
}

fn params(c: &Cloud, v: &BTreeMap<String, V>, bands: f32) -> SplatParams {
    let g = |k: &str, d: f32| num(v, k, d);
    let tint = if v.contains_key("tint_r") {
        Color::srgb(g("tint_r", 1.0), g("tint_g", 1.0), g("tint_b", 1.0))
    } else {
        colour(v.get("tint"), [1.0, 1.0, 1.0, 1.0])
    }
    .to_srgba();
    let glow = colour(v.get("glow"), [1.0, 0.92, 0.75, 1.0]).to_srgba();
    let rmode = match s(v, "reveal_mode").as_deref() {
        Some("plane") => 2,
        Some("radius") => 1,
        _ if v.contains_key("reveal") => 1,
        _ => 0,
    };
    SplatParams {
        sh_rot: c.rot.transpose(),
        center: c.center,
        radius: c.radius,
        count: c.n as u32,
        sh_deg: c.sh_deg,
        sh_words: c.sh_words,
        morph: g("morph", 0.0).clamp(0.0, 1.0),
        seed: g("seed", 0.0),
        twist: g("twist", 0.0),
        wave_amp: g("wave_amp", 0.0),
        wave_len: g("wave_len", 1.0),
        wave_phase: g("wave_phase", 0.0),
        explode: g("explode", 0.0).max(0.0),
        disperse: g("disperse", 0.0).clamp(0.0, 1.0),
        disperse_mode: if s(v, "disperse_mode").as_deref() == Some("dust") { 1 } else { 0 },
        wind: Vec3::new(g("wind_x", 1.0), g("wind_y", 0.15), g("wind_z", 0.0)),
        curl: g("curl", 1.0),
        reveal: g("reveal", 1.0).clamp(0.0, 1.0),
        reveal_mode: rmode,
        reveal_soft: g("reveal_soft", 0.15).max(1e-3),
        reveal_glow: g("reveal_glow", 0.0).max(0.0),
        reveal_origin: Vec3::new(g("reveal_x", c.center.x), g("reveal_y", c.center.y), g("reveal_z", c.center.z)),
        crop: g("crop", 0.0).max(0.0),
        reveal_axis: Vec3::new(g("reveal_ax", 0.0), g("reveal_ay", 1.0), g("reveal_az", 0.0)),
        bands: g("bands", bands).max(0.0),
        pulse: g("pulse", 0.0),
        pulse_scale: g("pulse_scale", 0.15),
        pulse_glow: g("pulse_glow", 0.0),
        splat_scale: g("splat_scale", 1.0).max(0.0),
        opacity: g("opacity", 1.0).clamp(0.0, 1.0),
        brightness: g("brightness", 1.0).max(0.0),
        saturation: g("saturation", 1.0).max(0.0),
        contrast: g("contrast", 1.0).max(0.0),
        tint: Vec4::new(tint.red, tint.green, tint.blue, 1.0),
        glow_color: Vec4::new(glow.red, glow.green, glow.blue, 1.0),
        paint: g("paint", 0.0).clamp(0.0, 1.0),
        spread: g("spread", 0.0).max(0.0),
    }
}

/// A splat row: its file read (once per file) and drawn as one mesh of a quad a splat, its morph target matched.
pub fn spawn(world: &mut World, e: Entity, t: &Tup, inserted: &[Tup], dir: &Path) -> Result<(), String> {
    let vals = &t.vals;
    let src = s(vals, "src").ok_or(format!("splat {}: needs src (a .ply, .splat or .spz file)", t.id))?;
    let c = cloud(&resolve_src(inserted, &src, dir)?)?;
    let target = match s(vals, "morph_to").filter(|s| !s.is_empty()) {
        Some(to) => {
            let b = cloud(&resolve_src(inserted, &to, dir)?)?;
            Some(Arc::new(matched(&c, &b, num(vals, "morph_crop", 0.0))))
        }
        None => None,
    };
    let n = c.n.max(1);
    // a quad a splat: its corners at (±1, ±1) and its place in the order in z
    let mut pos = Vec::with_capacity(n * 4);
    let mut idx = Vec::with_capacity(n * 6);
    for i in 0..n as u32 {
        let z = i as f32;
        pos.extend_from_slice(&[[-1.0, -1.0, z], [1.0, -1.0, z], [1.0, 1.0, z], [-1.0, 1.0, z]]);
        let b = i * 4;
        idx.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
    let mut m = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::RENDER_WORLD);
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    m.insert_indices(bevy::mesh::Indices::U32(idx));
    let mesh = world.resource_mut::<Assets<Mesh>>().add(m);
    let gpu = |w: &mut World, bytes: Vec<u8>, keep: bool| {
        let usage = if keep { bevy::asset::RenderAssetUsages::default() } else { bevy::asset::RenderAssetUsages::RENDER_WORLD };
        w.resource_mut::<Assets<ShaderBuffer>>().add(ShaderBuffer::new(bytes, usage))
    };
    let g = if c.n == 0 { vec![[0f32; 16]] } else { c.gauss.clone() };
    let splats = gpu(world, bytes_of(&g), false);
    let sh = gpu(world, words_of(&c.sh), false);
    let tgt = gpu(world, bytes_of(target.as_deref().map(|t| t.as_slice()).filter(|t| !t.is_empty()).unwrap_or(&[[0f32; 16]])), false);
    let order = gpu(world, words_of(&(0..n as u32).collect::<Vec<_>>()), true);
    let p0 = params(&c, vals, 0.0);
    let mask = world.resource_mut::<Assets<SplatMask>>().add(SplatMask { p: p0, splats: splats.clone(), sh: sh.clone(), target: tgt.clone(), order: order.clone() });
    let mat = world.resource_mut::<Assets<SplatMaterial>>().add(SplatMaterial { p: p0, splats, sh, target: tgt, order: order.clone() });
    // the eraser on the looks' layers, a child so it moves and hides with the splats
    let layers = bevy::camera::visibility::RenderLayers::from_layers(&(1..=30).collect::<Vec<_>>());
    let m2 = mesh.clone();
    let child = world.spawn((Mesh3d(m2), MeshMaterial3d(mask.clone()), Transform::default(), Visibility::Inherited, layers, NotShadowCaster, bevy::camera::visibility::NoFrustumCulling)).id();
    world.entity_mut(e).add_child(child);
    world.entity_mut(e).insert((
        Mesh3d(mesh),
        MeshMaterial3d(mat.clone()),
        transform_of(vals),
        NotShadowCaster,
        bevy::camera::visibility::NoFrustumCulling,
        Splats { mat, mask, cloud: c, target, order, last: None, keys: Vec::new() },
    ));
    Ok(())
}

/// Each splat row's effects from the frame's values, and its splats sorted for the frame's camera (after transforms
/// are propagated, so the camera and the splats are where this frame puts them). A frame whose camera, placement and
/// effects are the last sort's keeps that order (the same rows sort the same).
pub fn sync_splats(
    frame: Res<FrameRows>,
    mut mats: ResMut<Assets<SplatMaterial>>,
    mut masks: ResMut<Assets<SplatMask>>,
    mut bufs: ResMut<Assets<ShaderBuffer>>,
    cams: Query<(Entity, &GlobalTransform), With<Camera3d>>,
    mut q: Query<(Entity, &GlobalTransform, &mut Splats)>,
) {
    // the scene's camera: the one a row moves
    let Some((cam_e, cam)) = cams.iter().find(|(e, _)| frame.ix.contains_key(e)) else { return };
    let cv = frame.vals(cam_e);
    // a paint or cartoon <Look>: the splats' light in flat bands too (more of them than a surface's: a photograph's
    // light varies more within a part than a flat-coloured mesh's)
    let bands = match cv.as_ref().and_then(|v| s(v, "frame_look")).as_deref() {
        Some("paint") | Some("cartoon") => num(cv.as_ref().unwrap(), "paint_bands", 3.0) + 2.0,
        _ => 0.0,
    };
    let cam_pos = cam.translation();
    let fwd = cam.forward().as_vec3();
    for (e, gt, mut sp) in &mut q {
        let Some(v) = frame.vals(e) else { continue };
        let np = params(&sp.cloud, &v, bands);
        if mats.get(&sp.mat).is_some_and(|m| m.p != np) {
            if let Some(mut m) = mats.get_mut(&sp.mat) {
                m.p = np;
            }
            if let Some(mut m) = masks.get_mut(&sp.mask) {
                m.p = np;
            }
        }
        let model = gt.to_matrix();
        let key = (model, cam_pos, fwd, np);
        if sp.last.as_ref() == Some(&key) {
            continue;
        }
        let mut order = Vec::new();
        let sp = &mut *sp;
        sort(&sp.cloud, sp.target.as_deref().map(|t| t.as_slice()), &np, model, cam_pos, fwd, &mut order, &mut sp.keys);
        if order.is_empty() {
            order.push(0);
        }
        if let Some(mut b) = bufs.get_mut(&sp.order) {
            // Bevy 0.20: the data is a ShaderBufferData; the same bytes, the GPU buffer sized to them
            b.clear();
            b.extend_from_slice(&order);
        }
        sp.last = Some(key);
    }
}
