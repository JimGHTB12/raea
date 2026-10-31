//! B3 H11's scenarios (docs/blitz-bevy.feature): the hooks together, as an App uses them. bench/bridge/run.sh runs
//! these with each hook's own tests. (A click on a button on a tilted 3D screen is pick.rs's
//! a_page_on_a_tilted_mesh_is_clicked_through_its_uvs.)

use bevy::camera::{NormalizedRenderTarget, RenderTarget};
use bevy::ecs::system::RunSystemOnce;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::input_focus::InputFocus;
use bevy::picking::PickingSystems;
use bevy::picking::backend::ray::{RayId, RayMap};
use bevy::picking::events::PointerClick;
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
use bevy::prelude::*;
use bevy::reflect::structs::Struct;
use bevy::ui::{ComputedNode, ComputedUiTargetCamera, UiGlobalTransform};
use bevy::window::{PrimaryWindow, WindowEvent, WindowRef};

use raea_runtime_twins::bridge::{BridgePlugin, PageCommands, PageConflict, PageElement, PageQuery};

use raea_sys_page::PageSystems;
use raea_runtime_input::keys::KeysPlugin;
use raea_runtime_input::pick::{PageView, PickPlugin};
use raea_sys_page::{Docs, Page};

/// A window with a camera on it, picking with the page's backend and route, keys by focus; no page yet.
fn world(meshes: bool) -> (App, Entity, NormalizedRenderTarget) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::input::InputPlugin, bevy::picking::PickingPlugin, bevy::picking::InteractionPlugin, BridgePlugin::<raea_runtime_dom::host::Roots>::default(), PickPlugin, KeysPlugin))
        .add_message::<WindowEvent>()
        .init_non_send::<Docs>()
        .init_resource::<raea_sys_page::PageClock>()
        .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    if meshes {
        app.add_plugins(bevy::asset::AssetPlugin::default()).init_asset::<Mesh>();
    }
    let window = app.world_mut().spawn((Window::default(), PrimaryWindow)).id();
    let cam = app.world_mut().spawn((Camera::default(), RenderTarget::Window(WindowRef::Primary))).id();
    app.world_mut().spawn(PointerId::Mouse);
    let to = NormalizedRenderTarget::Window(WindowRef::Primary.normalize(Some(window)).unwrap());
    (app, cam, to)
}

fn html_page(app: &mut App, html: &str) -> Entity {
    let at = Page { size: UVec2::new(400, 300), scale: 1.0 };
    let page = app.world_mut().spawn(at).id();
    app.world_mut().non_send_mut::<Docs>().0.insert(page, blitz_html::HtmlDocument::from_html(html, raea_sys_page::config(at)).into_inner());
    page
}

/// `page` shown on a UI box `size` at the window's top left, through `cam`.
fn ui_view(app: &mut App, page: Entity, cam: Entity, size: Vec2) -> Entity {
    let mut target = ComputedUiTargetCamera::default();
    target.field_mut("camera").unwrap().apply(&Some(cam));
    app.world_mut().spawn((PageView::new(page), ComputedNode { size, ..default() }, UiGlobalTransform::from_translation(size / 2.0), target)).id()
}

fn pointer(app: &mut App, to: &NormalizedRenderTarget, at: Vec2, action: PointerAction) {
    app.world_mut().write_message(PointerInput::new(PointerId::Mouse, Location { target: to.clone(), position: at }, action));
    app.update();
}

fn click(app: &mut App, to: &NormalizedRenderTarget, at: Vec2) {
    for action in [PointerAction::Move { delta: Vec2::ZERO }, PointerAction::Press(PointerButton::Primary), PointerAction::Release(PointerButton::Primary)] {
        pointer(app, to, at, action);
    }
}

fn typed(app: &mut App, text: &str) {
    let window = app.world_mut().query_filtered::<Entity, With<PrimaryWindow>>().single(app.world()).unwrap();
    for c in text.chars() {
        for state in [ButtonState::Pressed, ButtonState::Released] {
            let k = KeyboardInput { key_code: KeyCode::KeyA, logical_key: Key::Character(c.to_string().into()), state, text: (state == ButtonState::Pressed).then(|| c.to_string().into()), repeat: false, window };
            app.world_mut().write_message(WindowEvent::KeyboardInput(k));
        }
        app.update();
    }
}

