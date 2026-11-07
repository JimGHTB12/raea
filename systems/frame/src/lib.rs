//! A frame's rows as the passes read them: FrameRows (each row's values at the frame, a live scene's rows, a frame's
//! own look), the loaded edit, the clock, the frame's error, and the helpers every pass reads a row with (num, colour,
//! sources found and resolved, the cover fit); bevy_api.rs is what the passes share of Bevy's render API.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use raea_core_program::eval::{self, Tup};
use raea_core_program::ir::{Program, V};

pub mod bevy_api;
// the edit's rows as entities, each column a component of its own, found by kind and id
pub mod store;
// the frame loop's own state (Stages, the frame tag, the overlay and output resources)
mod state;
pub use state::*;

/// A model row's entity (a glTF under it), whatever its loading stage.
#[derive(Component, Default)]
pub struct Model;

/// A model loaded, its scene spawned and its clip set up: what passes over a model's meshes wait for.
#[derive(Component)]
pub struct ModelReady;

/// The loaded edit: its program, its base as columns (gathered from the store once per load: row r is the store's
/// r-th row) and its media rows (for resolving sources).
#[derive(Resource, Clone)]
pub struct Edit {
    pub prog: Program,
    /// every frame, evaluated once per load from the base gathered out of the store (row r is the store's r-th row)
    pub timeline: std::sync::Arc<raea_core_program::timeline::Timeline>,
    pub media: Vec<Tup>,
}

/// The frame's values: the base after the keys, rules and transitions at the clock, by row; and each row's entity.
#[derive(Resource, Default)]
pub struct FrameRows {
    pub cols: eval::Cols,
    pub ix: bevy::ecs::entity::EntityHashMap<usize>,
    /// a live scene's rows (Engine::set_rows): a game's 3D comes from its play's state each frame, not from a timeline,
    /// so each row's latest values are kept here (in the store's order), changed in place where a frame changes them,
    /// and lent to the systems as they are (no columns made, nothing copied per frame)
    pub live: Option<Vec<BTreeMap<String, V>>>,
    /// rows whose frame look changes (the camera's `frame_looks`): that list as written, and each look's columns
    pub looks: HashMap<usize, (String, Vec<BTreeMap<String, V>>)>,
    /// the edit's timeline the columns came from (which rows never change: `still`)
    pub timeline: Option<std::sync::Arc<raea_core_program::timeline::Timeline>>,
}

impl FrameRows {
    /// Whether a row's values are the same on every frame of the edit, so a system applying them to its entity each
    /// frame can apply them once (a live scene's rows: never, since a play changes any of them).
    pub fn still(&self, e: Entity) -> bool {
        self.live.is_none() && self.timeline.as_ref().zip(self.ix.get(&e)).is_some_and(|(t, r)| t.still(*r))
    }

    /// Whether any row may have column `c` this frame (a live scene's or a frame look's rows: always), so a system
    /// looking for a rare column (a glitch's) passes over a frame without it at once, not row by row (FC1b)
    pub fn may_have(&self, c: &str) -> bool {
        self.live.is_some() || !self.looks.is_empty() || self.cols.cols.contains_key(c)
    }

    /// One of a row's values this frame, copying only it (with its frame look laid on, as `vals` has it).
    pub fn get(&self, e: Entity, c: &str) -> Option<V> {
        let r = *self.ix.get(&e)?;
        if self.looks.contains_key(&r) {
            return self.vals(e)?.get(c).cloned();
        }
        match &self.live {
            Some(rows) => rows.get(r)?.get(c).cloned(),
            None => self.cols.get(r, c),
        }
    }

    /// A row's values this frame.
    pub fn vals(&self, e: Entity) -> Option<std::borrow::Cow<'_, BTreeMap<String, V>>> {
        let r = *self.ix.get(&e)?;
        match &self.live {
            // a live row as it is, unless its frame look changes: then a copy with this frame's look laid on
            Some(rows) => {
                let row = rows.get(r)?;
                Some(match self.looks.get(&r) {
                    Some((_, looks)) => {
                        let mut v = row.clone();
                        frame_look_onto(&mut v, looks);
                        std::borrow::Cow::Owned(v)
                    }
                    None => std::borrow::Cow::Borrowed(row),
                })
            }
            None => {
                let mut v = self.cols.tup(r).vals;
                if let Some((_, looks)) = self.looks.get(&r) {
                    frame_look_onto(&mut v, looks);
                }
                Some(std::borrow::Cow::Owned(v))
            }
        }
    }

    /// The rows with a `frame_looks` list (the timeline's at the clock, or a live scene's rows), parsed again only
    /// where it changed.
    pub fn parse_looks(&mut self) {
        let found: Vec<(usize, String)> = match &self.live {
            Some(rows) => rows.iter().enumerate().filter_map(|(r, v)| match v.get("frame_looks") {
                Some(V::S(s)) => Some((r, s.clone())),
                _ => None,
            }).collect(),
            None => {
                let Some(col) = self.cols.cols.get("frame_looks") else {
                    self.looks.clear();
                    return;
                };
                (0..self.cols.len()).filter_map(|r| match col.get(r) {
                    Some(V::S(s)) => Some((r, s)),
                    _ => None,
                }).collect()
            }
        };
        let mut seen = Vec::new();
        for (r, s) in found {
            seen.push(r);
            if self.looks.get(&r).is_some_and(|(x, _)| *x == s) {
                continue;
            }
            self.looks.insert(r, (s.clone(), parse_frame_looks(&s)));
        }
        self.looks.retain(|r, _| seen.contains(r));
    }
}

