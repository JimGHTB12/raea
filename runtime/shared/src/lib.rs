//! State React and the World share (docs/blitz-bevy.feature B3 H4, sdk.feature S2b, S2d): a Resource or a component,
//! one copy of each value, the World's, crossing as Bevy's reflected form (what BRP reads and writes). Any type
//! registered for reflection with `#[reflect(Resource)]` or `#[reflect(Component)]` is React's to read and write with
//! no binding: `useResource(Score)` re-renders when a system changes it, `setResource(Score, v)` sets it in the World
//! (@raea/app's hooks.ts); components likewise, by entity: `useComponent(entity, Health)`, `setComponent`.
//!
//! - World to React: a hook asks the World to watch its value (`world.watch {path, entity?}`, `world.unwatch` when its
//!   last reader goes); each frame, in Last, the watched values that changed (or went) are sent to React as one
//!   `raea-world` event, `[{path, entity?, value}]` (value null: gone), which the JS host's prelude hands to hooks.ts's
//!   store. Any BRP read is watched the same way by a key of React's (`world.watch {key, method, params}`: useQuery's
//!   world.query), sent as `{key, value}` (or `{key, value: null, error}`); so is `raea/twin {op}`, the entity of an
//!   element's twin (B3 H1: a React page's element is keyed by its op id, what a ref holds), useEntity(ref)'s answer.
//! - Observers: `world.observe {key, event, entity?}` (or a component hook: `on`, `component`) is raea-bevy-world's
//!   observer in the World; what it saw each frame is sent as `{key, events: [{entity, event}]}` (useObserver). `app.share_resource::<Score>()` / `share_component::<Health>()` register the type and push it always (a
//!   component on every entity whose copy changed), watched or not.
//! - React to World: a write is the native call `world.write {path, entity?, value}`, queued and set in the World in
//!   PreUpdate (a system sees it that frame) as BRP's world.insert_resources / world.insert_components; the World's
//!   value then comes back as any change does, so React holds no second copy.
//!
//! Reads and writes go through raea-bevy-world, the one BRP layer every runtime shares (/world's queued calls are
//! drained here too unless an Engine in the process claimed them, so a plain App answers them), under the type path
//! bevy.ts's handles name.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::reflect::serde::TypedReflectSerializer;
use bevy::reflect::{GetTypeRegistration, TypePath};
use serde_json::{Value, json};

use std::sync::mpsc::Sender;

#[cfg(test)]
mod tests;

/// Bevy to React: events for the bridge's `op_next_event` (bevy-react's Outbound messages, as JSON).
#[derive(Resource)]
pub struct ReactOutbox(pub Mutex<Sender<String>>);

impl ReactOutbox {
    /// Any Outbound message (`{"t": "uiEvent", ...}`, `{"t": "elementEvent", ...}`, `{"t": "event", ...}`).
    pub fn send(&self, msg: Value) {
        let _ = self.0.lock().unwrap().send(msg.to_string());
    }
    /// A message already written as JSON
    pub fn send_raw(&self, msg: String) {
        let _ = self.0.lock().unwrap().send(msg);
    }
}

/// React's asks since the World last took them, in the order made.
static ASKS: Mutex<Vec<Ask>> = Mutex::new(Vec::new());

/// The Engine's reads (docs/sdk.feature S2d; on by default, RAEA_ENGINE_READS=0 turns them off): in a process whose
/// Engine runs raea-bevy-world's queue (render, live play), React's watches, reads, systems, observers and writes are
/// the Engine's World's, where a composition's <Mesh3d>s are, not React's App's. React's App hands them on here (twin
/// reads stay: twins are the page's) and the Engine's copy of this channel (engine_side) takes them; its pushes come
/// back through ENGINE_OUT.
static ENGINE_ASKS: Mutex<Vec<Ask>> = Mutex::new(Vec::new());

/// The Engine's pushes for React, sent with React's App's own in its Last.
static ENGINE_OUT: Mutex<Vec<Value>> = Mutex::new(Vec::new());

