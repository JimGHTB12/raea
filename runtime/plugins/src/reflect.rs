//! Reflection's side: loads awaited, values inserted as components or resources, and the schema of what a plugin
//! registers (fields, shapes, roles), for the generated TypeScript.

use super::*;
use bevy::reflect::structs::Struct;

/// Before the first frame: every asset a handle field loaded by path, loaded with its dependencies (a material's
/// textures), or the load's error.
pub fn await_loads(app: &mut App) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    return { let _ = app; Ok(()) };
    #[cfg(not(target_arch = "wasm32"))]
    await_loads_here(app)
}

#[cfg(not(target_arch = "wasm32"))]
fn await_loads_here(app: &mut App) -> Result<(), String> {
    let t0 = bevy::platform::time::Instant::now();
    loop {
        let world = app.world_mut();
        let Some(l) = world.get_resource::<Loading>() else { return Ok(()) };
        let server = world.resource::<AssetServer>();
        let mut waiting = 0;
        for (path, h) in &l.0 {
            match server.get_recursive_dependency_load_state(h.id()) {
                Some(RecursiveDependencyLoadState::Loaded) => {}
                Some(RecursiveDependencyLoadState::Failed(e)) => return Err(format!("{path}: {e}")),
                _ => waiting += 1,
            }
        }
        if waiting == 0 {
            return Ok(());
        }
        if t0.elapsed().as_secs_f64() > 60.0 {
            return Err(format!("{waiting} asset(s) a component's handle names didn't load in 60 s"));
        }
        app.update();
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// A reflected value from JSON in the shape Bevy's reflect serde reads (a struct an object, a newtype its inner value,
/// an enum "Unit" or {"Variant": ...}, glam vectors [x, y, z]; a Handle or Entity field as Fields reads it), checked to
/// be buildable before ReflectComponent::insert (which panics on a partial value with no Default).
pub fn value_of(world: &mut World, reg: &TypeRegistry, r: &TypeRegistration, path: &str, json: &Value) -> Result<Box<dyn PartialReflect>, String> {
    let refs = world.remove_resource::<Refs>().unwrap_or_default();
    let v = {
        let mut p = Fields { world: &mut *world, refs: &refs };
        TypedReflectDeserializer::with_processor(r, reg, &mut p).deserialize(json.clone()).map_err(|e| format!("{path}: {e}"))
    };
    world.insert_resource(refs);
    let v = v?;
    let whole = r.data::<ReflectFromReflect>().and_then(|f| f.from_reflect(v.as_partial_reflect())).is_some();
    if !whole && r.data::<ReflectDefault>().is_none() && r.data::<ReflectFromWorld>().is_none() {
        return Err(format!("{path} has no default, so give every field"));
    }
    Ok(v)
}

pub fn registration<'a>(reg: &'a TypeRegistry, path: &str) -> Result<&'a TypeRegistration, String> {
    reg.get_with_type_path(path).or_else(|| reg.get_with_short_type_path(path)).ok_or_else(|| format!("no reflected type {path} (is its plugin compiled in?)"))
}

/// One component from JSON onto an entity, through the TypeRegistry: the generic path every plugin component takes.
pub fn insert_reflect(world: &mut World, e: Entity, path: &str, json: &Value) -> Result<(), String> {
    if let Some(material) = path.strip_prefix("material:") {
        return patch_material(world, e, material, json);
    }
    // a three shader's uniforms this frame (raea-sys-three-shader, docs/r3f.feature R3F11)
    if path == "three:uniforms" {
        return raea_sys_three_shader::patch(world, e, json);
    }
    // a <Material>'s uniforms and storage buffers this frame (raea-sys-shader own_live.rs, docs/control.feature FC2a)
    if path == "own:uniforms" {
        return raea_sys_shader::own_live::patch(world, e, json);
    }
    let registry = world.resource::<AppTypeRegistry>().clone();
    let reg = registry.read();
    let r = registration(&reg, path)?;
    let rc = r.data::<ReflectComponent>().ok_or_else(|| format!("{path} isn't a component"))?;
    let v = value_of(world, &reg, r, path, json)?;
    // a component the entity already has (the engine's Camera on a <PerspectiveCamera>) takes the fields given over
    // its own; a new one is the type's default with them
    let mut em = world.entity_mut(e);
    if rc.contains(&em) {
        rc.apply(&mut em, v.as_partial_reflect());
    } else {
        rc.insert(&mut em, v.as_partial_reflect(), &reg);
    }
    Ok(())
}

