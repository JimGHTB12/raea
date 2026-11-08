//! The direct render's views: an edit's frames without the Bevy frame loop, on the GPU (gpuview.rs: the GPU pool,
//! the Blitz overlay and one composite.k dispatch a frame) or on the CPU (cpuview.rs, the same frames by the kernels'
//! native twins), and what they share with the engine's overlay: the 2D rows (rows.rs) drawn by Blitz (rows2d.rs),
//! clips' masks (masks.rs), and the paint device as Bevy's renderer with the frame pool's budget (device.rs).

pub mod cpuview;
mod device;
pub use device::*;
pub mod gpuview;
pub mod masks;
pub mod rows;
pub mod rows2d;

/// The camera's clear colour (its `bg`, #rrggbb or #rrggbbaa) as a premultiplied RGBA texel; black when it has none,
/// as raea-export's program clears under 2D pages (a fed video's program has no camera: its transparent parts were
/// raea's dark blue, 13,10,26, where a PNG of the same frame is black). An alpha under ff (an HTML page's canvas under
/// --alpha, #00000000) stays: it was read as opaque, so the clear canvas came out black (#32).
pub(crate) fn camera_bg(rows: &[raea_core_program::eval::Tup]) -> [u8; 4] {
    let hex = rows.iter().find(|t| t.kind == "camera").and_then(|t| match t.vals.get("bg") {
        Some(raea_core_program::ir::V::S(s)) => Some(s.trim_start_matches('#').to_string()),
        _ => None,
    });
    let b = |h: &str, i: usize| u8::from_str_radix(h.get(i..i + 2)?, 16).ok();
    let Some((rgb, a)) = hex.and_then(|h| Some(([b(&h, 0)?, b(&h, 2)?, b(&h, 4)?], if h.len() == 8 { b(&h, 6)? } else { 255 }))) else { return [0, 0, 0, 255] };
    let pre = |c: u8| ((c as u32 * a as u32 + 127) / 255) as u8;
    [pre(rgb[0]), pre(rgb[1]), pre(rgb[2]), a]
}
