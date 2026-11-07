//! Bevy plugins inside raea's engine (sdk/raea/docs/bevy-plugins.md). A plugin is compiled in by a crate that registers
//! it here (bevy-plugins/, behind cargo features); TSX drives it with generic rows, never a hand-written system per
//! component:
//!
//!   component  {target: a row's id, type: a reflected type path, value: JSON, from?: frame}
//!              the value deserialized through Bevy's TypeRegistry (TypedReflectDeserializer) and inserted with
//!              ReflectComponent::insert on the target's entity: at load, or once frame `from` is drawn (each frame
//!              takes the last row from at or before it, in every build: docs/control.feature FC1c)
//!   resource   {type, value, from?}   the same for a reflected resource
//!   mesh ... driven            the mesh's Transform is the plugin's (a rigid body's), not the rows'
//!
//! Render equals play: with a plugin installed, Bevy's clock is the frame index. Every update that draws a new frame f
//! from frame f - 1 advances Time by exactly 1/fps (TimeUpdateStrategy::ManualDuration, Time<Fixed> at the same step,
//! so FixedUpdate runs once); every other update (the same frame drawn again while its readback comes back) advances it
//! by zero. A frame reached by a jump is stepped to through the frames between, and a frame before the last stepped one
//! reloads the edit and steps from 0, so a plugin's state at frame f is always f steps from the load, however f was
//! reached (a render of one frame, a play, a seek).

use std::any::TypeId;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use bevy::app::PluginGroupBuilder;
use bevy::ecs::reflect::{AppTypeRegistry, ReflectComponent, ReflectEvent, ReflectFromWorld, ReflectMessage, ReflectResource};
use bevy::prelude::*;
use bevy::asset::{ReflectAsset, ReflectHandle, RecursiveDependencyLoadState, UntypedHandle};
use bevy::reflect::serde::{ReflectDeserializerProcessor, TypedReflectDeserializer, TypedReflectSerializer};
use bevy::reflect::std_traits::ReflectDefault;
use bevy::state::reflect::{ReflectFreelyMutableState, ReflectState};
use bevy::reflect::{PartialReflect, ReflectDeserialize, ReflectFromReflect, ReflectSerialize, TypeInfo, TypeRegistration, TypeRegistry, enums::VariantInfo};
use bevy::time::TimeUpdateStrategy;
use serde::de::DeserializeSeed;
use serde_json::{Value, json};

use raea_core_program::eval::Tup;
use raea_core_program::ir::V;

use raea_sys_frame::*;
use raea_sys_material::{mesh, shader_material};

mod world;
pub use world::*;
mod fields;
pub use fields::*;
mod reflect;
pub use reflect::*;
pub mod character;
#[cfg(test)]
mod tests;

/// What a plugin crate registers: its name, what it adds to the App, and optionally a change to DefaultPlugins (a
/// plugin that switches on bevy_winit must turn WinitPlugin off: raea-render drives its own updates and has no window).
/// `on_demand` is for a plugin that only an App which names it gets (a network peer's server or client half): raea-render's
/// Engine leaves it out, and `install_named` adds it.
pub struct Registered {
    pub name: &'static str,
    pub install: fn(&mut App),
    pub group: Option<fn(PluginGroupBuilder) -> PluginGroupBuilder>,
    pub on_demand: bool,
}

static PLUGINS: Mutex<Vec<Registered>> = Mutex::new(Vec::new());

/// Registers a plugin for every Engine made from now on (once per name).
pub fn register(r: Registered) {
    let mut p = PLUGINS.lock().unwrap();
    if !p.iter().any(|x| x.name == r.name) {
        p.push(r);
    }
}

/// The plugins registered, by name.
pub fn names() -> Vec<&'static str> {
    PLUGINS.lock().unwrap().iter().map(|r| r.name).collect()
}

pub fn group(mut b: PluginGroupBuilder) -> PluginGroupBuilder {
    for r in PLUGINS.lock().unwrap().iter().filter(|r| !r.on_demand) {
        if let Some(g) = r.group {
            b = g(b);
        }
    }
    b
}

