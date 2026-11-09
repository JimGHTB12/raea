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

#[test]
fn keyed_components_land_on_their_frame_forwards_backwards_and_jumping() {
    let mut app = app();
    let w = app.world_mut();
    let (a, b) = (w.spawn_empty().id(), w.spawn_empty().id());
    let ty = "bevy_transform::components::transform::Transform".to_string();
    let at = |x: f64| json!({"translation": [x, 0.0, 0.0]});
    // a keyed every frame 0-8; b at 0 and 5 only: in frame order, as apply_rows leaves them
    let mut rows: Vec<_> = (0..=8).map(|f| (Some(a), ty.clone(), at(f as f64), f)).collect();
    rows.extend([(Some(b), ty.clone(), at(100.0), 0), (Some(b), ty.clone(), at(105.0), 5)]);
    rows.sort_by_key(|r| r.3);
    w.insert_resource(Keyed { rows, ..default() });
    w.insert_resource(Error(None));
    let x = |w: &World, e: Entity| w.get::<Transform>(e).map(|t| t.translation.x);
    for (f, want_a, want_b) in [(0, 0.0, 100.0), (1, 1.0, 100.0), (2, 2.0, 100.0), (6, 6.0, 105.0), (3, 3.0, 100.0), (8, 8.0, 105.0)] {
        w.insert_resource(Clock(f));
        keyed_rows(w);
        assert_eq!((x(w, a), x(w, b)), (Some(want_a), Some(want_b)), "frame {f}");
    }
}

/// A live engine's rows (a fed render to MP4) still take a row held for its frame: the frames' rows only carry values
/// that change, and a `from` row never does, so a rigid body from frame 36 stayed asleep
#[test]
fn a_live_engine_takes_a_row_held_for_its_frame() {
    let mut app = app();
    let w = app.world_mut();
    let a = w.spawn_empty().id();
    let ty = "bevy_transform::components::transform::Transform".to_string();
    w.insert_resource(Keyed { rows: vec![(Some(a), ty, json!({"translation": [7.0, 0.0, 0.0]}), 3)], ..default() });
    w.insert_resource(Error(None));
    let mut fed = FrameRows::default();
    fed.live = Some(vec![]);
    w.insert_resource(fed);
    let x = |w: &World| w.get::<Transform>(a).map(|t| t.translation.x);
    w.insert_resource(Clock(2));
    keyed_rows(w);
    assert_eq!(x(w), None, "not before its frame");
    w.insert_resource(Clock(3));
    keyed_rows(w);
    assert_eq!(x(w), Some(7.0), "on its frame, in a live engine too");
}

/// A `material:` row sets only the fields it gives on the part's own copy of its material (#38): it took the whole
/// StandardMaterial from them, every other field at its default, so a patch of emissive_exposure_weight blacked out the
/// emissive and whitened the base, and a render's frames after the first drew the glowing cube grey
#[test]
fn a_material_row_sets_only_its_fields() {
    let mut app = app();
    app.register_type::<StandardMaterial>();
    let w = app.world_mut();
    let mut mats = Assets::<StandardMaterial>::default();
    let (black, red) = (Color::BLACK, LinearRgba::rgb(1.0, 0.0, 0.0));
    let shared = mats.add(StandardMaterial { base_color: black, emissive: red, ..default() });
    w.insert_resource(mats);
    let e = w.spawn(MeshMaterial3d(shared.clone())).id();
    insert_reflect(w, e, "material:bevy_pbr::pbr_material::StandardMaterial", &json!({"emissive_exposure_weight": 0.5})).unwrap();
    let own = w.get::<MeshMaterial3d<StandardMaterial>>(e).unwrap().0.clone();
    assert_ne!(own, shared, "the part's own copy, not the shared material");
    let m = w.resource::<Assets<StandardMaterial>>().get(&own).unwrap();
    assert_eq!((m.emissive_exposure_weight, m.emissive, m.base_color), (0.5, red, black));
    let err = insert_reflect(w, e, "material:bevy_pbr::pbr_material::StandardMaterial", &json!({"no_such": 1.0})).unwrap_err();
    assert!(err.contains("no field no_such"), "{err}");
}

#[test]
fn only_rows_a_plugin_carries_from_frame_to_frame_need_the_frames_before_a_still_stepped() {
    let row = |kind: &str, vals: &[(&str, V)]| Tup { kind: kind.into(), id: "r".into(), vals: vals.iter().map(|(k, v)| (k.to_string(), v.clone())).collect() };
    let ty = |t: &str| row("component", &[("type", V::S(t.into()))]);
    // a film's rows: parts, lights, a camera and Bevy's own components (#77)
    let film = vec![row("camera", &[]), row("light", &[]), row("mesh", &[]), row("model", &[]), row("panel", &[]), row("entity", &[]), ty("bevy_light::NotShadowCaster"), ty("bevy_post_process::bloom::settings::Bloom"), ty("material:bevy_pbr::pbr_material::StandardMaterial"), ty("three:uniforms")];
    assert!(!steps(&film));
    for more in [ty("avian3d::dynamics::rigid_body::RigidBody"), ty("bevy_hanabi::ParticleEffect"), ty("bevy_animation::AnimationPlayer"), ty("RigidBody"), row("mesh", &[("driven", V::N(1.0))]), row("compute", &[])] {
        let mut rows = film.clone();
        rows.push(more.clone());
        assert!(steps(&rows), "{more:?} needs its frames stepped");
    }
}
