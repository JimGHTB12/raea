//! The call bridge: TypeScript reaches a plugin's API through bevy_remote's method table, the Bevy Remote Protocol's
//! own registry of methods, here in process and with no HTTP. Every plugin App has BRP's built-in methods: world.query,
//! world.get_components, world.spawn_entity, world.insert_components, world.get_resources, world.insert_resources,
//! world.trigger_event, world.write_message, world.observe+watch, registry.schema and schedule.list. It also has raea's
//! methods for states (raea/state.get and raea/state.set), and whatever a plugin's calls module adds (for example
//! avian3d/cast_ray, or lightyear/connect and lightyear/input). A method is a Bevy system from `In<Option<Value>>` to
//! `BrpResult`, so one call runs on the World of the frame it's asked in.
//!
//! Two hosts use the bridge:
//! - **raea's native calls `bevy.app.*`.** These are for a TypeScript program (`raea-bevy-sdk motion run`), which gets
//!   a plain headless App with the plugins it names. The program steps it by frames (one TimeUpdateStrategy step and a
//!   frame of the plugin clock each, so a plugin steps exactly as it does under raea-render's Engine), calls its methods, and
//!   can serve it to a BRP client over HTTP (the dev tool). Apps stepped together go in lockstep; that is a network's
//!   peers, whose shared mock clock advances once a frame through `on_step`.
//! - **`remote_call`.** A host that holds its own World passes it, and a space's
//!   `call(name, args)` reaches the same methods.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

use bevy::prelude::*;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::time::TimeUpdateStrategy;
use bevy_remote::{BrpError, BrpResult, RemoteMethods, RemotePlugin};
use raea_runtime_plugins::PluginClock;
use serde::de::DeserializeSeed;
use serde_json::{Value, json};

type R<T> = Result<T, String>;

/// One headless App that TypeScript holds.
struct Peer {
    app: App,
    dt: Duration,
    frame: u64,
    /// a frame run schedule by schedule (`bevy.app.run`, for TypeScript systems): the next schedule of Bevy's frame
    /// (MainScheduleOrder) to run, while the frame is part way
    phase: Option<usize>,
}

thread_local! {
    static APPS: RefCell<BTreeMap<u64, Peer>> = const { RefCell::new(BTreeMap::new()) };
    static NEXT: RefCell<u64> = const { RefCell::new(0) };
}

/// What runs once per lockstep frame, before any App updates: a plugin whose clock is global (lightyear's mock
/// Instant) advances it here, once for all its peers.
static STEP_HOOKS: Mutex<Vec<fn(Duration)>> = Mutex::new(Vec::new());

/// Adds a hook that runs once per lockstep frame (see STEP_HOOKS).
pub fn on_step(f: fn(Duration)) {
    let mut h = STEP_HOOKS.lock().unwrap();
    if !h.iter().any(|x| std::ptr::fn_addr_eq(*x, f)) {
        h.push(f);
    }
}

/// A method of the World's table run on it now: an instant method's answer, or a watching method's answer for this
/// frame (null when it has nothing new). None if the World has no such method.
pub fn remote_call(world: &mut World, name: &str, args: Value) -> Option<R<Value>> {
    // the layer every runtime shares (engine/bevy-world): the same method table, run the same way
    raea_bevy_world::remote(world, name, args)
}

// the method helpers every calls module and adapter uses, from raea-bevy-calls (where most calls live)
pub use raea_bevy_calls::{add_method, entity, fail, params};

#[derive(serde::Deserialize)]
struct StateParams {
    state: String,
    #[serde(default)]
    value: Option<Value>,
}

/// `raea/state.get {state}`: the state's current value (a States type, by type path), or null when it isn't set up.
fn state_get(In(p): In<Option<Value>>, world: &World) -> BrpResult {
    let StateParams { state, .. } = params(p)?;
    raea_bevy_world::state_get(world, &state).map_err(fail)
}

/// `raea/state.set {state, value}`: the state's next state (it changes at the next StateTransition, in the next frame).
fn state_set(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let StateParams { state, value } = params(p)?;
    raea_bevy_world::state_set(world, &state, value.unwrap_or(Value::Null)).map(|_| Value::Null).map_err(fail)
}