/// The frame clock plugins step by: the last frame stepped to, the step, and whether the coming update steps.
#[derive(Resource)]
pub struct PluginClock {
    pub stepped: i64,
    pub dt: Duration,
    pub advancing: bool,
    pub steps: u64,
}

/// A mesh whose Transform a plugin owns (sync_meshes leaves it alone).
#[derive(Component)]
pub struct Driven;

/// The edit's component and resource rows keyed to frames (docs/control.feature FC1c): each one's target (None: a
/// resource), type, value and first frame, and the frame of the row last applied for each target and type. A frame
/// takes, for each, the last row from at or before it, so a seek (a still, live play's jump) lands on its value.
#[derive(Resource, Default)]
struct Keyed {
    rows: Vec<(Option<Entity>, String, Value, i64)>,
    applied: std::collections::HashMap<(Option<Entity>, String), i64>,
}

/// Adds the registered plugins (all but the on-demand ones) and the frame clock, when there are any. An App without
/// plugins is left as it was.
pub fn install(app: &mut App, fps: f64) {
    // the keyboard's and mouse buttons' state as resources TypeScript can read and write (Bevy's InputPlugin adds
    // them without registering them for reflection; the gamepad's ButtonInput it registers itself)
    app.register_type::<ButtonInput<KeyCode>>();
    app.register_type::<ButtonInput<MouseButton>>();
    // a Range field (a Viewport's depth) is reflected opaque without serde's data: given it, it reads as {start, end}
    app.register_type_data::<core::ops::Range<f32>, ReflectDeserialize>();
    app.register_type_data::<core::ops::Range<f32>, ReflectSerialize>();
    // a live game's world calls (sdk/raea/src/world.ts): queued by its TypeScript, run here at the update's start
    // (engine/bevy-world, the layer BevyApp programs and pages share), entities named as the edit names them
    raea_bevy_world::register_natives();
    // the queue is the process's: React's App in this process leaves it to this one (its names resolve here)
    raea_bevy_world::claim();
    app.add_systems(First, raea_bevy_world::drain);
    world_methods(app);
    character::register(app);
    // materials written in TypeScript (<ShaderMaterial>, shader_material.rs): the render Engine only
    shader_material::install(app);
    keyed(app);
    let list: Vec<fn(&mut App)> = PLUGINS.lock().unwrap().iter().filter(|r| !r.on_demand).map(|r| r.install).collect();
    install_list(app, fps, list);
}

/// Adds the plugins named, in that order, and the frame clock: a host's own headless App (raea-bevy-plugins' bridge,
/// which TypeScript drives) gets exactly the plugins it asks for. An unregistered name is an error that lists the
/// registered ones.
pub fn install_named(app: &mut App, fps: f64, names: &[&str]) -> Result<(), String> {
    let mut list = Vec::new();
    {
        let p = PLUGINS.lock().unwrap();
        for n in names {
            let r = p.iter().find(|r| r.name == *n).ok_or_else(|| format!("no plugin {n:?} in this build (it has: {})", p.iter().map(|r| r.name).collect::<Vec<_>>().join(", ")))?;
            list.push(r.install);
        }
    }
    world_methods(app);
    character::register(app);
    keyed(app);
    install_list(app, fps, list);
    Ok(())
}

/// Keyed component and resource rows, in every build: with a plugin or without one (docs/control.feature FC1c)
fn keyed(app: &mut App) {
    app.init_resource::<Keyed>();
    app.add_systems(Update, keyed_rows);
}

