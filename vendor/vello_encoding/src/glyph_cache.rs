// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::collections::HashMap;
use std::sync::Arc;

use super::{Encoding, FontEmbolden, StreamOffsets};

use peniko::{
    FontData, Style,
    kurbo::{BezPath, Join, Point},
};
use skrifa::instance::{NormalizedCoord, Size};
use skrifa::outline::{HintingInstance, HintingOptions, OutlineGlyphFormat, OutlinePen};
use skrifa::{GlyphId, MetadataProvider, OutlineGlyphCollection};

#[derive(Default)]
pub(crate) struct GlyphCache {
    free_list: Vec<Arc<Encoding>>,
    map: GlyphMap,
    var_map: HashMap<VarKey, GlyphMap>,
    cached_count: usize,
    hinting: HintCache,
    /// raea: each face, size and variation location's y map from the GlyphTops hook (None: drawn unhinted), when the
    /// hook is set
    y_maps: HashMap<(u64, u32, u32), HashMap<Box<[NormalizedCoord]>, Option<YMap>>>,
    serial: u64,
    last_prune_serial: u64,
}

impl GlyphCache {
    pub(crate) fn session<'a>(
        &'a mut self,
        font: &'a FontData,
        coords: &'a [NormalizedCoord],
        size: f32,
        embolden: FontEmbolden,
        hint: bool,
        style: &'a Style,
    ) -> Option<GlyphCacheSession<'a>> {
        let font_id = font.data.id();
        let font_index = font.index;
        let font_data = font.data.as_ref();
        let font = skrifa::FontRef::from_index(font_data, font.index).ok()?;
        let map = if !coords.is_empty() {
            // This is still ugly in rust. Choices are:
            // 1. multiple lookups in the hashmap (implemented here)
            // 2. always allocate and copy the key
            // 3. use unsafe
            // Pick 1 bad option :(
            if self.var_map.contains_key(coords) {
                self.var_map.get_mut(coords).unwrap()
            } else {
                self.var_map.entry(coords.into()).or_default()
            }
        } else {
            &mut self.map
        };
        let outlines = font.outline_glyphs();
        let size = Size::new(size);
        // raea: with the GlyphTops hook set, a hinted run is drawn through its face's y map at its size and variation
        // location, never by skrifa's hinter; one the hook gives no map for is drawn unhinted
        let tops = GLYPH_TOPS.get().filter(|_| hint);
        let y_map = match tops {
            Some(&tops) => {
                let maps = self.y_maps.entry((font_id, font_index, size.ppem().unwrap_or(0.0).to_bits())).or_default();
                match maps.get(coords) {
                    Some(map) => *map,
                    None => {
                        let map = y_map(tops, font_id, font_data, font_index, &outlines, size, coords);
                        maps.insert(coords.into(), map);
                        map
                    }
                }
            }
            None => None,
        };
        let hinter = if hint && tops.is_none() {
            let key = HintKey {
                font_id,
                font_index,
                outlines: &outlines,
                size,
                coords,
            };
            self.hinting.get(&key)
        } else {
            None
        };
        // TODO: we're ignoring dashing for now
        let style_bits = match style {
            Style::Fill(fill) => super::path::Style::from_fill(*fill),
            Style::Stroke(stroke) => super::path::Style::from_stroke(stroke)?,
        };
        let style_bits: [u32; 2] = bytemuck::cast(style_bits);
        Some(GlyphCacheSession {
            free_list: &mut self.free_list,
            map,
            font_id,
            font_index,
            coords,
            size,
            size_bits: size.ppem().unwrap().to_bits(),
            embolden,
            style,
            style_bits,
            outlines,
            hinter,
            y_map,
            outline_buf: BezPath::new(),
            serial: self.serial,
            cached_count: &mut self.cached_count,
        })
    }

    pub(crate) fn maintain(&mut self) {
        // Maximum number of resolve phases where we'll retain an unused glyph
        const MAX_ENTRY_AGE: u64 = 64;
        // Maximum number of resolve phases before we force a prune
        const PRUNE_FREQUENCY: u64 = 64;
        // Always prune if the cached count is greater than this value
        const CACHED_COUNT_THRESHOLD: usize = 256;
        // Number of encoding buffers we'll keep on the free list
        const MAX_FREE_LIST_SIZE: usize = 32;
        let free_list = &mut self.free_list;
        let serial = self.serial;
        self.serial += 1;
        // Don't iterate over the whole cache every frame
        if serial - self.last_prune_serial < PRUNE_FREQUENCY
            && self.cached_count < CACHED_COUNT_THRESHOLD
        {
            return;
        }
        self.last_prune_serial = serial;
        self.map.retain(|_, entry| {
            if serial - entry.serial > MAX_ENTRY_AGE {
                if free_list.len() < MAX_FREE_LIST_SIZE {
                    free_list.push(entry.encoding.clone());
                }
                self.cached_count -= 1;
                false
            } else {
                true
            }
        });
        self.var_map.retain(|_, map| {
            map.retain(|_, entry| {
                if serial - entry.serial > MAX_ENTRY_AGE {
                    if free_list.len() < MAX_FREE_LIST_SIZE {
                        free_list.push(entry.encoding.clone());
                    }
                    self.cached_count -= 1;
                    false
                } else {
                    true
                }
            });
            !map.is_empty()
        });
    }
}

