//! Text in the 3D world (<RichText> in raea/3d, raea-sys-text3d; docs/ts-native.feature P19): every glyph a Bevy entity of its
//! own, drawn by bevy_rich_text3d (a mesh of the glyph's quad over the plugin's glyph atlas) with a StandardMaterial of
//! its own, so it is lit, shaded, occluded and casts a letter-shaped shadow like any mesh, and moves, turns, scales,
//! recolours and fades on its own from its row.
//!
//! A `text` row is the block: its transform, its spans ({t, font, weight, italic, size, color}: the runs of rich text),
//! `size` (the em in scene units), `align` (0 left, 0.5 centre, 1 right edge at the origin), `line_height`, `max_width`
//! (wrapped there, in scene units; none by default) and its surface (rough, metal, emissive, emissive_k, unlit, shadow,
//! receive). The block is laid out once, at spawn, by cosmic-text through the plugin's own FontSystem (raea's bundled
//! faces loaded into it), so every glyph sits where the whole line's shaping and kerning put it.
//!
//! A `glyph` row is one glyph of a block (`target`: the block's row id): `at` its byte in the block's text, `span` its
//! run, `ch` its characters. Its entity hangs under a slot at the glyph's place in the block (the slot under the block),
//! so its own row (x, y, z, rot_*, scale_*, col_*, opacity, visible: the columns every mesh has) is the glyph's motion
//! about its own centre, and scene3d::sync_meshes draws it the frame it changes, in a render and in live play alike.
//! A glyph row's `name` is Bevy's Name on it ("title:3"), so world.* reads its Transform back.

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use raea_core_program::ir::V;
use raea_sys_frame::*;
use bevy_rich_text3d::{Text3d, Text3dPlugin, Text3dStyle, TextAnchor, TextAtlas, TextRenderer};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Wrap};
use serde_json::Value;

/// Raster pixels an em at a block's own size: the atlas resolution of its glyphs (a span twice the size is drawn at twice).
const RES: f32 = 128.0;

pub fn install(app: &mut App) {
    if !app.is_plugin_added::<Text3dPlugin>() {
        // the window's scale factor isn't the raster's (renders have no window): glyphs are drawn at RES an em
        app.add_plugins(Text3dPlugin { default_atlas_dimension: (512, 512), sync_scale_factor_with_main_window: false, ..default() });
    }
    app.init_resource::<Faces>();
}

/// The faces loaded into the plugin's FontSystem, by (family, weight, italic), and whether the system's were.
#[derive(Resource, Default)]
struct Faces {
    loaded: std::collections::HashSet<(String, u16, bool)>,
    system: bool,
}

/// One run of a block's rich text.
#[derive(Clone, Debug)]
struct Run {
    text: String,
    font: String,
    weight: u16,
    italic: bool,
    /// times the block's em
    size: f32,
}

fn runs(vals: &BTreeMap<String, V>) -> Result<Vec<Run>, String> {
    let Some(V::S(s)) = vals.get("spans") else { return Err("a text row needs spans".into()) };
    let v: Value = serde_json::from_str(s).map_err(|e| format!("spans: {e}"))?;
    let a = v.as_array().ok_or("spans: a list of {t, font, weight, italic, size}")?;
    Ok(a.iter().map(|r| Run {
        text: r.get("t").and_then(Value::as_str).unwrap_or("").to_string(),
        font: r.get("font").and_then(Value::as_str).unwrap_or("Inter").to_string(),
        weight: r.get("weight").and_then(Value::as_u64).unwrap_or(700) as u16,
        italic: r.get("italic").and_then(Value::as_bool).unwrap_or(false),
        size: r.get("size").and_then(Value::as_f64).unwrap_or(1.0) as f32,
    }).collect())
}

/// The family name cosmic-text matches: CSS's generic names as its own.
fn family(name: &str) -> Family<'_> {
    match name {
        "serif" => Family::Serif,
        "sans-serif" | "system-ui" => Family::SansSerif,
        "monospace" => Family::Monospace,
        "cursive" => Family::Cursive,
        "fantasy" => Family::Fantasy,
        _ => Family::Name(name),
    }
}

fn attrs(r: &Run) -> Attrs<'_> {
    Attrs::new().family(family(&r.font)).weight(cosmic_text::Weight(r.weight)).style(if r.italic { cosmic_text::Style::Italic } else { cosmic_text::Style::Normal })
}

/// Each run's face loaded into the FontSystem: raea's bundled one (the faces the page and Text3D draw with), else the
/// system's fonts, loaded once.
fn load_faces(fs: &mut FontSystem, faces: &mut Faces, runs: &[Run]) {
    for r in runs {
        let key = (r.font.clone(), r.weight, r.italic);
        if faces.loaded.contains(&key) {
            continue;
        }
        faces.loaded.insert(key);
        #[cfg(feature = "native")]
        if let Some(b) = raea_core_fonts::face_ttf(&r.font, r.weight, r.italic) {
            fs.db_mut().load_font_data(b);
            continue;
        }
        if !faces.system {
            faces.system = true;
            fs.db_mut().load_system_fonts();
        }
    }
}

