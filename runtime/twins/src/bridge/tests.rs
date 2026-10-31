use super::*;

/// An App with one HTML page, restyled and twinned each update as PagePlugin does (no GPU: nothing painted).
fn app(html: &str) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(BridgePlugin::<()>::default()).init_non_send::<Docs>().init_resource::<raea_sys_page::PageClock>();
    app.add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    let at = Page { size: UVec2::new(400, 300), scale: 1.0 };
    let page = app.world_mut().spawn(at).id();
    let doc = blitz_html::HtmlDocument::from_html(html, raea_sys_page::config(at)).into_inner();
    app.world_mut().non_send_mut::<Docs>().0.insert(page, doc);
    app.update();
    (app, page)
}

fn twin(app: &mut App, page: Entity, selector: &str) -> Option<Entity> {
    let selector = selector.to_string();
    app.world_mut().run_system_once(move |q: PageQuery| q.one(page, &selector)).unwrap()
}

use bevy::ecs::system::RunSystemOnce;

#[test]
fn twins_carry_boxes_and_follow_their_elements() {
    let (mut app, page) = app(r#"<body style="margin:0"><div id="a" class="x" style="width:100px;height:40px"></div><p class="x">hi</p></body>"#);
    let a = twin(&mut app, page, "#a").expect("a twin for #a");
    let b = app.world().get::<ElementBox>(a).unwrap().0;
    assert_eq!((b.min, b.size()), (Vec2::ZERO, Vec2::new(100.0, 40.0)));
    assert_eq!(app.world().get::<PageElement>(a).unwrap().page, page);
    let xs = app.world_mut().run_system_once(move |q: PageQuery| q.by_class(page, "x")).unwrap();
    assert_eq!(xs.len(), 2);
    assert_eq!(xs[0], a);

    // the element dropped: its twin with it
    let node = app.world().get::<PageElement>(a).unwrap().node;
    app.world_mut().non_send_mut::<Docs>().0.get_mut(&page).unwrap().mutate().remove_and_drop_node(node);
    app.update();
    assert!(app.world().get_entity(a).is_err());
    assert_eq!(twin(&mut app, page, "#a"), None);

    // the page gone: every twin with it
    let p = twin(&mut app, page, "p").unwrap();
    app.world_mut().despawn(page);
    app.update();
    assert!(app.world().get_entity(p).is_err());
}

#[test]
fn commands_land_before_restyle_in_the_same_frame() {
    let (mut app, page) = app(r#"<body style="margin:0"><div id="a" class="x" style="width:100px;height:40px">old</div></body>"#);
    let a = twin(&mut app, page, "#a").unwrap();
    app.world_mut()
        .run_system_once(move |mut c: PageCommands| {
            c.add_class(a, "y");
            c.remove_class(a, "x");
            c.set_style(a, "width", "250px");
            c.set_attribute(a, "data-k", "v");
            c.set_text(a, "new");
        })
        .unwrap();
    app.update();
    // one update: applied, restyled, laid out and the box written
    assert_eq!(app.world().get::<ElementBox>(a).unwrap().0.width(), 250.0);
    let docs = app.world().non_send::<Docs>();
    let doc = &docs.0[&page];
    let n = doc.get_node(app.world().get::<PageElement>(a).unwrap().node).unwrap();
    assert_eq!(n.attr(blitz_dom::LocalName::from("class")), Some("y"));
    assert_eq!(n.attr(blitz_dom::LocalName::from("data-k")), Some("v"));
    assert_eq!(n.text_content(), "new");
}

/// A plain handler, as the host's stands in for React: it prevents a press's default.
struct Prevent;
impl EventHandler for Prevent {
    fn handle_event(&mut self, _: &[NodeId], event: &mut DomEvent, _: &mut dyn Document, state: &mut EventState) {
        if matches!(event.data, DomEventData::PointerDown(_)) {
            state.prevent_default();
        }
    }
}

#[derive(Resource, Default)]
struct Heard(Vec<(&'static str, Entity, Entity, bool)>);

#[test]
fn dom_events_reach_the_twins_up_the_path() {
    use blitz_traits::events::{BlitzPointerEvent, MouseEventButton, MouseEventButtons, PointerCoords, PointerDetails, BlitzPointerId, UiEvent};
    let (mut app, page) = app(r#"<body style="margin:0"><div id="outer" style="padding:10px"><button id="b" style="width:80px;height:30px">go</button></div></body>"#);
    let (b, outer) = (twin(&mut app, page, "#b").unwrap(), twin(&mut app, page, "#outer").unwrap());
    app.init_resource::<Heard>();
    app.add_observer(|e: On<dom::PointerDown>, mut h: ResMut<Heard>| h.0.push(("down", e.entity, e.target, e.default_prevented)));
    app.add_observer(|e: On<dom::PointerUp>, mut h: ResMut<Heard>| h.0.push(("up", e.entity, e.target, e.default_prevented)));
    let at = |x: f32, y: f32| PointerCoords { page_x: x, page_y: y, screen_x: x, screen_y: y, client_x: x, client_y: y };
    let press = |ev: fn(BlitzPointerEvent) -> UiEvent, buttons| {
        ev(BlitzPointerEvent { id: BlitzPointerId::Mouse, is_primary: true, coords: at(30.0, 20.0), button: MouseEventButton::Main, buttons, mods: Default::default(), details: PointerDetails::default(), element: Default::default(), active_pointers: Default::default() })
    };
    for ev in [press(UiEvent::PointerDown, MouseEventButtons::Primary), press(UiEvent::PointerUp, MouseEventButtons::None)] {
        let mut doc = app.world_mut().non_send_mut::<Docs>().0.remove(&page).unwrap();
        let mut fired = vec![];
        blitz_dom::EventDriver::new(&mut doc, Forward { inner: Prevent, fired: &mut fired }).handle_ui_event(ev);
        app.world_mut().non_send_mut::<Docs>().0.insert(page, doc);
        deliver::<()>(app.world_mut(), page, fired);
    }
    let heard = &app.world().resource::<Heard>().0;
    // the press: the button's twin first, then its parent's, both with the button as target, prevented
    assert!(heard.starts_with(&[("down", b, b, true), ("down", outer, b, true)]), "{heard:?}");
    assert!(heard.iter().any(|h| *h == ("up", b, b, false)), "{heard:?}");
}

/// A handler whose press makes an element (React's commit in an event's handler): a `div#made` in the target's
/// parent.
struct Make;
impl EventHandler for Make {
    fn handle_event(&mut self, chain: &[NodeId], event: &mut DomEvent, doc: &mut dyn Document, _: &mut EventState) {
        if !matches!(event.data, DomEventData::PointerDown(_)) {
            return;
        }
        let Some(&parent) = chain.get(1) else { return };
        let mut doc = doc.inner_mut();
        let mut m = doc.mutate();
        let name = |n: &str| blitz_dom::QualName { prefix: None, ns: blitz_dom::ns!(html), local: blitz_dom::LocalName::from(n) };
        let made = m.create_element(name("div"), vec![blitz_dom::Attribute { name: name("id"), value: "made".into() }]);
        m.append_children(parent, &[made]);
    }
}

#[test]
fn an_element_the_events_commit_makes_has_a_twin_in_that_frame() {
    use blitz_traits::events::{BlitzPointerEvent, MouseEventButton, MouseEventButtons, PointerCoords, PointerDetails, BlitzPointerId, UiEvent};
    let (mut app, page) = app(r#"<body style="margin:0"><div id="outer"><button id="b" style="width:80px;height:30px">go</button></div></body>"#);
    assert!(twin(&mut app, page, "#made").is_none());
    let at = PointerCoords { page_x: 30.0, page_y: 15.0, screen_x: 30.0, screen_y: 15.0, client_x: 30.0, client_y: 15.0 };
    let ev = UiEvent::PointerDown(BlitzPointerEvent { id: BlitzPointerId::Mouse, is_primary: true, coords: at, button: MouseEventButton::Main, buttons: MouseEventButtons::Primary, mods: Default::default(), details: PointerDetails::default(), element: Default::default(), active_pointers: Default::default() });
    let mut doc = app.world_mut().non_send_mut::<Docs>().0.remove(&page).unwrap();
    let mut fired = vec![];
    blitz_dom::EventDriver::new(&mut doc, Forward { inner: Make, fired: &mut fired }).handle_ui_event(ev);
    app.world_mut().non_send_mut::<Docs>().0.insert(page, doc);
    deliver::<()>(app.world_mut(), page, fired);
    // no update since: the twin was made with the delivery, a child of its parent's
    let made = twin(&mut app, page, "#made").expect("a twin for the element the event's commit made");
    let outer = twin(&mut app, page, "#outer").unwrap();
    assert_eq!(app.world().get::<ChildOf>(made).map(|c| c.parent()), Some(outer));
    // and the frame's own twins step keeps it
    app.update();
    assert_eq!(twin(&mut app, page, "#made"), Some(made));
}

/// A handler that does nothing, so Blitz's default actions all run.
struct Nop;
impl EventHandler for Nop {
    fn handle_event(&mut self, _: &[NodeId], _: &mut DomEvent, _: &mut dyn Document, _: &mut EventState) {}
}

fn send(app: &mut App, page: Entity, ev: blitz_traits::events::UiEvent) {
    let mut doc = app.world_mut().non_send_mut::<Docs>().0.remove(&page).unwrap();
    let mut fired = vec![];
    blitz_dom::EventDriver::new(&mut doc, Forward { inner: Nop, fired: &mut fired }).handle_ui_event(ev);
    app.world_mut().non_send_mut::<Docs>().0.insert(page, doc);
    deliver::<()>(app.world_mut(), page, fired);
    app.update();
}

#[derive(Resource, Default)]
struct Kinds(Vec<(&'static str, Entity)>);

#[test]
fn keys_focus_typing_hover_and_wheel_reach_the_twins() {
    use blitz_traits::events::*;
    use keyboard_types::{Code, Key, Location, Modifiers};
    let (mut app, page) = app(r#"<body style="margin:0"><input id="f" style="display:block;width:200px;height:30px"><div id="s" style="height:100px;overflow:auto"><div style="height:1000px"></div></div></body>"#);
    let (f, sc) = (twin(&mut app, page, "#f").unwrap(), twin(&mut app, page, "#s").unwrap());
    app.init_resource::<Kinds>();
    macro_rules! hear {
        ($($k:ident),*) => { $(app.add_observer(|e: On<dom::$k>, mut h: ResMut<Kinds>| h.0.push((stringify!($k), e.entity)));)* };
    }
    hear!(PointerEnter, PointerLeave, Click, Focus, Blur, KeyDown, KeyUp, Input, Wheel, Scroll);
    let at = |x: f32, y: f32| PointerCoords { page_x: x, page_y: y, screen_x: x, screen_y: y, client_x: x, client_y: y };
    let ptr = |x, y, buttons| BlitzPointerEvent { id: BlitzPointerId::Mouse, is_primary: true, coords: at(x, y), button: MouseEventButton::Main, buttons, mods: Default::default(), details: PointerDetails::default(), element: Default::default(), active_pointers: Default::default() };
    let key = |c: &str, down| BlitzKeyEvent { key: Key::Character(c.into()), code: Code::KeyA, modifiers: Modifiers::empty(), location: Location::Standard, is_auto_repeating: false, is_composing: false, state: if down { KeyState::Pressed } else { KeyState::Released }, text: Some(c.into()) };
    // a hover, then a click into the field: enter, click, focus
    send(&mut app, page, UiEvent::PointerMove(ptr(20.0, 10.0, MouseEventButtons::None)));
    send(&mut app, page, UiEvent::PointerDown(ptr(20.0, 10.0, MouseEventButtons::Primary)));
    send(&mut app, page, UiEvent::PointerUp(ptr(20.0, 10.0, MouseEventButtons::None)));
    // typing: keydown, the field's input, keyup
    send(&mut app, page, UiEvent::KeyDown(key("a", true)));
    send(&mut app, page, UiEvent::KeyUp(key("a", false)));
    // the pointer over the scroller (leaving the field), and the wheel there: wheel, then its scroll
    send(&mut app, page, UiEvent::PointerMove(ptr(20.0, 80.0, MouseEventButtons::None)));
    send(&mut app, page, UiEvent::Wheel(BlitzWheelEvent { delta: BlitzWheelDelta::Pixels(0.0, -40.0), coords: at(20.0, 80.0), buttons: MouseEventButtons::None, mods: Modifiers::empty(), element: Default::default() }));
    let heard = &app.world().resource::<Kinds>().0;
    for want in [("PointerEnter", f), ("Click", f), ("Focus", f), ("KeyDown", f), ("Input", f), ("KeyUp", f), ("PointerLeave", f), ("PointerEnter", sc), ("Wheel", sc), ("Scroll", sc)] {
        assert!(heard.contains(&want), "{want:?} not in {heard:?}");
    }
}

#[test]
fn html_drag_events_reach_the_twins_up_their_path() {
    let (mut app, page) = app(r#"<body style="margin:0"><div id="list"><div id="card" draggable="true">card</div></div><div id="bin">bin</div></body>"#);
    let (list, card, bin) = (twin(&mut app, page, "#list").unwrap(), twin(&mut app, page, "#card").unwrap(), twin(&mut app, page, "#bin").unwrap());
    app.init_resource::<Kinds>();
    app.add_observer(|e: On<dom::DragStart>, mut h: ResMut<Kinds>| h.0.push(("DragStart", e.entity)));
    app.add_observer(|e: On<dom::Drop>, mut h: ResMut<Kinds>| {
        assert_eq!(e.data.types, ["text/plain"]);
        assert!(e.default_prevented);
        h.0.push(("Drop", e.entity))
    });
    let node = |app: &App, e: Entity| app.world().get::<PageElement>(e).unwrap().node;
    let data = dom::DragData { x: 10.0, y: 40.0, drop_effect: "move".into(), effect_allowed: "move".into(), types: vec!["text/plain".into()] };
    let fired = vec![
        Fired::drag(vec![node(&app, card), node(&app, list)], "dragstart", data.clone(), false, false),
        Fired::drag(vec![node(&app, bin)], "drop", data, true, false),
    ];
    deliver::<()>(app.world_mut(), page, fired);
    let heard = &app.world().resource::<Kinds>().0;
    assert_eq!(heard, &[("DragStart", card), ("DragStart", list), ("Drop", bin)]);
}

/// React's ops, as JSON, applied to a React page's document.
fn react(app: &mut App, page: Entity, nodes: &mut raea_sys_page::ops::Nodes, ops: serde_json::Value) {
    let ops: Vec<raea_core_wire::Op> = serde_json::from_value(ops).unwrap();
    raea_sys_page::ops::apply(app.world_mut().non_send_mut::<Docs>().0.get_mut(&page).unwrap(), nodes, ops);
}

fn conflicts(app: &mut App) -> Vec<(String, Owner)> {
    app.world_mut().resource_mut::<Messages<PageConflict>>().drain().map(|c| (c.property, c.owner)).collect()
}

/// A twin's own components are the page's (H9): a system writing a twin's PageElement or ElementBox has it put back
/// that frame, each named; a frame with no such write has none.
#[test]
fn a_twins_own_components_are_the_pages() {
    #[derive(Resource)]
    struct Write(Option<Entity>);
    let (mut app, page) = app(r#"<body style="margin:0"><div id="card" style="width:100px;height:50px"></div></body>"#);
    let card = twin(&mut app, page, "#card").unwrap();
    let (el, at) = (*app.world().get::<PageElement>(card).unwrap(), *app.world().get::<ElementBox>(card).unwrap());
    assert_eq!(at.0.size(), Vec2::new(100.0, 50.0));
    app.insert_resource(Write(Some(card))).add_systems(Update, |mut w: ResMut<Write>, mut q: Query<(&mut PageElement, &mut ElementBox)>| {
        if let Some((mut el, mut b)) = w.0.take().and_then(|e| q.get_mut(e).ok()) {
            el.op = Some(99);
            b.0 = Rect::new(0.0, 0.0, 1.0, 1.0);
        }
    });
    app.update();
    assert_eq!((*app.world().get::<PageElement>(card).unwrap(), *app.world().get::<ElementBox>(card).unwrap()), (el, at));
    assert_eq!(conflicts(&mut app), vec![("PageElement".into(), Owner::Page), ("ElementBox".into(), Owner::Page)]);
    app.update();
    assert_eq!(conflicts(&mut app), vec![]);
    // the page resizing its element isn't one
    app.world_mut().run_system_once(move |mut c: PageCommands| c.set_style(card, "width", "120px")).unwrap();
    app.update();
    assert_eq!(app.world().get::<ElementBox>(card).unwrap().0.width(), 120.0);
    assert_eq!(conflicts(&mut app), vec![]);
}

#[test]
fn each_property_has_one_owner() {
    use serde_json::json;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(BridgePlugin::<()>::default()).init_non_send::<Docs>().init_resource::<raea_sys_page::PageClock>();
    app.add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    let (page, mut nodes) = raea_sys_page::open_doc(app.world_mut(), 400, 300, None);
    react(&mut app, page, &mut nodes, json!([
        { "op": "create", "id": 1, "kind": "div", "props": { "className": "card", "style": { "width": "100px", "height": "40px", "background": "red" } } },
        { "op": "append", "parent": 0, "child": 1 },
    ]));
    app.update();
    let a = twin(&mut app, page, ".card").unwrap();
    let get = |app: &App, f: &dyn Fn(&blitz_dom::Node) -> String| {
        let docs = app.world().non_send::<Docs>();
        f(docs.0[&page].get_node(app.world().get::<PageElement>(a).unwrap().node).unwrap())
    };
    let opacity = |n: &blitz_dom::Node| n.primary_styles().unwrap().get_effects().opacity.to_string();
    let class = |n: &blitz_dom::Node| n.attr(blitz_dom::LocalName::from("class")).unwrap_or("").to_string();
    let data = |n: &blitz_dom::Node| n.attr(blitz_dom::LocalName::from("data-k")).unwrap_or("").to_string();

    // a system writes React's background (through its longhand): refused, named; its own opacity, class and
    // attribute land
    app.world_mut()
        .run_system_once(move |mut c: PageCommands| {
            c.set_style(a, "background-color", "blue");
            c.set_style(a, "opacity", "0.5");
            c.add_class(a, "lit");
            c.remove_class(a, "card");
            c.set_attribute(a, "data-k", "v");
        })
        .unwrap();
    app.update();
    assert_eq!(conflicts(&mut app), vec![("background-color".into(), Owner::React), (".card".into(), Owner::React)]);
    assert_eq!((get(&app, &opacity), get(&app, &class), get(&app, &data)), ("0.5".into(), "card lit".into(), "v".into()));

    // React renders again, its style and className whole: Bevy's opacity and class are kept, with no conflict
    react(&mut app, page, &mut nodes, json!([{ "op": "update", "id": 1, "set": { "className": "card big", "style": { "width": "120px", "height": "40px", "background": "red" } } }]));
    app.update();
    assert_eq!(conflicts(&mut app), vec![]);
    assert_eq!((get(&app, &opacity), get(&app, &class)), ("0.5".into(), "card big lit".into()));
    assert_eq!(app.world().get::<ElementBox>(a).unwrap().0.width(), 120.0);

    // React writes Bevy's: named, and Bevy's value is put back before restyle
    react(&mut app, page, &mut nodes, json!([{ "op": "update", "id": 1, "set": { "data-k": "w", "style": { "width": "120px", "height": "40px", "background": "red", "opacity": 1 } } }]));
    app.update();
    let mut got = conflicts(&mut app);
    got.sort_by(|x, y| x.0.cmp(&y.0));
    assert_eq!(got, vec![("[data-k]".into(), Owner::Bevy), ("opacity".into(), Owner::Bevy)]);
    assert_eq!((get(&app, &opacity), get(&app, &data)), ("0.5".into(), "v".into()));
}