pub(crate) struct GlyphCacheSession<'a> {
    free_list: &'a mut Vec<Arc<Encoding>>,
    map: &'a mut GlyphMap,
    font_id: u64,
    font_index: u32,
    coords: &'a [NormalizedCoord],
    size: Size,
    size_bits: u32,
    embolden: FontEmbolden,
    style: &'a Style,
    style_bits: [u32; 2],
    outlines: OutlineGlyphCollection<'a>,
    hinter: Option<&'a HintingInstance>,
    y_map: Option<YMap>,
    outline_buf: BezPath,
    serial: u64,
    cached_count: &'a mut usize,
}

impl GlyphCacheSession<'_> {
    pub(crate) fn get_or_insert(
        &mut self,
        glyph_id: u32,
    ) -> Option<(Arc<Encoding>, StreamOffsets)> {
        let key = GlyphKey {
            font_id: self.font_id,
            font_index: self.font_index,
            glyph_id,
            font_size_bits: self.size_bits,
            embolden_x_bits: f32_bits(self.embolden.amount.xx),
            embolden_y_bits: f32_bits(self.embolden.amount.yy),
            embolden_join_bits: join_bits(self.embolden.join),
            embolden_miter_limit_bits: f32_bits(self.embolden.miter_limit),
            embolden_tolerance_bits: f32_bits(self.embolden.tolerance),
            style_bits: self.style_bits,
            hint: self.hinter.is_some() || self.y_map.is_some(),
        };
        if let Some(entry) = self.map.get_mut(&key) {
            entry.serial = self.serial;
            return Some((entry.encoding.clone(), entry.stream_sizes));
        }
        let outline = self.outlines.get(GlyphId::new(key.glyph_id))?;
        let mut encoding = self.free_list.pop().unwrap_or_default();
        let encoding_ptr = Arc::make_mut(&mut encoding);
        encoding_ptr.reset();
        let is_fill = match &self.style {
            Style::Fill(fill) => {
                encoding_ptr.encode_fill_style(*fill);
                true
            }
            Style::Stroke(stroke) => {
                let encoded_stroke = encoding_ptr.encode_stroke_style(stroke);
                debug_assert!(encoded_stroke, "Stroke width is non-zero");
                false
            }
        };
        use skrifa::outline::DrawSettings;
        let draw_settings = if key.hint {
            if let Some(hinter) = self.hinter {
                DrawSettings::hinted(hinter, false)
            } else {
                DrawSettings::unhinted(self.size, self.coords)
            }
        } else {
            DrawSettings::unhinted(self.size, self.coords)
        };
        let n_path_segments = if self.embolden.amount != peniko::kurbo::Diagonal2::new(0.0, 0.0) {
            self.outline_buf.truncate(0);
            let mut path = BezPathOutline(&mut self.outline_buf);
            match self.y_map {
                Some(map) => outline.draw(draw_settings, &mut YMapPen { pen: &mut path, map }).ok()?,
                None => outline.draw(draw_settings, &mut path).ok()?,
            };
            let path = peniko::kurbo::expand_path(
                &self.outline_buf,
                self.embolden.amount,
                self.embolden.join,
                self.embolden.miter_limit,
                self.embolden.tolerance,
            );
            let mut encoder = encoding_ptr.encode_path(is_fill);
            encoder.path_elements(path.elements().iter().copied());
            encoder.finish(false)
        } else {
            let mut path = encoding_ptr.encode_path(is_fill);
            match self.y_map {
                Some(map) => outline.draw(draw_settings, &mut YMapPen { pen: &mut path, map }).ok()?,
                None => outline.draw(draw_settings, &mut path).ok()?,
            };
            path.finish(false)
        };
        if n_path_segments == 0 {
            encoding_ptr.reset();
        }
        let stream_sizes = encoding_ptr.stream_offsets();
        self.map.insert(
            key,
            GlyphEntry {
                encoding: encoding.clone(),
                stream_sizes,
                serial: self.serial,
            },
        );
        *self.cached_count += 1;
        Some((encoding, stream_sizes))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default, Debug)]
