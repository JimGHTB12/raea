//! The world API (docs/ts-native.feature P22, docs/bevy-ts.feature): a root reaches the whole Bevy world through the
//! Bevy Remote Protocol's own methods, run in-process by engine/bevy-world (the layer every raea runtime shares), so TS
//! can do whatever BRP can on anything Reflect and registered, with no binding of ours:
//!
//!   reads   world.get_components, world.get_resources, world.query, world.list_components, world.list_resources,
//!           registry.schema, schedule.list, schedule.graph
//!   writes  world.spawn_entity, world.despawn_entity, world.insert_components, world.mutate_components,
//!           world.remove_components, world.reparent_entities, world.insert_resources, world.mutate_resources,
//!           world.remove_resources, world.write_message, world.trigger_event
//!
//! A call (`__ui_call`, any of them) is answered with the method's result. A read can also be watched, as a signal:
//! the host runs it at every frame's step and hands the answer back whenever it differs from the last one it sent.
//!
//! Asks (JS -> host, `__ui_emit`): `watch` {key, method, params}, `unwatch` {key}. An entity in `params` (`entity`,
//! `parent`, each of `entities`) is its bits, or a name the host's `EntityNames` reads. Answers (host -> JS): {t: "ecs", key, value} or {t: "ecs", key, value: null, error}.
//! Each root's keys are its own (kept apart here by the root's entity).

use bevy::prelude::*;
use serde_json::Value;

pub use raea_bevy_world::{READS, WRITES, Watches};

const SEP: char = '\u{1f}';

fn key_of(root: Entity, key: &str) -> String {
    format!("{}{SEP}{key}", root.to_bits())
}

/// A `watch` ask: a new key starts at its next read; the same key again replaces what it reads.
pub fn watch(world: &mut World, root: Entity, v: &Value) {
    let mut v = v.clone();
    if let Some(k) = v["key"].as_str() {
        v["key"] = Value::from(key_of(root, k));
    }
    if let Err(e) = raea_bevy_world::watch(world, &v) {
        eprintln!("raea-runtime-js: {e}");
    }
}

pub fn unwatch(world: &mut World, root: Entity, v: &Value) {
    raea_bevy_world::unwatch(world, &key_of(root, v["key"].as_str().unwrap_or("")));
}

/// Each watch read now; the ones whose answer changed since it was last sent, as messages for their roots.
pub fn changed(world: &mut World) -> Vec<(Entity, Value)> {
    if !world.contains_resource::<Watches>() {
        return vec![];
    }
    raea_bevy_world::changed(world)
        .into_iter()
        .filter_map(|mut m| {
            let full = m["key"].as_str()?.to_string();
            let (bits, key) = full.split_once(SEP)?;
            let root = Entity::try_from_bits(bits.parse().ok()?)?;
            m["key"] = Value::from(key);
            m["t"] = Value::from("ecs");
            Some((root, m))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const T: &str = "bevy_transform::components::transform::Transform";

    fn world() -> World {
        let mut w = World::new();
        let reg = AppTypeRegistry::default();
        {
            let mut r = reg.write();
            r.register::<Transform>();
        }
        w.insert_resource(reg);
        w
    }

    #[test]
    fn spawn_mutate_get_query() {
        let mut w = world();
        let e = raea_bevy_world::call(&mut w, "world.spawn_entity", json!({"components": {T: {"translation": [1.0, 2.0, 3.0], "rotation": [0.0, 0.0, 0.0, 1.0], "scale": [1.0, 1.0, 1.0]}}})).unwrap();
        let e = e["entity"].clone();
        raea_bevy_world::call(&mut w, "world.mutate_components", json!({"entity": e, "component": T, "path": "translation.y", "value": 5.0})).unwrap();
        let got = raea_bevy_world::call(&mut w, "world.get_components", json!({"entity": e, "components": [T]})).unwrap();
        assert_eq!(got["components"][T]["translation"], json!([1.0, 5.0, 3.0]));
        assert!(raea_bevy_world::call(&mut w, "world.nope", Value::Null).is_err());
    }

    #[test]
    fn watches_are_each_root_s_own() {
        let mut w = world();
        w.init_resource::<Watches>();
        let (a, b) = (w.spawn_empty().id(), w.spawn_empty().id());
        let e = w.spawn(Transform::default()).id();
        let ask = json!({"key": "k", "method": "world.get_components", "params": {"entity": e.to_bits(), "components": [T]}});
        watch(&mut w, a, &ask);
        watch(&mut w, b, &ask);
        let m = changed(&mut w);
        assert_eq!(m.len(), 2);
        assert_eq!((m[0].0, m[1].0), (a, b));
        assert_eq!(m[0].1["key"], "k");
        assert!(changed(&mut w).is_empty());
        w.get_mut::<Transform>(e).unwrap().translation.x = 4.0;
        unwatch(&mut w, b, &json!({"key": "k"}));
        let m = changed(&mut w);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].0, a);
    }
}
