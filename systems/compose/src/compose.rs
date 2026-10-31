//! Composition passes: which of a piece's passes drew each pixel, so a frame-wide effect can leave some of the scene
//! as it was drawn (a page on a <Screen> crisp behind a painted body, 10-22, anything/intro/opening-build.tsx).
//!
//! A row's `pass` column (keyed, stepped: 0 "scene", the default, 1 to 3 the camera row's `passes` in order) goes on its
//! meshes as Bevy's MeshTag, and the prepass writes it where Bevy writes the normal (prepass_pass below patches Bevy's
//! two prepass shaders: the normal's alpha, which nothing reads, carries the tag's two low bits). So a pixel's pass is
//! the nearest surface's, depth-tested as the frame is: a painted body in front of a clean screen stays painted where
//! it covers it. A camera with passes keeps a depth and a normal prepass (and no MSAA: engine.rs).
//!
//! From that, each frame (compose.wesl):
//!   - the mask (`PassMask`, an image the camera's size; r g b passes 1 to 3's coverage): each pixel's pass, blurred as
//!     <DepthOfField> blurs the frame (its circle of confusion from the same depth, Bevy's Gaussian), so a defocused
//!     page's spill is the page's as far as the blur carries it. The looks' composite and the grade read it.
//!   - a <Bloom>'s passes: what a pass holds back neither glows nor takes the glow (the colour less the held share
//!     before Bevy's bloom, the kept colour put back after it, before depth of field).
//!   - a <Grade>'s passes: the colour kept after depth of field and put back after the effect stack (fringes, and the
//!     vignette where it is: on the scene's camera, or on the paint look's composite), and on an untonemapped camera
//!     after tone mapping too, so its colour grading is held back as well.
//! An <Outline> reads the pass from the normal itself (outline.wesl); the paint look's composite (look.wesl) and its
//! surfaces (paint.rs) take their own passes' settings.
//!
//! Camera row columns: passes (the names, "ui,fx"; at most three), bloom_passes and grade_passes (JSON: each named
//! pass's share of the effect, 0 to 1), look_skip, look_passes, outline_skip (read by look.rs, paint.rs).

