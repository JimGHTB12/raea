//! The target's side of a frame: the view box a tagged row draws in, supersampling and its downsample, the render
//! world's diagnostics forwarded.

use super::*;

/// Bevy's render diagnostics (each pass's elapsed time, `render/...`), into the run's trace once a second.
#[cfg(feature = "native")]
pub(crate) fn forward_render_diagnostics(store: Res<bevy::diagnostic::DiagnosticsStore>, mut last: Local<Option<std::time::Instant>>) {
    if last.is_some_and(|t| t.elapsed().as_secs_f64() < 1.0) {
        return;
    }
    *last = Some(std::time::Instant::now());
    let passes: serde_json::Map<String, serde_json::Value> = store
        .iter()
        .filter(|d| d.path().as_str().starts_with("render/"))
        .filter_map(|d| d.smoothed().map(|v| (d.path().as_str().trim_start_matches("render/").to_string(), serde_json::json!((v * 1000.0).round() / 1000.0))))
        .collect();
    if !passes.is_empty() {
        raea_io_trace::event("bevy_passes", serde_json::json!({"ms": passes}));
    }
}

/// The Engine's target: the frame, and its tag row under it.
#[derive(Resource)]
pub(crate) struct TagRowTarget(pub(crate) Handle<Image>);

/// Engine::set_view_box's box, in target px.
#[derive(Resource, Default)]
pub(crate) struct ViewBox(pub(crate) Option<[u32; 4]>);

/// A camera drawing to the Engine's target draws a w x h Viewport of it, the frame's own, above the tag row: a view of
/// w x (h + 1) drew the 3D 0.14% larger and half a pixel low against a camera on a w x h target, a plain App's
/// (docs/blitz-bevy.feature B3 H10), and against the looks' cells. With a view box, its own view is the box's (the
/// target a sub view of it, offset by the box's place).
pub(crate) fn tag_row_view(t: Res<TagRowTarget>, images: Res<Assets<Image>>, view_box: Option<Res<ViewBox>>, mut q: Query<(&mut Camera, &RenderTarget)>) {
    let Some(img) = images.get(&t.0) else { return };
    let s = img.size();
    let view = match view_box.and_then(|v| v.0) {
        Some([x, y, w, h]) => bevy::camera::SubCameraView { full_size: UVec2::new(w, h), offset: Vec2::new(-(x as f32), -(y as f32)), size: s },
        None => bevy::camera::SubCameraView { full_size: UVec2::new(s.x, s.y - 1), offset: Vec2::ZERO, size: s },
    };
    for (mut c, rt) in &mut q {
        if c.viewport.is_some() { if c.sub_camera_view.is_some() { c.sub_camera_view = None; } continue; } // a camera with its own Viewport (a ViewportNode's) sees its viewport's shape, as Bevy sizes it, never the tag row
        if matches!(rt, RenderTarget::Image(i) if i.handle == t.0) && c.sub_camera_view != Some(view) {
            if view.offset == Vec2::ZERO { c.viewport = Some(bevy::camera::Viewport { physical_size: view.full_size, ..default() }) } else { c.sub_camera_view = Some(view) } // a w x h Viewport: Bevy's orthographic sub view takes its area from the w x (h + 1) target, 0.14% small (2d/sprite_slice), and a draw in clip space (a PhaseItem, custom_phase_item) maps to its h + 1 rows
        }
    }
}

/// RAEA_SS (2, 3 or 4; 1 without it): how many times the frame's size it's drawn at. Not live, and only with the
/// lossless readback, which the downsampling reads.
pub(crate) fn supersampling(live: bool) -> u32 {
    let ss = std::env::var("RAEA_SS").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(1).clamp(1, 4);
    if ss > 1 && (live || std::env::var("RAEA_READBACK").map(|v| v != "rgba").unwrap_or(true)) {
        eprintln!("raea: RAEA_SS needs RAEA_READBACK=rgba and a render, not a live view; drawing at 1x");
        return 1;
    }
    ss
}

#[cfg(feature = "native")]
/// sRGB bytes -> linear, as a table
pub(crate) fn srgb_to_linear_table() -> &'static [f32; 256] {
    static T: std::sync::OnceLock<[f32; 256]> = std::sync::OnceLock::new();
    T.get_or_init(|| std::array::from_fn(|i| {
        let c = i as f32 / 255.0;
        if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    }))
}

#[cfg(feature = "native")]
/// an RGBA frame w x h shrunk by `s`: each s x s block's mean, in linear light (averaging the encoded bytes would darken
/// every edge between light and dark), encoded back to sRGB; alpha averaged as it is
pub(crate) fn downsample(px: &[u8], w: usize, h: usize, s: usize) -> Vec<u8> {
    let lin = srgb_to_linear_table();
    let (ow, oh) = (w / s, h / s);
    let n = (s * s) as f32;
    let mut out = vec![0u8; ow * oh * 4];
    for y in 0..oh {
        for x in 0..ow {
            let mut acc = [0f32; 4];
            for dy in 0..s {
                let row = ((y * s + dy) * w + x * s) * 4;
                for dx in 0..s {
                    let i = row + dx * 4;
                    acc[0] += lin[px[i] as usize];
                    acc[1] += lin[px[i + 1] as usize];
                    acc[2] += lin[px[i + 2] as usize];
                    acc[3] += px[i + 3] as f32;
                }
            }
            let o = (y * ow + x) * 4;
            for c in 0..3 {
                let v = acc[c] / n;
                let e = if v <= 0.0031308 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 };
                out[o + c] = (e.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
            out[o + 3] = (acc[3] / n + 0.5) as u8;
        }
    }
    out
}
