//! A frame's 3D read from the React tree: each part's props and names, cached across frames.

use super::*;

/// A part's props from its data-props text: parsed again only when the text changed since the last scene
pub(crate) fn props3d(e: Entity, text: Option<&str>) -> std::sync::Arc<Map<String, Value>> {
    let Some(text) = text else { return std::sync::Arc::default() };
    let now = SCENE3D.with(|g| g.get());
    PROPS3D.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((t, p, seen)) = c.get_mut(&e) {
            *seen = now;
            if t.as_str() == text {
                return p.clone();
            }
            // in place where no one else holds them (the last scene let go) and the keys are the same: each value
            // updated where it is (ZeroHeap: a moving part's props were parsed into a new map each frame)
            if let Some(m) = std::sync::Arc::get_mut(p)
                && props_in_place(m, text)
            {
                t.clear();
                t.push_str(text);
                return p.clone();
            }
            let parsed: Map<String, Value> = serde_json::from_str(text).unwrap_or_default();
            t.clear();
            t.push_str(text);
            *p = std::sync::Arc::new(parsed);
            return p.clone();
        }
        let p = std::sync::Arc::new(serde_json::from_str::<Map<String, Value>>(text).unwrap_or_default());
        c.insert(e, (text.to_string(), p.clone(), now));
        p
    })
}

/// A part's kind and shape, the shared ones from its last read when they're the same
pub(crate) fn names3d(e: Entity, kind: &str, shape: Option<&str>) -> (std::sync::Arc<str>, Option<std::sync::Arc<str>>) {
    NAMES3D.with_borrow_mut(|c| {
        if let Some((k, s)) = c.get(&e)
            && &**k == kind
            && s.as_deref() == shape
        {
            return (k.clone(), s.clone());
        }
        let v: (std::sync::Arc<str>, Option<std::sync::Arc<str>>) = (kind.into(), shape.map(Into::into));
        if c.len() > 8192 {
            c.clear();
        }
        c.insert(e, v.clone());
        v
    })
}

/// `m` set to the object `text` holds, value by value in place: false (`m` then partly updated) when it isn't an
/// object with the same keys
pub(crate) fn props_in_place(m: &mut Map<String, Value>, text: &str) -> bool {
    let Ok(raw) = serde_json::from_str::<&serde_json::value::RawValue>(text) else { return false };
    if !raea_core_wire::raw::is_object(raw) {
        return false;
    }
    let (mut n, mut ok) = (0usize, true);
    raea_core_wire::raw::each_entry(raw, |k, v| {
        n += 1;
        if m.contains_key(k) {
            raea_core_wire::raw::put_raw(m, k, v);
        } else {
            ok = false;
        }
    });
    ok && n == m.len()
}

pub(crate) fn read3d(world: &World, e: Entity) -> Option<Node3> {
    let p = world.get::<ReactProps>(e)?;
    let (kind, shape) = names3d(e, p.0.get("data-3d")?.as_str()?, p.0.get("data-shape").and_then(|v| v.as_str()));
    let props = props3d(e, p.0.get("data-props").and_then(|v| v.as_str()));
    // a world's entities (<Entities>): its scene, already in world space, as meshes with their materials
    if &*kind == "Entities" {
        let kids = props.get("scene").and_then(Value::as_array).into_iter().flatten().filter(|n| n["kind"] == "Mesh").map(|n| {
            let mesh = n["mesh"].as_object().cloned().unwrap_or_default();
            let mut p = Map::new();
            p.insert("position".into(), n["position"].clone());
            p.insert("rotation".into(), n["rotation"].clone());
            p.insert("scale".into(), n["scale"].clone());
            let mut mat = Map::new();
            for k in ["color", "roughness", "metalness"] {
                if let Some(x) = mesh.get(k) {
                    mat.insert(k.into(), x.clone());
                }
            }
            Node3 { kind: "Mesh".into(), props: std::sync::Arc::new(p), shape: mesh.get("shape").and_then(Value::as_str).map(Into::into), kids: vec![Node3 { kind: "MeshStandardMaterial".into(), props: std::sync::Arc::new(mat), shape: None, kids: Vec::new() }] }
        }).collect();
        return Some(Node3 { kind, props: std::sync::Arc::default(), shape: None, kids });
    }
    let mut kids = Vec::new();
    kids3d(world, e, &mut kids);
    Some(Node3 { kind, props, shape, kids })
}

/// The 3D children of `e`, looking through plain elements between them (a Sequence's or a component's wrapper div
/// inside a Canvas3D), so 3D timed by a Sequence is still the scene's.
pub(crate) fn kids3d(world: &World, e: Entity, out: &mut Vec<Node3>) {
    let Some(children) = world.get::<Children>(e) else { return };
    for k in children.iter() {
        match read3d(world, k) {
            Some(n) => out.push(n),
            None => kids3d(world, k, out),
        }
    }
}
