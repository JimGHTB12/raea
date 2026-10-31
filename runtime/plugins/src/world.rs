//! The World's methods for a plugin's rows and the shell: entities spawned with components, components inserted, the
//! World saved and loaded, diagnostics.

use super::*;

/// The World methods raea-render gives every host (engine/bevy-world's Methods table, asked before BRP's own):
///   world.spawn_entity, world.insert_components  BRP's, with each component read by the Fields processor (a Handle
///                       field as a path, an inline asset, a shape, buffers or a primitive; an Entity field as a name)
///   raea/world.save     {entities?: [bits]}: Bevy's DynamicWorld of those entities (default: every named one) and
///                       their reflected components, as JSON (DynamicWorldSerializer)
///   raea/world.load     {world}: that JSON written into the World (WorldDeserializer, entity references remapped);
///                       answers {entities: {saved bits: new bits}}
///   raea/diagnostics    every diagnostic in Bevy's DiagnosticsStore (FPS, frame time, entity count...) with its
///                       latest value, smoothed value and average
pub fn world_methods(app: &mut App) {
    use bevy::diagnostic::{DiagnosticsPlugin, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
    if !app.is_plugin_added::<DiagnosticsPlugin>() {
        app.add_plugins(DiagnosticsPlugin);
    }
    if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    }
    if !app.is_plugin_added::<EntityCountDiagnosticsPlugin>() {
        app.add_plugins(EntityCountDiagnosticsPlugin::default());
    }
    let mut m = app.world_mut().get_resource_or_init::<raea_bevy_world::Methods>();
    m.0.insert("world.spawn_entity".into(), spawn_entity);
    m.0.insert("world.insert_components".into(), insert_components);
    m.0.insert("raea/world.save".into(), world_save);
    m.0.insert("raea/world.load".into(), world_load);
    m.0.insert("raea/diagnostics".into(), diagnostics);
}

pub fn components_of(p: &Value) -> Result<serde_json::Map<String, Value>, String> {
    match p.get("components") {
        Some(Value::Object(o)) => Ok(o.clone()),
        None | Some(Value::Null) => Ok(Default::default()),
        Some(x) => Err(format!("components is an object of type path to value, got {x}")),
    }
}

pub fn spawn_entity(world: &mut World, p: Value) -> Result<Value, String> {
    let comps = components_of(&p)?;
    let e = world.spawn_empty().id();
    for (path, v) in &comps {
        if let Err(err) = insert_reflect(world, e, path, v) {
            world.despawn(e);
            return Err(err);
        }
    }
    Ok(json!({ "entity": e.to_bits() }))
}

pub fn insert_components(world: &mut World, p: Value) -> Result<Value, String> {
    let e = p.get("entity").and_then(Value::as_u64).and_then(Entity::try_from_bits).ok_or("world.insert_components needs entity")?;
    if world.get_entity(e).is_err() {
        return Err(format!("no entity {}", e.to_bits()));
    }
    for (path, v) in &components_of(&p)? {
        insert_reflect(world, e, path, v)?;
    }
    Ok(Value::Null)
}

pub fn world_save(world: &mut World, p: Value) -> Result<Value, String> {
    use bevy::world_serialization::{DynamicWorldBuilder, serde::DynamicWorldSerializer};
    let ents: Vec<Entity> = match p.get("entities").and_then(Value::as_array) {
        Some(a) => a.iter().map(|x| x.as_u64().and_then(Entity::try_from_bits).ok_or_else(|| format!("raea/world.save: an entity is its bits, got {x}"))).collect::<Result<_, _>>()?,
        None => {
            let mut q = world.query_filtered::<Entity, With<Name>>();
            q.iter(world).collect()
        }
    };
    let registry = world.resource::<AppTypeRegistry>().clone();
    let reg = registry.read();
    let dw = DynamicWorldBuilder::from_world(world, &reg).extract_entities(ents.into_iter()).build();
    serde_json::to_value(DynamicWorldSerializer::new(&dw, &reg)).map_err(|e| format!("raea/world.save: {e}"))
}

pub fn world_load(world: &mut World, p: Value) -> Result<Value, String> {
    use bevy::world_serialization::serde::WorldDeserializer;
    let saved = p.get("world").ok_or("raea/world.load needs world (what raea/world.save answered)")?;
    let registry = world.resource::<AppTypeRegistry>().clone();
    let dw = {
        let reg = registry.read();
        let mut server = world.get_resource::<AssetServer>().cloned().ok_or("raea/world.load: this World has no AssetServer to load handles with")?;
        WorldDeserializer { type_registry: &reg, load_from_path: &mut server }.deserialize(saved.clone()).map_err(|e| format!("raea/world.load: {e}"))?
    };
    let mut map = bevy::ecs::entity::EntityHashMap::default();
    dw.write_to_world(world, &mut map).map_err(|e| format!("raea/world.load: {e}"))?;
    let out: serde_json::Map<String, Value> = map.iter().map(|(a, b)| (a.to_bits().to_string(), json!(b.to_bits()))).collect();
    Ok(json!({ "entities": out }))
}

pub fn diagnostics(world: &mut World, _p: Value) -> Result<Value, String> {
    let store = world.get_resource::<bevy::diagnostic::DiagnosticsStore>().ok_or("raea/diagnostics: no DiagnosticsStore (Bevy's DiagnosticsPlugin)")?;
    let out: serde_json::Map<String, Value> = store.iter().map(|d| (d.path().as_str().to_string(), json!({
        "value": d.value(),
        "smoothed": d.smoothed(),
        "average": d.average(),
        "suffix": d.suffix.as_ref(),
        "history": d.history_len(),
    }))).collect();
    Ok(Value::Object(out))
}