struct GlyphKey {
    font_id: u64,
    font_index: u32,
    glyph_id: u32,
    font_size_bits: u32,
    embolden_x_bits: u32,
    embolden_y_bits: u32,
    embolden_join_bits: u8,
    embolden_miter_limit_bits: u32,
    embolden_tolerance_bits: u32,
    style_bits: [u32; 2],
    hint: bool,
}

#[inline(always)]
fn join_bits(join: Join) -> u8 {
    match join {
        Join::Bevel => 0,
        Join::Miter => 1,
        Join::Round => 2,
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "Cache keys intentionally store embolden parameters at f32 precision."
)]
#[inline(always)]
fn f32_bits(value: f64) -> u32 {
    (value as f32).to_bits()
}

struct BezPathOutline<'a>(&'a mut BezPath);

impl OutlinePen for BezPathOutline<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(Point::new(x.into(), y.into()));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(Point::new(x.into(), y.into()));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.0.quad_to(
            Point::new(cx0.into(), cy0.into()),
            Point::new(x.into(), y.into()),
        );
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.0.curve_to(
            Point::new(cx0.into(), cy0.into()),
            Point::new(cx1.into(), cy1.into()),
            Point::new(x.into(), y.into()),
        );
    }

    fn close(&mut self) {
        self.0.close_path();
    }
}

/// Outer level key for variable font caches.
///
/// Inline size of 8 maximizes the internal storage of the small vec.
type VarKey = smallvec::SmallVec<[NormalizedCoord; 8]>;

type GlyphMap = HashMap<GlyphKey, GlyphEntry>;

#[derive(Clone, Default)]
struct GlyphEntry {
    encoding: Arc<Encoding>,
    stream_sizes: StreamOffsets,
    /// Last use of this entry.
    serial: u64,
}

/// We keep this small to enable a simple LRU cache with a linear
/// search. Regenerating hinting data is low to medium cost so it's fine
/// to redo it occasionally.
const MAX_CACHED_HINT_INSTANCES: usize = 8;

pub(crate) struct HintKey<'a> {
    font_id: u64,
    font_index: u32,
    outlines: &'a OutlineGlyphCollection<'a>,
    size: Size,
    coords: &'a [NormalizedCoord],
}