fn twin(app: &mut App, page: Entity, sel: &'static str) -> Entity {
    app.world_mut().run_system_once(move |q: PageQuery| q.one(page, sel)).unwrap().unwrap()
}

/// `f` of the twin's element.
fn with_node<T>(app: &App, twin: Entity, f: impl FnOnce(&blitz_dom::Node) -> T) -> T {
    let el = app.world().get::<PageElement>(twin).unwrap();
    f(app.world().non_send::<Docs>().0[&el.page].get_node(el.node).unwrap())
}

/// Typing into a field on a phone in the scene, IME included: a click through the phone's screen (a page on a mesh,
/// turned) focuses the field, InputFocus is its twin, the window's IME turns on, and the keys reach the field.
#[test]
fn typing_into_a_field_on_a_phone_in_the_scene() {
    let (mut app, cam, to) = world(true);
    let page = html_page(&mut app, r#"<body style="margin:0"><input id="f" style="display:block;margin:40px 0 0 0;width:400px;height:40px"></body>"#);
    // the phone's screen: 3 x 2.25 (UV 0,0 at its top left), turned 25 degrees about Y and tipped back 10
    let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Rectangle::new(3.0, 2.25)));
    let phone = Transform::from_rotation(Quat::from_rotation_y(25f32.to_radians()) * Quat::from_rotation_x(-10f32.to_radians()));
    app.world_mut().spawn((PageView::new(page), Mesh3d(mesh), GlobalTransform::from(phone)));
    // aimed at the field's middle, page (200, 60): UV (0.5, 0.2), the screen's (0, 0.675, 0)
    let aim = phone.transform_point(Vec3::new(0.0, 0.675, 0.0));
    let ray = Ray3d::new(aim + Vec3::new(0.0, 0.0, 6.0), Dir3::new(-Vec3::new(0.0, 0.0, 6.0)).unwrap());
    app.add_systems(PreUpdate, (move |mut rays: ResMut<RayMap>| {
        rays.map.insert(RayId::new(cam, PointerId::Mouse), ray);
    }).after(PickingSystems::ProcessInput).before(PickingSystems::Backend));
    app.update();
    let f = twin(&mut app, page, "#f");
    click(&mut app, &to, Vec2::ZERO);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(f));
    assert!(app.world_mut().query::<&Window>().single(app.world()).unwrap().ime_enabled);
    typed(&mut app, "hi");
    assert_eq!(with_node(&app, f, |n| n.element_data().unwrap().text_input_data().unwrap().editor.raw_text().to_string()), "hi");
}

#[derive(Resource)]
struct Menu(Entity);

/// A system recolouring a menu in the same frame: a click on a React menu's item is heard by a Bevy observer, which
/// recolours the item through PageCommands; the colour is the element's in the update the click landed in, beside
/// React's own style, with no conflict (H9).
#[test]
fn a_system_recolours_a_menu_in_the_same_frame() {
    let (mut app, cam, to) = world(false);
    let (page, mut nodes) = raea_sys_page::open_doc(app.world_mut(), 400, 300, None);
    let ops: Vec<raea_core_wire::Op> = serde_json::from_value(serde_json::json!([
        { "op": "create", "id": 1, "kind": "div", "props": { "className": "menu", "style": { "width": "200px" } } },
        { "op": "create", "id": 2, "kind": "div", "props": { "className": "item", "style": { "height": "40px", "color": "white" } }, "text": "Play" },
        { "op": "create", "id": 3, "kind": "div", "props": { "className": "item", "style": { "height": "40px", "color": "white" } }, "text": "Quit" },
        { "op": "append", "parent": 1, "child": 2 },
        { "op": "append", "parent": 1, "child": 3 },
        { "op": "append", "parent": 0, "child": 1 },
    ]))
    .unwrap();
    raea_sys_page::ops::apply(app.world_mut().non_send_mut::<Docs>().0.get_mut(&page).unwrap(), &mut nodes, ops);
    ui_view(&mut app, page, cam, Vec2::new(400.0, 300.0));
    app.update();
    let menu = twin(&mut app, page, ".menu");
    app.insert_resource(Menu(menu));
    // the game's system: an item clicked is the one lit
    app.add_observer(|e: On<PointerClick>, menu: Res<Menu>, items: Query<&ChildOf, With<PageElement>>, mut page: PageCommands| {
        if items.get(e.entity).is_ok_and(|c| c.parent() == menu.0) {
            page.set_style(e.entity, "background-color", "rgb(255, 122, 26)");
        }
    });
    let quit = twin(&mut app, page, ".item:nth-child(2)");
    for action in [PointerAction::Move { delta: Vec2::ZERO }, PointerAction::Press(PointerButton::Primary)] {
        pointer(&mut app, &to, Vec2::new(20.0, 60.0), action);
    }
    // the release: Bevy's click, the system's command, restyled, all in this one update
    pointer(&mut app, &to, Vec2::new(20.0, 60.0), PointerAction::Release(PointerButton::Primary));
    let bg = with_node(&app, quit, |n| {
        let s = n.primary_styles().unwrap();
        let c = s.resolve_color(&s.clone_background_color()).into_srgb_legacy();
        c.raw_components().map(|x| (x * 255.0).round() as u8)
    });
    assert_eq!(bg, [255, 122, 26, 255]);
    assert_eq!(app.world_mut().resource_mut::<Messages<PageConflict>>().drain().count(), 0);
}

