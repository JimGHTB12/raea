//! The Bevy world from TypeScript, one layer for every runtime (docs/bevy-ts.feature G16). Each host used to carry its
//! own in-process Bevy Remote Protocol bridge (raea-bevy-plugins' bridge.rs for BevyApp programs, the legacy app's world.rs
//! for spaces); both now run their calls here, and so does a live game on raea-render's Engine (through the queue below).
//!
//!   call(world, method, params)   one BRP method run now on the World: the App's bevy_remote method table when it
//!                                 has one (BRP's built-ins plus whatever a plugin added: avian3d/cast_ray,
//!                                 lightyear/connect, raea/state.get), else BRP's built-in handlers called directly
//!                                 (an App without RemotePlugin: raea-render's)
//!   EntityNames                   how the host names entities: `entity`, `parent` and each of `entities` in the
//!                                 params may be a host name (the legacy app: {testId} or {el}; raea-render: a part's name or row id),
//!                                 resolved at each call; bits pass as they are
//!   Watches, watch, changed       a read run every frame, answered only when its answer changed (a signal)
//!   raea/observe {key, ...}      an observer inside the App (G16b): on a reflected event's trigger, or a component
//!                                 hook (`on`: Add, Insert, Discard (0.18 and before: Replace), Remove, Despawn, of `component`), it records
//!                                 what happened and runs its `then` calls right there, in the same flush, with
//!                                 "$entity" and "$event" in their params standing for the target and the event;
//!                                 raea/observed {key} drains what it recorded, raea/unobserve {key} removes it
//!   raea/state.get {state}       a reflected state's value (#[reflect(State)]), and raea/state.set {state, value} its
//!   raea/state.set               next one (at the next StateTransition): every World answers them, so a hook
//!                                (useAppState) or a BevyApp reads and sets an App's states alike
//!   raea/messages.read {message} this frame's messages of a type (Bevy's newer buffer of Messages<M>, as useMessages
//!   raea/messages.write          reads them; the App registers Messages<M> for reflection, which Bevy doesn't do for a
//!                                generic), and raea/messages.write {message, value} writes one (#[reflect(Message)])
//!   send / drain / answer         the live game's queue: TypeScript runs before the Engine's update, and holds no
//!                                 handle on its World, so a call is queued (`bevy.world.send`, a ticket), run at the
//!                                 Engine's next update (`drain`, BRP's own model), and its answer read from then on
//!                                 (`bevy.world.answer`)

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use bevy::ecs::lifecycle::{Add, Despawn, Discard, Insert, Remove};
use bevy::ecs::world::{DeferredWorld, WorldId};

use bevy::ecs::system::In;
use bevy::prelude::*;
use bevy_remote::builtin_methods as brp;
use bevy_remote::{RemoteMethodSystemId, RemoteMethods};
use serde_json::{Value, json};

mod observe;
pub use observe::*;
#[cfg(test)]
mod tests;

type R<T> = Result<T, String>;

/// BRP's read-only methods: the ones a watch may run.
pub const READS: &[&str] = &["raea/state.get", "raea/messages.read", "world.get_components", "world.get_resources", "world.query", "world.list_components", "world.list_resources", "registry.schema", "schedule.list", "schedule.graph"];
/// BRP's built-in methods that change the world.
pub const WRITES: &[&str] = &[
    "world.spawn_entity",
    "world.despawn_entity",
    "world.insert_components",
    "world.mutate_components",
    "world.remove_components",
    "world.reparent_entities",
    "world.insert_resources",
    "world.mutate_resources",
    "world.remove_resources",
    "world.write_message",
    "world.trigger_event",
];

/// How a host names its entities: a name in params (anything but a number) to an entity, Ok(None) when nothing has
/// that name now, Err when the name isn't one this host reads.
#[derive(Resource, Clone, Copy)]
pub struct EntityNames(pub fn(&mut World, &Value) -> R<Option<Entity>>);

fn entity(world: &mut World, e: &Value) -> R<Value> {
    if e.is_number() || e.is_null() {
        return Ok(e.clone());
    }
    let Some(names) = world.get_resource::<EntityNames>().copied() else {
        return Err(format!("an entity here is its bits, not {e}"));
    };
    names.0(world, e)?.map(|x| json!(x.to_bits())).ok_or_else(|| format!("no entity {e}"))
}