/// The Engine's live watches, reads, systems and observers: a new Engine App (a seek before the last stepped frame
/// replays from frame 0 in one) starts with them.
static ENGINE_LIVE: Mutex<Vec<Ask>> = Mutex::new(Vec::new());

/// The Engine's reads are on (by default since 10-25, after a full P26 with them; RAEA_ENGINE_READS=0 turns them off):
/// the Engine's copy of the channel is registered for every Engine.
pub fn engine_reads_asked() -> bool {
    std::env::var("RAEA_ENGINE_READS").as_deref() != Ok("0")
}

/// React's asks go to the Engine: the Engine's reads on and an Engine that runs the queue.
pub fn engine_reads() -> bool {
    engine_reads_asked() && raea_bevy_world::CLAIMED.load(std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug, Clone)]
enum Ask {
    Write { path: String, entity: Option<u64>, value: Value },
    Watch { path: String, entity: Option<u64> },
    Unwatch { path: String, entity: Option<u64> },
    WatchRead { key: String, method: String, params: Value },
    UnwatchRead { key: String },
    Observe { params: Value },
    Unobserve { key: String },
    System { key: String, query: Option<Value> },
    Unsystem { key: String },
}

/// The types pushed always (share_resource, share_component): a watch of one adds nothing.
#[derive(Resource, Default)]
struct Shared(HashSet<String>);

/// What React watches, by path and entity: the value last sent.
#[derive(Resource, Default)]
struct Watched(HashMap<(String, Option<u64>), Option<Value>>);

/// The BRP reads React watches, by its key: the method, its params and the answer last sent.
#[derive(Resource, Default)]
struct Reads(HashMap<String, (String, Value, Option<Value>)>);

/// React's observers in the World, by key.
#[derive(Resource, Default)]
struct Observed(HashSet<String>);

/// React's systems (useSystem), by key: the world.query each one reads, sent every frame whether or not it changed.
#[derive(Resource, Default)]
struct Systems(HashMap<String, Option<Value>>);

/// What changed this frame, for React.
#[derive(Resource, Default)]
pub struct WorldPushes(Vec<Value>);

/// This App is an Engine's, with the channel's Engine side (engine_side): what it pushes goes to React's App.
#[derive(Resource)]
struct EngineSide;

/// The shared values' systems, by name: React's asks are taken in `Writes` (PreUpdate), the changes collected in
/// `Push` and sent in `Send` (Last).
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShareSystems {
    Writes,
    Push,
    Send,
}

pub trait ShareExt {
    /// `R` registered for reflection and pushed to React whenever it changes (`useResource`, `setResource`).
    fn share_resource<R: Resource + Reflect + TypePath + GetTypeRegistration>(&mut self) -> &mut Self;
    /// `C` registered for reflection and pushed to React, by entity, whenever it changes (`useComponent`, `setComponent`).
    fn share_component<C: Component + Reflect + TypePath + GetTypeRegistration>(&mut self) -> &mut Self;
}

impl ShareExt for App {
    fn share_resource<R: Resource + Reflect + TypePath + GetTypeRegistration>(&mut self) -> &mut Self {
        setup(self);
        self.register_type::<R>();
        self.world_mut().resource_mut::<Shared>().0.insert(R::type_path().to_string());
        self.add_systems(Last, push_resource::<R>.in_set(ShareSystems::Push))
    }

    fn share_component<C: Component + Reflect + TypePath + GetTypeRegistration>(&mut self) -> &mut Self {
        setup(self);
        self.register_type::<C>();
        self.world_mut().resource_mut::<Shared>().0.insert(C::type_path().to_string());
        self.add_systems(Last, push_component::<C>.in_set(ShareSystems::Push))
    }
}