/// A page view resized: the page keeps its size and is drawn scaled, and a click lands on the element under it at the
/// view's new size.
#[test]
fn a_page_view_resized() {
    let (mut app, cam, to) = world(false);
    let page = html_page(&mut app, r#"<body style="margin:0"><div id="a" style="width:200px;height:300px"></div><div id="b" style="position:absolute;left:200px;top:0;width:200px;height:300px"></div></body>"#);
    let view = ui_view(&mut app, page, cam, Vec2::new(400.0, 300.0));
    app.update();
    let (a, b) = (twin(&mut app, page, "#a"), twin(&mut app, page, "#b"));
    #[derive(Resource, Default)]
    struct Clicked(Vec<Entity>);
    app.init_resource::<Clicked>();
    app.add_observer(|e: On<PointerClick>, mut c: ResMut<Clicked>| c.0.push(e.entity));
    // window x 300 is #b at 400 wide
    click(&mut app, &to, Vec2::new(300.0, 100.0));
    assert_eq!(app.world().resource::<Clicked>().0.first(), Some(&b));
    // the view twice as large: window x 300 is page x 150, #a
    let size = Vec2::new(800.0, 600.0);
    app.world_mut().entity_mut(view).insert((ComputedNode { size, ..default() }, UiGlobalTransform::from_translation(size / 2.0)));
    app.world_mut().resource_mut::<Clicked>().0.clear();
    click(&mut app, &to, Vec2::new(300.0, 100.0));
    assert_eq!(app.world().resource::<Clicked>().0.first(), Some(&a));
}

#[derive(Resource, Default)]
struct Clicks(Vec<Entity>);

/// A page on a 4 x 3 quad with a button at page (100, 50) (UV (0.25, 1/6), the quad's (-1, 1, 0)), and a ray from
/// `cam` that a system aims at the button wherever the quad is.
fn card(app: &mut App, cam: Entity) -> (Entity, Entity) {
    let page = html_page(app, r#"<body style="margin:0"><div id="btn" style="margin:30px 0 0 80px;width:40px;height:40px"></div></body>"#);
    let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Rectangle::new(4.0, 3.0)));
    let quad = app.world_mut().spawn((PageView::new(page), Mesh3d(mesh), GlobalTransform::IDENTITY)).id();
    app.add_systems(PreUpdate, (move |mut rays: ResMut<RayMap>, quads: Query<&GlobalTransform>| {
        let aim = quads.get(quad).unwrap().transform_point(Vec3::new(-1.0, 1.0, 0.0));
        rays.map.insert(RayId::new(cam, PointerId::Mouse), Ray3d::new(aim + Vec3::Z * 10.0, Dir3::NEG_Z));
    }).after(PickingSystems::ProcessInput).before(PickingSystems::Backend));
    app.init_resource::<Clicks>();
    app.add_observer(|e: On<PointerClick>, mut c: ResMut<Clicks>| c.0.push(e.entity));
    app.update();
    (page, quad)
}

fn clicked(app: &mut App, to: &NormalizedRenderTarget) -> Option<Entity> {
    app.world_mut().resource_mut::<Clicks>().0.clear();
    click(app, to, Vec2::ZERO);
    app.world().resource::<Clicks>().0.first().copied()
}