/// The params with every entity in them (`entity`, `parent`, each of `entities`) as BRP takes it.
fn resolve(world: &mut World, p: &Value) -> R<Value> {
    let mut p = p.clone();
    for k in ["entity", "parent"] {
        if let Some(e) = p.get(k).cloned() {
            p[k] = entity(world, &e)?;
        }
    }
    if let Some(es) = p.get("entities").and_then(Value::as_array).cloned() {
        p["entities"] = Value::Array(es.iter().map(|e| entity(world, e)).collect::<R<_>>()?);
    }
    Ok(p)
}

/// A method of the World's bevy_remote table run now (an instant method's answer, or a watching method's answer for
/// this frame: null when it has nothing new). None when the World has no table or no such method.
pub fn remote(world: &mut World, name: &str, params: Value) -> Option<R<Value>> {
    let r = match host_method(world, name) {
        Some(f) => Some(resolve(world, &params).and_then(|p| f(world, p))),
        None => remote_one(world, name, params),
    };
    // what observers queued while the method held the World's registry
    run_pending(world);
    r
}

fn remote_one(world: &mut World, name: &str, params: Value) -> Option<R<Value>> {
    let id = *world.get_resource::<RemoteMethods>()?.get(name)?;
    let params = if params.is_null() { None } else { Some(params) };
    let err = |e: bevy_remote::BrpError| match e.data {
        Some(d) => format!("{} ({d})", e.message),
        None => e.message,
    };
    Some(match id {
        RemoteMethodSystemId::Instant(id) => match world.run_system_with(id, params) {
            Ok(r) => r.map_err(err),
            Err(e) => Err(format!("{name}: {e}")),
        },
        RemoteMethodSystemId::Watching(id) => match world.run_system_with(id, params) {
            Ok(r) => r.map(|v| v.unwrap_or(Value::Null)).map_err(err),
            Err(e) => Err(format!("{name}: {e}")),
        },
    })
}

/// Methods a host adds or takes over (raea-render: world.spawn_entity and world.insert_components through its reflect
/// processor, so a Handle field takes a path or an inline asset and an Entity field a name; raea/world.save and
/// raea/world.load; raea/diagnostics). Asked before BRP's own table, with the params' entities already resolved.
#[derive(Resource, Default, Clone)]
pub struct Methods(pub BTreeMap<String, HostMethod>);

pub type HostMethod = fn(&mut World, Value) -> R<Value>;

fn host_method(world: &World, name: &str) -> Option<HostMethod> {
    world.get_resource::<Methods>()?.0.get(name).copied()
}

/// One BRP method run now on the World, its entities resolved by the host's names: what it answers over BRP.
pub fn call(world: &mut World, method: &str, params: Value) -> R<Value> {
    let r = call_one(world, method, params);
    run_pending(world);
    r
}

