//! The MCP `ui` tool's answer (sdk/ui/conformance/conformance.feature, "What the window shows comes out as a tree"),
//! kept at the end of every frame: the tree a reader put in [`Tree`] this frame (src/page.rs reads the page's
//! document), the state beside it, and the `frame_png` readbacks asked for.
//!
//! Frames: the counter `input` answers (src/inject.rs). A tree answered at the end of frame N is what frame N laid out
//! and drew: the app renders each update itself (no pipelined render thread), so it's also the frame presented to the
//! render target.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use serde_json::{Value, json};

use crate::inject::{Frame, Pending, Target};
use crate::mcp::Answer;

pub fn r1(v: f32) -> f64 {
    ((v as f64) * 10.0).round() / 10.0
}

/// The tree's nodes in the order a reader expects: top to bottom, then left to right.
pub fn sort(nodes: &mut [Value]) {
    nodes.sort_by(|a, b| {
        let k = |n: &Value| (n["rect"]["y"].as_f64().unwrap_or(0.0), n["rect"]["x"].as_f64().unwrap_or(0.0));
        k(a).partial_cmp(&k(b)).unwrap_or(std::cmp::Ordering::Equal)
    });
}

pub fn panels(nodes: &[Value], scale: f32) -> Vec<Value> {
    // the regions a compositing substrate presents as textures (sdk/ui/conformance/conformance.feature: `panels`). A
    // page has none: every panel is laid out and drawn in the window's own pass, in the frame, so what's presented for
    // one is its box at the window's scale and it is never stale. Listed so a case about panels' freshness reads the
    // same here
    nodes
        .iter()
        .filter_map(|n| {
            let id = n["id"].as_str()?;
            let name = id.strip_prefix("panel.").or((id == "dock").then_some(id))?;
            let (w, h) = (n["rect"]["w"].as_f64()?, n["rect"]["h"].as_f64()?);
            Some(json!({"name": name, "shown": n["visible"], "rect": n["rect"], "texture": {"w": (w * scale as f64).round(), "h": (h * scale as f64).round(), "scale": scale}, "stale": false, "drawn": "window pass"}))
        })
        .collect()
}

/// What the host knows of the window on screen (an app's view of it), for `state`.
#[derive(Resource, Default)]
pub struct WindowFacts(pub Value);

/// This frame's tree, put here by whoever read it (src/page.rs, or an app's own reader), taken by `answer`.
#[derive(Resource, Default)]
pub struct Tree(pub Option<Value>);

/// What an app adds to `state`: its fields, over the host's.
#[derive(Resource, Default)]
pub struct Extra(pub Value);

/// The latest frame's tree, kept at the end of every frame: the `ui` method answers with it (src/brp.rs), so the
/// frame it reports is the one that laid out what it shows.
pub static LATEST: std::sync::Mutex<Option<Value>> = std::sync::Mutex::new(None);
/// The last few frames' trees, oldest first, so a reader polling from its own thread (the MCP bridge, src/mcp.rs) is
/// handed each frame in turn and never one later than the first that showed what it waits for: a BRP call is answered
/// at the end of the frame it arrives in, which would count a frame the tree had already shown as one more.
pub static TREES: (std::sync::Mutex<std::collections::VecDeque<Value>>, std::sync::Condvar) = (std::sync::Mutex::new(std::collections::VecDeque::new()), std::sync::Condvar::new());

/// The earliest kept tree of frame `after` or later, waiting up to `limit` for one.
pub fn tree_after(after: u64, limit: std::time::Duration) -> Option<Value> {
    let (m, cv) = &TREES;
    let at = |t: &Value| t["frame"].as_u64().unwrap_or(0);
    let g = m.lock().unwrap();
    let (g, _) = cv.wait_timeout_while(g, limit, |q| !q.back().is_some_and(|t| at(t) >= after)).unwrap();
    g.iter().find(|t| at(t) >= after).cloned()
}

/// The latest frame's state (the `state` method).
pub static STATE: std::sync::Mutex<Option<Value>> = std::sync::Mutex::new(None);

/// The window's scale: its own on screen, the image's headless.
pub fn scale(target: &Target, win: Option<&Window>) -> f32 {
    if target.windowed { win.map(|w| w.scale_factor()).unwrap_or(1.0) } else { target.scale }
}

/// The frame's tree and state, kept for the `ui` and `state` methods (src/brp.rs), and the `frame_png` readbacks
/// asked for, at its end.
#[allow(clippy::too_many_arguments)]
pub fn answer(
    mut pending: ResMut<Pending>,
    frame: Res<Frame>,
    target: Res<Target>,
    win: Query<&Window, With<bevy::window::PrimaryWindow>>,
    facts: Res<WindowFacts>,
    extra: Res<Extra>,
    mut tree: ResMut<Tree>,
    mut commands: Commands,
    mut waiting: ResMut<PngWaiting>,
) {
    let w = win.single().ok();
    let scale = scale(&target, w);
    if let Some(t) = tree.0.take() {
        {
            let mut kept = TREES.0.lock().unwrap();
            kept.push_back(t.clone());
            while kept.len() > 240 {
                kept.pop_front();
            }
            TREES.1.notify_all();
        }
        *LATEST.lock().unwrap() = Some(t);
    }
    let mut state = json!({
        "frame": frame.0,
        "window": {"w": w.map_or(0.0, |w| w.width()), "h": w.map_or(0.0, |w| w.height())},
        "on_screen": target.windowed,
        "scale": scale,
        "native_window": facts.0,
    });
    if let Some(o) = extra.0.as_object() {
        for (k, v) in o {
            state[k] = v.clone();
        }
    }
    *STATE.lock().unwrap() = Some(state);
    if !pending.png.is_empty() {
        let first = waiting.0.is_empty();
        waiting.0.extend(pending.png.drain(..).map(|tx| (tx, frame.0)));
        if first {
            let shot = if target.windowed { Screenshot::primary_window() } else { Screenshot::image(target.image.clone()) };
            commands.spawn(shot).observe(captured);
        }
    }
}

/// `frame_png` callers waiting for the render target's readback.
#[derive(Resource, Default)]
pub struct PngWaiting(pub Vec<(std::sync::mpsc::Sender<Answer>, u64)>);

fn captured(ev: On<ScreenshotCaptured>, mut waiting: ResMut<PngWaiting>) {
    let img = ev.image.clone();
    let (w, h) = (img.width(), img.height());
    let png = img.try_into_dynamic().ok().and_then(|d| {
        let mut out = std::io::Cursor::new(Vec::new());
        d.to_rgba8().write_to(&mut out, image::ImageFormat::Png).ok()?;
        Some(out.into_inner())
    });
    for (tx, f) in waiting.0.drain(..) {
        let _ = tx.send(match &png {
            Some(p) => Answer::Png(p.clone(), json!({"frame": f, "w": w, "h": h})),
            None => Answer::Err("the render target couldn't be read back as an image".into()),
        });
    }
}
