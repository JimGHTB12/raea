//! The frame loop's own state: what a load spawned, the overlay and output resources, clips and their decoders, the
//! overlay's digest, the render flags, where the time goes (Stages) and the frame tag each readback carries.

use super::*;
#[cfg(feature = "native")]
use crate::store::RowKey;

/// A 2D row's props as installed (its constants: what its columns started as, the image's asset path).
#[derive(Component, Clone)]
pub struct Base2d(pub serde_json::Map<String, serde_json::Value>);

#[derive(Component)]
pub struct FrameTag;

/// `places` (as the rows stood before any went) dropped from `rows`, from the last back so none moves another.
pub fn drop_places<T>(rows: &mut Vec<T>, mut places: Vec<usize>) {
    places.sort_unstable_by(|a, b| b.cmp(a));
    places.dedup();
    for r in places {
        if r < rows.len() {
            rows.remove(r);
        }
    }
}

#[cfg(test)]
mod drop_places_tests {
    #[test]
    fn rows_dropped_together_take_only_their_own_places() {
        let mut rows = vec!["a", "b", "c", "d", "e"];
        // in any order, found before any went (remove_rows drops 375 words at once)
        super::drop_places(&mut rows, vec![1, 3]);
        assert_eq!(rows, ["a", "c", "e"]);
        let mut rows = vec!["a", "b", "c", "d", "e"];
        super::drop_places(&mut rows, vec![4, 0, 2, 2, 9]);
        assert_eq!(rows, ["b", "d"]);
    }
}

#[cfg(feature = "native")]
#[derive(Resource)]
pub struct Overlay(pub Handle<Image>);

/// Readbacks as they arrive (several can land in one update once frames are pipelined); each carries its tag.
#[derive(Resource, Default)]
pub struct Out(pub Vec<Vec<u8>>);

/// Which load the World is on: the tag carries it, so a readback in flight from before a move is never taken.
#[derive(Resource, Default, Clone, Copy)]
pub struct Gen(pub u8);

/// A clip row's picture: a texture the decoded frame is written into, under the Blitz overlay.
#[cfg(feature = "native")]
#[derive(Component)]
pub struct Clip {
    pub image: Handle<Image>,
    /// (source, slot, source time in microseconds) last written, so an update that asks again does no work
    pub last: Option<(String, usize, i64)>,
}

/// The overlay's state digest: every 2D row's values and whether it shows. A 2D row's pixels are a function of
/// these, so when the digest hasn't changed the Blitz overlay (style, layout, raster, upload) is reused as it is.
#[cfg(feature = "native")]
#[derive(Resource, Default)]
pub struct OverlayKey {
    pub last: Option<u64>,
    pub dirty: bool,
    /// overlay redraws and reuses since the load (for the bench)
    pub drawn: u64,
    pub reused: u64,
}

#[cfg(feature = "native")]
pub fn overlay_key(clock: Res<Clock>, generation: Res<Gen>, frame: Res<FrameRows>, mut key: ResMut<OverlayKey>, q: Query<(Entity, &RowKey), With<Base2d>>) {
    use std::hash::{Hash, Hasher};
    let mut rows: Vec<(&RowKey, BTreeMap<String, V>)> = q.iter().filter_map(|(e, k)| frame.vals(e).map(|v| (k, v.into_owned()))).collect();
    rows.sort_by(|a, b| (&a.0.kind, &a.0.id).cmp(&(&b.0.kind, &b.0.id)));
    let mut h = std::collections::hash_map::DefaultHasher::new();
    generation.0.hash(&mut h);
    for (row, v) in &rows {
        let v = v;
        let on = clock.0 >= num(v, "from", f32::MIN) as i64 && v.get("to").map(|t| clock.0 < t.num() as i64).unwrap_or(true);
        (&row.kind, &row.id, on).hash(&mut h);
        if on {
            format!("{v:?}").hash(&mut h);
        }
    }
    let d = h.finish();
    key.dirty = key.last != Some(d) || std::env::var_os("RAEA_NO_MEMO").is_some();
    key.last = Some(d);
    if key.dirty { key.drawn += 1 } else { key.reused += 1 }
}

#[cfg(feature = "native")]
pub fn overlay_dirty(key: Res<OverlayKey>) -> bool {
    key.dirty
}

/// Set by a render whose frames are kept as pictures (PNG stills or a folder of them): they come back as RGBA, never
/// through NV12, whose 4:2:0 chroma and limited range belong to the video encoder alone.
pub static STILLS_RGBA: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Where the time goes, accumulated over frames (G8's baseline): nanoseconds of CPU per stage.
#[derive(Resource, Default, Clone, Debug)]
pub struct Stages {
    pub rules: u64,
    /// clips: decode (frame_at), of which conversion is counted by raea_io_media::CONVERT_NS
    pub clips: u64,
    /// writing decoded pixels into the clips' textures (the CPU side of the upload)
    pub clip_prep: u64,
    /// the Blitz overlay: rows, style, layout, raster, unpremultiply and upload (only when its digest changed)
    pub overlay: u64,
    #[cfg(feature = "native")]
    pub overlay_t0: Option<std::time::Instant>,
    /// the main world's Update, first system to last
    pub main: u64,
    pub main_t0: Option<std::time::Instant>,
    /// whole app.update() (main world + render world: extract, prepare incl. texture uploads, queue, submit)
    pub update: u64,
    /// copying readbacks out of the padded rows
    pub readback_copy: u64,
    /// serial probe (RAEA_SERIAL): waiting for the GPU to finish after each update
    pub gpu_wait: u64,
    pub updates: u64,
    pub frames: u64,
}

pub fn mark_main_start(mut st: ResMut<Stages>) {
    st.main_t0 = Some(std::time::Instant::now());
}

pub fn mark_main_end(mut st: ResMut<Stages>) {
    if let Some(t) = st.main_t0.take() {
        st.main += t.elapsed().as_nanos() as u64;
    }
}

#[cfg(feature = "native")]
pub fn mark_overlay_start(mut st: ResMut<Stages>) {
    st.overlay_t0 = Some(std::time::Instant::now());
}

#[cfg(feature = "native")]
pub fn mark_overlay_end(mut st: ResMut<Stages>) {
    if let Some(t) = st.overlay_t0.take() {
        st.overlay += t.elapsed().as_nanos() as u64;
    }
}

pub fn tag_colour(f: i64, generation: u8) -> (u8, u8, u8) {
    ((f & 0xff) as u8, ((f >> 8) & 0xff) as u8, generation)
}

/// An update drawing the frame before only to set motion blur's previous poses: its readback carries generation 0,
/// which no load has, so nothing takes it for a frame.
#[derive(Resource, Default)]
pub struct PreStep(pub bool);

pub fn tag(clock: Res<Clock>, generation: Res<Gen>, pre: Res<PreStep>, mut q: Query<&mut BackgroundColor, With<FrameTag>>) {
    let (r, g, b) = tag_colour(clock.0, if pre.0 { 0 } else { generation.0 });
    for mut c in &mut q {
        *c = BackgroundColor(Color::srgb_u8(r, g, b));
    }
}
