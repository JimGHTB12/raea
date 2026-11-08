//! The parity probe (docs/control.feature FC0a). The same plugin goes into a Bevy example's App (bench/bevy-parity's
//! host) and into raea's (raea_plugins.rs), so the two runs of a scene are measured by one piece of code. Off unless
//! RAEA_PARITY is set:
//!
//!   RAEA_PARITY=FILE.jsonl     one JSON line per probed frame (below)
//!   RAEA_PARITY_FRAMES=30,60   the frames probed, counted from the App's first update ("all": every frame; default all)
//!   RAEA_PARITY_PNG=DIR        also the primary window's picture at each probed frame, as DIR/FRAME.png
//!   RAEA_PARITY_EXIT=1         exit a few frames after the last probed frame (the screenshot's readback lands first)
//!   RAEA_PARITY_TYPES=A,B      also each listed Bevy type's value on every entity that has it, reflected as Bevy
//!                              serializes it, one line a probed frame in FILE.world.jsonl (FC0c's defaults test)
//!   RAEA_PARITY_INPUT=FILE     scripted input, the script `raea play --input-script` takes (input.rs)
//!
//! A line holds what decides a frame's cost, read from the main World:
//!   entities          every entity
//!   mesh_entities     entities with a Mesh3d; `visible` those whose ViewVisibility is set this frame
//!   meshes, materials the Mesh and StandardMaterial assets held
//!   batch_keys        distinct (mesh, standard material) handle pairs among the visible: with Bevy's automatic batching
//!                     (and GPU preprocessing) a pair is about one draw, so this stands in for the draw-call count Bevy
//!                     doesn't report; `other_material` counts visible meshes drawn by another material type
//!   extents           the distinct half-extents of the visible meshes' bounds (up to 8), a check on what was drawn
//!   frame_ms          real time per frame, the mean since the line before
//! Wall time is the machine's, so compare it between runs of one machine only.

use std::collections::{BTreeSet, HashSet};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::ViewVisibility;
use bevy::prelude::*;
use bevy::render::view::window::screenshot::{Screenshot, save_to_disk};
use bevy::time::TimeUpdateStrategy;

mod input;

#[derive(Resource)]
struct Probe {
    out: PathBuf,
    frames: Option<BTreeSet<u64>>,
    png: Option<PathBuf>,
    exit: bool,
    types: Vec<String>,
    frame: u64,
    since: Instant,
    ticks: u32,
}

/// Adds the probe when RAEA_PARITY is set: the example host's entry and RaeaPlugins' (raea_plugins.rs).
pub fn install(app: &mut App) {
    let Ok(out) = std::env::var("RAEA_PARITY") else { return };
    let frames: Option<BTreeSet<u64>> = match std::env::var("RAEA_PARITY_FRAMES").ok().as_deref() {
        None | Some("all") | Some("") => None,
        Some(s) => Some(s.split(',').filter_map(|f| f.trim().parse().ok()).collect()),
    };
    let png = std::env::var("RAEA_PARITY_PNG").ok().map(PathBuf::from);
    if let Some(d) = &png {
        let _ = std::fs::create_dir_all(d);
        // an earlier run's pictures would end RAEA_PARITY_EXIT's wait for this run's
        for k in frames.iter().flatten() {
            let _ = std::fs::remove_file(d.join(format!("{k}.png")));
        }
    }
    let exit = std::env::var("RAEA_PARITY_EXIT").is_ok_and(|v| v == "1");
    let types: Vec<String> = std::env::var("RAEA_PARITY_TYPES")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    if !types.is_empty() {
        let _ = std::fs::write(world_file(&out), "");
    }
    let _ = std::fs::write(&out, "");
    if png.is_some() {
        app.add_systems(First, offscreen);
        app.add_systems(PreUpdate, no_input.after(bevy::input::InputSystems));
    }
    if let Some(script) = input::load() {
        app.insert_resource(script);
        app.add_systems(First, input::scripted);
    }
    app.insert_resource(Probe { out: out.into(), frames, png, exit, types, frame: 0, since: Instant::now(), ticks: 0 });
    app.add_systems(Last, (probe, dump).chain());
}

