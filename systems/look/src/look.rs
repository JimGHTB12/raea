//! A model's look: the model drawn at one texel a cell and laid back over the frame cell by cell (look.wesl), so a
//! character can be pixel art or raea.io's dithered grain while everything else in the scene draws as it is.
//!
//! Columns on a model row (all optional; without `look` the model draws as before):
//!   look (0 solid, 1 pixel, 2 site; keyed, so a model changes look mid-piece), look_px (a cell's size in the frame's
//!   pixels, 6 by default: the site's), look_edge (the pixel look's outline colour, '' for none), look_theme ('dark',
//!   the default, or 'light': the site look's greys), look_lift look_gain (the site look: ink = lift + light * gain, on light greys lift + (1 - light) * gain),
//!   flat (1: the model unlit, its shape and colours alone, in every look but site)
//!
//! Each looked model gets its own render layer, which its meshes move onto while it isn't solid, and a camera on that
//! layer that follows the scene's camera into a target a cell per texel, cleared see-through. One composite camera
//! after the scene's draws every looked model's cells over the frame (a full-frame quad each, nearest the camera
//! last), before the page goes on top. The lights shine on every layer, so a looked model is lit as it was and still
//! casts its shadow on the scene. Parts of the scene in front of a looked model don't hide it.

use super::*;
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::pbr::{Material, MaterialPlugin, MeshMaterial3d};
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

mod frame_look;
pub use frame_look::*;

/// The composite camera's layer, past any look's.
const COMPOSITE: usize = 31;

