use super::*;

fn app() -> App {
    let mut app = App::new();
    app.register_type::<Transform>().register_type::<Visibility>().register_type::<bevy::light::DirectionalLightShadowMap>();
    app
}

#[test]
fn a_component_from_json_through_the_registry() {
    let mut app = app();
    let w = app.world_mut();
    let e = w.spawn_empty().id();
    // a struct with a default: the fields given, the rest default
    insert_reflect(w, e, "bevy_transform::components::transform::Transform", &json!({"translation": [1.0, 2.0, 3.0]})).unwrap();
    let t = w.get::<Transform>(e).unwrap();
    assert_eq!(t.translation, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(t.scale, Vec3::ONE);
    // a short path names it too; an enum is its variant's name
    insert_reflect(w, e, "Visibility", &json!("Hidden")).unwrap();
    assert_eq!(*w.get::<Visibility>(e).unwrap(), Visibility::Hidden);
    assert!(insert_reflect(w, e, "no::Such", &json!({})).unwrap_err().contains("no reflected type"));
    assert!(insert_reflect(w, e, "Transform", &json!({"translation": "up"})).is_err());
}

#[test]
fn a_resource_from_json_and_the_schema() {
    let mut app = app();
    let w = app.world_mut();
    insert_reflect_resource(w, "bevy_light::directional_light::DirectionalLightShadowMap", &json!({"size": 512})).unwrap();
    assert_eq!(w.resource::<bevy::light::DirectionalLightShadowMap>().size, 512);
    let s = schema(w, &|p| p.starts_with("bevy_transform::components::transform::Transform"));
    let t = &s["types"]["bevy_transform::components::transform::Transform"];
    assert_eq!(t["kind"], "struct");
    assert_eq!(t["component"], true);
    assert_eq!(t["fields"][0]["name"], "translation");
    assert_eq!(t["default"]["scale"], json!([1.0, 1.0, 1.0]));
    // the types its fields reach come along
    assert!(s["types"].get("glam::Vec3").is_some());
}

/// raea-render's host methods: world.spawn_entity reads an Entity field by name (a ChildOf naming its parent), a saved
/// world loads back with its references remapped to the new entities, and the diagnostics list entity_count
#[test]
fn world_save_and_load() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
    app.register_type::<Transform>().register_type::<Name>().register_type::<ChildOf>().register_type::<Children>();
    world_methods(&mut app);
    app.update();
    app.update();
    let w = app.world_mut();
    let dad = raea_bevy_world::call(w, "world.spawn_entity", json!({"components": {"bevy_ecs::name::Name": "dad"}})).unwrap()["entity"].as_u64().unwrap();
    let kid = raea_bevy_world::call(w, "world.spawn_entity", json!({"components": {"bevy_ecs::name::Name": "kid", "bevy_ecs::hierarchy::ChildOf": "dad"}})).unwrap()["entity"].as_u64().unwrap();
    let (dad, kid) = (Entity::from_bits(dad), Entity::from_bits(kid));
    assert_eq!(w.get::<ChildOf>(kid).map(|c| c.parent()), Some(dad));
    let saved = raea_bevy_world::call(w, "raea/world.save", json!({})).unwrap();
    assert_eq!(saved["entities"].as_object().map(|o| o.len()), Some(2), "{saved}");
    let loaded = raea_bevy_world::call(w, "raea/world.load", json!({"world": saved})).unwrap();
    let map = loaded["entities"].as_object().unwrap();
    let new = |e: Entity| Entity::from_bits(map[&e.to_bits().to_string()].as_u64().unwrap());
    let (dad2, kid2) = (new(dad), new(kid));
    assert_ne!(dad2, dad);
    assert_eq!(w.get::<Name>(kid2).map(|n| n.as_str()), Some("kid"));
    assert_eq!(w.get::<ChildOf>(kid2).map(|c| c.parent()), Some(dad2), "the loaded kid's parent is the loaded dad");
    let d = raea_bevy_world::call(w, "raea/diagnostics", json!({})).unwrap();
    assert!(d.get("entity_count").is_some() && d.get("fps").is_some(), "{d}");
}