/// A client asked the App's program to stop (raea/stop), which the program reads with raea/stopped: how a BRP client
/// over HTTP ends a served session.
#[derive(Resource, Default)]
struct Stop(bool);

fn stop(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    world.insert_resource(Stop(true));
    Ok(Value::Null)
}

fn stopped(In(_): In<Option<Value>>, s: Option<Res<Stop>>) -> BrpResult {
    Ok(json!(s.is_some_and(|s| s.0)))
}

/// A reflected value of type T from JSON ("Space", "Left"), through the TypeRegistry: no serde feature needed.
pub(crate) fn reflected<T: bevy::reflect::FromReflect + bevy::reflect::TypePath>(world: &World, v: Value) -> R<T> {
    let registry = world.resource::<AppTypeRegistry>().read();
    let r = registry.get(std::any::TypeId::of::<T>()).ok_or_else(|| format!("{} isn't registered", T::type_path()))?;
    let d = TypedReflectDeserializer::new(r, &registry).deserialize(v).map_err(|e| format!("{}: {e}", T::type_path()))?;
    T::from_reflect(d.as_partial_reflect()).ok_or_else(|| format!("{}: not a whole value", T::type_path()))
}

/// BRP and raea's methods on an App (before any plugin's calls module adds its own).
pub fn install_remote(app: &mut App) {
    if app.world().contains_resource::<RemoteMethods>() {
        return;
    }
    app.add_plugins(RemotePlugin::default());
    add_method(app, "raea/state.get", state_get);
    add_method(app, "raea/state.set", state_set);
    add_method(app, "raea/stop", stop);
    add_method(app, "raea/stopped", stopped);
    add_method(app, "raea/input", crate::input::input);
    add_method(app, "raea/input.gamepad", crate::input::gamepad);
    crate::asset::install(app);
    // BRP's spawn and insert with a value read as a TSX <BevyComponent>'s is: fields left out take the type's reflected
    // Default (BRP's own wants every field); a component the entity has is replaced, as BRP's insert does
    add_method(app, "world.spawn_entity", spawn_entity);
    add_method(app, "world.insert_components", insert_components);
}

#[derive(serde::Deserialize)]
struct Insert {
    entity: Option<Entity>,
    components: serde_json::Map<String, Value>,
}

fn insert_all(world: &mut World, e: Entity, components: &serde_json::Map<String, Value>) -> Result<(), BrpError> {
    let registry = world.resource::<AppTypeRegistry>().clone();
    let reg = registry.read();
    for (path, v) in components {
        let r = raea_runtime_plugins::registration(&reg, path).map_err(fail)?;
        let rc = r.data::<ReflectComponent>().ok_or_else(|| fail(format!("{path} isn't a component")))?.clone();
        let value = raea_runtime_plugins::value_of(world, &reg, r, path, v).map_err(fail)?;
        rc.insert(&mut world.entity_mut(e), value.as_partial_reflect(), &reg);
    }
    Ok(())
}

fn spawn_entity(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: Insert = params(p)?;
    let e = world.spawn_empty().id();
    if let Err(err) = insert_all(world, e, &p.components) {
        world.despawn(e);
        return Err(err);
    }
    Ok(json!({ "entity": e }))
}

fn insert_components(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: Insert = params(p)?;
    let e = p.entity.ok_or_else(|| fail("world.insert_components: no entity"))?;
    if world.get_entity(e).is_err() {
        return Err(fail(format!("world.insert_components: no entity {e}")));
    }
    insert_all(world, e, &p.components)?;
    Ok(Value::Null)
}