/// One resource from JSON, the same way.
pub fn insert_reflect_resource(world: &mut World, path: &str, json: &Value) -> Result<(), String> {
    let registry = world.resource::<AppTypeRegistry>().clone();
    let reg = registry.read();
    let r = registration(&reg, path)?;
    if r.data::<ReflectResource>().is_none() {
        return Err(format!("{path} isn't a resource"));
    }
    let rc = r.data::<ReflectComponent>().ok_or_else(|| format!("{path} isn't reflected as a resource"))?;
    let v = value_of(world, &reg, r, path, json)?;
    let id = rc.register_component(world);
    let e = match world.resource_entities().get(id) {
        Some(e) => e,
        None => world.spawn_empty().id(),
    };
    rc.insert(&mut world.entity_mut(e), v.as_partial_reflect(), &reg);
    Ok(())
}

// ---- the registry as a schema (what bevy-plugins' generator turns into TypeScript)

pub fn field_list<'a>(it: impl Iterator<Item = (Option<&'static str>, &'static str)> + 'a) -> Vec<Value> {
    it.map(|(n, t)| match n {
        Some(n) => json!({"name": n, "type": t}),
        None => json!({"type": t}),
    }).collect()
}

pub fn shape_of(ti: &TypeInfo) -> Value {
    match ti {
        TypeInfo::Struct(s) => json!({"kind": "struct", "fields": field_list(s.iter().map(|f| (Some(f.name()), f.type_path())))}),
        TypeInfo::TupleStruct(s) => json!({"kind": "tuple_struct", "fields": field_list(s.iter().map(|f| (None, f.type_path())))}),
        TypeInfo::Tuple(s) => json!({"kind": "tuple", "fields": field_list(s.iter().map(|f| (None, f.type_path())))}),
        TypeInfo::List(l) => json!({"kind": "list", "item": l.item_ty().path()}),
        TypeInfo::Array(a) => json!({"kind": "array", "item": a.item_ty().path(), "len": a.capacity()}),
        TypeInfo::Map(m) => json!({"kind": "map", "key": m.key_ty().path(), "value": m.value_ty().path()}),
        TypeInfo::Set(s) => json!({"kind": "set", "item": s.value_ty().path()}),
        TypeInfo::Enum(e) => json!({"kind": "enum", "variants": e.iter().map(|v| match v {
            VariantInfo::Unit(u) => json!({"name": u.name(), "kind": "unit"}),
            VariantInfo::Tuple(t) => json!({"name": t.name(), "kind": "tuple", "fields": field_list(t.iter().map(|f| (None, f.type_path())))}),
            VariantInfo::Struct(s) => json!({"name": s.name(), "kind": "struct", "fields": field_list(s.iter().map(|f| (Some(f.name()), f.type_path())))}),
        }).collect::<Vec<_>>()}),
        TypeInfo::Opaque(_) => json!({"kind": "opaque"}),
    }
}

pub fn referenced(shape: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for k in ["item", "key", "value"] {
        if let Some(t) = shape.get(k).and_then(Value::as_str) {
            out.push(t.to_string());
        }
    }
    let mut fields = |fs: Option<&Value>| {
        for f in fs.and_then(Value::as_array).into_iter().flatten() {
            if let Some(t) = f.get("type").and_then(Value::as_str) {
                out.push(t.to_string());
            }
        }
    };
    fields(shape.get("fields"));
    for v in shape.get("variants").and_then(Value::as_array).into_iter().flatten() {
        fields(v.get("fields"));
    }
    out
}

/// What a registered type is to TypeScript, read from its type data: a resource, a component, an event (ReflectEvent,
/// which an observer sees when it's triggered), a message (ReflectMessage, which is buffered), or a state
/// (ReflectState; it is "state" when its next state can be set, or "computed_state" when it can't). None for plain data.
pub fn role(r: &TypeRegistration) -> Option<&'static str> {
    if r.data::<ReflectState>().is_some() {
        Some(if r.data::<ReflectFreelyMutableState>().is_some() { "state" } else { "computed_state" })
    } else if r.data::<ReflectResource>().is_some() {
        Some("resource")
    } else if r.data::<ReflectComponent>().is_some() {
        Some("component")
    } else if r.data::<ReflectEvent>().is_some() {
        Some("event")
    } else if r.data::<ReflectMessage>().is_some() {
        Some("message")
    } else {
        None
    }
}

