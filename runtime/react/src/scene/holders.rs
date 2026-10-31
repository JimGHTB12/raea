//! The page elements that hold a frame's 3D, and the 3D read from React's tree.

use super::*;

/// The frame's 3D (each data-3d root in the page's order, its 3D children under it): what a render turns into rows.
pub fn scene_of(world: &mut World) -> Vec<Node3> {
    let now = SCENE3D.with(|g| {
        g.set(g.get() + 1);
        g.get()
    });
    if now % 120 == 0 {
        PROPS3D.with(|c| c.borrow_mut().retain(|_, v| v.2 + 2 >= now));
    }
    let mut q = world.query::<(Entity, &ReactKind, Option<&ReactProps>)>();
    let Some(surf) = q.iter(world).find(|(_, k, p)| k.0 == "surface" && p.and_then(|p| p.0.get("id")).and_then(|v| v.as_str()) == Some("motion")).map(|(e, _, _)| e) else { return Vec::new() };
    fn walk(world: &World, e: Entity, out: &mut Vec<Node3>) {
        if world.get::<ReactProps>(e).is_some_and(|p| p.0.contains_key("data-3d")) {
            if let Some(n) = read3d(world, e) {
                out.push(n);
            }
            return;
        }
        if let Some(c) = world.get::<Children>(e) {
            for k in c.iter() {
                walk(world, k, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(world, surf, &mut out);
    out
}

/// The page elements holding the frame's 3D (each 3D root's ancestors up to the surface, nearest first) with their
/// props: a render clears their backgrounds (the 3D is drawn under the page) and the nearest one's colour is the 3D's.
/// The elements holding the 3D (holders_of's), into `out` (ZeroHeap: live play's present asks each frame)
pub fn holder_entities(world: &mut World, out: &mut Vec<Entity>) {
    out.clear();
    with_holder_query(world, |world, q| {
        let is3d = |w: &World, e: Entity| w.get::<ReactProps>(e).is_some_and(|p| p.0.contains_key("data-3d"));
        for (r, p, c) in q.0.iter(world) {
            if !p.0.contains_key("data-3d") || c.is_some_and(|c| is3d(world, c.parent())) {
                continue;
            }
            let mut at = world.get::<ChildOf>(r).map(|c| c.parent());
            while let Some(e) = at {
                if world.get::<ReactKind>(e).is_some_and(|k| k.0 == "surface") {
                    break;
                }
                if !out.contains(&e) {
                    out.push(e);
                }
                at = world.get::<ChildOf>(e).map(|c| c.parent());
            }
        }
    })
}

#[derive(Resource)]
pub struct HolderQuery(QueryState<(Entity, &'static ReactProps, Option<&'static ChildOf>)>);

pub fn with_holder_query<T>(world: &mut World, f: impl FnOnce(&mut World, &mut HolderQuery) -> T) -> T {
    if !world.contains_resource::<HolderQuery>() {
        let q = world.query();
        world.insert_resource(HolderQuery(q));
    }
    world.resource_scope(|world, mut q: Mut<HolderQuery>| f(world, &mut q))
}

pub fn holders_of(world: &mut World) -> Vec<(Entity, Map<String, Value>)> {
    let mut q = world.query::<(Entity, &ReactProps, Option<&ChildOf>)>();
    let is3d = |w: &World, e: Entity| w.get::<ReactProps>(e).is_some_and(|p| p.0.contains_key("data-3d"));
    let roots: Vec<Entity> = q.iter(world).filter(|(_, p, c)| p.0.contains_key("data-3d") && !c.is_some_and(|c| is3d(world, c.parent()))).map(|(e, _, _)| e).collect();
    let mut out: Vec<(Entity, Map<String, Value>)> = Vec::new();
    for r in roots {
        let mut at = world.get::<ChildOf>(r).map(|c| c.parent());
        while let Some(e) = at {
            if world.get::<ReactKind>(e).is_some_and(|k| k.0 == "surface") {
                break;
            }
            if !out.iter().any(|(x, _)| *x == e) {
                out.push((e, world.get::<ReactProps>(e).map(|p| p.0.clone()).unwrap_or_default()));
            }
            at = world.get::<ChildOf>(e).map(|c| c.parent());
        }
    }
    out
}

/// A holder of the 3D made see-through, as every path that draws the 3D under the page does: its inline background
/// goes (the nearest one's colour is the 3D's clear colour, returned here), and when it had one or has a class, an
/// inline transparent background and no image override whatever a class gives it. Before, only an inline background
/// went, so `className="bg-brand-bg"` (the brand's own utility) painted an opaque page over the 3D and every frame
/// rendered as the background colour alone (2026-10-18, void's agent's perfume shot, black for 24 renders). Whether
/// the props changed, and the clear colour.
pub fn see_through(p: &mut Map<String, Value>) -> (bool, Option<String>) {
    let classed = p.get("className").and_then(Value::as_str).is_some_and(|c| !c.trim().is_empty());
    let style = p.entry("style").or_insert_with(|| Value::Object(Map::new()));
    let Value::Object(st) = style else { return (false, None) };
    let mut clear = None;
    let mut had = false;
    for k in ["background", "backgroundColor"] {
        if let Some(v) = st.remove(k) {
            had = true;
            if clear.is_none() {
                clear = v.as_str().map(str::to_string);
            }
        }
    }
    if had || classed {
        st.insert("backgroundColor".into(), Value::String("transparent".into()));
        st.insert("backgroundImage".into(), Value::String("none".into()));
    }
    (had || classed, clear)
}

/// A layered Canvas3D's slot: the `<img data-raea-scene>` it renders, whose place in the page's paint order the 3D's
/// frame takes (as a texture), so page elements before it in the tree are under the 3D and those after it over it.
pub fn scene_slot(world: &mut World) -> Option<u64> {
    let mut q = world.query::<(&ReactProps, Option<&crate::ReactNode>)>();
    q.iter(world).find(|(p, _)| p.0.contains_key("data-raea-scene")).and_then(|(_, n)| n.map(|n| n.0))
}

/// The 3D's clear colour for a render or play: transparent (#00000000) for a layered Canvas3D, whose background is the
/// page under it; else the nearest holder's background.
pub fn scene_clear(world: &mut World) -> Option<String> {
    if scene_slot(world).is_some() {
        return Some("#00000000".into());
    }
    clear_of(&holders_of(world))
}

/// The 3D's clear colour: the nearest holder's background, as a render takes it.
pub fn clear_of(holders: &[(Entity, Map<String, Value>)]) -> Option<String> {
    holders.iter().find_map(|(_, p)| p.get("style").and_then(|s| s.as_object()).and_then(|st| st.get("background").or(st.get("backgroundColor"))).and_then(|v| v.as_str()).map(str::to_string))
}