use super::*;
use bevy::{
    core_pipeline::{
        prepass::{DepthPrepass, NormalPrepass, ViewPrepassTextures},
        schedule::Core3d,
        tonemapping::tonemapping,
        Core3dSystems, FullscreenShader,
    },
    ecs::schedule::{FlattenedDependencies, NodeId, ScheduleBuildError, ScheduleBuildPass, ScheduleGraph, SystemKey, SystemSetKey},
    mesh::MeshTag,
    post_process::bloom::bloom,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{texture_2d, texture_depth_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        texture::{CachedTexture, GpuImage, TextureCache},
        view::ViewTarget,
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};

mod passes;
pub use passes::*;

/// The most passes a piece names besides "scene": the normal's alpha has two bits, and "scene" is one of its values.
pub const MOST: usize = 3;

/// The camera row's pass names (`passes`), in id order from 1.
pub fn names(v: &BTreeMap<String, V>) -> Vec<String> {
    match v.get("passes") {
        Some(V::S(s)) => s.split(',').filter(|n| !n.is_empty()).take(MOST).map(str::to_string).collect(),
        _ => Vec::new(),
    }
}

/// A pass's id by name: "scene" 0, the camera's passes from 1.
pub fn id_of(names: &[String], name: &str) -> Option<usize> {
    if name == "scene" {
        return Some(0);
    }
    names.iter().position(|n| n == name).map(|i| i + 1)
}

/// A comma list of pass names (`look_skip`, `outline_skip`) as bits, 1 << id.
pub fn bits(v: &BTreeMap<String, V>, col: &str) -> u32 {
    let ns = names(v);
    match v.get(col) {
        Some(V::S(s)) => s.split(',').filter_map(|n| id_of(&ns, n)).fold(0, |b, i| b | 1 << i),
        _ => 0,
    }
}

/// A JSON column of settings by pass name (`look_passes`, `bloom_passes`) by id: [scene, 1, 2, 3].
pub fn per_pass(v: &BTreeMap<String, V>, col: &str) -> [Option<serde_json::Value>; 4] {
    let mut out: [Option<serde_json::Value>; 4] = Default::default();
    let ns = names(v);
    if let Some(V::S(s)) = v.get(col) {
        if let Ok(serde_json::Value::Object(m)) = serde_json::from_str::<serde_json::Value>(s) {
            for (k, x) in m {
                if let Some(i) = id_of(&ns, &k) {
                    out[i] = Some(x);
                }
            }
        }
    }
    out
}

/// How much each pass holds back from an effect: 1 - its share (`{ ui: 0 }`: all of it), [scene, 1, 2, 3].
fn held(v: &BTreeMap<String, V>, col: &str) -> Vec4 {
    let p = per_pass(v, col);
    let h = |i: usize| p[i].as_ref().and_then(serde_json::Value::as_f64).map_or(0.0, |s| 1.0 - s.clamp(0.0, 1.0) as f32);
    Vec4::new(h(0), h(1), h(2), h(3))
}

/// A camera drawing with passes: the mask it writes (the scene's camera) or reads (the looks' composite), what its
/// bloom and grade hold back by pass, and its depth of field's lens for the mask's blur.
#[derive(Component, Clone, PartialEq, ExtractComponent)]
#[extract_app(bevy::render::RenderApp)]
pub struct ComposeView {
    mask: Handle<Image>,
    write: bool,
    bloom: Vec4,
    grade: Vec4,
    /// the grade's kept colour goes back after tone mapping (an untonemapped camera: its grading held back too)
    late: bool,
    /// depth of field: focal distance, focal length, the circle of confusion's scale, its widest (0 scale: none)
    coc: Vec4,
    /// Bevy's bokeh (else its Gaussian)
    bokeh: bool,
    /// the near plane and the farthest depth the blur reads
    near: f32,
    far: f32,
}

/// The mask this frame's passes are in (its size the scene camera's target), for the looks' composite.
#[derive(Resource, Default)]
pub struct PassMask {
    image: Option<(Handle<Image>, UVec2)>,
    on: bool,
}

impl PassMask {
    /// The mask, while the camera has passes.
    pub fn image(&self) -> Option<Handle<Image>> {
        self.image.as_ref().filter(|_| self.on).map(|(h, _)| h.clone())
    }
}

pub fn install(app: &mut App) {
    bevy::asset::embedded_asset!(app, "compose.wesl");
    app.init_resource::<PassMask>().add_plugins(ExtractComponentPlugin::<ComposeView>::default()).add_systems(Update, prepass_pass);
    let Some(render) = app.get_sub_app_mut(RenderApp) else { return };
    render
        .add_systems(RenderStartup, init)
        .add_systems(Render, prepare.in_set(RenderSystems::PrepareResources).after(bevy::render::view::prepare_view_targets))
        .add_systems(
            Core3d,
            (
                mask.in_set(Core3dSystems::EarlyPostProcess),
                bloom_hold.before(bloom).in_set(Core3dSystems::PostProcess),
                bloom_give.after(bloom).before(grade_hold).in_set(Core3dSystems::PostProcess),
                grade_hold.before(grade_give).in_set(Core3dSystems::PostProcess),
                grade_give.before(tonemapping).in_set(Core3dSystems::PostProcess),
                grade_give_late.after(tonemapping).before(bevy::core_pipeline::upscaling::upscaling).in_set(Core3dSystems::PostProcess),
            ),
        );
    // Bevy's depth of field and effect stack are crate-private systems, so they can't be named in an ordering: a build
    // pass puts the bloom's give before the depth of field, the grade's hold after it and before the effect stack, and
    // the grade's give after that (and its late give before lens.rs's pass, private to it)
    render.edit_schedule(Core3d, |s| {
        s.add_build_pass(Around);
    });
}

/// Orders compose's systems around Bevy's depth of field and effect stack, found by name.
#[derive(Debug)]
struct Around;

impl ScheduleBuildPass for Around {
    type EdgeOptions = ();

    fn add_dependency(&mut self, _from: NodeId, _to: NodeId, _options: Option<&Self::EdgeOptions>) {}

    fn collapse_set(&mut self, _set: SystemSetKey, _systems: &indexmap::IndexSet<SystemKey, bevy::platform::hash::FixedHasher>, _graph: &bevy::ecs::schedule::graph::DiGraph<NodeId>) -> impl Iterator<Item = (NodeId, NodeId)> {
        std::iter::empty()
    }

    fn build(&mut self, _world: &mut World, graph: &mut ScheduleGraph, mut deps: FlattenedDependencies<'_>) -> Result<(), ScheduleBuildError> {
        let keys: Vec<SystemKey> = deps.toposort_and_graph().map_err(ScheduleBuildError::FlatDependencySort)?.0.to_vec();
        let find = |short: &str| keys.iter().copied().find(|k| {
            // a system's name, less the " (in set …)" Bevy appends to one in a set
            let n = graph.get_node_name(&NodeId::System(*k));
            let n = n.split(" (").next().unwrap_or_default();
            n == short || n.ends_with(&format!("::{short}"))
        });
        let (Some(dof), Some(stack)) = (find("depth_of_field"), find("post_processing")) else {
            // a Bevy release renamed them: the passes' grade and bloom would be held in the wrong place
            bevy::log::warn!("compose: Bevy's depth_of_field or post_processing system not found in Core3d; composition passes can't order around them");
            return Ok(());
        };
        if let Some(b) = find("bloom_give") {
            deps.add_edge(b, dof);
        }
        if let Some(h) = find("grade_hold") {
            deps.add_edge(dof, h);
            deps.add_edge(h, stack);
        }
        if let Some(g) = find("grade_give") {
            deps.add_edge(stack, g);
        }
        // and the late give before raea's lens (a lens bends the page with the rest of the frame)
        if let (Some(g), Some(l)) = (find("grade_give_late"), find("lens_pass")) {
            deps.add_edge(g, l);
        }
        Ok(())
    }
}

// ------------------------------------------------------------------------------------------------ the prepass

/// Bevy 0.20.0's prepass shaders, FNV-1a, which the patch below was written against.
const PREPASS_FNV: u64 = 0x6f99_db37_c7ad_5c56;
const PBR_PREPASS_FNV: u64 = 0x07c2_2c70_e879_b290;

const ALPHA_FN: &str = "/// raea (raea-render engine/compose.rs): a mesh's composition pass, its MeshTag's two low bits, in the normal's alpha: 1 for
/// the scene's (as Bevy writes it), 0, 1/3 and 2/3 for passes 1 to 3
fn raea_pass_alpha(tag: u32) -> f32 {
    let t = tag & 3u;
    if t == 0u {
        return 1.0;
    }
    return f32(t - 1u) / 3.0;
}

@if(PREPASS_FRAGMENT)
@fragment";

fn replace_once(s: &str, from: &str, to: &str, file: &str) -> String {
    assert_eq!(s.matches(from).count(), 1, "raea: expected `{from}` once in Bevy's {file}");
    s.replacen(from, to, 1)
}

/// Bevy's prepass.wesl with the pass in the normal's alpha.
fn patch_prepass(bevy: &str) -> String {
    let s = replace_once(bevy, "@if(PREPASS_FRAGMENT)\n@fragment", ALPHA_FN, "prepass.wesl");
    replace_once(&s, "    out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);", "    out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), raea_pass_alpha(mesh[in.instance_index].tag));", "prepass.wesl")
}

/// Bevy's pbr_prepass.wesl (StandardMaterial's, and every extension's of it) with the pass in the normal's alpha.
fn patch_pbr_prepass(bevy: &str) -> String {
    let s = replace_once(bevy, "@if(PREPASS_FRAGMENT)\n@fragment", ALPHA_FN, "pbr_prepass.wesl");
    replace_once(
        &s,
        "        out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);\n    }\n",
        "        out.normal = vec4(in.world_normal * 0.5 + vec3(0.5), 1.0);\n    }\n@if(NORMAL_PREPASS && !MESHLET_MESH_MATERIAL_PASS)\n    out.normal.a = raea_pass_alpha(mesh[in.instance_index].tag);\n",
        "pbr_prepass.wesl",
    )
}

/// The mask: each pixel's pass from the normal prepass, blurred along x into `half`, then along y into the mask.
fn mask(view: ViewQuery<(&ComposeView, &ViewPrepassTextures, &Kept)>, p: Option<Res<Pipelines>>, cache: Res<PipelineCache>, images: Res<RenderAssets<GpuImage>>, queue: Res<RenderQueue>, mut ctx: RenderContext) {
    let Some(p) = p else { return };
    let (cv, prepass, kept) = view.into_inner();
    if !cv.write {
        return;
    }
    let (Some(out), Some(depth), Some(normal)) = (images.get(&cv.mask), prepass.depth_only_view(), prepass.normal_view()) else { return };
    let (Some(h), Some(v)) = (p.get(&cache, "mask_h", TextureFormat::Rgba8Unorm), p.get(&cache, "mask_v", TextureFormat::Rgba8Unorm)) else { return };
    if out.texture.size() != kept.half.texture.size() {
        return;
    }
    let device = ctx.render_device().clone();
    let layout = cache.get_bind_group_layout(&p.mask_layout);
    for (dir, pipeline, from, to) in [(Vec2::X, h, normal, &kept.half.default_view), (Vec2::Y, v, &kept.half.default_view, &out.texture_view)] {
        let u = params(&device, &queue, Params { hold: Vec4::new(cv.bokeh as u8 as f32, 0.0, 0.0, 0.0), coc: cv.coc, z: Vec4::new(cv.near, cv.far, dir.x, dir.y) });
        let group = device.create_bind_group("raea_compose_mask", &layout, &BindGroupEntries::with_indices(((0, u.binding().unwrap()), (1, from), (3, depth))));
        draw(&mut ctx, "raea_compose_mask", pipeline, &group, to);
    }
}

/// The colour as it is now, kept for a give.
fn keep(ctx: &mut RenderContext, target: &ViewTarget, kept: &Kept) {
    let src = target.main_texture();
    if src.format() == kept.kept.texture.format() && src.size() == kept.kept.texture.size() {
        ctx.command_encoder().copy_texture_to_texture(src.as_image_copy(), kept.kept.texture.as_image_copy(), src.size());
    }
}

/// An effect's give: the frame after it, with what each pass holds back put back from the kept colour (by the sharp
/// pass, `aux` the normal prepass, before depth of field; by the mask after it).
#[allow(clippy::too_many_arguments)]
fn give(ctx: &mut RenderContext, p: &Pipelines, cache: &PipelineCache, queue: &RenderQueue, target: &ViewTarget, kept: &Kept, hold: Vec4, entry: &'static str, aux: &TextureView) {
    let Some(pipeline) = p.get(cache, entry, target.main_texture_format()) else { return };
    let device = ctx.render_device().clone();
    let u = params(&device, queue, Params { hold, coc: Vec4::ZERO, z: Vec4::ZERO });
    let post = target.post_process_write();
    let group = device.create_bind_group("raea_compose_give", &cache.get_bind_group_layout(&p.post_layout), &BindGroupEntries::with_indices(((0, u.binding().unwrap()), (1, post.source), (2, aux), (4, &kept.kept.default_view))));
    draw(ctx, "raea_compose_give", pipeline, &group, post.destination);
}

/// Before Bevy's bloom: the colour kept, and each pixel's held share taken out of what blooms.
fn bloom_hold(view: ViewQuery<(&ComposeView, &ViewTarget, &ViewPrepassTextures, &Kept)>, p: Option<Res<Pipelines>>, cache: Res<PipelineCache>, queue: Res<RenderQueue>, mut ctx: RenderContext) {
    let Some(p) = p else { return };
    let (cv, target, prepass, kept) = view.into_inner();
    if cv.bloom == Vec4::ZERO {
        return;
    }
    let (Some(normal), Some(pipeline)) = (prepass.normal_view(), p.get(&cache, "hold", target.main_texture_format())) else { return };
    keep(&mut ctx, target, kept);
    let device = ctx.render_device().clone();
    let u = params(&device, &queue, Params { hold: cv.bloom, coc: Vec4::ZERO, z: Vec4::ZERO });
    let post = target.post_process_write();
    let group = device.create_bind_group("raea_compose_hold", &cache.get_bind_group_layout(&p.post_layout), &BindGroupEntries::with_indices(((0, u.binding().unwrap()), (1, post.source), (2, normal), (4, &kept.kept.default_view))));
    draw(&mut ctx, "raea_compose_hold", pipeline, &group, post.destination);
}

/// After Bevy's bloom, before depth of field: the held share back as it was drawn, unbloomed.
fn bloom_give(view: ViewQuery<(&ComposeView, &ViewTarget, &ViewPrepassTextures, &Kept)>, p: Option<Res<Pipelines>>, cache: Res<PipelineCache>, queue: Res<RenderQueue>, mut ctx: RenderContext) {
    let Some(p) = p else { return };
    let (cv, target, prepass, kept) = view.into_inner();
    let Some(normal) = prepass.normal_view() else { return };
    if cv.bloom != Vec4::ZERO {
        give(&mut ctx, &p, &cache, &queue, target, kept, cv.bloom, "give_sharp", normal);
    }
}

/// After depth of field, before the effect stack: the colour kept for the grade's give.
fn grade_hold(view: ViewQuery<(&ComposeView, &ViewTarget, &Kept)>, mut ctx: RenderContext) {
    let (cv, target, kept) = view.into_inner();
    if cv.grade != Vec4::ZERO {
        keep(&mut ctx, target, kept);
    }
}

fn grade_give_at(late: bool, view: ViewQuery<(&ComposeView, &ViewTarget, &Kept)>, p: Option<Res<Pipelines>>, cache: Res<PipelineCache>, images: Res<RenderAssets<GpuImage>>, queue: Res<RenderQueue>, mut ctx: RenderContext) {
    let Some(p) = p else { return };
    let (cv, target, kept) = view.into_inner();
    if cv.grade == Vec4::ZERO || cv.late != late {
        return;
    }
    let Some(m) = images.get(&cv.mask) else { return };
    give(&mut ctx, &p, &cache, &queue, target, kept, cv.grade, "give_mask", &m.texture_view);
}

/// After the effect stack (fringes, vignette): the kept colour back where a pass holds the grade back.
fn grade_give(view: ViewQuery<(&ComposeView, &ViewTarget, &Kept)>, p: Option<Res<Pipelines>>, cache: Res<PipelineCache>, images: Res<RenderAssets<GpuImage>>, queue: Res<RenderQueue>, ctx: RenderContext) {
    grade_give_at(false, view, p, cache, images, queue, ctx)
}

/// The same after tone mapping, on an untonemapped camera, so its colour grading is held back as well.
fn grade_give_late(view: ViewQuery<(&ComposeView, &ViewTarget, &Kept)>, p: Option<Res<Pipelines>>, cache: Res<PipelineCache>, images: Res<RenderAssets<GpuImage>>, queue: Res<RenderQueue>, ctx: RenderContext) {
    grade_give_at(true, view, p, cache, images, queue, ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_names_and_bits() {
        let mut v = BTreeMap::new();
        v.insert("passes".to_string(), V::S("ui,fx".into()));
        v.insert("look_skip".to_string(), V::S("ui,nope".into()));
        v.insert("bloom_passes".to_string(), V::S(r#"{"ui":0,"fx":0.25}"#.into()));
        assert_eq!(id_of(&names(&v), "scene"), Some(0));
        assert_eq!(id_of(&names(&v), "fx"), Some(2));
        assert_eq!(bits(&v, "look_skip"), 1 << 1);
        assert_eq!(held(&v, "bloom_passes"), Vec4::new(0.0, 1.0, 0.75, 0.0));
    }
}