fn call_one(world: &mut World, method: &str, params: Value) -> R<Value> {
    match method {
        "raea/observe" => return observe(world, &params),
        "raea/observed" => return Ok(observed(world, params["key"].as_str().unwrap_or(""))),
        "raea/unobserve" => {
            unobserve(world, params["key"].as_str().unwrap_or(""));
            return Ok(Value::Null);
        }
        // a host's own table (a BevyApp's) answers them as it does; otherwise here
        "raea/state.get" | "raea/state.set" if host_method(world, method).is_none() && world.get_resource::<RemoteMethods>().and_then(|m| m.get(method)).is_none() => {
            let state = params.get("state").and_then(Value::as_str).ok_or_else(|| format!("{method}: needs a state (its type path)"))?;
            return if method == "raea/state.get" { state_get(world, state) } else { state_set(world, state, params.get("value").cloned().unwrap_or(Value::Null)).map(|_| Value::Null) };
        }
        "raea/messages.read" | "raea/messages.write" if host_method(world, method).is_none() && world.get_resource::<RemoteMethods>().and_then(|m| m.get(method)).is_none() => {
            let m = params.get("message").and_then(Value::as_str).ok_or_else(|| format!("{method}: needs a message (its type path)"))?;
            return if method == "raea/messages.read" { messages_read(world, m) } else { message_write(world, m, params.get("value").cloned().unwrap_or(Value::Null)).map(|_| Value::Null) };
        }
        _ => {}
    }
    let p = resolve(world, &params)?;
    if let Some(f) = host_method(world, method) {
        return f(world, p);
    }
    if let Some(r) = remote_one(world, method, p.clone()) {
        return r;
    }
    let p = (!p.is_null()).then_some(p);
    let r = match method {
        "world.get_components" => brp::process_remote_get_components_request(In(p), world),
        "world.get_resources" => brp::process_remote_get_resources_request(In(p), world),
        "world.query" => brp::process_remote_query_request(In(p), world),
        "world.list_components" => brp::process_remote_list_components_request(In(p), world),
        "world.list_resources" => brp::process_remote_list_resources_request(In(p), world),
        "registry.schema" => brp::export_registry_types(In(p), world),
        "schedule.list" => brp::schedule_list(In(p), world),
        "schedule.graph" => brp::schedule_graph(In(p), world),
        "world.spawn_entity" => brp::process_remote_spawn_entity_request(In(p), world),
        "world.despawn_entity" => brp::process_remote_despawn_entity_request(In(p), world),
        "world.insert_components" => brp::process_remote_insert_components_request(In(p), world),
        "world.mutate_components" => brp::process_remote_mutate_components_request(In(p), world),
        "world.remove_components" => brp::process_remote_remove_components_request(In(p), world),
        "world.reparent_entities" => brp::process_remote_reparent_entities_request(In(p), world),
        "world.insert_resources" => brp::process_remote_insert_resources_request(In(p), world),
        "world.mutate_resources" => brp::process_remote_mutate_resources_request(In(p), world),
        "world.remove_resources" => brp::process_remote_remove_resources_request(In(p), world),
        "world.write_message" => brp::process_remote_write_message_request(In(p), world),
        "world.trigger_event" => brp::process_remote_trigger_event_request(In(p), world),
        _ => return Err(format!("no world method {method}: {}, {}", READS.join(", "), WRITES.join(", "))),
    };
    r.map_err(|e| e.message)
}

// ---- states

/// A reflected state's value (#[reflect(State)]), as BRP writes values; null when the App hasn't the state now.
pub fn state_get(world: &World, state: &str) -> R<Value> {
    use bevy::reflect::serde::ReflectSerializer;
    use bevy::state::reflect::ReflectState;
    let registry = world.resource::<AppTypeRegistry>().read();
    let r = registry.get_with_type_path(state).ok_or_else(|| format!("no reflected type {state}"))?;
    let rs = r.data::<ReflectState>().ok_or_else(|| format!("{state} isn't a reflected state (#[reflect(State)])"))?;
    match rs.reflect(world) {
        Some(v) => serde_json::to_value(ReflectSerializer::new(v.as_partial_reflect(), &registry)).map(|j| j.get(state).cloned().unwrap_or(j)).map_err(|e| e.to_string()),
        None => Ok(Value::Null),
    }
}

/// A state's next value (it changes at the next StateTransition, in the next frame).
pub fn state_set(world: &mut World, state: &str, value: Value) -> R<()> {
    use bevy::reflect::serde::TypedReflectDeserializer;
    use bevy::state::reflect::ReflectFreelyMutableState;
    use serde::de::DeserializeSeed;
    let registry = world.resource::<AppTypeRegistry>().clone();
    let reg = registry.read();
    let r = reg.get_with_type_path(state).ok_or_else(|| format!("no reflected type {state}"))?;
    let fm = r.data::<ReflectFreelyMutableState>().ok_or_else(|| format!("{state} can't be set (a computed or sub state, or not #[reflect(State, FreelyMutableState)])"))?.clone();
    let v = TypedReflectDeserializer::new(r, &reg).deserialize(value).map_err(|e| format!("{state}: {e}"))?;
    let full = r.data::<bevy::reflect::ReflectFromReflect>().and_then(|f| f.from_reflect(v.as_partial_reflect())).ok_or_else(|| format!("{state}: give the whole value"))?;
    fm.set_next_state(world, full.as_reflect(), &reg);
    Ok(())
}

// ---- messages

/// This frame's messages of type `message` (Messages<M>'s newer buffer: what was written since the last
/// message_update_system, in First), as BRP writes values. Messages<M> must be registered for reflection.
pub fn messages_read(world: &mut World, message: &str) -> R<Value> {
    // Messages<M>'s type path, as bevy_reflect writes a generic's: Bevy's own prefix and M's path
    let generic = <bevy::ecs::message::Messages<bevy::app::AppExit> as TypePath>::type_path();
    let want = format!("{}<{message}>", &generic[..generic.find('<').unwrap_or(generic.len())]);
    if world.resource::<AppTypeRegistry>().read().get_with_type_path(&want).is_none() {
        return Err(format!("{message}: register Messages<{message}> for reflection (app.register_type::<Messages<M>>())"));
    }
    let got = match call_one(world, "world.get_resources", json!({"resource": want})) {
        Ok(v) => v,
        // not in the World (no add_message yet): none
        Err(_) => return Ok(json!([])),
    };
    let list = got["value"]["messages_b"]["messages"].as_array().cloned().unwrap_or_default();
    Ok(Value::Array(list.into_iter().map(|m| m["message"].clone()).collect()))
}