/// A headless App with the plugins named, BRP's method table, and (when `serve`) BRP's HTTP server on that port.
fn open(names: &[String], fps: f64, serve: Option<u16>) -> R<Peer> {
    let declared = crate::declared();
    let mut needs: Vec<&str> = Vec::new();
    for n in names {
        if let Some(d) = declared.iter().find(|d| d.name == n || n.starts_with(&format!("{}_", d.name))) {
            needs.extend(d.headless.iter().copied());
        }
    }
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    crate::needs::add(&mut app, &needs);
    install_remote(&mut app);
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    raea_runtime_plugins::install_named(&mut app, fps, &refs)?;
    if !app.world().contains_resource::<PluginClock>() {
        // no plugin named: the clock still steps by frames
        let dt = Duration::from_secs_f64(1.0 / fps.max(1.0));
        app.world_mut().resource_mut::<Time<Fixed>>().set_timestep(dt);
        app.insert_resource(PluginClock { stepped: 0, dt, advancing: false, steps: 0 });
    }
    if let Some(port) = serve {
        #[cfg(feature = "remote")]
        app.add_plugins(bevy_remote::http::RemoteHttpPlugin::default().with_port(port));
        #[cfg(not(feature = "remote"))]
        return Err(format!("serve {port}: this build has no BRP HTTP server (the remote feature)"));
    }
    app.finish();
    app.cleanup();
    // Startup runs here, on an update that doesn't step (Time advances by zero)
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.update();
    let dt = app.world().resource::<PluginClock>().dt;
    Ok(Peer { app, dt, frame: 0, phase: None })
}

/// One frame on every App given, in lockstep: the step hooks once, then each App's update with Time one frame on.
fn step(ids: &[u64], frames: u64) -> R<Value> {
    APPS.with(|apps| {
        let mut apps = apps.borrow_mut();
        for id in ids {
            match apps.get(id) {
                None => return Err(format!("no app {id}")),
                Some(p) if p.phase.is_some() => return Err(format!("app {id} is part way through frame {} (bevy.app.run): finish it first", p.frame)),
                _ => {}
            }
        }
        let dt = apps[&ids[0]].dt;
        for _ in 0..frames {
            for h in STEP_HOOKS.lock().unwrap().iter() {
                h(dt);
            }
            for id in ids {
                let p = apps.get_mut(id).unwrap();
                p.frame += 1;
                {
                    let w = p.app.world_mut();
                    let mut c = w.resource_mut::<PluginClock>();
                    c.advancing = true;
                    c.stepped = p.frame as i64;
                    c.steps += 1;
                    w.insert_resource(TimeUpdateStrategy::ManualDuration(p.dt));
                }
                p.app.update();
                p.app.world_mut().resource_mut::<PluginClock>().advancing = false;
            }
        }
        Ok(json!(ids.iter().map(|id| apps[id].frame).collect::<Vec<_>>()))
    })
}

/// Bevy's frame run schedule by schedule, as App::update runs it (Main's MainScheduleOrder: First, PreUpdate,
/// StateTransition, RunFixedMainLoop, Update, SpawnScene, PostUpdate, Last), stopping after `until`, so TypeScript can
/// run its systems on the World between one schedule and the next. A frame starts (Time one frame on, as `step`) on
/// the first call after the last finished; `until` absent finishes it.
///
/// `fixed`: RunFixedMainLoop opened up into the fixed step's own schedules (FixedFirst, FixedPreUpdate, FixedUpdate,
/// FixedPostUpdate, FixedLast), run in Time<Fixed>'s context as run_fixed_main_schedule runs them, so a system can sit
/// in FixedUpdate. The plugin clock's step is the fixed timestep, so a frame is one fixed step; a frame that owes more
/// runs the rest after FixedLast without stopping. RunFixedMainLoop's own before and after systems don't run in such a
/// frame (none of raea's plugins has any).
/// Time<Fixed>'s step, as its private `expend` takes one: a timestep out of the overstep, and the clock on by it.
fn expend(t: &mut Time<bevy::time::Fixed>) -> bool {
    let step = t.timestep();
    if t.overstep() < step {
        return false;
    }
    t.discard_overstep(step);
    t.advance_by(step);
    true
}