/// The channel in `app`, once (ReactPlugin adds it; share_resource too).
pub fn setup(app: &mut App) {
    if app.world().contains_resource::<Shared>() {
        return;
    }
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        fn named(v: &Value) -> Result<(String, Option<u64>), String> {
            let path = v.get("path").and_then(Value::as_str).ok_or("no path: a type path, as bevy.ts's handles name it")?;
            Ok((path.to_string(), v.get("entity").and_then(Value::as_u64)))
        }
        raea_native::register("world.write", "{ path: string; entity?: number; value: unknown }", "null", "a value React sets in a resource or a component, by its type path and reflected form (@raea/app's setResource, setComponent; B3 H4)", |v| {
            let (path, entity) = named(&v)?;
            ASKS.lock().unwrap().push(Ask::Write { path, entity, value: v.get("value").cloned().unwrap_or(Value::Null) });
            Ok(Value::Null)
        });
        raea_native::register("world.watch", "{ path: string; entity?: number } | { key: string; method: string; params?: unknown }", "null", "React reads a resource or a component (useResource, useComponent), or any BRP read by its key (useQuery): the World sends it each frame it changes", |v| {
            let ask = match v.get("key").and_then(Value::as_str) {
                Some(key) => {
                    let method = v.get("method").and_then(Value::as_str).ok_or("world.watch: a key's watch names its method")?;
                    if !raea_bevy_world::READS.contains(&method) && method != TWIN {
                        return Err(format!("world.watch: {method} isn't a read that can be watched: {}", raea_bevy_world::READS.join(", ")));
                    }
                    Ask::WatchRead { key: key.into(), method: method.into(), params: v.get("params").cloned().unwrap_or(Value::Null) }
                }
                None => {
                    let (path, entity) = named(&v)?;
                    Ask::Watch { path, entity }
                }
            };
            ASKS.lock().unwrap().push(ask);
            Ok(Value::Null)
        });
        raea_native::register("world.observe", "{ key: string; event?: string; entity?: number; on?: string; component?: string }", "null", "React observes an event (or a component's hook) in the World (useObserver): what it sees each frame is sent to React", |v| {
            if v.get("key").and_then(Value::as_str).is_none() {
                return Err("world.observe: a key".into());
            }
            ASKS.lock().unwrap().push(Ask::Observe { params: v.clone() });
            Ok(Value::Null)
        });
        raea_native::register("world.unobserve", "{ key: string }", "null", "React's observer is gone", |v| {
            let key = v.get("key").and_then(Value::as_str).ok_or("world.unobserve: a key")?;
            ASKS.lock().unwrap().push(Ask::Unobserve { key: key.into() });
            Ok(Value::Null)
        });
        raea_native::register("world.system", "{ key: string; query?: unknown }", "null", "a React system (useSystem): each frame, the frame's number and its world.query's rows are sent to React, which runs it and writes back", |v| {
            let key = v.get("key").and_then(Value::as_str).ok_or("world.system: a key")?;
            ASKS.lock().unwrap().push(Ask::System { key: key.into(), query: v.get("query").filter(|q| !q.is_null()).cloned() });
            Ok(Value::Null)
        });
        raea_native::register("world.unsystem", "{ key: string }", "null", "React's system is gone", |v| {
            let key = v.get("key").and_then(Value::as_str).ok_or("world.unsystem: a key")?;
            ASKS.lock().unwrap().push(Ask::Unsystem { key: key.into() });
            Ok(Value::Null)
        });
        raea_native::register("world.unwatch", "{ path: string; entity?: number } | { key: string }", "null", "React's last reader of a watched value is gone", |v| {
            let ask = match v.get("key").and_then(Value::as_str) {
                Some(key) => Ask::UnwatchRead { key: key.into() },
                None => {
                    let (path, entity) = named(&v)?;
                    Ask::Unwatch { path, entity }
                }
            };
            ASKS.lock().unwrap().push(ask);
            Ok(Value::Null)
        });
    });
    raea_bevy_world::register_natives();
    channel(app).add_systems(PreUpdate, (asks, raea_bevy_world::drain_unclaimed).chain().in_set(ShareSystems::Writes));
}

