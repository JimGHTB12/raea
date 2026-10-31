//! React's `on*` props on the entities React makes (a mesh, a sprite, a UI node; docs/sdk.feature S2c, B3 H3): Bevy
//! picking's per-kind pointer events (bevy_picking 0.20's PointerClick, PointerPress, ...) go to the element's handler
//! as React spells it, `onClick` for PointerClick, `onPointerDown` for PointerPress, `onWheel` for PointerScroll.
//! One global observer per kind sends `{"t": "uiEvent"}` for each entity up the event's path that has the prop, so a
//! handler on a parent sees a child's event as React's bubbling has it. A page element's events are Blitz's own
//! (its twin has `raea_runtime_twins::bridge::dom::*`, not ReactNode), so nothing is sent twice.
//!
//! Not yet (docs/sdk.feature S3a, M6, one tree): a Canvas3D mesh is a row of the Engine's scene program, not the
//! React node's own entity, so a hit on it has no ReactNode to route to; nor is MeshPickingPlugin added (it ray-casts
//! every mesh on each pointer move). Both come with the one tree, when React's elements are the entities drawn.
//!
//! The handler gets `{id, kind, x, y}` (the pointer in its target's px), with `button` (0 primary, 1 middle, 2
//! secondary, as the DOM numbers them), `point` (the hit in world space, when the backend has one) and, for onWheel,
//! `deltaX` / `deltaY`.

use bevy::picking::events::*;
use bevy::picking::backend::HitData;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use serde_json::{json, Map, Value};

use crate::{ReactNode, ReactOutbox, ReactProps};

/// An observer per kind.
pub(crate) fn setup(app: &mut App) {
    macro_rules! route {
        ($($ty:ident => $prop:literal, $extra:expr;)*) => {$(
            app.add_observer(|ev: On<$ty>, q: Query<(&ReactNode, &ReactProps)>, out: Option<Res<ReactOutbox>>| {
                let mut m = Map::new();
                m.insert("x".into(), json!(ev.pointer.position.x));
                m.insert("y".into(), json!(ev.pointer.position.y));
                ($extra)(&*ev, &mut m);
                send(&q, out.as_deref(), ev.entity, $prop, m);
            });
        )*};
    }
    // `point`: the hit in world space, when the backend has one (a drag and its end have no hit of their own)
    fn hit(h: &HitData, m: &mut Map<String, Value>) {
        if let Some(p) = h.position {
            m.insert("point".into(), json!([p.x, p.y, p.z]));
        }
    }
    fn button(b: PointerButton, m: &mut Map<String, Value>) {
        m.insert("button".into(), json!(match b { PointerButton::Primary => 0, PointerButton::Middle => 1, PointerButton::Secondary => 2 }));
    }
    type M = Map<String, Value>;
    route! {
        PointerClick => "onClick", |e: &PointerClick, m: &mut M| { hit(&e.hit, m); button(e.button, m) };
        PointerPress => "onPointerDown", |e: &PointerPress, m: &mut M| { hit(&e.hit, m); button(e.button, m) };
        PointerRelease => "onPointerUp", |e: &PointerRelease, m: &mut M| { hit(&e.hit, m); button(e.button, m) };
        PointerOver => "onPointerOver", |e: &PointerOver, m: &mut M| hit(&e.hit, m);
        PointerOut => "onPointerOut", |e: &PointerOut, m: &mut M| hit(&e.hit, m);
        PointerEnter => "onPointerEnter", |e: &PointerEnter, m: &mut M| hit(&e.hit, m);
        PointerLeave => "onPointerLeave", |e: &PointerLeave, m: &mut M| hit(&e.hit, m);
        PointerMove => "onPointerMove", |e: &PointerMove, m: &mut M| hit(&e.hit, m);
        PointerCancel => "onPointerCancel", |e: &PointerCancel, m: &mut M| hit(&e.hit, m);
        PointerScroll => "onWheel", |e: &PointerScroll, m: &mut M| {
            hit(&e.hit, m);
            m.insert("deltaX".into(), json!(e.x));
            m.insert("deltaY".into(), json!(e.y));
        };
        PointerDragStart => "onDragStart", |e: &PointerDragStart, m: &mut M| { hit(&e.hit, m); button(e.button, m) };
        PointerDrag => "onDrag", |e: &PointerDrag, m: &mut M| button(e.button, m);
        PointerDragEnd => "onDragEnd", |e: &PointerDragEnd, m: &mut M| button(e.button, m);
        PointerDragEnter => "onDragEnter", |e: &PointerDragEnter, m: &mut M| { hit(&e.hit, m); button(e.button, m) };
        PointerDragOver => "onDragOver", |e: &PointerDragOver, m: &mut M| { hit(&e.hit, m); button(e.button, m) };
        PointerDragLeave => "onDragLeave", |e: &PointerDragLeave, m: &mut M| { hit(&e.hit, m); button(e.button, m) };
        PointerDragDrop => "onDrop", |e: &PointerDragDrop, m: &mut M| { hit(&e.hit, m); button(e.button, m) };
    }
}