fn run_until(id: u64, until: Option<&str>, fixed: bool) -> R<Value> {
    use bevy::app::{FixedMain, FixedMainScheduleOrder, MainScheduleOrder, RunFixedMainLoop};
    use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
    use bevy::time::{Fixed, Virtual};
    enum Step {
        Run(InternedScheduleLabel),
        FixedIn,
        Fixed(InternedScheduleLabel),
        FixedOut,
    }
    #[derive(Resource)]
    struct Stepping(bool);
    APPS.with(|apps| {
        let mut apps = apps.borrow_mut();
        let p = apps.get_mut(&id).ok_or(format!("no app {id}"))?;
        let mut steps: Vec<(String, Step)> = Vec::new();
        for l in p.app.world().resource::<MainScheduleOrder>().labels.clone() {
            if fixed && l == RunFixedMainLoop.intern() {
                steps.push(("~fixed in".into(), Step::FixedIn));
                for f in p.app.world().resource::<FixedMainScheduleOrder>().labels.clone() {
                    steps.push((format!("{f:?}"), Step::Fixed(f)));
                }
                // a system "in RunFixedMainLoop" runs once the fixed step is done, as it would after the loop
                steps.push(("RunFixedMainLoop".into(), Step::FixedOut));
            } else {
                steps.push((format!("{l:?}"), Step::Run(l)));
            }
        }
        let names: Vec<String> = steps.iter().map(|s| s.0.clone()).filter(|n| !n.starts_with('~')).collect();
        let from = p.phase.unwrap_or(0);
        let to = match until {
            Some(u) => {
                let i = steps.iter().position(|s| s.0 == u).ok_or_else(|| format!("no schedule {u:?} in Bevy's frame (it runs: {})", names.join(", ")))?;
                if i < from {
                    return Err(format!("{u} already ran in frame {} (systems run in the frame's order: {})", p.frame, names.join(", ")));
                }
                i + 1
            }
            None => steps.len(),
        };
        if p.phase.is_none() {
            for h in STEP_HOOKS.lock().unwrap().iter() {
                h(p.dt);
            }
            p.frame += 1;
            let w = p.app.world_mut();
            let mut c = w.resource_mut::<PluginClock>();
            c.advancing = true;
            c.stepped = p.frame as i64;
            c.steps += 1;
            w.insert_resource(TimeUpdateStrategy::ManualDuration(p.dt));
        }
        let w = p.app.world_mut();
        for (_, st) in &steps[from..to] {
            match st {
                Step::Run(l) => {
                    let _ = w.try_run_schedule(*l);
                }
                Step::FixedIn => {
                    let delta = w.resource::<Time<Virtual>>().delta();
                    w.resource_mut::<Time<Fixed>>().accumulate_overstep(delta);
                    let on = expend(&mut w.resource_mut::<Time<Fixed>>());
                    if on {
                        *w.resource_mut::<Time>() = w.resource::<Time<Fixed>>().as_generic();
                    }
                    w.insert_resource(Stepping(on));
                }
                Step::Fixed(l) => {
                    if w.get_resource::<Stepping>().is_some_and(|s| s.0) {
                        let _ = w.try_run_schedule(*l);
                    }
                }
                Step::FixedOut => {
                    if w.get_resource::<Stepping>().is_some_and(|s| s.0) {
                        while expend(&mut w.resource_mut::<Time<Fixed>>()) {
                            *w.resource_mut::<Time>() = w.resource::<Time<Fixed>>().as_generic();
                            let _ = w.try_run_schedule(FixedMain);
                        }
                    }
                    *w.resource_mut::<Time>() = w.resource::<Time<Virtual>>().as_generic();
                    w.remove_resource::<Stepping>();
                }
            }
        }
        if to == steps.len() {
            w.clear_trackers();
            w.resource_mut::<PluginClock>().advancing = false;
            p.phase = None;
        } else {
            p.phase = Some(to);
        }
        Ok(json!({"frame": p.frame, "done": p.phase.is_none(), "schedules": names}))
    })
}

fn id_of(v: &Value) -> R<u64> {
    v.get("app").and_then(Value::as_u64).ok_or_else(|| "app: an app's id (from bevy.app.open)".to_string())
}

