//! The paint look's surfaces (a <Look mode="paint"> or "cartoon": frame_look 'paint' on the camera row): every
//! standard material in the scene drawn as paint.wesl paints it, its suns' light in a few flat tones (`paint_bands`):
//! lit, its own colour; turned from the sun, times `paint_mid`; in a cast shadow (cut hard), times `paint_shadow`. No
//! specular or ambient; a point or spot light only when its row asks (`paint`: a <PointLight paint>), in a band of its
//! own colour where it reaches (paint_lights.wesl, `PaintLights`). The frame's ragged edges, grain, sky and palette
//! are the look's composite (look.rs, mode 6).
//!
//! Each mesh keeps its standard material (`Painted::std`) for the rows to animate (sync_meshes writes it, as for any
//! mesh), and its paint copy follows it here. Models with a look of their own (`look`, `flat`) keep theirs. A model's
//! meshes (no rows of their own, so the camera's paint unchanged) that share a standard material share one paint copy
//! too, so Bevy still batches them: a town of 20k pack pieces on a few dozen materials draws a few dozen paints.
//!
//! Columns on the camera row: paint_bands (keyed), paint_shadow and paint_mid (hex, the first frame's).

use super::*;
use std::collections::HashSet;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

pub type PaintMaterial = ExtendedMaterial<StandardMaterial, Paint>;

