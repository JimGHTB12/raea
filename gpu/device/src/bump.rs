//! What a page's Vello scene uses of each bump buffer, counted by running Vello's coarse pipeline on the CPU
//! (vello_shaders' cpu stages, in the order vello's render.rs records them, as its own use_cpu engine runs them) on the
//! resolved encoding, glyphs included. No GPU work, so a scene past a buffer can't fault the GPU here: the stages
//! don't bounds-check, so the buffers are 4x the render's or Vello's own sizes, the larger, and a scene past even those
//! is reported as over.
//! RAEA_VELLO_BUMP_USE=1 prints a line per page drawn (lib.rs `draw`) against the sizes the GPU render has.

use std::cell::RefCell;
use vello::low_level::vello_encoding::{BumpAllocators, IndirectCount, RenderConfig, Resolver};
use vello_shaders::cpu::{self, CpuBinding};

/// The buffers' use and the sizes the GPU render gives them, in each buffer's elements.
pub struct Use {
    /// lines, binning, tiles, seg_counts, segments, ptcl, blend: (used, size)
    pub buffers: [(&'static str, u32, u32); 7],
}

impl Use {
    /// The fullest buffer: its name and its use as a fraction of its size.
    pub fn fullest(&self) -> (&'static str, f64) {
        self.buffers.iter().map(|&(n, u, s)| (n, u as f64 / s.max(1) as f64)).fold(("", 0.0), |a, b| if b.1 > a.1 { b } else { a })
    }

    pub fn line(&self) -> String {
        self.buffers.iter().map(|&(n, u, s)| format!("{n} {u}/{s} ({:.1}%)", 100.0 * u as f64 / s.max(1) as f64)).collect::<Vec<_>>().join(", ")
    }
}

/// `scene` at `w` x `h`: Err when it's past 4x its sizes (a stage indexed past its buffer).
pub fn measure(scene: &vello::Scene, w: u32, h: u32) -> Result<Use, String> {
    let mut packed = Vec::new();
    let (layout, _, _) = Resolver::new().resolve(scene.encoding(), &mut packed);
    if packed.is_empty() {
        packed.resize(4, u8::MAX);
    }
    // the GPU render's sizes (raea's, or Vello's own while redraw_full is on)
    let given = RenderConfig::new(&layout, w, h, &vello::peniko::Color::TRANSPARENT);
    let mut big = RenderConfig::new(&layout, w, h, &vello::peniko::Color::TRANSPARENT);
    // 4x the larger of the render's sizes and Vello's own, each
    let d = vello::low_level::vello_encoding::BumpAllocationSizes::default();
    let s = &given.gpu;
    let x4 = |a: u32, b: u32| 4 * a.max(b);
    let bin = x4(given.buffer_sizes.bin_data.len(), d.bin_data);
    let (g, b) = (&mut big.gpu, &mut big.buffer_sizes);
    g.lines_size = x4(s.lines_size, d.lines);
    g.binning_size = bin - layout.bin_data_start;
    g.tiles_size = x4(s.tiles_size, d.tiles);
    g.seg_counts_size = x4(s.seg_counts_size, d.seg_counts);
    g.segments_size = x4(s.segments_size, d.segments);
    g.blend_size = x4(s.blend_size, d.blend_spill);
    g.ptcl_size = x4(s.ptcl_size, d.ptcl);
    b.lines = vello::low_level::vello_encoding::BufferSize::new(g.lines_size);
    b.bin_data = vello::low_level::vello_encoding::BufferSize::new(bin);
    b.tiles = vello::low_level::vello_encoding::BufferSize::new(g.tiles_size);
    b.seg_counts = vello::low_level::vello_encoding::BufferSize::new(g.seg_counts_size);
    b.segments = vello::low_level::vello_encoding::BufferSize::new(g.segments_size);
    b.blend_spill = vello::low_level::vello_encoding::BufferSize::new(g.blend_size);
    b.ptcl = vello::low_level::vello_encoding::BufferSize::new(g.ptcl_size);
    let bump = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&big, &packed))).map_err(|_| "past 4x its bump sizes".to_string())?;
    // ptcl's words in use: every tile's initial allocation, then what coarse allocated past them
    let ptcl = s.width_in_tiles * s.height_in_tiles * vello::low_level::vello_encoding::PTCL_INITIAL_ALLOC + bump.ptcl;
    Ok(Use {
        buffers: [
            ("lines", bump.lines, s.lines_size),
            ("binning", bump.binning, s.binning_size),
            ("tiles", bump.tile, s.tiles_size),
            ("seg_counts", bump.seg_counts, s.seg_counts_size),
            ("segments", bump.segments, s.segments_size),
            ("ptcl", ptcl, s.ptcl_size),
            ("blend", bump.blend, s.blend_size),
        ],
    })
}