/// The bridge's native calls (raea's native-call registry, raea-core-calls), for TypeScript (sdk/raea/src/bevy.ts).
pub fn register_natives() {
    raea_core_calls::register(
        "bevy.app.open",
        "{ plugins: string[]; fps?: number; serve?: number }",
        "{ app: number; methods: string[] }",
        "a headless Bevy App with the plugins named (this build's), BRP's method table, and BRP over HTTP on port `serve` when given",
        |a| {
            let names: Vec<String> = a.get("plugins").and_then(Value::as_array).map(|v| v.iter().filter_map(|s| s.as_str().map(str::to_string)).collect()).unwrap_or_default();
            let fps = a.get("fps").and_then(Value::as_f64).unwrap_or(60.0);
            let serve = a.get("serve").and_then(Value::as_u64).map(|p| p as u16);
            let peer = open(&names, fps, serve)?;
            let mut methods = peer.app.world().resource::<RemoteMethods>().methods();
            methods.sort();
            let id = NEXT.with(|n| {
                *n.borrow_mut() += 1;
                *n.borrow()
            });
            APPS.with(|apps| apps.borrow_mut().insert(id, peer));
            Ok(json!({"app": id, "methods": methods}))
        },
    );
    raea_core_calls::register("bevy.app.step", "{ apps: number[]; frames?: number }", "number[]", "the apps stepped together, a frame at a time (Time one frame on in each); each one's frame after", |a| {
        let ids: Vec<u64> = a.get("apps").and_then(Value::as_array).map(|v| v.iter().filter_map(Value::as_u64).collect()).unwrap_or_default();
        if ids.is_empty() {
            return Err("apps: one or more app ids".into());
        }
        step(&ids, a.get("frames").and_then(Value::as_u64).unwrap_or(1))
    });
    raea_core_calls::register("bevy.app.run", "{ app: number; until?: string; fixed?: boolean }", "{ frame: number; done: boolean; schedules: string[] }", "the app's frame run schedule by schedule up to and including `until` (a schedule of Bevy's frame: PreUpdate, Update, PostUpdate...), or to its end; starts the next frame when none is part way", |a| {
        run_until(id_of(a)?, a.get("until").and_then(Value::as_str), a.get("fixed").and_then(Value::as_bool).unwrap_or(false))
    });
    raea_core_calls::register("bevy.app.schedules", "{ app: number }", "string[]", "the schedules a frame of the app runs, in order (Bevy's MainScheduleOrder)", |a| {
        let id = id_of(a)?;
        APPS.with(|apps| {
            let apps = apps.borrow();
            let p = apps.get(&id).ok_or(format!("no app {id}"))?;
            let w = p.app.world();
            let fixed: Vec<String> = w.resource::<bevy::app::FixedMainScheduleOrder>().labels.iter().map(|l| format!("{l:?}")).collect();
            // the fixed step's schedules in the place RunFixedMainLoop runs them (a system can sit in either)
            Ok(json!(w.resource::<bevy::app::MainScheduleOrder>().labels.iter().flat_map(|l| {
                let n = format!("{l:?}");
                if n == "RunFixedMainLoop" { std::iter::once(n).chain(fixed.clone()).collect::<Vec<_>>() } else { vec![n] }
            }).collect::<Vec<_>>()))
        })
    });
    raea_core_calls::register("bevy.app.call", "{ app: number; method: string; params?: unknown }", "unknown", "a method of the app's BRP table (world.query, world.trigger_event, avian3d/cast_ray, ...) run now", |a| {
        let id = id_of(a)?;
        let method = a.get("method").and_then(Value::as_str).ok_or("method: a BRP method's name")?.to_string();
        let params = a.get("params").cloned().unwrap_or(Value::Null);
        APPS.with(|apps| {
            let mut apps = apps.borrow_mut();
            let p = apps.get_mut(&id).ok_or(format!("no app {id}"))?;
            // the App's table, then the shared layer's own methods (raea/observe, raea/observed, raea/unobserve)
            match remote_call(p.app.world_mut(), &method, params.clone()) {
                Some(r) => r,
                None if method.starts_with("raea/") => raea_bevy_world::call(p.app.world_mut(), &method, params),
                None => {
                    let mut m = p.app.world().resource::<RemoteMethods>().methods();
                    m.sort();
                    Err(format!("the app has no method {method:?} (it has: {}, raea/observe, raea/observed, raea/unobserve)", m.join(", ")))
                }
            }
        })
    });
    raea_core_calls::register("bevy.app.close", "{ app: number }", "null", "drop an app", |a| {
        let id = id_of(a)?;
        APPS.with(|apps| apps.borrow_mut().remove(&id));
        Ok(Value::Null)
    });
}