impl HintKey<'_> {
    fn instance(&self) -> Option<HintingInstance> {
        HintingInstance::new(self.outlines, self.size, self.coords, HINTING_OPTIONS).ok()
    }
}

// TODO: We might want to expose making these configurable in future.
// However, these options are probably fine for most users.
const HINTING_OPTIONS: HintingOptions = HintingOptions {
    engine: skrifa::outline::Engine::AutoFallback,
    target: skrifa::outline::Target::Smooth {
        mode: skrifa::outline::SmoothMode::Lcd,
        symmetric_rendering: false,
        preserve_linear_metrics: true,
    },
};

#[derive(Default)]
struct HintCache {
    // Split caches for glyf/cff because the instance type can reuse
    // internal memory when reconfigured for the same format.
    glyf_entries: Vec<HintEntry>,
    cff_entries: Vec<HintEntry>,
    varc_entries: Vec<HintEntry>,
    serial: u64,
}

impl HintCache {
    fn get(&mut self, key: &HintKey<'_>) -> Option<&HintingInstance> {
        let entries = match key.outlines.format()? {
            OutlineGlyphFormat::Glyf => &mut self.glyf_entries,
            OutlineGlyphFormat::Cff | OutlineGlyphFormat::Cff2 => &mut self.cff_entries,
            OutlineGlyphFormat::Varc => &mut self.varc_entries,
        };
        let (entry_ix, is_current) = find_hint_entry(entries, key)?;
        let entry = entries.get_mut(entry_ix)?;
        self.serial += 1;
        entry.serial = self.serial;
        if !is_current {
            entry.font_id = key.font_id;
            entry.font_index = key.font_index;
            entry
                .instance
                .reconfigure(key.outlines, key.size, key.coords, HINTING_OPTIONS)
                .ok()?;
        }
        Some(&entry.instance)
    }
}

struct HintEntry {
    font_id: u64,
    font_index: u32,
    instance: HintingInstance,
    serial: u64,
}

fn find_hint_entry(entries: &mut Vec<HintEntry>, key: &HintKey<'_>) -> Option<(usize, bool)> {
    let mut found_serial = u64::MAX;
    let mut found_index = 0;
    for (ix, entry) in entries.iter().enumerate() {
        if entry.font_id == key.font_id
            && entry.font_index == key.font_index
            && entry.instance.size() == key.size
            && entry.instance.location().coords() == key.coords
        {
            return Some((ix, true));
        }
        if entry.serial < found_serial {
            found_serial = entry.serial;
            found_index = ix;
        }
    }
    if entries.len() < MAX_CACHED_HINT_INSTANCES {
        let instance = key.instance()?;
        let ix = entries.len();
        entries.push(HintEntry {
            font_id: key.font_id,
            font_index: key.font_index,
            instance,
            serial: 0,
        });
        Some((ix, true))
    } else {
        Some((found_index, false))
    }
}

/// raea: where the platform's rasterizer puts two reference heights of a face at a size: given the face's data (its
/// id, the data, its index in a collection), the size in px, a variable face's location as user-space values by axis
/// tag (empty for a static face or the default location), and the glyph ids of `o` and `H`, the tops of the two as
/// it draws them, in px above the baseline. CoreGraphics moves a TrueType face's glyphs in y at small whole sizes so
/// that, roughly, the round x-height and the cap height land on whole rows, by a rule of its own per face; Chrome on
/// macOS draws through it. None leaves the face unhinted at that size.
pub type GlyphTops = fn(u64, &[u8], u32, f32, &[(u32, f32)], [u32; 2]) -> Option<[f32; 2]>;

static GLYPH_TOPS: std::sync::OnceLock<GlyphTops> = std::sync::OnceLock::new();

/// raea: sets the process's GlyphTops hook, once, before the first render; false when one was already set. Hinted
/// glyph runs are then drawn through each face's y map instead of skrifa's hinting.
pub fn set_glyph_tops(tops: GlyphTops) -> bool {
    GLYPH_TOPS.set(tops).is_ok()
}