/// The channel's state and its Last systems (what changed collected, then sent), in React's App or an Engine's.
fn channel(app: &mut App) -> &mut App {
    app.init_resource::<Shared>()
        .init_resource::<Watched>()
        .init_resource::<Reads>()
        .init_resource::<Observed>()
        .init_resource::<Systems>()
        .init_resource::<WorldPushes>()
        .configure_sets(Last, (ShareSystems::Push, ShareSystems::Send).chain())
        .add_systems(Last, (watched.in_set(ShareSystems::Push), send.in_set(ShareSystems::Send)))
}

/// React's asks taken by React's App, in the order made; with engine_reads, all but twin reads are handed to the Engine
/// (an unwatch goes to both: a key's read is in one of them).
fn asks(world: &mut World) {
    let queued = std::mem::take(&mut *ASKS.lock().unwrap());
    let (here, engine) = split(queued, engine_reads());
    ENGINE_ASKS.lock().unwrap().extend(engine);
    apply(world, here);
}

/// React's asks as React's App's and the Engine's: all React's App's unless `to_engine`, else all the Engine's but twin
/// reads, and an unwatch both's.
fn split(queued: Vec<Ask>, to_engine: bool) -> (Vec<Ask>, Vec<Ask>) {
    if !to_engine {
        return (queued, Vec::new());
    }
    let (mut here, mut engine) = (Vec::new(), Vec::new());
    for a in queued {
        match a {
            Ask::WatchRead { ref method, .. } if method == TWIN => here.push(a),
            Ask::UnwatchRead { .. } => {
                here.push(a.clone());
                engine.push(a);
            }
            a => engine.push(a),
        }
    }
    (here, engine)
}

/// React's asks handed to the Engine, taken by its copy of the channel; ENGINE_LIVE keeps what's still watched.
fn engine_asks(world: &mut World) {
    let queued = std::mem::take(&mut *ENGINE_ASKS.lock().unwrap());
    if queued.is_empty() {
        return;
    }
    {
        let mut live = ENGINE_LIVE.lock().unwrap();
        for a in &queued {
            match a {
                Ask::Watch { .. } | Ask::WatchRead { .. } | Ask::Observe { .. } | Ask::System { .. } => live.push(a.clone()),
                Ask::Unwatch { path, entity } => live.retain(|l| !matches!(l, Ask::Watch { path: p, entity: e } if p == path && e == entity)),
                Ask::UnwatchRead { key } => live.retain(|l| !matches!(l, Ask::WatchRead { key: k, .. } if k == key)),
                Ask::Unobserve { key } => live.retain(|l| !matches!(l, Ask::Observe { params } if params["key"].as_str() == Some(key))),
                Ask::Unsystem { key } => live.retain(|l| !matches!(l, Ask::System { key: k, .. } if k == key)),
                Ask::Write { .. } => {}
            }
        }
    }
    apply(world, queued);
}

/// The channel in an Engine's App (raea-render's plugin "react_share", registered unless RAEA_ENGINE_READS=0): React's
/// asks handed on by its App run here, after the queue's calls, and what changes goes back to React. It starts with
/// the watches still live, for an Engine made after them.
pub fn engine_side(app: &mut App) {
    channel(app.insert_resource(EngineSide)).add_systems(PreUpdate, engine_asks.in_set(ShareSystems::Writes));
    let live = ENGINE_LIVE.lock().unwrap().clone();
    apply(app.world_mut(), live);
}