/// The registry's components, resources, events, messages and states whose type path `keep` accepts, and every type
/// their fields reach. For each type it records the kind, the fields, the enum variants, its role (`role`), whether it
/// has its own serde form, and its default when it has one (serialized as the reflect deserializer reads it back).
pub fn schema(world: &World, keep: &dyn Fn(&str) -> bool) -> Value {
    let registry = world.resource::<AppTypeRegistry>().clone();
    let reg = registry.read();
    let mut todo: Vec<String> = reg.iter().filter(|r| keep(r.type_info().type_path()) && role(r).is_some()).map(|r| r.type_info().type_path().to_string()).collect();
    todo.sort();
    let roots = todo.clone();
    let mut types = serde_json::Map::new();
    while let Some(path) = todo.pop() {
        if types.contains_key(&path) {
            continue;
        }
        let Some(r) = reg.get_with_type_path(&path) else {
            types.insert(path.clone(), json!({"kind": "unregistered"}));
            continue;
        };
        let ti = r.type_info();
        let mut t = shape_of(ti);
        for p in referenced(&t) {
            if !types.contains_key(&p) {
                todo.push(p);
            }
        }
        let o = t.as_object_mut().unwrap();
        o.insert("short".into(), json!(ti.type_path_table().short_path()));
        o.insert("generic".into(), json!(!ti.generics().is_empty()));
        match role(r) {
            Some("resource") => {
                o.insert("resource".into(), json!(true));
            }
            Some("component") => {
                o.insert("component".into(), json!(true));
            }
            Some(other) => {
                o.insert("role".into(), json!(other));
            }
            None => {}
        }
        if r.data::<ReflectSerialize>().is_some() && r.data::<ReflectDeserialize>().is_some() {
            o.insert("serde".into(), json!(true));
        }
        if let Some(d) = r.data::<ReflectDefault>() {
            let v = d.default();
            if let Ok(j) = serde_json::to_value(TypedReflectSerializer::new(v.as_partial_reflect(), &reg)) {
                o.insert("default".into(), j);
            }
        }
        types.insert(path, t);
    }
    json!({"roots": roots, "types": types})
}

/// A part whose material a `material:` row patched has a copy of its own (its handle may be one other parts share,
/// runtime/scene3d share.rs).
#[derive(Component)]
pub struct OwnMaterial;

/// The fields given (a partial StandardMaterial in reflect's JSON) set on the part's own material asset: a
/// StandardMaterial's `animate` of a field the row columns don't carry (docs/control.feature FC1c), keyed like a
/// component. The first patch makes the part's copy of its material.
pub fn patch_material(world: &mut World, e: Entity, path: &str, json: &Value) -> Result<(), String> {
    if registration(&world.resource::<AppTypeRegistry>().read(), path)?.type_id() != std::any::TypeId::of::<StandardMaterial>() {
        return Err(format!("material:{path}: only a StandardMaterial's fields animate this way"));
    }
    let registry = world.resource::<AppTypeRegistry>().clone();
    let reg = registry.read();
    // each field given, by its own type: the whole StandardMaterial deserialized from a partial one took every field
    // it wasn't given at its default, so a patch of emissive_exposure_weight blacked out the emissive (#38: a render's
    // frames after the first, which the scene sync leaves alone, drew the glowing cube grey)
    let Value::Object(given) = json else { return Err(format!("material:{path}: give its fields as an object")) };
    let TypeInfo::Struct(info) = registration(&reg, path)?.type_info() else { return Err(format!("material:{path}: not a struct")) };
    let mut fields = Vec::with_capacity(given.len());
    for (name, x) in given {
        let ty = info.field(name).ok_or_else(|| format!("material:{path}: no field {name}"))?.type_path();
        fields.push((name.as_str(), value_of(world, &reg, registration(&reg, ty)?, &format!("{path}.{name}"), x)?));
    }
    let Some(h) = world.get::<MeshMaterial3d<StandardMaterial>>(e).map(|m| m.0.clone()) else {
        return Err(format!("material:{path}: the part has no StandardMaterial"));
    };
    let h = if world.get::<OwnMaterial>(e).is_some() {
        h
    } else {
        let mut mats = world.resource_mut::<Assets<StandardMaterial>>();
        let own = mats.get(&h).cloned().ok_or_else(|| format!("material:{path}: its material isn't loaded"))?;
        let nh = mats.add(own);
        world.entity_mut(e).insert((MeshMaterial3d(nh.clone()), OwnMaterial));
        nh
    };
    let mut mats = world.resource_mut::<Assets<StandardMaterial>>();
    let Some(mut m) = mats.get_mut(&h) else { return Err(format!("material:{path}: its material isn't loaded")) };
    for (name, v) in fields {
        let f = Struct::field_mut(&mut *m, name).ok_or_else(|| format!("material:{path}: no field {name}"))?;
        f.try_apply(v.as_partial_reflect()).map_err(|err| format!("material:{path}.{name}: {err}"))?;
    }
    Ok(())
}