/// vello's render_encoding_coarse, its dispatches run here in order (the CPU path takes the small path-tag scan).
fn run(cfg: &RenderConfig, scene: &[u8]) -> BumpAllocators {
    let (s, wg) = (&cfg.buffer_sizes, &cfg.workgroup_counts);
    let buf = |bytes: u32| RefCell::new(vec![0u8; bytes as usize]);
    let config = bytemuck::bytes_of(&cfg.gpu);
    let (reduced, tagmonoid, path_bbox) = (buf(s.path_reduced.size_in_bytes()), buf(s.path_monoids.size_in_bytes()), buf(s.path_bboxes.size_in_bytes()));
    let (bump, lines, draw_reduced) = (buf(s.bump_alloc.size_in_bytes()), buf(s.lines.size_in_bytes()), buf(s.draw_reduced.size_in_bytes()));
    let (draw_monoid, info_bin, clip_inp) = (buf(s.draw_monoids.size_in_bytes()), buf(s.bin_data.size_in_bytes()), buf(s.clip_inps.size_in_bytes()));
    let (clip_el, clip_bic, clip_bbox) = (buf(s.clip_els.size_in_bytes()), buf(s.clip_bics.size_in_bytes()), buf(s.clip_bboxes.size_in_bytes()));
    let (draw_bbox, bin_header, path) = (buf(s.draw_bboxes.size_in_bytes()), buf(s.bin_headers.size_in_bytes()), buf(s.paths.size_in_bytes()));
    let (tile, indirect, seg_counts) = (buf(s.tiles.size_in_bytes()), buf(s.indirect_count.size_in_bytes()), buf(s.seg_counts.size_in_bytes()));
    let (ptcl, segments) = (buf(s.ptcl.size_in_bytes()), buf(s.segments.size_in_bytes()));
    let (c, sc) = (CpuBinding::Buffer(config), CpuBinding::Buffer(scene));
    let rw = CpuBinding::BufferRW;
    cpu::pathtag_reduce(wg.path_reduce.0, &[c, sc, rw(&reduced)]);
    cpu::pathtag_scan(wg.path_scan.0, &[c, sc, rw(&reduced), rw(&tagmonoid)]);
    cpu::bbox_clear(wg.bbox_clear.0, &[c, rw(&path_bbox)]);
    cpu::flatten(wg.flatten.0, &[c, sc, rw(&tagmonoid), rw(&path_bbox), rw(&bump), rw(&lines)]);
    cpu::draw_reduce(wg.draw_reduce.0, &[c, sc, rw(&draw_reduced)]);
    cpu::draw_leaf(wg.draw_leaf.0, &[c, sc, rw(&draw_reduced), rw(&path_bbox), rw(&draw_monoid), rw(&info_bin), rw(&clip_inp)]);
    if wg.clip_reduce.0 > 0 {
        cpu::clip_reduce(wg.clip_reduce.0, &[rw(&clip_inp), rw(&path_bbox), rw(&clip_bic), rw(&clip_el)]);
    }
    if wg.clip_leaf.0 > 0 {
        cpu::clip_leaf(wg.clip_leaf.0, &[c, rw(&clip_inp), rw(&path_bbox), rw(&clip_bic), rw(&clip_el), rw(&draw_monoid), rw(&clip_bbox)]);
    }
    cpu::binning(wg.binning.0, &[c, rw(&draw_monoid), rw(&path_bbox), rw(&clip_bbox), rw(&draw_bbox), rw(&bump), rw(&info_bin), rw(&bin_header)]);
    cpu::tile_alloc(wg.tile_alloc.0, &[c, sc, rw(&draw_bbox), rw(&bump), rw(&path), rw(&tile)]);
    cpu::path_count_setup(wg.path_count_setup.0, &[rw(&bump), rw(&indirect)]);
    let n = |b: &RefCell<Vec<u8>>| bytemuck::from_bytes::<IndirectCount>(&b.borrow()[..std::mem::size_of::<IndirectCount>()]).count_x;
    cpu::path_count(n(&indirect), &[c, rw(&bump), rw(&lines), rw(&path), rw(&tile), rw(&seg_counts)]);
    cpu::backdrop(wg.backdrop.0, &[c, rw(&bump), rw(&path), rw(&tile)]);
    cpu::coarse(wg.coarse.0, &[c, sc, rw(&draw_monoid), rw(&bin_header), rw(&info_bin), rw(&path), rw(&tile), rw(&bump), rw(&ptcl)]);
    cpu::path_tiling_setup(wg.path_tiling_setup.0, &[rw(&bump), rw(&indirect), rw(&ptcl)]);
    cpu::path_tiling(n(&indirect), &[rw(&bump), rw(&seg_counts), rw(&lines), rw(&path), rw(&tile), rw(&segments)]);
    *bytemuck::from_bytes::<BumpAllocators>(&bump.borrow()[..std::mem::size_of::<BumpAllocators>()])
}