/// One message written (MessageWriter's write), from its value: a #[reflect(Message)] type.
pub fn message_write(world: &mut World, message: &str, value: Value) -> R<()> {
    use bevy::ecs::reflect::ReflectMessage;
    use bevy::reflect::serde::TypedReflectDeserializer;
    use serde::de::DeserializeSeed;
    let registry = world.resource::<AppTypeRegistry>().clone();
    let reg = registry.read();
    let r = reg.get_with_type_path(message).ok_or_else(|| format!("no reflected type {message}"))?;
    let w = r.data::<ReflectMessage>().ok_or_else(|| format!("{message} isn't a reflected message (#[reflect(Message)])"))?.clone();
    let v = TypedReflectDeserializer::new(r, &reg).deserialize(value).map_err(|e| format!("{message}: {e}"))?;
    w.write_message(world, v.as_partial_reflect(), &reg);
    Ok(())
}

// ---- observers and hooks inside the App

struct Observing {
    observer: Entity,
    seen: Arc<Mutex<Vec<Value>>>,
}

/// The observers TypeScript made, by key.
#[derive(Resource, Default)]
pub struct Observers(BTreeMap<String, Observing>);

/// `then`'s params with "$entity" and "$event" put in.
fn fill(v: &Value, entity: &Value, event: &Value) -> Value {
    match v {
        Value::String(s) if s == "$entity" => entity.clone(),
        Value::String(s) if s == "$event" => event.clone(),
        Value::Array(a) => Value::Array(a.iter().map(|x| fill(x, entity, event)).collect()),
        Value::Object(o) => Value::Object(o.iter().map(|(k, x)| (k.clone(), fill(x, entity, event))).collect()),
        x => x.clone(),
    }
}

/// Each watch read now; the ones whose answer changed since it was last sent: {key, value} or {key, value: null, error}.
pub fn changed(world: &mut World) -> Vec<Value> {
    let Some(ws) = world.get_resource::<Watches>() else { return vec![] };
    let asks: Vec<(usize, String, Value)> = ws.0.iter().enumerate().map(|(i, w)| (i, w.method.clone(), w.params.clone())).collect();
    let mut out = vec![];
    for (i, method, params) in asks {
        let now = match READS.contains(&method.as_str()).then(|| call(world, &method, params)).unwrap_or_else(|| Err(format!("{method} isn't a read that can be watched: {}", READS.join(", ")))) {
            Ok(v) => json!({"value": v}),
            Err(e) => json!({"value": null, "error": e}),
        };
        let mut ws = world.resource_mut::<Watches>();
        let Some(w) = ws.0.get_mut(i) else { continue };
        if w.last.as_ref() != Some(&now) {
            let mut m = json!({"key": w.key});
            m.as_object_mut().unwrap().extend(now.as_object().unwrap().clone());
            out.push(m);
            w.last = Some(now);
        }
    }
    out
}

// ---- the live game's queue

struct Queue {
    next: u64,
    asked: Vec<(u64, String, Value)>,
    answers: BTreeMap<u64, Value>,
}

static QUEUE: Mutex<Queue> = Mutex::new(Queue { next: 0, asked: Vec::new(), answers: BTreeMap::new() });
/// answers kept for reading, the oldest let go past this
const KEPT: usize = 4096;

/// A call queued for the Engine's next update: its ticket.
pub fn send(method: &str, params: Value) -> u64 {
    let mut q = QUEUE.lock().unwrap();
    q.next += 1;
    let t = q.next;
    q.asked.push((t, method.to_string(), params));
    t
}

/// A ticket's answer once its call has run: {value} or {error}; None before.
pub fn answer(ticket: u64) -> Option<Value> {
    QUEUE.lock().unwrap().answers.get(&ticket).cloned()
}