/// The picture the window's cameras draw into instead of its swapchain, when the example's pictures are taken
/// (RAEA_PARITY_PNG): a window shows black once macOS stops compositing it (an unattended Mac mini's display asleep,
/// 2026-11-07: frames after the first second came back black), an image never does. Same size and format the window
/// would have. The UI's camera is the window's would be (bevy_ui's DefaultUiCamera): the one the example marks
/// IsDefaultUiCamera, else the highest order of the window's cameras; the marker is added only when the example gave none.
#[derive(Resource)]
struct Offscreen(Handle<Image>);

fn offscreen(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cams: Query<(Entity, &bevy::camera::RenderTarget, &bevy::camera::Camera), Added<bevy::camera::RenderTarget>>,
    marked: Query<(), With<bevy::ui::IsDefaultUiCamera>>,
    held: Option<Res<Offscreen>>,
) {
    let Ok(w) = window.single() else { return };
    let mut target = held.map(|h| h.0.clone());
    let mut ui: Option<(isize, Entity)> = None;
    for (e, t, cam) in &cams {
        if !matches!(t, bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Primary)) {
            continue;
        }
        let first = target.is_none();
        let h = target.get_or_insert_with(|| {
            let h = images.add(Image::new_target_texture(w.physical_width(), w.physical_height(), bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, None));
            commands.insert_resource(Offscreen(h.clone()));
            h
        });
        commands.entity(e).insert(bevy::camera::RenderTarget::Image(h.clone().into()));
        if (first || ui.is_some()) && ui.is_none_or(|(o, _)| cam.order > o) {
            ui = Some((cam.order, e));
        }
    }
    if let Some((_, e)) = ui.filter(|_| marked.is_empty()) {
        commands.entity(e).insert(bevy::ui::IsDefaultUiCamera);
    }
}

/// No real input while the pictures are taken: a run must repeat, and a mouse moved over the Mac (2026-11-06:
/// camera_orbit's camera swung by AccumulatedMouseMotion) would steer the example. Cleared after Bevy's input systems
/// fill them, before the example's Update reads them.
fn no_input(
    motion: Option<ResMut<bevy::input::mouse::AccumulatedMouseMotion>>,
    scroll: Option<ResMut<bevy::input::mouse::AccumulatedMouseScroll>>,
    buttons: Option<ResMut<ButtonInput<MouseButton>>>,
    keys: Option<ResMut<ButtonInput<KeyCode>>>,
    mut windows: Query<&mut Window>,
) {
    // and no cursor in the window: the Mac's pointer resting over it turned rotate_to_cursor's ship (11-07)
    for mut w in &mut windows {
        if w.cursor_position().is_some() {
            w.set_cursor_position(None);
        }
    }
    if let Some(mut m) = motion {
        *m = default();
    }
    if let Some(mut s) = scroll {
        *s = default();
    }
    if let Some(mut b) = buttons {
        b.reset_all();
    }
    if let Some(mut k) = keys {
        k.reset_all();
    }
}

/// A fixed clock of `fps` frames a second (Bevy's TimeUpdateStrategy::ManualDuration), for a run that must repeat.
pub fn fixed_clock(app: &mut App, fps: f64) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / fps)));
}