/// A glyph's place in its block: its centre, in the block's own space (scene units).
#[derive(Clone, Copy, Debug)]
pub struct Place {
    centre: Vec3,
}

/// A block: where each of its glyphs goes (by its byte in the text), its runs and its surface's fixed values.
#[derive(Component)]
pub struct TextBlock {
    places: HashMap<usize, Place>,
    runs: Vec<Run>,
    em: f32,
    surface: BTreeMap<String, V>,
}

/// The block laid out as one: each glyph's centre (by the byte it starts at), in scene units, y up, the origin where
/// `align` and the block's middle put it. A glyph is drawn alone (its own Text3d, anchored at its centre): its centre is
/// where its own pen origin, laid out alone as the plugin lays it out, meets the block's pen for it.
fn layout(fs: &mut FontSystem, runs: &[Run], em: f32, line_height: f32, max_width: Option<f32>, align: f32) -> HashMap<usize, Place> {
    let unit = em / RES;
    let mut buf = Buffer::new(fs, Metrics::new(RES, RES * line_height));
    buf.set_wrap(if max_width.is_some() { Wrap::WordOrGlyph } else { Wrap::None });
    buf.set_size(max_width.map(|w| w / unit), None);
    let spans: Vec<(&str, Attrs)> = runs.iter().enumerate().map(|(i, r)| (r.text.as_str(), attrs(r).metrics(Metrics::new(RES * r.size, RES * r.size * line_height)).metadata(i))).collect();
    buf.set_rich_text(spans.iter().map(|(t, a)| (*t, a.clone())), &Attrs::new(), Shaping::Advanced, None);
    buf.shape_until_scroll(fs, false);
    let (mut w, mut h) = (0.0f32, 0.0f32);
    for run in buf.layout_runs() {
        w = w.max(run.line_w);
        h = h.max(run.line_top + run.line_height);
    }
    // each glyph alone: its advance and baseline, as the plugin's Buffer lays a one-glyph Text3d out
    let mut alone: HashMap<(usize, usize), (f32, f32, f32)> = HashMap::new();
    let mut pens: Vec<(usize, usize, usize, f32, f32)> = Vec::new();
    // the runs' texts joined, as the block's Buffer has them, and each run's first byte
    let mut starts = Vec::new();
    let mut at = 0;
    for r in runs {
        starts.push(at);
        at += r.text.len();
    }
    for run in buf.layout_runs() {
        let x0 = (w - run.line_w) * align - w * align;
        for g in run.glyphs {
            pens.push((g.start, g.end, g.metadata, x0 + g.x, -run.line_y + h / 2.0));
        }
    }
    let text: String = runs.iter().map(|r| r.text.as_str()).collect();
    let mut out = HashMap::new();
    for (start, end, span, px, py) in pens {
        let Some(r) = runs.get(span) else { continue };
        let key = (start, end);
        let (w1, h1, y1) = *alone.entry(key).or_insert_with(|| {
            let ch = text.get(start..end).unwrap_or("");
            let size = RES * r.size;
            let mut one = Buffer::new(fs, Metrics::new(size, size));
            one.set_rich_text([(ch, attrs(r))], &attrs(r), Shaping::Advanced, None);
            one.shape_until_scroll(fs, false);
            let mut m = (0.0, size, size * 0.8);
            if let Some(l) = one.layout_runs().next() {
                m = (l.line_w, l.line_top + l.line_height, l.line_y);
            }
            m
        });
        // the glyph alone is drawn with its block's centre at its origin: its pen at (-w1/2, h1/2 - y1)
        let c = Vec3::new((px + w1 / 2.0) * unit, (py - h1 / 2.0 + y1) * unit, 0.0);
        out.entry(start).or_insert(Place { centre: c });
    }
    out
}

/// A `text` row's entity: the block placed, laid out, its glyphs' places kept for their rows.
pub fn spawn_text(world: &mut World, e: Entity, vals: &BTreeMap<String, V>) -> Result<(), String> {
    let runs = runs(vals)?;
    let em = num(vals, "size", 0.5).max(1e-4);
    let mut renderer = world.get_resource::<TextRenderer>().cloned().ok_or("text: the 3D text plugin's fonts aren't loaded")?;
    let places = world.resource_scope(|_, mut faces: Mut<Faces>| {
        let mut fs = renderer.lock();
        load_faces(&mut fs, &mut faces, &runs);
        layout(&mut fs, &runs, em, num(vals, "line_height", 1.2), vals.get("max_width").map(|v| v.num() as f32).filter(|w| *w > 0.0), num(vals, "align", 0.5).clamp(0.0, 1.0))
    });
    let surface: BTreeMap<String, V> = vals.iter().filter(|(k, _)| ["rough", "metal", "emissive", "emissive_k", "unlit", "shadow", "receive", "double"].contains(&k.as_str())).map(|(k, v)| (k.clone(), v.clone())).collect();
    world.entity_mut(e).insert((transform_of(vals), Visibility::default(), TextBlock { places, runs, em, surface }));
    if let Some(V::S(n)) = vals.get("name") {
        world.entity_mut(e).insert(Name::new(n.clone()));
    }
    layers(world, e, vals);
    Ok(())
}