/// A live game's component and resource rows (Engine::set_rows), each as its live values now stand: a component's
/// value set on the row it targets (patched over what the part has), a resource's on the World. A live frame's
/// <BevyComponent value> may change every frame, as a render's are set at load.
pub fn apply_live_rows(world: &mut World, rows: &[(String, String)]) -> Result<(), String> {
    for (kind, id) in rows {
        let Some(e) = raea_sys_frame::store::find(world, kind, id) else { continue };
        let vals = {
            let fr = world.resource::<FrameRows>();
            match fr.ix.get(&e).and_then(|r| fr.live.as_ref().and_then(|l| l.get(*r))) {
                Some(v) => v.clone(),
                None => continue,
            }
        };
        let ty = s(&vals, "type").ok_or_else(|| format!("{kind} {id}: needs type"))?;
        let v: Value = serde_json::from_str(&s(&vals, "value").unwrap_or_else(|| "null".into())).map_err(|e| format!("{kind} {id}: value isn't JSON: {e}"))?;
        if kind == "resource" {
            insert_reflect_resource(world, &ty, &v).map_err(|err| format!("resource {id} ({ty}): {err}"))?;
            continue;
        }
        let target = s(&vals, "target").ok_or_else(|| format!("component {id}: needs target (a row's id)"))?;
        let found = raea_sys_frame::store::rows(world).iter().copied().find(|r| raea_sys_frame::store::key(world, *r).is_some_and(|k| k.id == target));
        let Some(t) = found else { return Err(format!("component {id}: no row {target}")) };
        insert_reflect(world, t, &ty, &v).map_err(|err| format!("component {id} ({ty}): {err}"))?;
    }
    Ok(())
}

fn install_list(app: &mut App, fps: f64, list: Vec<fn(&mut App)>) {
    if list.is_empty() {
        return;
    }
    let dt = Duration::from_secs_f64(1.0 / fps.max(1.0));
    for f in list {
        f(app);
    }
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.world_mut().resource_mut::<Time<Fixed>>().set_timestep(dt);
    app.insert_resource(PluginClock { stepped: 0, dt, advancing: false, steps: 0 });
}

pub fn active(world: &World) -> bool {
    world.contains_resource::<PluginClock>()
}

/// Before an update: Time advances one frame when this update draws the frame after the last stepped, else not at all.
pub fn arm(world: &mut World) {
    let f = world.resource::<Clock>().0;
    let Some(mut c) = world.get_resource_mut::<PluginClock>() else { return };
    let step = f == c.stepped + 1;
    c.advancing = step;
    if std::env::var_os("RAEA_PLUGIN_TRACE").is_some() {
        eprintln!("plugin clock: frame {f}, stepped {}, {}", c.stepped, if step { "step" } else { "hold" });
    }
    let dt = if step {
        c.stepped = f;
        c.steps += 1;
        c.dt
    } else {
        Duration::ZERO
    };
    world.insert_resource(TimeUpdateStrategy::ManualDuration(dt));
}

/// Keyed rows whose frame has come, set on the update that draws it (so a plugin's steps after it see them): with a
/// plugin, on the frame its clock steps to; without one, on the frame drawn, whichever way it was reached.
fn keyed_rows(world: &mut World) {
    // live play sets its rows from the game's state each frame (apply_live_rows), not from a timeline
    if world.get_resource::<FrameRows>().is_some_and(|r| r.live.is_some()) {
        return;
    }
    let f = match world.get_resource::<PluginClock>() {
        Some(c) if !c.advancing => return,
        Some(c) => c.stepped,
        None => match world.get_resource::<Clock>() {
            Some(c) => c.0,
            None => return,
        },
    };
    let due: Vec<(Option<Entity>, String, Value, i64)> = {
        let k = world.resource::<Keyed>();
        let mut last: std::collections::HashMap<(Option<Entity>, &str), &(Option<Entity>, String, Value, i64)> = Default::default();
        for r in k.rows.iter().filter(|r| r.3 <= f) {
            let at = last.entry((r.0, r.1.as_str())).or_insert(r);
            if r.3 >= at.3 {
                *at = r;
            }
        }
        last.into_values().filter(|r| k.applied.get(&(r.0, r.1.clone())) != Some(&r.3)).cloned().collect()
    };
    for (e, ty, v, from) in due {
        let done = match e {
            Some(e) => insert_reflect(world, e, &ty, &v).map_err(|err| format!("component {ty}: {err}")),
            None => insert_reflect_resource(world, &ty, &v).map_err(|err| format!("resource {ty}: {err}")),
        };
        if let Err(err) = done {
            world.resource_mut::<Error>().0 = Some(err);
        }
        world.resource_mut::<Keyed>().applied.insert((e, ty), from);
    }
}