/// raea: an outline's y in px through two knots (from, to), the lower first, from (0, 0): linear between them and
/// past the upper one on the last segment's slope, and below the baseline on the first's.
#[derive(Clone, Copy, Debug)]
struct YMap([(f32, f32); 2]);

impl YMap {
    fn y(&self, y: f32) -> f32 {
        let [(a0, b0), (a1, b1)] = self.0;
        if y <= a0 { y * b0 / a0 } else { b0 + (y - a0) * (b1 - b0) / (a1 - a0) }
    }
}

/// raea: a face's y map at `size` and `coords`: its own unhinted tops of `o` and `H` taken to where the hook says
/// they're drawn.
fn y_map(
    tops: GlyphTops,
    font_id: u64,
    data: &[u8],
    index: u32,
    outlines: &OutlineGlyphCollection<'_>,
    size: Size,
    coords: &[NormalizedCoord],
) -> Option<YMap> {
    let font = skrifa::FontRef::from_index(data, index).ok()?;
    let location = user_location(&font, coords);
    let charmap = font.charmap();
    let ids = [charmap.map('o')?, charmap.map('H')?];
    let top = |id: GlyphId| {
        let mut path = BezPath::new();
        outlines.get(id)?.draw(skrifa::outline::DrawSettings::unhinted(size, skrifa::instance::LocationRef::new(coords)), &mut BezPathOutline(&mut path)).ok()?;
        Some(peniko::kurbo::Shape::bounding_box(&path).y1 as f32)
    };
    let own = [top(ids[0])?, top(ids[1])?];
    let drawn = tops(font_id, data, index, size.ppem()?, &location, [ids[0].to_u32(), ids[1].to_u32()])?;
    let mut knots = [(own[0], drawn[0]), (own[1], drawn[1])];
    knots.sort_by(|a, b| a.0.total_cmp(&b.0));
    // tops too close to divide between, or not above the baseline: one scale from the higher
    if !(knots[0].0 > 0.5 && knots[0].1 > 0.5) {
        return None;
    }
    if knots[1].0 - knots[0].0 < 0.5 {
        let s = knots[1].1 / knots[1].0;
        knots[0] = (knots[1].0 * 0.5, knots[1].0 * 0.5 * s);
    }
    Some(YMap(knots))
}

/// raea: a variable face's normalized location in user space, (axis tag, value) for each axis off its default: each
/// axis's value is found by bisection on its own normalization (fvar's, then avar's segment maps), which only rises
fn user_location(font: &skrifa::FontRef<'_>, coords: &[NormalizedCoord]) -> Vec<(u32, f32)> {
    let axes = font.axes();
    axes.iter()
        .enumerate()
        .filter_map(|(i, axis)| {
            let want = coords.get(i).copied().filter(|c| c.to_bits() != 0)?.to_f32();
            let at = |v: f32| axes.location([(axis.tag(), v)]).coords().get(i).map_or(0.0, |c| c.to_f32());
            let (mut lo, mut hi) = (axis.min_value(), axis.max_value());
            for _ in 0..32 {
                let mid = (lo + hi) / 2.0;
                if at(mid) < want { lo = mid } else { hi = mid }
            }
            Some((u32::from_be_bytes(axis.tag().to_be_bytes()), (lo + hi) / 2.0))
        })
        .collect()
}

/// raea: a pen that hands its outline on through a YMap
struct YMapPen<'p, P: OutlinePen> {
    pen: &'p mut P,
    map: YMap,
}

impl<P: OutlinePen> OutlinePen for YMapPen<'_, P> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pen.move_to(x, self.map.y(y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.pen.line_to(x, self.map.y(y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.pen.quad_to(cx0, self.map.y(cy0), x, self.map.y(y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.pen.curve_to(cx0, self.map.y(cy0), cx1, self.map.y(cy1), x, self.map.y(y));
    }

    fn close(&mut self) {
        self.pen.close();
    }
}
