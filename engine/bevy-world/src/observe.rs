//! Observers: what fires them, recorded, and the calls queued when they do.

use super::*;

/// What an observer does when it fires: records it, and queues its `then` calls for this same flush.
pub(crate) fn fired(world: WorldId, seen: &Arc<Mutex<Vec<Value>>>, then: &[Value], mut commands: Commands, entity: Value, event: Value) {
    seen.lock().unwrap().push(json!({"entity": entity, "event": event}));
    if then.is_empty() {
        return;
    }
    let mut p = PENDING.lock().unwrap();
    for t in then {
        p.push((world, t["method"].as_str().unwrap_or("").to_string(), fill(t.get("params").unwrap_or(&Value::Null), &entity, &event)));
    }
    // run at this flush's end; a trigger inside a BRP method (its registry taken out while it inserts) runs them when
    // the method returns instead (call)
    commands.queue(|w: &mut World| run_pending(w));
}

/// Observers' `then` calls waiting for the World to be whole, each for the World its observer is in (a process may
/// hold several: tests in parallel, a host's Apps).
pub(crate) static PENDING: Mutex<Vec<(WorldId, String, Value)>> = Mutex::new(Vec::new());

/// The observers' waiting `then` calls, run now in order (and any they cause), unless the registry is out.
pub fn run_pending(world: &mut World) {
    if !world.contains_resource::<AppTypeRegistry>() {
        return;
    }
    loop {
        let next: Vec<(String, Value)> = {
            let mut p = PENDING.lock().unwrap();
            let (mine, others): (Vec<_>, Vec<_>) = std::mem::take(&mut *p).into_iter().partition(|(w, ..)| *w == world.id());
            *p = others;
            mine.into_iter().map(|(_, m, v)| (m, v)).collect()
        };
        if next.is_empty() {
            return;
        }
        for (method, params) in next {
            if let Err(e) = call_one(world, &method, params) {
                eprintln!("raea-bevy-world: an observer's {method}: {e}");
            }
        }
    }
}

/// `raea/observe {key, event?, on?, component?, entity?, then?}`: an observer spawned in the World (see the top).
pub fn observe(world: &mut World, p: &Value) -> R<Value> {
    let key = p["key"].as_str().ok_or("raea/observe: a key")?.to_string();
    unobserve(world, &key);
    let then: Vec<Value> = p.get("then").and_then(Value::as_array).cloned().unwrap_or_default();
    let seen: Arc<Mutex<Vec<Value>>> = Arc::default();
    let registry = world.resource::<AppTypeRegistry>().clone();
    let observer = if let Some(on) = p["on"].as_str() {
        let path = p["component"].as_str().ok_or("raea/observe: a hook needs `component` (its type path)")?;
        let id = {
            let reg = registry.read();
            let r = reg.get_with_type_path(path).ok_or_else(|| format!("no reflected type {path}"))?;
            let rc = r.data::<bevy::ecs::reflect::ReflectComponent>().ok_or_else(|| format!("{path} isn't a component"))?.clone();
            drop(reg);
            rc.register_component(world)
        };
        let (s, t, wid) = (seen.clone(), then.clone(), world.id());
        macro_rules! hook {
            ($E:ty) => {
                Observer::new(move |e: On<$E>, c: Commands| fired(wid, &s, &t, c, json!(e.event_target().to_bits()), Value::Null))
            };
        }
        let o = match on {
            "Add" => hook!(Add<()>),
            "Insert" => hook!(Insert<()>),
            "Discard" | "Replace" => hook!(Discard<()>),
            "Remove" => hook!(Remove<()>),
            "Despawn" => hook!(Despawn<()>),
            _ => return Err(format!("raea/observe: `on` is Add, Insert, Discard, Remove or Despawn, not {on}")),
        };
        o.with_component(id)
    } else {
        let path = p["event"].as_str().ok_or("raea/observe: an `event` (a reflected event's type path) or a hook (`on`, `component`)")?.to_string();
        let re = {
            let reg = registry.read();
            let r = reg.get_with_type_path(&path).ok_or_else(|| format!("no reflected type {path}"))?;
            r.data::<bevy::ecs::reflect::ReflectEvent>().ok_or_else(|| format!("{path} isn't a reflected event (#[reflect(Event)])"))?.clone()
        };
        let (s, t, reg2, wid) = (seen.clone(), then.clone(), registry.clone(), world.id());
        re.create_observer(Box::new(move |ev: &dyn Reflect, mut w: DeferredWorld| {
            let value = {
                let reg = reg2.read();
                serde_json::to_value(bevy::reflect::serde::TypedReflectSerializer::new(ev.as_partial_reflect(), &reg)).unwrap_or(Value::Null)
            };
            let target = value.get("entity").cloned().unwrap_or(Value::Null);
            fired(wid, &s, &t, w.commands(), target, value);
        }))
    };
    let observer = match p.get("entity").filter(|e| !e.is_null()) {
        Some(e) => {
            let bits = entity(world, e)?;
            observer.with_entity(Entity::try_from_bits(bits.as_u64().unwrap_or(u64::MAX)).ok_or("raea/observe: entity")?)
        }
        None => observer,
    };
    let e = world.spawn(observer).id();
    world.get_resource_or_init::<Observers>().0.insert(key.clone(), Observing { observer: e, seen });
    Ok(json!({"key": key, "observer": e.to_bits()}))
}

/// What an observer recorded since the last read: [{entity, event}].
pub fn observed(world: &mut World, key: &str) -> Value {
    world.get_resource::<Observers>().and_then(|o| o.0.get(key)).map(|o| Value::Array(std::mem::take(&mut *o.seen.lock().unwrap()))).unwrap_or(Value::Array(vec![]))
}

pub fn unobserve(world: &mut World, key: &str) {
    let gone = world.get_resource_mut::<Observers>().and_then(|mut o| o.0.remove(key));
    if let Some(o) = gone {
        if let Ok(e) = world.get_entity_mut(o.observer) {
            e.despawn();
        }
    }
}

// ---- watched reads

pub(crate) struct Watch {
    pub(crate) key: String,
    pub(crate) method: String,
    pub(crate) params: Value,
    /// what was last sent (the answer, or {error})
    pub(crate) last: Option<Value>,
}

/// The watched reads, in the order asked.
#[derive(Resource, Default)]
pub struct Watches(pub(crate) Vec<Watch>);

/// A `watch` ask {key, method, params}: a new key starts at its next read; the same key again replaces what it reads.
pub fn watch(world: &mut World, v: &Value) -> R<()> {
    let (Some(key), Some(method)) = (v["key"].as_str(), v["method"].as_str()) else {
        return Err(format!("watch: needs a key and a method: {v}"));
    };
    let w = Watch { key: key.into(), method: method.into(), params: v.get("params").cloned().unwrap_or(Value::Null), last: None };
    let mut ws = world.get_resource_or_init::<Watches>();
    match ws.0.iter_mut().find(|w| w.key == key) {
        Some(old) => *old = w,
        None => ws.0.push(w),
    }
    Ok(())
}

pub fn unwatch(world: &mut World, key: &str) {
    if let Some(mut ws) = world.get_resource_mut::<Watches>() {
        ws.0.retain(|w| w.key != key);
    }
}