pub fn install(app: &mut App) {
    bevy::asset::embedded_asset!(app, "look.wesl");
    app.add_plugins(MaterialPlugin::<LookMaterial>::default());
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct LookMaterial {
    #[uniform(0)]
    p: LookParams,
    #[texture(1)]
    #[sampler(2)]
    cells: Handle<Image>,
    /// the composition passes' mask (compose.rs), read by the frame's looks while `masked`
    #[texture(3)]
    #[sampler(4)]
    mask: Handle<Image>,
}

#[derive(ShaderType, Clone, Copy, Default, PartialEq)]
struct LookParams {
    mode: u32,
    dark: u32,
    size: Vec2,
    edge: Vec4,
    lift: f32,
    gain: f32,
    /// the frame look: how much of a cell's hue its grey keeps
    color: f32,
    /// the ink look's paper (its ink is `edge`) and how big a step in light draws a line (0: none)
    paper: Vec4,
    line: f32,
    /// the ink look's split between ink and paper, and how wide a band of light either side of it is dithered
    threshold: f32,
    band: f32,
    /// the paint look (mode 6): paper grain (0 none), how far edges wander in pixels and the size of their wander,
    /// the wander's seed (a boiling line's changes on its beat), the sky where nothing's drawn (top and bottom; alpha
    /// 0, none: the scene's own background), and a palette every pixel snaps to (`npal` colours, 0 for none)
    grain: f32,
    rag: f32,
    rag_scale: f32,
    seed: f32,
    sky_top: Vec4,
    sky_bottom: Vec4,
    npal: u32,
    /// paint drawn a cell per `frame_look_px` pixels (pixel art): each cell read whole, not wandered or blended with
    /// its neighbours, and its grain a cell's
    crisp: u32,
    pal: [Vec4; 16],
    /// the frame's looks over composition passes (compose.rs's mask, 1: on), and each pass's own (x "scene", y z w
    /// passes 1 to 3): drawn as it is (1, a <Look skip>), the paint's rag, grain and palette (1: snapped)
    masked: u32,
    pass_raw: Vec4,
    pass_rag: Vec4,
    pass_grain: Vec4,
    pass_snap: Vec4,
}

impl Material for LookMaterial {
    fn vertex_shader() -> ShaderRef {
        "embedded://raea_sys_look/look.wesl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "embedded://raea_sys_look/look.wesl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        // the frame's own cells (modes 3 and 4) cover it; a model's are laid over them
        if self.p.mode >= 3 { AlphaMode::Opaque } else { AlphaMode::Blend }
    }
    fn enable_prepass() -> bool {
        false
    }
    fn enable_shadows() -> bool {
        false
    }
}

/// A looked model's layer, its cell camera and its quad on the composite.
#[derive(Component)]
pub struct Look {
    layer: usize,
    cam: Entity,
    quad: Entity,
    mat: Handle<LookMaterial>,
}

fn mode_of(v: &BTreeMap<String, V>) -> u32 {
    match v.get("look") {
        Some(V::S(s)) => match s.as_str() {
            "pixel" => 1,
            "site" | "dither" => 2,
            _ => 0,
        },
        Some(x) => x.num().clamp(0.0, 2.0) as u32,
        None => 0,
    }
}

/// The frame look's state: its quad and material on the composite, the cells the scene's camera draws into while a
/// look is on, the target it draws into otherwise, and the camera's own tone mapping and clear colour to go back to.
#[derive(Default)]
pub struct FrameLookState {
    mat: Option<Handle<LookMaterial>>,
    quad: Option<Entity>,
    cells: Option<(Handle<Image>, u32, u32)>,
    frame_target: Option<RenderTarget>,
    own: Option<(Option<Tonemapping>, ClearColorConfig, bool)>,
    on: bool,
}

fn same_clear(a: &ClearColorConfig, b: &ClearColorConfig) -> bool {
    match (a, b) {
        (ClearColorConfig::Default, ClearColorConfig::Default) | (ClearColorConfig::None, ClearColorConfig::None) => true,
        (ClearColorConfig::Custom(x), ClearColorConfig::Custom(y)) => x == y,
        _ => false,
    }
}

/// A frame look's parameters from the camera's row this frame (cells, size and the paint look's sky aside).
fn frame_params(v: &BTreeMap<String, V>, mode: &str) -> LookParams {
    let sky = |k: &str| match v.get(k) {
        Some(V::S(s)) if !s.is_empty() => Vec4::from_array(colour(v.get(k), [0.0; 4]).to_linear().to_f32_array()),
        _ => Vec4::ZERO,
    };
    let (sky_top, sky_bottom) = (sky("paint_sky_top"), sky("paint_sky_bottom"));
    let mut pal = [Vec4::ZERO; 16];
    let mut npal = 0;
    if let Some(V::S(list)) = v.get("paint_palette") {
        for h in list.split(',').filter(|h| !h.is_empty()).take(16) {
            pal[npal] = Vec4::from_array(colour(Some(&V::S(h.to_string())), [0.0; 4]).to_linear().to_f32_array());
            npal += 1;
        }
    }
    LookParams {
        mode: match mode {
            "pixel" => 4,
            "ink" => 5,
            "paint" => 6,
            _ => 3,
        },
        dark: !matches!(v.get("frame_look_theme"), Some(V::S(s)) if s == "light") as u32,
        size: Vec2::ZERO,
        edge: Vec4::from_array(colour(v.get("frame_look_ink"), [0.13, 0.12, 0.10, 1.0]).to_linear().to_f32_array()),
        paper: Vec4::from_array(colour(v.get("frame_look_paper"), [0.90, 0.88, 0.80, 1.0]).to_linear().to_f32_array()),
        line: num(v, "frame_look_line", 0.0),
        threshold: num(v, "frame_look_threshold", 0.5),
        band: num(v, "frame_look_band", 1.0),
        lift: num(v, "frame_look_lift", 0.0),
        gain: num(v, "frame_look_gain", 1.0),
        color: num(v, "frame_look_color", 1.0),
        grain: num(v, "paint_grain", 0.3),
        rag: num(v, "paint_rag", 2.0),
        rag_scale: num(v, "paint_rag_scale", 24.0).max(1.0),
        seed: num(v, "paint_seed", 0.0),
        sky_top,
        sky_bottom: if sky_bottom.w > 0.0 { sky_bottom } else { sky_top },
        npal: npal as u32,
        crisp: 0,
        pal,
        ..default()
    }
    .passes(v)
}

impl LookParams {
    /// The look's own settings for each composition pass (the camera row's look_skip and look_passes: compose.rs),
    /// masked only when some pass has any.
    fn passes(mut self, v: &BTreeMap<String, V>) -> Self {
        let skip = super::compose::bits(v, "look_skip");
        let own = super::compose::per_pass(v, "look_passes");
        if super::compose::names(v).is_empty() || (skip == 0 && own.iter().all(Option::is_none)) {
            return self;
        }
        let f = |i: usize, k: &str, d: f32| own[i].as_ref().and_then(|o| o.get(k)).and_then(serde_json::Value::as_f64).map_or(d, |x| x as f32);
        let snap = |i: usize| own[i].as_ref().and_then(|o| o.get("palette")).and_then(serde_json::Value::as_bool).map_or(1.0, |b| b as u8 as f32);
        let each = |g: &dyn Fn(usize) -> f32| Vec4::new(g(0), g(1), g(2), g(3));
        self.masked = 1;
        self.pass_raw = each(&|i| ((skip >> i) & 1) as f32);
        self.pass_rag = each(&|i| f(i, "rag", self.rag));
        self.pass_grain = each(&|i| f(i, "grain", self.grain));
        self.pass_snap = each(&snap);
        self
    }
}

/// Looks set up for models that ask for one (once each, when the model is ready), then each frame: the model's
/// meshes on its layer or the scene's, its camera where the scene's is, its composite's mode.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn sync_looks(
    mut commands: Commands,
    frame: Res<FrameRows>,
    edit: Res<Edit>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<LookMaterial>>,
    models: Query<(Entity, Option<&Look>), With<ModelReady>>,
    kids: Query<&Children>,
    layered: Query<(Entity, Option<&RenderLayers>), With<Mesh3d>>,
    lights: Query<Entity, Or<(With<DirectionalLight>, With<PointLight>, With<SpotLight>)>>,
    main: Query<(Entity, &Transform, Option<&Projection>, Option<&AmbientLight>, Option<&Tonemapping>, Option<&bevy::camera::Exposure>, &RenderTarget, &Msaa, Has<bevy::camera::Hdr>, Has<super::bevy_api::Untonemapped>), (With<Camera3d>, Without<LookCam>)>,
    mut cams: Query<(&mut Camera, &mut Transform, Option<&mut Projection>), (With<LookCam>, Without<Look>)>,
    mut quads: Query<&mut Visibility, (With<Mesh3d>, With<LookCam>)>,
    composite: Query<(Entity, Has<bevy::camera::Hdr>, Has<bevy::ui::IsDefaultUiCamera>), (With<LookCam>, With<Camera3d>, Without<Mesh3d>)>,
    mut next: Local<usize>,
) {
    let Ok((scene_cam, ct, proj, amb, tm, ev, target, msaa, hdr, untonemapped)) = main.single() else { return };
    let (w, h) = (edit.prog.width, edit.prog.height);
    let mut made = false;
    for (e, look) in &models {
        let Some(v) = frame.vals(e) else { continue };
        let v = &v;
        if look.is_none() {
            if !v.contains_key("look") {
                continue;
            }
            // a cell's size in the frame's pixels; the cells' target is the frame's size over it, rounded up
            let px = num(v, "look_px", 6.0).max(1.0);
            let (cw, ch) = (((w as f32) / px).ceil().max(1.0) as u32, ((h as f32) / px).ceil().max(1.0) as u32);
            let mut img = Image::new_target_texture(cw, ch, TextureFormat::Rgba8UnormSrgb, None);
            img.sampler = bevy::image::ImageSampler::nearest();
            let cells = images.add(img);
            *next += 1;
            let layer = *next;
            if layer >= COMPOSITE {
                continue; // thirty looks at most
            }
            let mut cam = commands.spawn((
                Camera3d::default(),
                Camera { order: -1 - layer as isize, clear_color: ClearColorConfig::Custom(Color::NONE), ..default() },
                RenderTarget::Image(cells.clone().into()),
                Msaa::Off,
                RenderLayers::layer(layer),
                *ct,
                tm.copied().unwrap_or_default(),
                LookCam,
            ));
            if let Some(p) = proj {
                cam.insert(p.clone());
            }
            if let Some(a) = amb {
                cam.insert(a.clone());
            }
            if let Some(x) = ev {
                cam.insert(*x);
            }
            if untonemapped {
                cam.insert(super::bevy_api::Untonemapped);
            }
            let cam = cam.id();
            let edge = match v.get("look_edge") {
                Some(V::S(s)) if s.is_empty() || s == "none" => Vec4::ZERO,
                c => Vec4::from_array(colour(c, [0.0, 0.0, 0.0, 1.0]).to_linear().to_f32_array()),
            };
            let theme_dark = !matches!(v.get("look_theme"), Some(V::S(s)) if s == "light");
            let mat = mats.add(LookMaterial {
                p: LookParams {
                    mode: 0,
                    dark: theme_dark as u32,
                    size: Vec2::new(cw as f32, ch as f32),
                    edge,
                    // on the light greys ink is darkness, and a dark model needs more of the range to show its light
                    lift: num(v, "look_lift", if theme_dark { 0.28 } else { 0.35 }),
                    gain: num(v, "look_gain", if theme_dark { 0.9 } else { 0.75 }),
                    color: 0.0,
                    paper: Vec4::ZERO,
                    line: 0.0,
                    threshold: 0.5,
                    band: 1.0,
                    ..default()
                },
                cells,
                mask: Handle::default(),
            });
            let quad = commands
                .spawn((Mesh3d(meshes.add(Rectangle::new(1.0, 1.0))), MeshMaterial3d(mat.clone()), RenderLayers::layer(COMPOSITE), Transform::default(), Visibility::Hidden, bevy::camera::visibility::NoFrustumCulling, LookCam))
                .id();
            commands.entity(e).insert(Look { layer, cam, quad, mat });
            made = true;
        }
    }
    if made {
        // every light on every look's layer as well as the scene's
        let mut all: Vec<usize> = vec![0];
        all.extend(1..=*next);
        for l in &lights {
            commands.entity(l).insert(RenderLayers::from_layers(&all));
        }
        if composite.is_empty() {
            let mut c = commands.spawn((
                Camera3d::default(),
                Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },
                target.clone(),
                *msaa,
                RenderLayers::layer(COMPOSITE),
                Tonemapping::None,
                super::bevy_api::Untonemapped,
                DebandDither::Disabled,
                Transform::default(),
                LookCam,
            ));
            if hdr {
                c.insert(bevy::camera::Hdr);
            }
        }
        // the page's UI stays on the scene's camera (with a frame look, which draws into cells, on the composite)
        if !matches!(frame.vals(scene_cam).as_ref().and_then(|v| v.get("frame_look")), Some(V::S(_))) && !composite.iter().any(|(_, _, ui)| ui) {
            commands.entity(scene_cam).insert(bevy::ui::IsDefaultUiCamera);
        }
    }
    // the composite draws over the scene's own frame only if it's as HDR as the scene's camera (Bloom makes it so,
    // perhaps a few updates in): otherwise it gets a frame of its own and covers the scene with it
    for (c, has, _) in &composite {
        if has != hdr {
            if hdr {
                commands.entity(c).insert(bevy::camera::Hdr);
            } else {
                commands.entity(c).remove::<bevy::camera::Hdr>();
            }
        }
    }
    for (e, look) in &models {
        let Some(look) = look else { continue };
        let Some(v) = frame.vals(e) else { continue };
        let mode = mode_of(&v);
        let on = mode != 0;
        // the model's meshes (a scene can spawn more of them a few updates after it's ready) on its layer or the scene's
        let want = RenderLayers::layer(if on { look.layer } else { 0 });
        for d in descendants(e, &kids) {
            if let Ok((m, have)) = layered.get(d) {
                if have != Some(&want) {
                    commands.entity(m).insert(want.clone());
                }
            }
        }
        if let Ok((mut c, mut t, p)) = cams.get_mut(look.cam) {
            if c.is_active != on {
                c.is_active = on;
            }
            if *t != *ct {
                *t = *ct;
            }
            if let (Some(mut p), Some(sp)) = (p, proj) {
                if let (Projection::Perspective(a), Projection::Perspective(b)) = (p.as_mut(), sp) {
                    if a.fov != b.fov {
                        a.fov = b.fov;
                    }
                }
            }
        }
        if let Ok(mut vis) = quads.get_mut(look.quad) {
            let want = if on { Visibility::Inherited } else { Visibility::Hidden };
            if *vis != want {
                *vis = want;
            }
        }
        if let Some(m) = mats.get(&look.mat) {
            if m.p.mode != mode {
                if let Some(mut m) = mats.get_mut(&look.mat) {
                    m.p.mode = mode;
                }
            }
        }
    }
}