pub fn install(app: &mut App) {
    bevy::asset::embedded_asset!(app, "paint.wesl");
    bevy::asset::embedded_asset!(app, "paint_vertex.wesl");
    bevy::shader::load_shader_library!(app, "paint_lights.wesl");
    app.add_plugins(MaterialPlugin::<PaintMaterial>::default());
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct Paint {
    #[uniform(100)]
    p: PaintParams,
    #[uniform(101)]
    lights: PaintLights,
}

/// The point and spot lights that ask for paint (a light row's `paint`), as paint_lights.wesl finds them in the
/// fragment's cluster: by where they are. None (the default): the paint is as it was without them.
#[derive(ShaderType, Clone, Copy, Default, PartialEq)]
pub struct PaintLights {
    pos: [Vec4; 8],
    /// how many, the band's lux, the hot band's multiple of it, the hot band's brightness
    k: Vec4,
}

/// a quarter of a DirectionalLight of 1 (8000 lux): a PointLight of intensity I bands what faces it within about
/// 6 * sqrt(I) metres
const LIGHT_BAND_LUX: f32 = 2000.0;

/// this frame's asking lights, at most 8 (in a fixed order, so an unchanged set compares equal)
fn paint_lights(frame: &FrameRows, lights: &Query<Entity, Or<(With<PointLight>, With<SpotLight>)>>) -> PaintLights {
    let mut at: Vec<[f32; 3]> = Vec::new();
    for e in lights {
        let Some(v) = frame.vals(e) else { continue };
        if v.get("paint").is_some_and(V::truthy) && num(&v, "lumens", 1.0e6) > 0.0 {
            at.push([num(&v, "x", 0.0), num(&v, "y", 0.0), num(&v, "z", 0.0)]);
        }
    }
    if at.is_empty() {
        return PaintLights::default();
    }
    at.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    at.truncate(8);
    let mut l = PaintLights { k: Vec4::new(at.len() as f32, LIGHT_BAND_LUX, 4.0, 1.25), ..default() };
    for (i, p) in at.iter().enumerate() {
        l.pos[i] = Vec3::from_array(*p).extend(0.0);
    }
    l
}

#[derive(ShaderType, Clone, Copy, Default, PartialEq)]
pub struct PaintParams {
    shadow: Vec4,
    mid: Vec4,
    /// the ambient-occlusion band's tint (a: on)
    ao: Vec4,
    /// foam's colour where surfaces meet the water (a: on)
    foam: Vec4,
    /// the mesh's breakup patches' colour (a: how strongly they tint it)
    patch: Vec4,
    /// bands, ao height, ground y, water y
    k0: Vec4,
    /// foam height, ssao threshold, shadow-edge raggedness, breakup amount
    k1: Vec4,
    /// breakup scale, edge scale, wash, wash patch size (uv)
    k2: Vec4,
    /// stroke amount, stroke length (m), saturation, foliage
    k3: Vec4,
    /// foliage height (m), calm from (m), calm to (m), calm amount
    k4: Vec4,
    /// the colour the low ground settles toward with distance (a: on)
    calm: Vec4,
    /// texel (m; 0: none), each cell's shade jitter, cloud shadows' drift (x, z: m)
    k5: Vec4,
    /// cloud shadows' cover (0: none), their size (m), this material sways (1: cut-out, set per mesh), -
    k6: Vec4,
    /// wind: a blade's tip's bend (m; 0: none), its direction (x, z), the time it has blown (s)
    k7: Vec4,
    /// haze: how thick (per m), how fast it thins with height (m), where it starts (m), its most (0 to 1)
    k8: Vec4,
    /// the haze's colour (a: on), and its colour toward the sun (a: on)
    haze: Vec4,
    haze_sun: Vec4,
}

impl MaterialExtension for Paint {
    fn fragment_shader() -> ShaderRef {
        "embedded://raea_sys_look/paint.wesl".into()
    }
    // the wind bends cut-out foliage in the vertex stage, in the main pass and the prepass alike (depth, normals and
    // shadow maps), so outlines and cast shadows move with it
    fn vertex_shader() -> ShaderRef {
        "embedded://raea_sys_look/paint_vertex.wesl".into()
    }
    fn prepass_vertex_shader() -> ShaderRef {
        "embedded://raea_sys_look/paint_vertex.wesl".into()
    }
}

/// a mesh's paint with its material's own part: whether it sways in the wind (a cut-out: masked, blended or to
/// coverage)
fn for_material(mut p: PaintParams, m: &StandardMaterial) -> PaintParams {
    p.k6.z = if matches!(m.alpha_mode, AlphaMode::Opaque) { 0.0 } else { 1.0 };
    p
}

/// A painted mesh: the standard material the rows animate, the paint copy drawn, and the composition pass it was
/// painted for (its MeshTag's, compose.rs).
#[derive(Component)]
pub struct Painted {
    pub std: Handle<StandardMaterial>,
    paint: Handle<PaintMaterial>,
    pass: u32,
}

/// The camera's paint as one composition pass has it (the camera row's look_passes: compose.rs): its own bands, torn
/// shadow edge, strokes, saturation and wash.
fn pass_params(mut p: PaintParams, own: Option<&serde_json::Value>) -> PaintParams {
    let Some(o) = own else { return p };
    let f = |k: &str| o.get(k).and_then(serde_json::Value::as_f64).map(|x| x as f32);
    if let Some(b) = f("bands") {
        p.k0.x = b.clamp(1.0, 8.0);
    }
    if let Some(e) = f("edge") {
        p.k1.z = e;
    }
    if let Some(w) = f("wash") {
        p.k2.z = w.clamp(0.0, 1.0);
    }
    if let Some(s) = f("stroke") {
        p.k3.x = s.clamp(0.0, 1.0);
    }
    if let Some(s) = f("saturate") {
        p.k3.z = s;
    }
    p
}

fn lin(v: Option<&V>, or: [f32; 4]) -> Vec4 {
    Vec4::from_array(colour(v, or).to_linear().to_f32_array())
}

/// the camera row's paint (every surface's)
fn params(v: &BTreeMap<String, V>) -> PaintParams {
    let on = |k: &str| if v.contains_key(k) { 1.0f32 } else { 0.0 };
    let mut ao = lin(v.get("paint_ao"), [0.7, 0.68, 0.72, 1.0]);
    ao.w = on("paint_ao_height").max(on("paint_ssao"));
    let mut foam = lin(v.get("paint_foam_color"), [1.0, 1.0, 1.0, 1.0]);
    foam.w = on("paint_water");
    let mut calm = lin(v.get("paint_calm"), [0.0, 0.0, 0.0, 1.0]);
    calm.w = on("paint_calm");
    let mut haze = lin(v.get("paint_haze"), [1.0, 1.0, 1.0, 1.0]);
    haze.w = on("paint_haze");
    let mut haze_sun = lin(v.get("paint_haze_sun"), [1.0, 0.9, 0.75, 1.0]);
    haze_sun.w = on("paint_haze_sun");
    PaintParams {
        shadow: lin(v.get("paint_shadow"), [0.55, 0.58, 0.68, 1.0]),
        mid: lin(v.get("paint_mid"), [0.80, 0.76, 0.78, 1.0]),
        ao,
        foam,
        patch: Vec4::ZERO,
        k0: Vec4::new(num(v, "paint_bands", 3.0).clamp(1.0, 8.0), num(v, "paint_ao_height", 0.0), num(v, "paint_ground", 0.0), num(v, "paint_water", 0.0)),
        k1: Vec4::new(num(v, "paint_foam", 0.25), num(v, "paint_ssao", 0.0), num(v, "paint_edge", 0.0), 0.0),
        k2: Vec4::new(1.0, num(v, "paint_edge_scale", 1.5).max(0.01), num(v, "paint_wash", 0.0).clamp(0.0, 1.0), num(v, "paint_wash_size", 0.05).max(0.0)),
        k3: Vec4::new(num(v, "paint_stroke", 0.0).clamp(0.0, 1.0), num(v, "paint_stroke_scale", 0.08).max(0.001), num(v, "paint_saturate", 1.0), num(v, "paint_foliage", 0.0).clamp(0.0, 1.0)),
        k4: Vec4::new(num(v, "paint_foliage_height", 0.6), num(v, "paint_calm_from", 8.0), num(v, "paint_calm_to", 40.0).max(num(v, "paint_calm_from", 8.0) + 0.01), num(v, "paint_calm_amount", 0.85).clamp(0.0, 1.0)),
        calm,
        k5: Vec4::new(num(v, "paint_texel", 0.0).max(0.0), num(v, "paint_texel_jitter", 0.08).clamp(0.0, 0.5), num(v, "paint_cloud_shadow_x", 0.0), num(v, "paint_cloud_shadow_z", 0.0)),
        k6: Vec4::new(num(v, "paint_cloud_shadows", 0.0).clamp(0.0, 1.0), num(v, "paint_cloud_shadow_size", 40.0).max(1.0), 0.0, 0.0),
        k7: Vec4::new(num(v, "paint_wind", 0.0).max(0.0), num(v, "paint_wind_x", 1.0), num(v, "paint_wind_z", 0.0), num(v, "paint_wind_time", 0.0)),
        k8: Vec4::new(num(v, "paint_haze_density", 0.02).max(0.0), num(v, "paint_haze_height", 3.0).max(0.01), num(v, "paint_haze_start", 8.0).max(0.0), num(v, "paint_haze_most", 0.7).clamp(0.0, 1.0)),
        haze,
        haze_sun,
    }
}

/// one mesh's own paint on top of the camera's: its breakup (paint_breakup, paint_breakup_scale, paint_patch)
fn mesh_params(mut p: PaintParams, v: Option<&BTreeMap<String, V>>) -> PaintParams {
    let Some(v) = v else { return p };
    if v.contains_key("paint_breakup") {
        p.k1.w = num(v, "paint_breakup", 0.0);
        p.k2.x = num(v, "paint_breakup_scale", 1.0).max(0.001);
        p.patch = lin(v.get("paint_patch"), [0.0, 0.0, 0.0, 0.0]);
        p.patch.w = if v.contains_key("paint_patch") { num(v, "paint_patch_amount", 0.6) } else { 0.0 };
    }
    p
}

/// Once the camera's row asks for paint: each standard-material mesh (but a looked or flat model's) swapped onto a
/// paint copy, as meshes appear; then each frame the copies follow their standard materials and the row's bands.
#[allow(clippy::type_complexity)]
pub fn sync_paint(
    mut commands: Commands,
    frame: Res<FrameRows>,
    cams: Query<Entity, (With<Camera3d>, Without<LookCam>)>,
    fresh: Query<(Entity, &MeshMaterial3d<StandardMaterial>), (With<Mesh3d>, Without<Painted>)>,
    painted: Query<(Entity, &Painted)>,
    models: Query<Entity, With<Model>>,
    kids: Query<&Children>,
    stds: Res<Assets<StandardMaterial>>,
    mut paints: ResMut<Assets<PaintMaterial>>,
    mut last: Local<Option<PaintParams>>,
    mut shared: Local<HashMap<(AssetId<StandardMaterial>, u32), Handle<PaintMaterial>>>,
    tags: Query<&bevy::mesh::MeshTag>,
    lamps: Query<Entity, Or<(With<PointLight>, With<SpotLight>)>>,
    mut last_lights: Local<PaintLights>,
) {
    let Ok(cam) = cams.single() else { return };
    let Some(v) = frame.vals(cam) else { return };
    if !matches!(v.get("frame_look"), Some(V::S(s)) if s == "paint") {
        // a scene without paint after one with it: each painted mesh back on its standard material
        if last.is_some() {
            for (e, pm) in &painted {
                commands.entity(e).remove::<MeshMaterial3d<PaintMaterial>>().insert(MeshMaterial3d(pm.std.clone()));
            }
            *last = None;
        }
        return;
    }
    let p = params(&v);
    // composition passes (compose.rs): those the look skips draw their own materials, the rest each its own paint
    let skip = super::compose::bits(&v, "look_skip");
    let own = super::compose::per_pass(&v, "look_passes");
    let pass = |e: Entity| tags.get(e).map_or(0, |t| t.value & 3);
    let skipped = |e: Entity| (skip >> pass(e)) & 1 == 1;
    let p_of = |e: Entity| pass_params(p, own[pass(e) as usize].as_ref());
    let lights = paint_lights(&frame, &lamps);
    // and on again when paint comes back (bands_changed below brings their parameters up to date)
    if last.is_none() {
        for (e, pm) in &painted {
            commands.entity(e).remove::<MeshMaterial3d<StandardMaterial>>().insert(MeshMaterial3d(pm.paint.clone()));
        }
    }
    // a model with its own look or flat surfaces keeps them
    let mut keep: HashSet<Entity> = HashSet::new();
    for m in &models {
        if let Some(mv) = frame.vals(m) {
            if mv.contains_key("look") || mv.get("flat").is_some_and(V::truthy) {
                keep.extend(descendants(m, &kids));
            }
        }
    }
    // a shared copy goes with its standard material (a scene unloaded)
    shared.retain(|(id, _), _| stds.contains(*id));
    for (e, s) in &fresh {
        if keep.contains(&e) || skipped(e) {
            continue;
        }
        let own = frame.vals(e);
        let ps = pass(e);
        // a mesh without a row (a model's) is painted as the camera says, so its material's copy is everyone's in its pass
        if own.is_none() {
            if let Some(paint) = shared.get(&(s.0.id(), ps)) {
                commands.entity(e).remove::<MeshMaterial3d<StandardMaterial>>().insert((MeshMaterial3d(paint.clone()), Painted { std: s.0.clone(), paint: paint.clone(), pass: ps }));
                continue;
            }
        }
        let Some(base) = stds.get(&s.0).cloned() else { continue };
        let paint = paints.add(PaintMaterial { extension: Paint { p: for_material(mesh_params(p_of(e), own.as_deref()), &base), lights }, base });
        if own.is_none() {
            shared.insert((s.0.id(), ps), paint.clone());
        }
        commands.entity(e).remove::<MeshMaterial3d<StandardMaterial>>().insert((MeshMaterial3d(paint.clone()), Painted { std: s.0.clone(), paint, pass: ps }));
    }
    // a painted mesh whose pass the look now skips, or that moved to another pass: back on its standard material
    // (painted again for its new pass, above, on the next frame)
    for (e, pm) in &painted {
        if pm.pass != pass(e) || skipped(e) {
            commands.entity(e).remove::<(MeshMaterial3d<PaintMaterial>, Painted)>().insert(MeshMaterial3d(pm.std.clone()));
        }
    }
    let bands_changed = *last != Some(p) || *last_lights != lights;
    *last = Some(p);
    *last_lights = lights;
    // each copy once, however many meshes draw it
    let mut seen: HashSet<AssetId<PaintMaterial>> = HashSet::new();
    for (e, pm) in &painted {
        if !seen.insert(pm.paint.id()) {
            continue;
        }
        let Some(std) = stds.get(&pm.std) else { continue };
        let Some(cur) = paints.get(&pm.paint) else { continue };
        let moved = cur.base.base_color != std.base_color || cur.base.emissive != std.emissive || cur.base.alpha_mode != std.alpha_mode;
        if !moved && !bands_changed {
            continue;
        }
        let p = for_material(mesh_params(p_of(e), frame.vals(e).as_deref()), std);
        let std = std.clone();
        if let Some(mut m) = paints.get_mut(&pm.paint) {
            if moved {
                m.base = std;
            }
            m.extension.p = p;
            m.extension.lights = lights;
        }
    }
}