/// The entity's handler, when its props have one: bevy-react's dispatch finds it by id and kind (`onClick` → `click`).
fn send(q: &Query<(&ReactNode, &ReactProps)>, out: Option<&ReactOutbox>, entity: Entity, prop: &str, mut m: Map<String, Value>) {
    let (Ok((node, props)), Some(out)) = (q.get(entity), out) else { return };
    if props.0.get(prop) != Some(&Value::Bool(true)) {
        return;
    }
    m.insert("id".into(), json!(node.0));
    m.insert("kind".into(), json!(format!("{}{}", prop[2..3].to_lowercase(), &prop[3..])));
    out.send(json!({"t": "uiEvent", "event": m}));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::NormalizedRenderTarget;
    use bevy::picking::pointer::{Location, PointerId};
    use std::sync::mpsc::channel;
    use std::sync::Mutex;
    use std::time::Duration;

    #[test]
    fn a_click_on_an_entity_goes_to_its_on_click_and_bubbles() {
        let mut app = App::new();
        let (tx, rx) = channel();
        app.add_plugins(MinimalPlugins).insert_resource(ReactOutbox(Mutex::new(tx)));
        setup(&mut app);
        // picking's traversal reads Window (a real App's WindowPlugin registers it)
        app.world_mut().register_component::<bevy::window::Window>();
        let props = |keys: &[&str]| ReactProps(keys.iter().map(|k| (k.to_string(), Value::Bool(true))).collect());
        let parent = app.world_mut().spawn((ReactNode(1), props(&["onClick"]))).id();
        let mesh = app.world_mut().spawn((ReactNode(2), props(&["onClick", "onWheel"]), ChildOf(parent))).id();
        let quiet = app.world_mut().spawn((ReactNode(3), props(&[]))).id();
        app.update();
        let at = Location { target: NormalizedRenderTarget::None { width: 100, height: 100 }, position: Vec2::new(10.0, 20.0) };
        let hit = HitData::new(Entity::PLACEHOLDER, 1.0, Some(Vec3::new(0.0, 1.0, 0.0)), None);
        let click = |e| PointerClick { entity: e, pointer: Pointer::new(PointerId::Mouse, at.clone()), button: PointerButton::Primary, hit: hit.clone(), duration: Duration::ZERO, count: 1 };
        app.world_mut().trigger(click(mesh));
        let got: Vec<Value> = rx.try_iter().map(|s| serde_json::from_str(&s).unwrap()).collect();
        let ev = |id: u64| json!({"t": "uiEvent", "event": {"id": id, "kind": "click", "x": 10.0, "y": 20.0, "point": [0.0, 1.0, 0.0], "button": 0}});
        assert_eq!(got, vec![ev(2), ev(1)]);
        app.world_mut().trigger(click(quiet));
        assert_eq!(rx.try_iter().count(), 0);
        app.world_mut().trigger(PointerScroll { entity: mesh, pointer: Pointer::new(PointerId::Mouse, at), unit: bevy::input::mouse::MouseScrollUnit::Line, x: 0.0, y: -3.0, hit, phase: bevy::input::touch::TouchPhase::Moved });
        let got: Vec<Value> = rx.try_iter().map(|s| serde_json::from_str(&s).unwrap()).collect();
        assert_eq!(got.len(), 1);
        assert_eq!((got[0]["event"]["kind"].clone(), got[0]["event"]["deltaY"].clone()), (json!("wheel"), json!(-3.0)));
    }
}
