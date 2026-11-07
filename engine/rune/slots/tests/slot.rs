//! A Blitz page with a Rune layer, through the page host (raea-runtime-dom) in a headless Bevy app: what's under the
//! marked element is the layer's Rune page's and not the document's, the layer is its slot's size and shown there,
//! and a press on the layer's button reaches the page's JS as an event of that button.

use bevy::prelude::*;
use bevy::window::ExitCondition;
use raea_rune_bevy::{Input, RuneLayer, Trees};
use raea_rune_page::page::Page;
use raea_rune_slots::{RuneSlotsPlugin, Slots};
use raea_runtime_dom::host::{self, HostStepPlugin, PageAsks, Roots};
use raea_sys_page::view3d::ElementImages;
use raea_sys_page::{Docs, PageClock, PageSystems};
use serde_json::Value;

const CODE: &str = r#"
    globalThis.__ui_deliver = (json) => { const m = JSON.parse(json); if (m.t === "event") __ui_emit("heard", JSON.stringify(m)); };
    __ui_flush(JSON.stringify({ v: 1, ops: [
        { op: "create", id: 1, kind: "div", props: {} },
        { op: "create", id: 2, kind: "div", props: { "data-raea-renderer": "rune", style: { width: "200px", height: "100px" } } },
        { op: "create", id: 3, kind: "button", props: { style: { width: "200px", height: "100px" } } },
        { op: "create", id: 4, kind: "text", props: {}, text: "Press" },
        { op: "append", parent: 3, child: 4 },
        { op: "listen", id: 3, events: ["pointerdown"] },
        { op: "append", parent: 2, child: 3 },
        { op: "create", id: 5, kind: "p", props: {} },
        { op: "append", parent: 1, child: 2 },
        { op: "append", parent: 1, child: 5 },
        { op: "append", parent: 0, child: 1 },
    ] }));
"#;

fn app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: true, ..default() }),
    );
    app.add_plugins((HostStepPlugin, RuneSlotsPlugin))
        .init_non_send::<Docs>()
        .init_resource::<PageClock>()
        .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    app.finish();
    app.cleanup();
    let page = host::mount(app.world_mut(), 400, 300, CODE, &[], None).unwrap();
    for _ in 0..6 {
        app.update();
    }
    (app, page)
}

fn layer(app: &App, page: Entity) -> Entity {
    app.world().non_send::<Slots>().layers.get(&(page, 2)).expect("a layer for the marked element").entity
}

#[test]
fn whats_under_the_slot_is_the_layers() {
    let (app, page) = app();
    let w = app.world();
    // the document: the slot, empty, beside the page's own <p>
    let roots = w.non_send::<Roots>();
    let nodes = &roots.0.iter().find(|r| r.page == page).unwrap().nodes;
    assert!(nodes.by_id.contains_key(&2) && nodes.by_id.contains_key(&5));
    assert!(!nodes.by_id.contains_key(&3) && !nodes.by_id.contains_key(&4), "the layer's elements reached Blitz");
    let doc = w.non_send::<Docs>();
    let slot = doc.0[&page].get_node(nodes.by_id[&2]).unwrap();
    assert!(slot.children.is_empty(), "the slot has Blitz children");
    // the layer's page: the button under its root
    let e = layer(&app, page);
    let trees = w.non_send::<Trees>();
    let tree = &trees.0[&e];
    let p = tree.state().downcast_ref::<Page>().unwrap();
    assert_eq!(p.tree.get(0).unwrap().children, vec![3]);
    assert_eq!(p.tree.get(3).unwrap().children, vec![4]);
    // sized to the slot's content box, its Image shown in the slot
    assert_eq!(tree.viewport(), ((200.0, 100.0), 1.0));
    let image = &w.get::<RuneLayer>(e).unwrap().image;
    assert_eq!(w.resource::<ElementImages>().0.get(&(page, nodes.by_id[&2])), Some(image));
    assert!(w.non_send::<Slots>().report.iter().all(|r| !r.contains("moved")), "{:?}", w.non_send::<Slots>().report);
}

#[test]
fn a_press_on_the_layer_reaches_the_pages_js() {
    let (mut app, page) = app();
    let e = layer(&app, page);
    for ev in [Input::Move(40.0, 30.0), Input::Down(40.0, 30.0), Input::Up(40.0, 30.0)] {
        app.world_mut().non_send_mut::<Trees>().0.get_mut(&e).unwrap().input(ev);
        app.update();
    }
    for _ in 0..3 {
        app.update();
    }
    let asks = &app.world().resource::<PageAsks>().0;
    let heard: Vec<&Value> = asks.iter().filter(|(p, n, _)| *p == page && n == "heard").map(|(_, _, v)| v).collect();
    assert!(heard.iter().any(|m| m["id"] == 3 && m["event"] == "pointerdown"), "the JS heard {heard:?}");
}

/// Picking's press on the slot's twin, and its release on another element's: the layer has the press, and the release
/// still ends it there (the pointer captured, as setPointerCapture does), so its button isn't left held.
#[test]
fn a_press_held_off_the_layer_still_ends_there() {
    use bevy::camera::NormalizedRenderTarget;
    use bevy::picking::backend::HitData;
    use bevy::picking::events::{Pointer, PointerPress, PointerRelease};
    use bevy::picking::pointer::{Location, PointerButton, PointerId};
    use raea_runtime_twins::bridge::PageElement;

    let (mut app, page) = app();
    let w = app.world_mut();
    let (slot, p) = {
        let roots = w.non_send::<Roots>();
        let nodes = &roots.0.iter().find(|r| r.page == page).unwrap().nodes;
        (nodes.by_id[&2], nodes.by_id[&5])
    };
    let at = {
        let docs = w.non_send::<Docs>();
        let n = docs.0[&page].get_node(slot).unwrap();
        let (abs, l) = (n.absolute_position(0.0, 0.0), n.final_layout());
        Vec2::new(abs.x + l.border.left + l.padding.left, abs.y + l.border.top + l.padding.top) + Vec2::new(40.0, 30.0)
    };
    let slot_twin = w.spawn(PageElement { page, node: slot, op: Some(2) }).id();
    let p_twin = w.spawn(PageElement { page, node: p, op: Some(5) }).id();
    let pointer = Pointer::new(PointerId::Mouse, Location { target: NormalizedRenderTarget::None { width: 400, height: 300 }, position: Vec2::ZERO });
    let hit = |p: Vec2| HitData::new(Entity::PLACEHOLDER, 0.0, Some(p.extend(0.0)), None);
    w.trigger(PointerPress { entity: slot_twin, pointer: pointer.clone(), button: PointerButton::Primary, hit: hit(at), count: 1 });
    app.update();
    app.update();
    let e = layer(&app, page);
    let pressed = |app: &App| app.world().non_send::<Trees>().0[&e].state().downcast_ref::<Page>().unwrap().pressed.clone();
    // (the press holds the button and each element up to the layer's root, as :active does)
    assert!(pressed(&app).contains(&3), "the press didn't reach the layer's button: {:?}", pressed(&app));
    app.world_mut().trigger(PointerRelease { entity: p_twin, pointer, button: PointerButton::Primary, hit: hit(at + Vec2::new(0.0, 200.0)) });
    for _ in 0..3 {
        app.update();
    }
    assert!(pressed(&app).is_empty(), "the release off the layer left its button held");
}
