//! Screens (`<Screen>` in raea/3d): a live picture on a surface in the 3D scene, drawn as a CRT or a flat
//! panel. The picture is a nested composition or game (or a video in one) that the page lays out at its own size and
//! the host paints into a texture each frame (react's runtime/page/src/screen.rs, Dom::paint_node_texture); the host copies it
//! into the screen's image here (`Engine::screen_targets`) before the frame draws, so a render and a live play show the
//! same picture on the same frame. Nothing in here reads a clock: the flicker is seeded by the film frame.
//!
//! Where a screen goes:
//!   - a model's material by name: the model row's `screens` column, JSON `[{material, name, w, h, crt}]`; every mesh
//!     of the model whose glTF material has that name is drawn with the screen's material instead, the picture fitted
//!     to the mesh's texture coordinates (an affine map from its UVs to the extent of the mesh's face, faced along its
//!     mean normal, so a picture is upright whichever way the UVs were laid out). The face is the vertices within 30
//!     degrees of that normal: a tube's glass is a bulging face and a skirt folding back under the bezel at a crease,
//!     and the picture belongs on the face alone (fitted to the whole mesh, its edges ran onto the skirt and out of
//!     sight, and the skirt's UVs bent the fit by several percent); the skirt shows as bare glass;
//!   - a free-standing quad: a panel row with a `screen` column, JSON `{name, w, h, crt}`, its `w` x `h` (metres).
//!
//! A model row's `hide` column (JSON list of node names) hides those nodes (a robot's painted-on eyes under its screen).
//!
//! `crt` (all optional): curvature (barrel, 0 none), scanlines (0..1), mask (0..1), maskKind ("aperture", "shadow",
//! "none"), maskSize (mask triads per source pixel, or per line with `lines`, 1), lines (the tube's own scanlines top
//! to bottom, 0 by default: the picture's rows; set it for a sharp picture on a tube of a few hundred lines), glow
//! (phosphor halo, 0..1), convergence (texels), vignette
//! (0..1), flicker (0..1), brightness (the emitted light's multiplier, 2.5), glass (the tube's own colour, hex),
//! reflect (0..1, the glass's sheen), smudge (0..1, 0 clean: fingerprints, a wiped arc and dust specks on the glass,
//! seen mostly in its sheen; the pattern is fixed per screen, seeded by its name), overscan (0..0.3: the picture grown
//! past the glass's edge), power (0..1: the tube warming up, a line opening to the picture; keyed as the row's
//! `screen_power`). `crt: false` draws the picture flat and sharp (an LCD): unlit, at the page's own colours times the
//! spec's `brightness` (1), and with `alpha: true` (Screen's background see-through) blended by the page's alpha.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::texture::GpuImage;
use raea_core_program::eval::Tup;
use raea_core_program::ir::V;
use raea_sys_frame::*;
use bevy::light::NotShadowCaster;
use bevy::pbr::{Material, MaterialPlugin, MeshMaterial3d};
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

mod uv;
pub use uv::*;

/// screens' pictures from `screenpage` rows (raea's bevy_ui page)
pub mod screen_page;

pub fn install(app: &mut App) {
    screen_page::install(app);
    bevy::asset::embedded_asset!(app, "screen.wesl");
    app.add_plugins(MaterialPlugin::<CrtMaterial>::default())
        .init_resource::<ScreenImages>()
        .insert_resource(raea_sys_material::maps::NamedImages(named_image))
        .add_systems(Update, (model_screens, mesh_screens, hide_nodes, pose_nodes, sync_screens).chain().after(FrameSystems::Rows).before(FrameSystems::Tag));
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct CrtMaterial {
    #[uniform(0)]
    p: CrtParams,
    #[texture(1)]
    image: Handle<Image>,
    /// a see-through flat screen (its page's background isn't opaque: the spec's `alpha`): blended over what's behind
    /// it, where an opaque one is drawn opaque and writes depth
    blend: bool,
}

#[derive(ShaderType, Clone, Copy, Default, PartialEq, Debug)]
struct CrtParams {
    /// the texture coordinates to the picture's (x right, y down, 0..1): x = dot(map_a.xy, uv) + map_b.x,
    /// y = dot(map_a.zw, uv) + map_b.y
    map_a: Vec4,
    map_b: Vec4,
    /// source width, height (texels), the film frame, the power (0..1)
    res: Vec4,
    /// curvature, scanlines, mask, mask kind (0 none, 1 aperture grille, 2 shadow mask)
    a: Vec4,
    /// glow, convergence, vignette, flicker
    b: Vec4,
    /// brightness, reflect, overscan, crt on (1) or flat (0)
    c: Vec4,
    /// the glass's colour (linear) and the mask's triads per source pixel
    glass: Vec4,
    /// smudge (0..1), the smudge pattern's seed (from the screen's name), the tube's lines (0: the picture's rows), see-through (1: a flat
    /// screen drawn with its page's alpha)
    d: Vec4,
}

impl Material for CrtMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://raea_sys_screen/screen.wesl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        if self.blend { AlphaMode::Blend } else { AlphaMode::Opaque }
    }
    fn enable_shadows() -> bool {
        false
    }
}