/// Under a block, at a glyph's place: the glyph's entity hangs from it, so the glyph's own motion is about its centre.
#[derive(Component)]
pub struct GlyphSlot;

/// A `glyph` row's entity: its character drawn alone, at its place in its block, with a surface of its own.
pub fn spawn_glyph(world: &mut World, e: Entity, id: &str, vals: &BTreeMap<String, V>) -> Result<(), String> {
    let Some(V::S(target)) = vals.get("target") else { return Err(format!("glyph {id}: needs target (its text row)")) };
    let block = raea_sys_frame::store::find(world, "text", target).ok_or_else(|| format!("glyph {id}: no text row {target}"))?;
    let at = num(vals, "at", 0.0) as usize;
    let span = num(vals, "span", 0.0) as usize;
    let ch = match vals.get("ch") { Some(V::S(s)) => s.clone(), _ => return Err(format!("glyph {id}: needs ch (its characters)")) };
    let (place, run, em, surface) = {
        let b = world.get::<TextBlock>(block).ok_or_else(|| format!("glyph {id}: {target} isn't a text block"))?;
        let run = b.runs.get(span).cloned().ok_or_else(|| format!("glyph {id}: its block has no span {span}"))?;
        (b.places.get(&at).copied(), run, b.em, b.surface.clone())
    };
    // a glyph the layout has no place for (a ligature's second letter, a mark joined to the one before) stays where its
    // cluster is: its first byte's place before it
    let place = place.unwrap_or(Place { centre: Vec3::ZERO });
    let slot = world.spawn((Transform::from_translation(place.centre), Visibility::Inherited, GlyphSlot, ChildOf(block))).id();
    let s = |k: &str, d: f32| surface.get(k).map(|v| v.num() as f32).unwrap_or(d);
    let base = colour(vals.get("color"), [1.0, 1.0, 1.0, 1.0]);
    let fades = vals.contains_key("opacity");
    let mut base_color = base;
    if fades {
        base_color.set_alpha(num(vals, "opacity", 1.0).clamp(0.0, 1.0));
    }
    let emissive = match surface.get("emissive") {
        Some(c) => colour(Some(c), [0.0, 0.0, 0.0, 1.0]).to_linear() * s("emissive_k", 1.0),
        None => LinearRgba::BLACK,
    };
    let mat = StandardMaterial {
        base_color,
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        // a glyph that fades is blended; one that doesn't is cut at half its coverage, so it writes depth and its
        // shadow is the letter's
        alpha_mode: if fades { AlphaMode::Blend } else { AlphaMode::Mask(0.5) },
        perceptual_roughness: s("rough", 0.5),
        metallic: s("metal", 0.0),
        emissive: LinearRgba::rgb(emissive.red, emissive.green, emissive.blue),
        unlit: surface.get("unlit").is_some_and(V::truthy),
        // turned round, a glyph shows its back (mirrored), as a card does
        double_sided: true,
        cull_mode: None,
        ..default()
    };
    let mat = world.resource_mut::<Assets<StandardMaterial>>().add(mat);
    let style = Text3dStyle {
        size: RES * run.size,
        font: run.font.clone().into(),
        weight: bevy_rich_text3d::Weight(run.weight),
        style: if run.italic { bevy_rich_text3d::Style::Italic } else { bevy_rich_text3d::Style::Normal },
        anchor: TextAnchor::CENTER,
        line_height: 1.0,
        layer_offset: 0.0,
        world_scale: Some(Vec2::splat(em * run.size)),
        ..default()
    };
    world.entity_mut(e).insert((Text3d::new(ch), style, Mesh3d::default(), MeshMaterial3d(mat), transform_of(vals), Visibility::Inherited, ChildOf(slot)));
    if let Some(V::S(n)) = vals.get("name") {
        world.entity_mut(e).insert(Name::new(n.clone()));
    }
    if surface.get("shadow").is_some_and(|v| !v.truthy()) {
        world.entity_mut(e).insert(bevy::light::NotShadowCaster);
    }
    if surface.get("receive").is_some_and(|v| !v.truthy()) {
        world.entity_mut(e).insert(bevy::light::NotShadowReceiver);
    }
    layers(world, e, vals);
    Ok(())
}

/// Each block where the frame's row puts it, shown or hidden (its glyphs move with their own rows: sync_meshes).
pub fn sync_texts(frame: Res<FrameRows>, mut q: Query<(Entity, &mut Transform, &mut Visibility), With<TextBlock>>) {
    for (e, mut t, mut vis) in &mut q {
        let Some(v) = frame.vals(e) else { continue };
        let nt = transform_of(&v);
        if *t != nt {
            *t = nt;
        }
        if let Some(on) = v.get("visible") {
            let want = if on.truthy() { Visibility::Inherited } else { Visibility::Hidden };
            if *vis != want {
                *vis = want;
            }
        }
    }
}