/// A flat model's surfaces (its row's `flat`): shape and colour with no light on them, each of its meshes' materials
/// copied unlit, except under the site look, whose grain is made of the light, where the lit originals come back.
#[derive(Component, Default)]
pub struct Flat(Vec<(Entity, Handle<StandardMaterial>, Handle<StandardMaterial>)>);

pub fn sync_flat(
    mut commands: Commands,
    frame: Res<FrameRows>,
    mut models: Query<(Entity, Option<&mut Flat>), With<ModelReady>>,
    kids: Query<&Children>,
    mut surfaces: Query<&mut MeshMaterial3d<StandardMaterial>>,
    // a see-through mesh drawn with its froxel fog copy keeps its standard material here
    mut fogged: Query<&mut super::froxel_glass::Fogged>,
    mut std: ResMut<Assets<StandardMaterial>>,
) {
    for (e, flat) in &mut models {
        let Some(v) = frame.vals(e) else { continue };
        if !v.get("flat").is_some_and(V::truthy) {
            continue;
        }
        let Some(mut flat) = flat else {
            commands.entity(e).insert(Flat::default());
            continue;
        };
        // the copies made once per mesh (the model's own, not the file's, which other copies of it share); a scene
        // can spawn more meshes a few updates after it's ready
        for d in descendants(e, &kids) {
            if flat.0.iter().any(|(x, ..)| *x == d) {
                continue;
            }
            let Some(lit) = surfaces.get(d).map(|s| s.0.clone()).ok().or_else(|| fogged.get(d).ok().map(|f| f.std.clone())) else { continue };
            let Some(mut copy) = std.get(&lit).cloned() else { continue };
            copy.unlit = true;
            flat.0.push((d, lit, std.add(copy)));
        }
        let unlit = mode_of(&v) != 2;
        for (d, lit, un) in &flat.0 {
            let want = if unlit { un } else { lit };
            if let Ok(mut s) = surfaces.get_mut(*d) {
                if s.0 != *want {
                    s.0 = want.clone();
                }
            } else if let Ok(mut f) = fogged.get_mut(*d) {
                if f.std != *want {
                    f.std = want.clone();
                }
            }
        }
    }
}