/// Each screen's image by name, its size: what the host copies each frame's picture into.
#[derive(Resource, Default)]
pub struct ScreenImages(pub HashMap<String, (Handle<Image>, u32, u32)>);

/// A screen's material on an entity (a panel's, or a model's mesh), and the row whose `screen_power` and frame it takes.
#[derive(Component)]
pub struct Screen {
    mat: Handle<CrtMaterial>,
    row: Entity,
}

impl Screen {
    /// its material, which holds the screen's image
    pub fn material(&self) -> &Handle<CrtMaterial> {
        &self.mat
    }
}

impl CrtMaterial {
    /// the screen's picture
    pub fn image(&self) -> &Handle<Image> {
        &self.image
    }
}

/// A flat panel of a screen (a <Screen>'s quad, `spawn_panel`), whose texture coordinates are its picture's, so a
/// point on it is a point on the page it shows (a page layer in the world takes the pointer through it)
#[derive(Component)]
pub struct ScreenPanel;

/// A model whose screens and hidden nodes are set up (or that has none).
#[derive(Component)]
pub struct ScreensDone;
#[derive(Component)]
pub struct HideDone;

fn json(v: Option<&V>) -> Option<serde_json::Value> {
    match v {
        Some(V::S(s)) => serde_json::from_str(s).ok(),
        _ => None,
    }
}

fn f(o: &serde_json::Value, k: &str, d: f32) -> f32 {
    o.get(k).and_then(serde_json::Value::as_f64).map(|x| x as f32).unwrap_or(d)
}

/// A screen's spec: its name, source size, and the CRT's settings (`crt: false` flat).
fn params(spec: &serde_json::Value, map_a: Vec4, map_b: Vec4) -> CrtParams {
    let w = f(spec, "w", 320.0).max(1.0);
    let h = f(spec, "h", 200.0).max(1.0);
    let flat = spec.get("crt") == Some(&serde_json::Value::Bool(false));
    let none = serde_json::Value::Object(Default::default());
    let c = spec.get("crt").filter(|c| c.is_object()).unwrap_or(&none);
    let kind = match c.get("maskKind").and_then(serde_json::Value::as_str) {
        Some("none") => 0.0,
        Some("shadow") => 2.0,
        _ => 1.0,
    };
    let glass = c.get("glass").and_then(serde_json::Value::as_str).map(|h| colour(Some(&V::S(h.to_string())), [0.1, 0.11, 0.1, 1.0])).unwrap_or(Color::srgb(0.10, 0.115, 0.105)).to_linear();
    let z = |k: &str, d: f32| if flat { 0.0 } else { f(c, k, d) };
    CrtParams {
        map_a,
        map_b,
        res: Vec4::new(w, h, 0.0, 1.0),
        a: Vec4::new(z("curvature", 0.0), z("scanlines", 0.55), z("mask", 0.35), if flat { 0.0 } else { kind }),
        b: Vec4::new(z("glow", 0.35), z("convergence", 0.35), z("vignette", 0.35), z("flicker", 0.04)),
        // a flat screen's page at its own colours (1) unless its `brightness` says; a CRT's glow (2.5) from its settings
        c: Vec4::new(if flat { f(spec, "brightness", 1.0) } else { f(c, "brightness", 2.5) }, z("reflect", 0.5), f(c, "overscan", 0.0), if flat { 0.0 } else { 1.0 }),
        glass: Vec4::new(glass.red, glass.green, glass.blue, f(c, "maskSize", 1.0).max(0.1)),
        d: Vec4::new(f(c, "smudge", 0.0).clamp(0.0, 1.0), seed(spec), z("lines", 0.0).max(0.0), if see_through(spec) { 1.0 } else { 0.0 }),
    }
}

/// A flat screen whose page's background lets what's behind show (the spec's `alpha`, from Screen's background): a CRT
/// is a tube, opaque whatever its picture. */
fn see_through(spec: &serde_json::Value) -> bool {
    spec.get("crt") == Some(&serde_json::Value::Bool(false)) && spec.get("alpha") == Some(&serde_json::Value::Bool(true))
}