/// `frame_looks` (scene3d's writer: a JSON list of objects, each a look's columns) as rows of columns.
pub fn parse_frame_looks(s: &str) -> Vec<BTreeMap<String, V>> {
    let Ok(serde_json::Value::Array(list)) = serde_json::from_str::<serde_json::Value>(s) else { return Vec::new() };
    list.iter()
        .map(|o| {
            o.as_object()
                .map(|o| {
                    o.iter()
                        .filter_map(|(k, v)| match v {
                            serde_json::Value::Number(n) => Some((k.clone(), V::N(n.as_f64().unwrap_or(0.0)))),
                            serde_json::Value::String(x) => Some((k.clone(), V::S(x.clone()))),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect()
}

/// This frame's look laid onto a row: every look's columns off it, then the one `frame_look_i` names (none below 0).
pub fn frame_look_onto(v: &mut BTreeMap<String, V>, looks: &[BTreeMap<String, V>]) {
    let i = v.get("frame_look_i").map(V::num).unwrap_or(-1.0).round();
    v.remove("frame_looks");
    for l in looks {
        for k in l.keys() {
            v.remove(k);
        }
    }
    if i >= 0.0 {
        if let Some(l) = looks.get(i as usize) {
            v.extend(l.iter().map(|(k, x)| (k.clone(), x.clone())));
        }
    }
}

#[cfg(test)]
mod frame_look_tests {
    use super::*;

    #[test]
    fn a_frame_takes_its_own_look_and_none_goes_plain() {
        let looks = parse_frame_looks(r##"[{"frame_look":"paint","paint_sky_top":"#5b9497"},{"frame_look":"paint","outline":"#2b1d14"}]"##);
        assert_eq!(looks.len(), 2);
        let row = |i: f64| BTreeMap::from([("frame_looks".to_string(), V::S("..".into())), ("frame_look_i".to_string(), V::N(i)), ("x".to_string(), V::N(1.0))]);
        let mut v = row(1.0);
        frame_look_onto(&mut v, &looks);
        assert_eq!(v.get("outline"), Some(&V::S("#2b1d14".into())));
        assert!(!v.contains_key("paint_sky_top") && !v.contains_key("frame_looks"));
        let mut v = row(-1.0);
        frame_look_onto(&mut v, &looks);
        assert!(!v.contains_key("frame_look") && !v.contains_key("outline"));
        assert_eq!(v.get("x"), Some(&V::N(1.0)));
    }
}

#[derive(Resource, Default)]
pub struct Clock(pub i64);

#[derive(Resource, Default)]
pub struct Error(pub Option<String>);

/// A bare <Entity>: its Bevy components come from its rows, so the camera row's systems (sync_camera) leave it be.
#[derive(Component)]
pub struct BareEntity;

pub fn colour(v: Option<&V>, d: [f32; 4]) -> Color {
    let c = match v {
        // CSS's hex forms, #rgb and #rgba too (a holder's "#000" was the default clear)
        Some(V::S(s)) => Srgba::hex(s.trim()).map(|c| c.to_f32_array()).unwrap_or(d),
        _ => d,
    };
    Color::srgba(c[0], c[1], c[2], c[3])
}

/// A picture file Bevy reads itself (a .ktx2 cubemap or lightmap, with its own format and mips) decoded into a new
/// image (`what` names the part in the error): an environment map's, a skybox's or a lightmap's.
pub fn image_file(world: &mut World, what: &str, p: &str) -> Result<Handle<Image>, String> {
    let bytes = std::fs::read(p).map_err(|err| format!("{what}: {p}: {err}"))?;
    let ext = Path::new(p).extension().and_then(|x| x.to_str()).unwrap_or("ktx2");
    let img = Image::from_buffer(&bytes, bevy::image::ImageType::Extension(ext), bevy::image::CompressedImageFormats::NONE, false, bevy::image::ImageSampler::Default, bevy::asset::RenderAssetUsages::RENDER_WORLD)
        .map_err(|err| format!("{what}: {p}: {err}"))?;
    Ok(world.resource_mut::<Assets<Image>>().add(img))
}

/// A clip's or audio row's source: `media:<sha256>` names a row of the media table (content-addressed footage, music
/// and voice; the row holds the file's path and size), anything else is a path. The size is checked on every
/// resolve (cheap); the hash was checked when the row was made (`raea media`).
pub fn resolve_src(rows: &[Tup], src: &str, dir: &Path) -> Result<PathBuf, String> {
    let Some(hash) = src.strip_prefix("media:") else { return Ok(find(dir, src)) };
    let m = rows.iter().find(|t| t.kind == "media" && t.id == hash).ok_or(format!("{src}: no such row in media"))?;
    // the portable decoders read the row's edit mezzanine when it has one (the platform's read the original)
    let portable = matches!(media_backend().as_str(), "soft" | "av1");
    let (pcol, bcol) = match (portable, m.vals.get("mezz")) {
        (true, Some(V::S(p))) if !p.is_empty() => ("mezz", "mezz_bytes"),
        _ => ("path", "bytes"),
    };
    // `src` reads as `path` (a row written by hand rather than by `media`)
    let path = match m.vals.get(pcol).or_else(|| m.vals.get("src")) {
        Some(V::S(p)) => find(dir, p),
        _ => return Err(format!("media {hash}: no path (INSERT INTO media (id, path) VALUES ('{hash}', 'takes/<take>/video.mov'))")),
    };
    let size = std::fs::metadata(&path).map_err(|e| format!("media {hash}: {}: {e}", path.display()))?.len();
    if let Some(b) = m.vals.get(bcol) {
        if b.num() as u64 != size {
            return Err(format!("media {hash}: {} has changed (size {size}, the row says {})", path.display(), b.num()));
        }
    }
    Ok(path)
}

/// A relative path from the edit's folder, or from a folder above it (a project's `takes/...` from `edits/x.rdb`);
/// the edit's folder when it's in none of them.
pub fn find(dir: &Path, p: &str) -> PathBuf {
    dir.ancestors().take(3).map(|d| d.join(p)).find(|f| f.exists()).unwrap_or_else(|| dir.join(p))
}

/// A clip's source rect for its box: the largest rect with the box's aspect, zoomed about the centre, then panned within
/// the slack (pan -1..1 on each axis; 0 centres). Returns its centre and size, in source pixels. The one cover rule every
/// renderer uses (the engine, gpuview, the sketch, previz); stills reframe a subject with the pan.
/// Generic over the float type so each renderer keeps its own arithmetic (the engine's UI clips are f32, the rest f64)
/// and every frame stays bit-identical.
pub fn cover<T: Real>(sw: T, sh: T, w: T, h: T, zoom: T, pan: (T, T)) -> (T, T, T, T) {
    let (cw, ch) = if sw / sh > w / h { (sh * w / h, sh) } else { (sw, sw * h / w) };
    let (cw, ch) = (cw / zoom, ch / zoom);
    (sw / T::TWO + pan.0 * (sw - cw) / T::TWO, sh / T::TWO + pan.1 * (sh - ch) / T::TWO, cw, ch)
}

/// f32 or f64, for `cover`.
pub trait Real: Copy + PartialOrd + std::ops::Add<Output = Self> + std::ops::Sub<Output = Self> + std::ops::Mul<Output = Self> + std::ops::Div<Output = Self> {
    const TWO: Self;
}

impl Real for f32 {
    const TWO: f32 = 2.0;
}

impl Real for f64 {
    const TWO: f64 = 2.0;
}

/// The media backend clips decode with: RAEA_MEDIA, else platform hardware where there is one, else portable software.
pub fn media_backend() -> String {
    std::env::var("RAEA_MEDIA").unwrap_or_else(|_| if cfg!(target_vendor = "apple") { "apple".into() } else { "soft".into() })
}

pub fn num(v: &BTreeMap<String, V>, k: &str, d: f32) -> f32 {
    v.get(k).map(|x| x.num() as f32).unwrap_or(d)
}

/// A number from a JSON spec (a row's JSON column: grass, volumes), else `d`.
pub fn json_f32(j: &serde_json::Value, k: &str, d: f32) -> f32 {
    j.get(k).and_then(serde_json::Value::as_f64).map_or(d, |x| x as f32)
}

pub fn flag(vals: &BTreeMap<String, V>, k: &str, d: bool) -> bool {
    vals.get(k).map(V::truthy).unwrap_or(d)
}

pub fn scale_of(vals: &BTreeMap<String, V>) -> Vec3 {
    let s = num(vals, "scale", 1.0);
    Vec3::new(num(vals, "scale_x", s), num(vals, "scale_y", s), num(vals, "scale_z", s))
}

/// A row's rotation: its quaternion (q_x q_y q_z q_w, what runtime/scene writes: docs/control.feature FC1d), or
/// rot_x rot_y rot_z (radians, YXZ) where a row was written that way
pub fn rot_of(vals: &BTreeMap<String, V>) -> Quat {
    if vals.contains_key("q_w") {
        return Quat::from_xyzw(num(vals, "q_x", 0.0), num(vals, "q_y", 0.0), num(vals, "q_z", 0.0), num(vals, "q_w", 1.0)).normalize();
    }
    Quat::from_euler(EulerRot::YXZ, num(vals, "rot_y", 0.0), num(vals, "rot_x", 0.0), num(vals, "rot_z", 0.0))
}

/// A mesh or model row's transform.
pub fn transform_of(vals: &BTreeMap<String, V>) -> Transform {
    Transform::from_xyz(num(vals, "x", 0.0), num(vals, "y", 0.0), num(vals, "z", 0.0)).with_rotation(rot_of(vals)).with_scale(scale_of(vals))
}

/// A row's render layers (`layers`: "0,2"), as Bevy's RenderLayers on its entity (a mesh, a light, the camera).
pub fn layers(world: &mut World, e: Entity, vals: &BTreeMap<String, V>) {
    if let Some(V::S(l)) = vals.get("layers") {
        let ls: Vec<usize> = l.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        world.entity_mut(e).insert(bevy::camera::visibility::RenderLayers::from_layers(&ls));
    }
}

/// Bevy's SystemSet for Scene3dPlugin's systems, to order a plugin's own systems before or after them.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Scene3dSystems;

/// Engine::new's frame in Update, in this order: the stage's start mark, the frame's rows (relational), the 3D they move
/// (scene3d_systems, also in Scene3dSystems), the clips, the page overlay, the frame's tag and the end mark. A module
/// orders its own systems by them, as `.after(FrameSystems::Rows).before(FrameSystems::Tag)`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameSystems {
    Start,
    Rows,
    Scene3d,
    Clips,
    Overlay,
    Tag,
    End,
}

/// A camera a pass made (a look's model cells, the composite): not the scene's.
#[derive(Component)]
pub struct LookCam;

/// Every entity under `e`, depth first.
pub fn descendants(e: Entity, kids: &Query<&Children>) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut stack = vec![e];
    while let Some(x) = stack.pop() {
        if let Ok(c) = kids.get(x) {
            for k in c.iter() {
                out.push(k);
                stack.push(k);
            }
        }
    }
    out
}

/// `s` with `from` replaced, which must appear exactly once.
pub fn replace_once(s: &str, from: &str, to: &str) -> String {
    assert_eq!(s.matches(from).count(), 1, "raea: expected `{from}` exactly once in a patched shader");
    s.replacen(from, to, 1)
}

/// Preview quality (RAEA_PREVIEW_QUALITY=1, from outside the process: a shell's preview, a measurement): smaller shadow
/// maps and environment filters.
pub fn preview() -> bool {
    std::env::var_os("RAEA_PREVIEW_QUALITY").is_some_and(|v| v != "0")
}

/// The clips' decoders, one per (source, slot): a slot per clip of the same source showing at once. Decoders answer
/// "the frame showing at t" by as-of lookup, so a frame doesn't depend on which frames were asked for before it.
#[cfg(feature = "native")]
pub struct Media {
    pub backend: Box<dyn raea_io_media::Backend>,
    pub decoders: HashMap<(String, usize), Box<dyn raea_io_media::VideoDecoder>>,
    pub dir: PathBuf,
}

/// A mesh's triangles as vertex indices: its index buffer three at a time, or its vertices in threes when it has none.
pub fn triangles(m: &Mesh, vertices: usize) -> Vec<[usize; 3]> {
    match m.indices() {
        Some(ix) => {
            let v: Vec<usize> = ix.iter().collect();
            v.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
        }
        None => (0..vertices / 3).map(|i| [3 * i, 3 * i + 1, 3 * i + 2]).collect(),
    }
}

/// What a load spawned besides the rows (overlay, tag), despawned by the next load; the rows go through the store.
#[derive(Component)]
pub struct Mirror;