fn s(vals: &BTreeMap<String, V>, k: &str) -> Option<String> {
    match vals.get(k) {
        Some(V::S(x)) => Some(x.clone()),
        _ => None,
    }
}

/// The edit's component and resource rows applied at load (the rows' entities are `ents`, in `inserted`'s order), the
/// timed ones kept for their frame, and the plugin clock back at frame 0.
pub fn apply_rows(world: &mut World, inserted: &[Tup], ents: &[Entity], dir: &Path) -> Result<(), String> {
    // what a Handle or an Entity field resolves against: the edit's folder, its media rows, and each row by id and name
    let mut by_name: BTreeMap<String, Entity> = BTreeMap::new();
    for (t, &e) in inserted.iter().zip(ents) {
        if let Some(n) = s(&t.vals, "name") {
            by_name.entry(n).or_insert(e);
        }
    }
    for (t, &e) in inserted.iter().zip(ents) {
        by_name.insert(t.id.clone(), e);
    }
    let media = inserted.iter().filter(|t| t.kind == "media").cloned().collect();
    world.insert_resource(Refs { dir: dir.to_path_buf(), names: by_name, media });
    world.insert_resource(raea_bevy_world::EntityNames(row_names));
    world.insert_resource(Loading::default());
    world.insert_resource(RenderImages::default());
    if active(world) {
        let mut c = world.resource_mut::<PluginClock>();
        c.stepped = 0;
        c.advancing = false;
    }
    let by_id: BTreeMap<&str, Entity> = inserted.iter().zip(ents).map(|(t, e)| (t.id.as_str(), *e)).collect();
    for (t, &e) in inserted.iter().zip(ents) {
        if t.kind == "mesh" && t.vals.get("driven").is_some_and(V::truthy) {
            world.entity_mut(e).insert(Driven);
        }
    }
    // Bevy's own reflected components and resources (a camera's Projection, a mesh's NotShadowReceiver) and a plugin's:
    // those from frame 0 now, the rest kept for their frame (keyed_rows), in every build
    let mut keyed = Keyed::default();
    for t in inserted.iter().filter(|t| t.kind == "component" || t.kind == "resource") {
        let ty = s(&t.vals, "type").ok_or_else(|| format!("{} {}: needs type", t.kind, t.id))?;
        let v: Value = serde_json::from_str(&s(&t.vals, "value").unwrap_or_else(|| "null".into())).map_err(|e| format!("{} {}: value isn't JSON: {e}", t.kind, t.id))?;
        let target = if t.kind == "resource" {
            None
        } else {
            let target = s(&t.vals, "target").ok_or_else(|| format!("component {}: needs target (a row's id)", t.id))?;
            Some(*by_id.get(target.as_str()).ok_or_else(|| format!("component {}: no row {target}", t.id))?)
        };
        let from = t.vals.get("from").map(V::num).unwrap_or(0.0) as i64;
        if from <= 0 {
            match target {
                Some(e) => insert_reflect(world, e, &ty, &v).map_err(|err| format!("component {} ({ty}): {err}", t.id))?,
                None => insert_reflect_resource(world, &ty, &v).map_err(|err| format!("resource {} ({ty}): {err}", t.id))?,
            }
            keyed.applied.insert((target, ty.clone()), 0);
        }
        keyed.rows.push((target, ty, v, from.max(0)));
    }
    // kept only when a row waits for a frame (a still edit's rows are all applied now)
    if !keyed.rows.iter().any(|r| r.3 > 0) {
        keyed = Keyed::default();
    }
    world.insert_resource(keyed);
    Ok(())
}