/// A card passing behind a 3D object: a cube (PickableMesh) between the camera and the card's button takes the click;
/// once the card has moved out from behind it, the button has it again.
#[test]
fn a_card_passing_behind_a_3d_object() {
    let (mut app, cam, to) = world(true);
    let (page, quad) = card(&mut app, cam);
    let btn = twin(&mut app, page, "#btn");
    let cube = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
    // the cube 3 in front of the card's button
    let blocker = app.world_mut().spawn((raea_runtime_input::pick::PickableMesh, Mesh3d(cube), GlobalTransform::from_translation(Vec3::new(-1.0, 1.0, 3.0)))).id();
    // the card slides right: behind the cube, the cube has the click; past it, the button
    assert_eq!(clicked(&mut app, &to), Some(blocker));
    app.world_mut().entity_mut(quad).insert(GlobalTransform::from_translation(Vec3::new(2.5, 0.0, 0.0)));
    assert_eq!(clicked(&mut app, &to), Some(btn));
    // and a cube behind the card doesn't
    app.world_mut().entity_mut(blocker).insert(GlobalTransform::from_translation(Vec3::new(1.5, 1.0, -3.0)));
    assert_eq!(clicked(&mut app, &to), Some(btn));
}

/// A layer flown through the scene and clicked: the page's quad moves, turns and scales along a path, and a click
/// at the button wherever it is that frame lands on the button's twin.
#[test]
fn a_layer_flown_through_the_scene_and_clicked() {
    let (mut app, cam, to) = world(true);
    let (page, quad) = card(&mut app, cam);
    let btn = twin(&mut app, page, "#btn");
    for i in 0..6 {
        let t = i as f32 / 5.0;
        let at = Transform::from_translation(Vec3::new(-4.0 + 8.0 * t, (t * 6.0).sin(), -6.0 * t)).with_rotation(Quat::from_rotation_y(0.8 * t - 0.4) * Quat::from_rotation_x(0.3 * t)).with_scale(Vec3::splat(0.5 + t));
        app.world_mut().entity_mut(quad).insert(GlobalTransform::from(at));
        assert_eq!(clicked(&mut app, &to), Some(btn), "at step {i}");
    }
}

/// The frame's order (H8): a host page's step runs in PreUpdate's [`PageSystems::Step`], after the pages' events, so
/// what its script commits from a host event has its twin for the game systems in Update that frame, and keeps it.
#[test]
fn a_host_pages_commit_has_its_twins_for_that_frames_game_systems() {
    #[derive(Resource, Default)]
    struct Seen(Vec<Option<Entity>>);
    #[derive(Resource)]
    struct ThePage(Entity);
    let (mut app, _, _) = world(false);
    app.add_plugins(raea_runtime_dom::host::HostStepPlugin).init_resource::<Seen>();
    let js = r#"
        const flush = (ops) => __ui_flush(JSON.stringify({ v: 1, ops }));
        globalThis.__ui_deliver = (json) => {
          const m = JSON.parse(json);
          if (m.t === "named" && m.name === "make")
            flush([{ op: "create", id: 2, kind: "div", props: { id: "made" } }, { op: "append", parent: 1, child: 2 }]);
        };
        flush([{ op: "create", id: 1, kind: "div", props: { id: "root" } }, { op: "append", parent: 0, child: 1 }]);
    "#;
    let page = raea_runtime_dom::host::mount(app.world_mut(), 400, 300, js, &[], None).unwrap();
    app.insert_resource(ThePage(page)).add_systems(Update, |q: PageQuery, page: Res<ThePage>, mut seen: ResMut<Seen>| seen.0.push(q.by_id(page.0, "made")));
    app.update();
    assert_eq!(app.world().resource::<Seen>().0, vec![None]);
    raea_runtime_dom::host::post(app.world_mut(), "make", serde_json::Value::Null);
    app.update();
    app.update();
    let seen = app.world().resource::<Seen>().0.clone();
    assert!(seen[1].is_some(), "the game system didn't see #made's twin the frame it was made: {seen:?}");
    assert_eq!(seen[1], seen[2], "#made's twin changed the frame after: {seen:?}");
    let root = twin(&mut app, page, "#root");
    assert_eq!(app.world().get::<ChildOf>(seen[1].unwrap()).map(|c| c.parent()), Some(root));
}
