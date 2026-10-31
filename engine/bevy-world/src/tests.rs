use super::*;

const T: &str = "bevy_transform::components::transform::Transform";

fn world() -> World {
    let mut w = World::new();
    let reg = AppTypeRegistry::default();
    reg.write().register::<Transform>();
    reg.write().register::<Name>();
    w.insert_resource(reg);
    w
}

fn by_name(world: &mut World, v: &Value) -> R<Option<Entity>> {
    let n = v.as_str().ok_or("a name")?;
    Ok(world.query::<(Entity, &Name)>().iter(world).find(|(_, m)| m.as_str() == n).map(|(e, _)| e))
}

#[test]
fn spawn_mutate_get_query() {
    let mut w = world();
    let e = call(&mut w, "world.spawn_entity", json!({"components": {T: {"translation": [1.0, 2.0, 3.0], "rotation": [0.0, 0.0, 0.0, 1.0], "scale": [1.0, 1.0, 1.0]}}})).unwrap()["entity"].clone();
    call(&mut w, "world.mutate_components", json!({"entity": e, "component": T, "path": "translation.y", "value": 5.0})).unwrap();
    let got = call(&mut w, "world.get_components", json!({"entity": e, "components": [T]})).unwrap();
    assert_eq!(got["components"][T]["translation"], json!([1.0, 5.0, 3.0]));
    assert!(call(&mut w, "world.nope", Value::Null).is_err());
}

#[derive(Event, Reflect, Clone)]
#[reflect(Event)]
struct Ping {
    to: f32,
}

#[test]
fn hooks_and_observers_run_inside_the_world() {
    let mut w = world();
    w.resource::<AppTypeRegistry>().write().register::<Ping>();
    w.insert_resource(EntityNames(by_name));
    let stage = w.spawn((Name::new("stage"), Transform::default())).id();
    // a hook: a Name added gets a Transform at x 9, in the same flush as the spawn
    observe(&mut w, &json!({"key": "named", "on": "Add", "component": "bevy_ecs::name::Name",
        "then": [{"method": "world.insert_components", "params": {"entity": "$entity", "components": {T: {"translation": [9.0, 0.0, 0.0], "rotation": [0.0, 0.0, 0.0, 1.0], "scale": [1.0, 1.0, 1.0]}}}}]})).unwrap();
    let e = call(&mut w, "world.spawn_entity", json!({"components": {"bevy_ecs::name::Name": "kid"}})).unwrap()["entity"].clone();
    let got = call(&mut w, "world.get_components", json!({"entity": e, "components": [T]})).unwrap();
    assert_eq!(got["components"][T]["translation"][0], json!(9.0));
    assert_eq!(observed(&mut w, "named")[0]["entity"], e);
    assert_eq!(observed(&mut w, "named"), json!([]));
    // an observer of a reflected event: its value moves the stage
    call(&mut w, "raea/observe", json!({"key": "ping", "event": std::any::type_name::<Ping>(),
        "then": [{"method": "world.mutate_components", "params": {"entity": "stage", "component": T, "path": "translation.y", "value": 3.0}}]})).unwrap();
    call(&mut w, "world.trigger_event", json!({"event": std::any::type_name::<Ping>(), "value": {"to": 1.0}})).unwrap();
    w.flush();
    assert_eq!(w.get::<Transform>(stage).unwrap().translation.y, 3.0);
    assert_eq!(call(&mut w, "raea/observed", json!({"key": "ping"})).unwrap()[0]["event"]["to"], json!(1.0));
    call(&mut w, "raea/unobserve", json!({"key": "named"})).unwrap();
    let e2 = call(&mut w, "world.spawn_entity", json!({"components": {"bevy_ecs::name::Name": "late"}})).unwrap()["entity"].clone();
    assert!(call(&mut w, "world.get_components", json!({"entity": e2, "components": [T], "strict": false})).unwrap()["components"].get(T).is_none());
}

#[derive(Message, Reflect, Clone)]
#[reflect(Message)]
struct Hit {
    dmg: u32,
}

#[test]
fn messages_read_this_frames_and_write_one() {
    let mut w = world();
    w.init_resource::<bevy::ecs::message::Messages<Hit>>();
    {
        let reg = w.resource::<AppTypeRegistry>().clone();
        let mut reg = reg.write();
        reg.register::<Hit>();
        reg.register::<bevy::ecs::message::Messages<Hit>>();
    }
    let path = std::any::type_name::<Hit>();
    assert_eq!(call(&mut w, "raea/messages.read", json!({"message": path})).unwrap(), json!([]));
    w.write_message(Hit { dmg: 3 });
    call(&mut w, "raea/messages.write", json!({"message": path, "value": {"dmg": 4}})).unwrap();
    assert_eq!(call(&mut w, "raea/messages.read", json!({"message": path})).unwrap(), json!([{"dmg": 3}, {"dmg": 4}]));
    // the next frame's buffer: these are the older one's now
    w.resource_mut::<bevy::ecs::message::Messages<Hit>>().update();
    assert_eq!(call(&mut w, "raea/messages.read", json!({"message": path})).unwrap(), json!([]));
    assert!(call(&mut w, "raea/messages.read", json!({"message": "nowhere::Gone"})).unwrap_err().contains("register Messages"));
}

#[test]
fn host_names_and_watches_and_the_queue() {
    let mut w = world();
    w.insert_resource(EntityNames(by_name));
    let e = w.spawn((Name::new("stage"), Transform::from_xyz(7.0, 0.0, 0.0))).id();
    let got = call(&mut w, "world.get_components", json!({"entity": "stage", "components": [T]})).unwrap();
    assert_eq!(got["components"][T]["translation"][0], json!(7.0));
    assert!(call(&mut w, "world.get_components", json!({"entity": "nowhere", "components": [T]})).unwrap_err().contains("no entity"));
    watch(&mut w, &json!({"key": "k", "method": "world.get_components", "params": {"entity": "stage", "components": [T]}})).unwrap();
    assert_eq!(changed(&mut w).len(), 1);
    assert!(changed(&mut w).is_empty());
    w.get_mut::<Transform>(e).unwrap().translation.x = 4.0;
    assert_eq!(changed(&mut w)[0]["value"]["components"][T]["translation"][0], json!(4.0));
    let t = send("world.mutate_components", json!({"entity": "stage", "component": T, "path": "translation.x", "value": 9.0}));
    assert!(answer(t).is_none());
    drain(&mut w);
    assert_eq!(answer(t), Some(json!({"value": null})));
    assert_eq!(w.get::<Transform>(e).unwrap().translation.x, 9.0);
}