#[allow(clippy::type_complexity)]
fn probe(
    offscreen: Option<Res<Offscreen>>,
    mut p: ResMut<Probe>,
    all: Query<Entity>,
    meshes_q: Query<(&Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>, Option<&ViewVisibility>, Option<&Aabb>)>,
    meshes: Option<Res<Assets<Mesh>>>,
    materials: Option<Res<Assets<StandardMaterial>>>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    let f = p.frame;
    p.frame += 1;
    p.ticks += 1;
    let last = p.frames.as_ref().and_then(|s| s.last().copied());
    // a screenshot's readback lands some frames after it's asked for (save_to_disk writes it on the main thread), so
    // the run ends once every probed frame's picture is on disk, or 600 frames on without it
    if p.exit && last.is_some_and(|l| f >= l + 4) {
        let saved = |d: &PathBuf| p.frames.iter().flatten().all(|k| d.join(format!("{k}.png")).exists());
        if p.png.as_ref().is_none_or(saved) || last.is_some_and(|l| f >= l + 600) {
            exit.write(AppExit::Success);
        }
        return;
    }
    if p.frames.as_ref().is_some_and(|s| !s.contains(&f)) {
        return;
    }
    let frame_ms = p.since.elapsed().as_secs_f64() * 1000.0 / p.ticks.max(1) as f64;
    p.since = Instant::now();
    p.ticks = 0;
    let (mut mesh_entities, mut visible, mut other) = (0u64, 0u64, 0u64);
    let mut keys = HashSet::new();
    let mut extents = BTreeSet::new();
    for (mesh, mat, vis, aabb) in &meshes_q {
        mesh_entities += 1;
        if !vis.is_some_and(|v| v.get()) {
            continue;
        }
        visible += 1;
        match mat {
            Some(m) => {
                keys.insert((mesh.0.id(), m.0.id()));
            }
            None => other += 1,
        }
        if let Some(a) = aabb
            && extents.len() < 8
        {
            let h = a.half_extents;
            extents.insert(format!("{:.4} {:.4} {:.4}", h.x, h.y, h.z));
        }
    }
    let line = serde_json::json!({
        "frame": f,
        "entities": all.iter().count(),
        "mesh_entities": mesh_entities,
        "visible": visible,
        "meshes": meshes.map_or(0, |m| m.len()),
        "materials": materials.map_or(0, |m| m.len()),
        "batch_keys": keys.len(),
        "other_material": other,
        "extents": extents.into_iter().collect::<Vec<_>>(),
        "frame_ms": (frame_ms * 1000.0).round() / 1000.0,
    });
    if let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(&p.out) {
        let _ = writeln!(file, "{line}");
    }
    if let Some(d) = &p.png {
        let shot = offscreen.as_ref().map_or_else(Screenshot::primary_window, |o| Screenshot::image(o.0.clone()));
        commands.spawn(shot).observe(save_to_disk(d.join(format!("{f}.png"))));
    }
}

fn world_file(out: &str) -> PathBuf {
    PathBuf::from(format!("{}.world.jsonl", out.strip_suffix(".jsonl").unwrap_or(out)))
}

/// The listed types' values on the frame `probe` just wrote, reflected as Bevy serializes them (TypedReflectSerializer).
fn dump(world: &mut World) {
    let Some(p) = world.get_resource::<Probe>() else { return };
    let f = p.frame.saturating_sub(1);
    // which entities there are, by archetype (RAEA_PARITY_ARCHETYPES=1, at the probed frames): what a growing entity
    // count is made of (mesh_ray_cast's grew by one a frame per moving gizmo)
    if std::env::var_os("RAEA_PARITY_ARCHETYPES").is_some() && p.frames.as_ref().is_none_or(|s| s.contains(&f)) {
        for a in world.archetypes().iter().filter(|a| a.len() > 20) {
            let names: Vec<String> = a.components().iter().filter_map(|c| world.components().get_name(*c)).map(|n| n.shortname().to_string()).collect();
            eprintln!("archetypes frame {f}: {} x [{}]", a.len(), names.join(", "));
        }
    }
    if p.types.is_empty() || p.frames.as_ref().is_some_and(|s| !s.contains(&f)) {
        return;
    }
    let (types, out) = (p.types.clone(), world_file(&p.out.to_string_lossy()));
    let registry = world.resource::<AppTypeRegistry>().clone();
    let registry = registry.read();
    let mut by_type = serde_json::Map::new();
    for path in &types {
        let Some(reg) = registry.get_with_type_path(path) else {
            by_type.insert(path.clone(), serde_json::json!({"error": "not in the type registry"}));
            continue;
        };
        // a resource too: Bevy 0.20 keeps each on an entity of its own, as a component
        let Some(rc) = reg.data::<bevy::ecs::reflect::ReflectComponent>() else {
            by_type.insert(path.clone(), serde_json::json!({"error": "not a reflected component or resource"}));
            continue;
        };
        let mut values = vec![];
        for e in world.iter_entities() {
            if let Some(v) = rc.reflect(e) {
                let ser = bevy::reflect::serde::TypedReflectSerializer::new(v.as_partial_reflect(), &registry);
                values.push(serde_json::to_value(&ser).unwrap_or_else(|err| serde_json::json!({"error": err.to_string()})));
            }
        }
        by_type.insert(path.clone(), serde_json::Value::Array(values));
    }
    if let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(&out) {
        let _ = writeln!(file, "{}", serde_json::json!({"frame": f, "world": by_type}));
    }
}