/// Set once an App that names entities as the edit does (raea-render's Engine) runs the queue: the process's other
/// Apps (React's, sdk/react shared.rs) then leave the calls to it, and with RAEA_ENGINE_READS their reads too.
pub static CLAIMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The queue taken by this process's Engine: drain_unclaimed does nothing from now on.
pub fn claim() {
    CLAIMED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// drain, unless an Engine has claimed the queue (a plain App's: React's in a process without one).
pub fn drain_unclaimed(world: &mut World) {
    if !CLAIMED.load(std::sync::atomic::Ordering::Relaxed) {
        drain(world);
    }
}

/// The queued calls run on this World, in the order sent (an exclusive system: raea-render's Engine runs it each update).
pub fn drain(world: &mut World) {
    let asked = std::mem::take(&mut QUEUE.lock().unwrap().asked);
    if asked.is_empty() {
        return;
    }
    let done: Vec<(u64, Value)> = asked.into_iter().map(|(t, m, p)| (t, match call(world, &m, p) {
        Ok(v) => json!({"value": v}),
        Err(e) => json!({"error": e}),
    })).collect();
    let mut q = QUEUE.lock().unwrap();
    q.answers.extend(done);
    while q.answers.len() > KEPT {
        q.answers.pop_first();
    }
}

/// The queue's native calls (raea-core-calls), for a live game's TypeScript (sdk/raea/src/world.ts).
/// A live game's window, as its TypeScript asks for it (raea/window, docs/bevy-ts.feature G19): title, size
/// [w, h], fullscreen, resizable, cursor ("visible" | "hidden" | "locked" | "confined"), decorations. The host that
/// owns the window (raea play, the legacy app's stage) applies what changed; `version` goes up on each set.
static WINDOW: Mutex<(u64, Option<Value>)> = Mutex::new((0, None));

/// The window settings and their version (0 before any).
pub fn window_settings() -> (u64, Option<Value>) {
    WINDOW.lock().unwrap().clone()
}

fn set_window(v: &Value) -> R<Value> {
    let o = v.as_object().ok_or("window settings are an object")?;
    for (k, x) in o {
        let ok = match k.as_str() {
            "title" => x.is_string(),
            "size" => x.as_array().is_some_and(|a| a.len() == 2 && a.iter().all(|n| n.as_f64().is_some_and(|n| n >= 1.0))),
            "fullscreen" | "resizable" | "decorations" => x.is_boolean(),
            "cursor" => matches!(x.as_str(), Some("visible" | "hidden" | "locked" | "confined")),
            _ => return Err(format!("no window setting {k:?} (title, size, fullscreen, resizable, decorations, cursor)")),
        };
        if !ok {
            return Err(format!("window setting {k}: {x} isn't one it takes"));
        }
    }
    let mut w = WINDOW.lock().unwrap();
    let mut merged = w.1.clone().unwrap_or_else(|| json!({}));
    for (k, x) in o {
        merged[k] = x.clone();
    }
    w.0 += 1;
    w.1 = Some(merged.clone());
    Ok(merged)
}

pub fn register_natives() {
    if raea_core_calls::has("bevy.world.send") {
        return;
    }
    raea_core_calls::register("raea.window.set", "{ title?: string; size?: [number, number]; fullscreen?: boolean; resizable?: boolean; decorations?: boolean; cursor?: \"visible\" | \"hidden\" | \"locked\" | \"confined\" }", "Record<string, unknown>", "the live game's window settings changed (merged over the ones set before); the window's host applies them", set_window);
    raea_core_calls::register("raea.window.get", "{}", "Record<string, unknown> | null", "the window settings set so far", |_| Ok(WINDOW.lock().unwrap().1.clone().unwrap_or(Value::Null)));
    raea_core_calls::register("bevy.world.send", "{ method: string; params?: unknown }", "number", "a BRP method (world.query, world.mutate_components, a plugin's method...) queued for the live Engine's next update: its ticket", |a| {
        let method = a.get("method").and_then(Value::as_str).ok_or("method: a BRP method's name")?;
        Ok(json!(send(method, a.get("params").cloned().unwrap_or(Value::Null))))
    });
    raea_core_calls::register("bevy.world.answer", "{ ticket: number }", "{ value?: unknown; error?: string } | null", "a queued call's answer once the Engine has run it (null before)", |a| {
        let t = a.get("ticket").and_then(Value::as_u64).ok_or("ticket: from bevy.world.send")?;
        Ok(answer(t).unwrap_or(Value::Null))
    });
}