/// Asks applied to `world`: writes set, watches started and stopped.
fn apply(world: &mut World, queued: Vec<Ask>) {
    for a in queued {
        match a {
            Ask::Write { path, entity, value } => {
                let r = match entity {
                    None => raea_bevy_world::call(world, "world.insert_resources", json!({"resource": path, "value": value})),
                    Some(e) => raea_bevy_world::call(world, "world.insert_components", json!({"entity": e, "components": {&path: value}})),
                };
                if let Err(e) = r {
                    error!("raea: React's write to {path}: {e}");
                }
            }
            Ask::Watch { path, entity } => {
                world.resource_mut::<Watched>().0.entry((path, entity)).or_insert(None);
            }
            Ask::Unwatch { path, entity } => {
                world.resource_mut::<Watched>().0.remove(&(path, entity));
            }
            Ask::WatchRead { key, method, params } => {
                world.resource_mut::<Reads>().0.insert(key, (method, params, None));
            }
            Ask::UnwatchRead { key } => {
                world.resource_mut::<Reads>().0.remove(&key);
            }
            Ask::Observe { params } => {
                let key = params["key"].as_str().unwrap_or_default().to_string();
                match raea_bevy_world::observe(world, &params) {
                    Ok(_) => {
                        world.resource_mut::<Observed>().0.insert(key);
                    }
                    Err(e) => {
                        error!("raea: React's observer {key}: {e}");
                        world.resource_mut::<WorldPushes>().0.push(json!({"key": key, "events": [], "error": e}));
                    }
                }
            }
            Ask::Unobserve { key } => {
                raea_bevy_world::unobserve(world, &key);
                world.resource_mut::<Observed>().0.remove(&key);
            }
            Ask::System { key, query } => {
                world.resource_mut::<Systems>().0.insert(key, query);
            }
            Ask::Unsystem { key } => {
                world.resource_mut::<Systems>().0.remove(&key);
            }
        }
    }
}

/// Each watched value read now (BRP's world.get_resources / world.get_components); the ones that changed pushed.
fn watched(world: &mut World) {
    let asked: Vec<(String, Option<u64>)> = {
        let shared = &world.resource::<Shared>().0;
        world.resource::<Watched>().0.keys().filter(|(p, _)| !shared.contains(p)).cloned().collect()
    };
    for (path, entity) in asked {
        let now = match entity {
            None => raea_bevy_world::call(world, "world.get_resources", json!({"resource": path})).ok().and_then(|v| v.get("value").cloned()),
            Some(e) => raea_bevy_world::call(world, "world.get_components", json!({"entity": e, "components": [path], "strict": false}))
                .ok()
                .and_then(|v| v.get("components")?.get(&path).cloned()),
        };
        let mut w = world.resource_mut::<Watched>();
        let Some(last) = w.0.get_mut(&(path.clone(), entity)) else { continue };
        // a value not seen yet is sent even when absent, so a reader's first answer comes
        if last.as_ref() != Some(&now.clone().unwrap_or(Value::Null)) {
            *last = Some(now.clone().unwrap_or(Value::Null));
            let mut p = json!({"path": path, "value": now.unwrap_or(Value::Null)});
            if let Some(e) = entity {
                p["entity"] = json!(e);
            }
            world.resource_mut::<WorldPushes>().0.push(p);
        }
    }
    // React's systems: every frame, its number and the rows (React runs them on its thread, a frame behind)
    let systems: Vec<(String, Option<Value>)> = world.resource::<Systems>().0.iter().map(|(k, q)| (k.clone(), q.clone())).collect();
    let frame = world.get_resource::<bevy::diagnostic::FrameCount>().map_or(0, |f| f.0);
    for (key, query) in systems {
        let mut p = json!({"key": key, "tick": frame});
        if let Some(q) = query {
            match raea_bevy_world::call(world, "world.query", q) {
                Ok(v) => p["value"] = v,
                Err(e) => p["error"] = json!(e),
            }
        }
        world.resource_mut::<WorldPushes>().0.push(p);
    }
    let keys: Vec<String> = world.resource::<Observed>().0.iter().cloned().collect();
    for key in keys {
        let events = raea_bevy_world::observed(world, &key);
        if events.as_array().is_some_and(|a| !a.is_empty()) {
            world.resource_mut::<WorldPushes>().0.push(json!({"key": key, "events": events}));
        }
    }
    let reads: Vec<(String, String, Value)> = world.resource::<Reads>().0.iter().map(|(k, (m, p, _))| (k.clone(), m.clone(), p.clone())).collect();
    for (key, method, params) in reads {
        let r = if method == TWIN { Ok(twin(world, &params)) } else { raea_bevy_world::call(world, &method, params) };
        let now = match r {
            Ok(v) => json!({"key": key, "value": v}),
            Err(e) => json!({"key": key, "value": null, "error": e}),
        };
        let mut r = world.resource_mut::<Reads>();
        let Some(last) = r.0.get_mut(&key).map(|x| &mut x.2) else { continue };
        if last.as_ref() != Some(&now) {
            *last = Some(now.clone());
            world.resource_mut::<WorldPushes>().0.push(now);
        }
    }
}