/// A screen's smudge seed from its name (FNV-1a), 0..1024: the same glass looks the same every frame and every render.
fn seed(spec: &serde_json::Value) -> f32 {
    let name = spec.get("name").and_then(serde_json::Value::as_str).unwrap_or("");
    let h = name.bytes().fold(0x811c9dc5u32, |h, b| (h ^ b as u32).wrapping_mul(0x01000193));
    (h % 1024) as f32
}

/// The screen image a material's texture names (`raea-screen:name@WIDTHxHEIGHT`, the SDK's targetImage: Bevy's
/// render_ui_to_texture, a UiTexture's UI drawn into it as a <Screen>'s picture is), made if no screen has it yet; None
/// for any other source, or a World without screens.
pub fn named_image(world: &mut World, src: &str) -> Option<Handle<Image>> {
    let (name, size) = src.strip_prefix("raea-screen:")?.rsplit_once('@')?;
    let (w, h) = size.split_once('x')?;
    let (w, h) = (w.parse::<u32>().ok()?.max(1), h.parse::<u32>().ok()?.max(1));
    world.contains_resource::<ScreenImages>().then(|| image(world, name, w, h))
}

/// A screen's image: black, the source's size, filled each frame by the host's copy.
fn image(world: &mut World, name: &str, w: u32, h: u32) -> Handle<Image> {
    if let Some((hd, iw, ih)) = world.resource::<ScreenImages>().0.get(name) {
        if (*iw, *ih) == (w, h) {
            return hd.clone();
        }
    }
    let mut img = Image::new_fill(
        bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    img.texture_descriptor.usage |= TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
    let hd = world.resource_mut::<Assets<Image>>().add(img);
    world.resource_mut::<ScreenImages>().0.insert(name.to_string(), (hd.clone(), w, h));
    hd
}

fn spec_size(spec: &serde_json::Value) -> (u32, u32) {
    (f(spec, "w", 320.0).max(1.0) as u32, f(spec, "h", 200.0).max(1.0) as u32)
}

/// A panel row with a `screen` column: a w x h quad (metres) showing the screen, its texture coordinates the picture's.
pub fn spawn_panel(world: &mut World, e: Entity, t: &Tup) -> Result<(), String> {
    let vals = &t.vals;
    let spec = json(vals.get("screen")).ok_or_else(|| format!("panel {}: its screen isn't JSON", t.id))?;
    let name = spec.get("name").and_then(serde_json::Value::as_str).unwrap_or(&t.id).to_string();
    let (w, h) = spec_size(&spec);
    let img = image(world, &name, w, h);
    let mut p = params(&spec, Vec4::new(1.0, 0.0, 0.0, 1.0), Vec4::ZERO);
    p.res.w = num(vals, "screen_power", 1.0);
    let mat = world.resource_mut::<Assets<CrtMaterial>>().add(CrtMaterial { p, image: img, blend: see_through(&spec) });
    let (qw, qh) = (num(vals, "w", 1.0), num(vals, "h", num(vals, "w", 1.0) * h as f32 / w as f32));
    let mesh = world.resource_mut::<Assets<Mesh>>().add(Mesh::from(Rectangle::new(qw, qh)));
    world.entity_mut(e).insert((Mesh3d(mesh), MeshMaterial3d(mat.clone()), transform_of(vals), NotShadowCaster, Screen { mat, row: e }, ScreenPanel));
    Ok(())
}

/// Each model's `hide` nodes hidden, once it's loaded.
/// Each model's <Model nodes> local transforms set on its glTF nodes, by name, every frame (what a row doesn't give
/// stays as the node has it).
fn pose_nodes(frame: Res<FrameRows>, models: Query<Entity, With<ModelReady>>, kids: Query<&Children>, mut nodes: Query<(&Name, &mut Transform)>) {
    for e in &models {
        if !frame.may_have("node0") {
            return;
        }
        let Some(v) = frame.vals(e) else { continue };
        let mut i = 0;
        while let Some(V::S(name)) = v.get(&format!("node{i}")) {
            let col = |c: &str| { let k = format!("node{i}_{c}"); v.contains_key(&k).then(|| num(&v, &k, 0.0)) };
            for d in descendants(e, &kids) {
                let Ok((n, mut t)) = nodes.get_mut(d) else { continue };
                if n.as_str() != name {
                    continue;
                }
                if let (Some(x), Some(y), Some(z)) = (col("x"), col("y"), col("z")) {
                    t.translation = Vec3::new(x, y, z);
                }
                if let (Some(x), Some(y), Some(z), Some(w)) = (col("q_x"), col("q_y"), col("q_z"), col("q_w")) {
                    t.rotation = Quat::from_xyzw(x, y, z, w);
                }
                if let (Some(x), Some(y), Some(z)) = (col("s_x"), col("s_y"), col("s_z")) {
                    t.scale = Vec3::new(x, y, z);
                }
            }
            i += 1;
        }
    }
}

fn hide_nodes(mut commands: Commands, frame: Res<FrameRows>, models: Query<Entity, (With<ModelReady>, Without<HideDone>)>, kids: Query<&Children>, names: Query<&Name>) {
    for e in &models {
        commands.entity(e).insert(HideDone);
        let Some(v) = frame.vals(e) else { continue };
        let Some(serde_json::Value::Array(list)) = json(v.get("hide")) else { continue };
        let want: Vec<&str> = list.iter().filter_map(serde_json::Value::as_str).collect();
        for d in descendants(e, &kids) {
            if names.get(d).is_ok_and(|n| want.contains(&n.as_str())) {
                commands.entity(d).insert(Visibility::Hidden);
            }
        }
    }
}

/// Each screen's frame (its flicker's seed), power and, for a flat one, brightness, from the clock and its row: a
/// screen's brightness can move over a shot (a phone dimmed under words) without the screen being made again.
fn sync_screens(clock: Res<Clock>, frame: Res<FrameRows>, mut mats: ResMut<Assets<CrtMaterial>>, q: Query<&Screen>) {
    for s in &q {
        let vals = frame.vals(s.row);
        let power = vals.as_ref().map(|v| num(v, "screen_power", 1.0)).unwrap_or(1.0);
        let Some(m) = mats.get(&s.mat) else { continue };
        // (raea-runtime-scene writes `screen_bright` each frame for a flat screen given a brightness)
        let bright = if m.p.c.w == 0.0 { vals.as_ref().map(|v| num(v, "screen_bright", m.p.c.x)).unwrap_or(m.p.c.x) } else { m.p.c.x };
        let want = (clock.0 as f32, power, bright);
        if (m.p.res.z, m.p.res.w, m.p.c.x) != want {
            if let Some(mut m) = mats.get_mut(&s.mat) {
                m.p.res.z = want.0;
                m.p.res.w = want.1;
                m.p.c.x = want.2;
            }
        }
    }
}

/// Every screen's texture on the paint device by name, with its size: where the host copies each frame's picture
/// before `draw`. A screen whose image the GPU doesn't hold yet (the frame it first appears) is left out.
pub fn screen_targets(app: &App) -> Vec<(String, wgpu::Texture, u32, u32)> {
    let Some(rw) = app.get_sub_app(bevy::render::RenderApp).map(|a| a.world()) else { return Vec::new() };
    let Some(gpu) = rw.get_resource::<RenderAssets<GpuImage>>() else { return Vec::new() };
    let Some(si) = app.world().get_resource::<ScreenImages>() else { return Vec::new() };
    si.0.iter().filter_map(|(name, (hd, w, h))| gpu.get(hd).map(|g| (name.clone(), (*g.texture).clone(), *w, *h))).collect()
}

/// The screen under a point of the frame (`nx`, `ny`: 0..1 across and down it), seen from the drawing camera: its
/// name and where the point lands on its picture (0..1 across and down), the nearest along the ray. Every screen's
/// mesh is tested, triangle by triangle, its texture coordinates mapped to the picture's as its material maps them
/// (uv_map); other geometry in front of a screen isn't tested. A game on a screen takes clicks this way (react's
/// runtime/page/src/screen.rs routes them into an app screen's document).
pub fn screen_at(app: &mut App, nx: f64, ny: f64) -> Option<(String, f64, f64)> {
    use bevy::mesh::VertexAttributeValues as A;
    let world = app.world_mut();
    let mut cams = world.query_filtered::<(&Camera, &GlobalTransform), (With<Camera3d>, Without<LookCam>)>();
    let (cam, cgt) = cams.iter(world).filter(|c| c.0.is_active).max_by_key(|c| c.0.order)?;
    let ndc = |z: f32| cam.ndc_to_world(cgt, Vec3::new(nx as f32 * 2.0 - 1.0, 1.0 - ny as f32 * 2.0, z));
    let (near, far) = (ndc(1.0)?, ndc(0.01)?);
    let dir = (far - near).normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }
    let names: HashMap<bevy::asset::AssetId<Image>, String> = world.resource::<ScreenImages>().0.iter().map(|(n, (hd, _, _))| (hd.id(), n.clone())).collect();
    let mut q = world.query::<(&Screen, &Mesh3d, &GlobalTransform, Option<&InheritedVisibility>)>();
    let (meshes, mats) = (world.resource::<Assets<Mesh>>(), world.resource::<Assets<CrtMaterial>>());
    let mut best: Option<(f32, String, f64, f64)> = None;
    for (s, mesh, gt, vis) in q.iter(world) {
        if vis.is_some_and(|v| !v.get()) {
            continue;
        }
        let (Some(mesh), Some(mat)) = (meshes.get(&mesh.0), mats.get(&s.mat)) else { continue };
        let Some(name) = names.get(&mat.image.id()) else { continue };
        let (Some(A::Float32x3(pos)), Some(A::Float32x2(uv))) = (mesh.attribute(Mesh::ATTRIBUTE_POSITION), mesh.attribute(Mesh::ATTRIBUTE_UV_0)) else { continue };
        // the ray in the mesh's own space, where its triangles are
        let inv = gt.affine().inverse();
        let (o, d) = (inv.transform_point3(near), inv.transform_vector3(dir));
        for [i0, i1, i2] in triangles(mesh, pos.len()) {
            let (Some(p0), Some(p1), Some(p2)) = (pos.get(i0), pos.get(i1), pos.get(i2)) else { continue };
            let (p0, p1, p2) = (Vec3::from(*p0), Vec3::from(*p1), Vec3::from(*p2));
            // Möller-Trumbore, both faces
            let (e1, e2) = (p1 - p0, p2 - p0);
            let pv = d.cross(e2);
            let det = e1.dot(pv);
            if det.abs() < 1e-12 {
                continue;
            }
            let tv = o - p0;
            let u = tv.dot(pv) / det;
            let qv = tv.cross(e1);
            let v = d.dot(qv) / det;
            let t = e2.dot(qv) / det;
            if u < 0.0 || v < 0.0 || u + v > 1.0 || t <= 0.0 {
                continue;
            }
            // the distance in the world's units, comparable across meshes
            let dist = gt.transform_point(o + d * t).distance(near);
            if best.as_ref().is_some_and(|b| b.0 <= dist) {
                continue;
            }
            let at = |k: usize| Vec2::from(uv[k]);
            let tex = at(i0) * (1.0 - u - v) + at(i1) * u + at(i2) * v;
            let (a, b) = (mat.p.map_a, mat.p.map_b);
            let s = Vec2::new(a.x * tex.x + a.y * tex.y + b.x, a.z * tex.x + a.w * tex.y + b.y);
            // where the tube draws that point of the glass from, as screen.wesl does: overscan, then the barrel,
            // then the warming-up line (curvature 0.2 moved a corner click by tens of the picture's pixels)
            let p = &mat.p;
            let mut q = (s * 2.0 - 1.0) / (1.0 + p.c.z);
            q *= 1.0 + p.a.x * q.dot(q) * 0.25;
            let power = p.res.w.clamp(0.0, 1.0);
            if p.c.w > 0.5 && power < 1.0 {
                q.y /= (power * power * 1.05 - 0.05).max(0.004);
            }
            let s = q * 0.5 + 0.5;
            // on the glass around the picture: the screen's, though nothing of the picture is there
            let on = (0.0..=1.0).contains(&s.x) && (0.0..=1.0).contains(&s.y);
            best = Some((dist, if on { name.clone() } else { String::new() }, s.x as f64, s.y as f64));
        }
    }
    best.filter(|b| !b.1.is_empty()).map(|b| (b.1, b.2, b.3))
}

/// Screen `name`'s picture for the frame about to draw, copied from `src` (a texture on the paint device, which
/// the engine shares: Dom::paint_node_texture's), its top-left corner, as much as both hold. False when there's no
/// such screen, or the GPU doesn't hold its image yet (its first frame).
pub fn copy_screen(app: &App, name: &str, src: &wgpu::Texture) -> bool {
    let Some((_, dst, w, h)) = screen_targets(app).into_iter().find(|t| t.0 == name) else { return false };
    let Some(rw) = app.get_sub_app(bevy::render::RenderApp).map(|a| a.world()) else { return false };
    let device = rw.resource::<bevy::render::renderer::RenderDevice>().wgpu_device();
    let queue = rw.resource::<bevy::render::renderer::RenderQueue>();
    let (cw, ch) = (w.min(src.width()), h.min(src.height()));
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("screen copy") });
    enc.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo { texture: src, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyTextureInfo { texture: &dst, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::Extent3d { width: cw, height: ch, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    true
}