/// The read that answers an element's twin: `{op}` (a React page's op id for the element) to the twin's entity bits, or
/// null while the element has none.
const TWIN: &str = "raea/twin";

fn twin(world: &mut World, p: &Value) -> Value {
    let Some(op) = p.get("op").and_then(Value::as_u64) else { return Value::Null };
    world.query::<(Entity, &raea_runtime_twins::PageElement)>().iter(world).find(|(_, el)| el.op == Some(op)).map(|(e, _)| json!(e.to_bits())).unwrap_or(Value::Null)
}

#[derive(SystemParam)]
struct Push<'w> {
    out: ResMut<'w, WorldPushes>,
    registry: Res<'w, AppTypeRegistry>,
}

impl Push<'_> {
    fn value(&mut self, path: &str, entity: Option<Entity>, value: Option<&dyn Reflect>) {
        let value = value.map(|v| {
            let reg = self.registry.read();
            serde_json::to_value(TypedReflectSerializer::new(v.as_partial_reflect(), &reg)).unwrap_or(Value::Null)
        });
        let mut p = json!({"path": path, "value": value.unwrap_or(Value::Null)});
        if let Some(e) = entity {
            p["entity"] = json!(e.to_bits());
        }
        self.out.0.push(p);
    }
}

fn push_resource<R: Resource + Reflect + TypePath>(r: Option<Res<R>>, mut had: Local<bool>, mut push: Push) {
    match &r {
        Some(r) if r.is_changed() => push.value(R::type_path(), None, Some(&**r)),
        None if *had => push.value(R::type_path(), None, None),
        _ => {}
    }
    *had = r.is_some();
}

fn push_component<C: Component + Reflect + TypePath>(changed: Query<(Entity, &C), Changed<C>>, mut gone: RemovedComponents<C>, mut push: Push) {
    for e in gone.read() {
        push.value(C::type_path(), Some(e), None);
    }
    for (e, c) in &changed {
        push.value(C::type_path(), Some(e), Some(c));
    }
}

/// This frame's changes to React, as one event, with the Engine's since the last; an Engine's App (no outbox) leaves
/// its own in ENGINE_OUT for React's.
fn send(mut pushes: ResMut<WorldPushes>, outbox: Option<Res<ReactOutbox>>, engine: Option<Res<EngineSide>>) {
    let Some(o) = outbox else {
        if engine.is_some() {
            ENGINE_OUT.lock().unwrap().append(&mut pushes.0);
        } else {
            pushes.0.clear();
        }
        return;
    };
    pushes.0.append(&mut ENGINE_OUT.lock().unwrap());
    if pushes.0.is_empty() {
        return;
    }
    let value = Value::Array(std::mem::take(&mut pushes.0));
    o.send(json!({"t": "event", "name": "raea-world", "value": value}));
}

/// The Engine's pushes sent to React now, ahead of the frame asked for after them (motion.rs engine_before), rather than
/// with React's App's next update.
pub fn engine_flush(outbox: &ReactOutbox) {
    let pushes = std::mem::take(&mut *ENGINE_OUT.lock().unwrap());
    if !pushes.is_empty() {
        outbox.send(json!({"t": "event", "name": "raea-world", "value": Value::Array(pushes)}));
    }
}
