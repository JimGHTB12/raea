use super::*;
use raea_runtime_twins::bridge::{BridgePlugin, PageQuery, dom};
use raea_sys_page::PageSystems;
use bevy::camera::{NormalizedRenderTarget, RenderTarget};
use bevy::ecs::system::RunSystemOnce;
use bevy::picking::events::{PointerClick, PointerDrag, PointerDragDrop, PointerDragStart, PointerPress};
use bevy::reflect::structs::Struct;
use bevy::window::WindowRef;

#[derive(Resource, Default)]
struct Heard(Vec<(&'static str, Entity)>);

/// A window, a camera on it and an HTML page shown on a 400x300 UI box at the window's top left; Bevy's picking
/// with the page's backend and route. The pointer input is written as picking's own input plugin writes it.
fn app(html: &str) -> (App, Entity, NormalizedRenderTarget) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::input::InputPlugin, bevy::picking::PickingPlugin, bevy::picking::InteractionPlugin, BridgePlugin::<()>::default(), PickPlugin))
        .init_non_send::<Docs>()
        .init_resource::<raea_sys_page::PageClock>()
        .init_resource::<Heard>()
        .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    let window = app.world_mut().spawn((Window::default(), PrimaryWindow)).id();
    let cam = app.world_mut().spawn((Camera::default(), RenderTarget::Window(WindowRef::Primary))).id();
    let at = Page { size: UVec2::new(400, 300), scale: 1.0 };
    let page = app.world_mut().spawn(at).id();
    let doc = blitz_html::HtmlDocument::from_html(html, raea_sys_page::config(at)).into_inner();
    app.world_mut().non_send_mut::<Docs>().0.insert(page, doc);
    let mut target = ComputedUiTargetCamera::default();
    target.field_mut("camera").unwrap().apply(&Some(cam));
    app.world_mut().spawn((PageView::new(page), ComputedNode { size: Vec2::new(400.0, 300.0), ..default() }, UiGlobalTransform::from_translation(Vec2::new(200.0, 150.0)), target));
    app.world_mut().spawn(PointerId::Mouse);
    app.update();
    let to = NormalizedRenderTarget::Window(WindowRef::Primary.normalize(Some(window)).unwrap());
    (app, page, to)
}

fn input(app: &mut App, to: &NormalizedRenderTarget, x: f32, y: f32, action: PointerAction) {
    app.world_mut().write_message(PointerInput::new(PointerId::Mouse, Location { target: to.clone(), position: Vec2::new(x, y) }, action));
    app.update();
}

fn twin(app: &mut App, page: Entity, sel: &'static str) -> Entity {
    app.world_mut().run_system_once(move |q: PageQuery| q.one(page, sel)).unwrap().unwrap()
}

#[test]
fn bevy_picking_events_reach_twins_and_the_page_hears_the_press() {
    let (mut app, page, to) = app(r#"<body style="margin:0"><div id="card" style="width:100px;height:50px"><span id="label" style="display:block;width:50px;height:20px">x</span></div><div id="slot" style="width:100px;height:50px"></div></body>"#);
    let (card, label, slot) = (twin(&mut app, page, "#card"), twin(&mut app, page, "#label"), twin(&mut app, page, "#slot"));
    // twins are the elements' hierarchy: Bevy's bubbling runs the DOM's path
    assert_eq!(app.world().get::<ChildOf>(label).map(|c| c.parent()), Some(card));
    app.add_observer(|e: On<PointerPress>, mut h: ResMut<Heard>| h.0.push(("press", e.entity)));
    app.add_observer(|e: On<PointerClick>, mut h: ResMut<Heard>| h.0.push(("click", e.entity)));
    app.add_observer(|e: On<PointerDragStart>, mut h: ResMut<Heard>| h.0.push(("dragstart", e.entity)));
    app.add_observer(|e: On<PointerDrag>, mut h: ResMut<Heard>| h.0.push(("drag", e.entity)));
    app.add_observer(|e: On<PointerDragDrop>, mut h: ResMut<Heard>| h.0.push(("drop", e.entity)));
    app.add_observer(|e: On<dom::PointerDown>, mut h: ResMut<Heard>| h.0.push(("dom-down", e.entity)));
    app.add_observer(|e: On<dom::PointerUp>, mut h: ResMut<Heard>| h.0.push(("dom-up", e.entity)));

    // a click on the label: Bevy's press and click on its twin, bubbled to the card's; the page heard it too
    input(&mut app, &to, 10.0, 10.0, PointerAction::Move { delta: Vec2::ZERO });
    input(&mut app, &to, 10.0, 10.0, PointerAction::Press(PointerButton::Primary));
    input(&mut app, &to, 10.0, 10.0, PointerAction::Release(PointerButton::Primary));
    let heard = std::mem::take(&mut app.world_mut().resource_mut::<Heard>().0);
    for want in [("press", label), ("press", card), ("click", label), ("click", card), ("dom-down", label), ("dom-up", label)] {
        assert!(heard.contains(&want), "{want:?} not in {heard:?}");
    }
    // the page's own events came first (the route runs before picking's pointer events)
    let first = |k| heard.iter().position(|h| h.0 == k).unwrap();
    assert!(first("dom-down") < first("press"), "{heard:?}");

    // a drag from the card to the slot: Bevy's drag stays on the card (pointer capture), the drop lands on the
    // slot; the page's pointerup goes to the page it started on
    input(&mut app, &to, 80.0, 40.0, PointerAction::Move { delta: Vec2::ZERO });
    input(&mut app, &to, 80.0, 40.0, PointerAction::Press(PointerButton::Primary));
    input(&mut app, &to, 80.0, 70.0, PointerAction::Move { delta: Vec2::new(0.0, 30.0) });
    input(&mut app, &to, 80.0, 75.0, PointerAction::Move { delta: Vec2::new(0.0, 5.0) });
    input(&mut app, &to, 80.0, 75.0, PointerAction::Release(PointerButton::Primary));
    let heard = &app.world().resource::<Heard>().0;
    for want in [("dragstart", card), ("drag", card), ("drop", slot), ("dom-up", slot)] {
        assert!(heard.contains(&want), "{want:?} not in {heard:?}");
    }
    assert!(!heard.contains(&("drag", slot)), "{heard:?}");
}

#[test]
fn a_held_pointer_stays_on_its_page_past_its_edge() {
    let (mut app, page, to) = app(r#"<body style="margin:0"><div id="a" style="width:400px;height:300px"></div></body>"#);
    input(&mut app, &to, 390.0, 10.0, PointerAction::Move { delta: Vec2::ZERO });
    input(&mut app, &to, 390.0, 10.0, PointerAction::Press(PointerButton::Primary));
    // off the page's box: the press holds the pointer on the page, at a point past its edge
    input(&mut app, &to, 450.0, 10.0, PointerAction::Move { delta: Vec2::new(60.0, 0.0) });
    let r = &app.world().resource::<Routes>().0[&PointerId::Mouse];
    assert_eq!(r.held, Some(page));
    assert!(r.at.distance(Vec2::new(450.0, 10.0)) < 1e-3, "{:?}", r.at);
    input(&mut app, &to, 450.0, 10.0, PointerAction::Release(PointerButton::Primary));
    let r = &app.world().resource::<Routes>().0[&PointerId::Mouse];
    assert_eq!((r.held, r.over), (None, None));
}

#[test]
fn a_page_on_a_tilted_mesh_is_clicked_through_its_uvs() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default(), bevy::input::InputPlugin, bevy::picking::PickingPlugin, bevy::picking::InteractionPlugin, BridgePlugin::<()>::default(), PickPlugin))
        .init_asset::<Mesh>()
        .init_non_send::<Docs>()
        .init_resource::<raea_sys_page::PageClock>()
        .init_resource::<Heard>()
        .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    let window = app.world_mut().spawn((Window::default(), PrimaryWindow)).id();
    let cam = app.world_mut().spawn((Camera::default(), RenderTarget::Window(WindowRef::Primary))).id();
    let at = Page { size: UVec2::new(400, 300), scale: 1.0 };
    let page = app.world_mut().spawn(at).id();
    let html = r#"<body style="margin:0"><div id="btn" style="margin:30px 0 0 80px;width:40px;height:40px"></div></body>"#;
    app.world_mut().non_send_mut::<Docs>().0.insert(page, blitz_html::HtmlDocument::from_html(html, raea_sys_page::config(at)).into_inner());
    // a 4 x 3 screen (UV 0,0 at its top left), turned 30 degrees about Y
    let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Rectangle::new(4.0, 3.0)));
    let turn = Transform::from_rotation(Quat::from_rotation_y(30f32.to_radians()));
    app.world_mut().spawn((PageView::new(page), Mesh3d(mesh), GlobalTransform::from(turn)));
    app.world_mut().spawn(PointerId::Mouse);
    // the button's middle, page (100, 50): UV (0.25, 1/6), the screen's (-1, 1, 0), turned; a ray to it from 10 in front
    let aim = turn.transform_point(Vec3::new(-1.0, 1.0, 0.0));
    let ray = Ray3d::new(aim + Vec3::Z * 10.0, Dir3::NEG_Z);
    app.add_systems(PreUpdate, (move |mut rays: ResMut<RayMap>| {
        rays.map.insert(RayId::new(cam, PointerId::Mouse), ray);
    }).after(PickingSystems::ProcessInput).before(PickingSystems::Backend));
    app.update();
    let btn = twin(&mut app, page, "#btn");
    app.add_observer(|e: On<PointerClick>, mut h: ResMut<Heard>| h.0.push(("click", e.entity)));
    app.add_observer(|e: On<dom::PointerDown>, mut h: ResMut<Heard>| h.0.push(("dom-down", e.entity)));
    let to = NormalizedRenderTarget::Window(WindowRef::Primary.normalize(Some(window)).unwrap());
    for action in [PointerAction::Move { delta: Vec2::ZERO }, PointerAction::Press(PointerButton::Primary), PointerAction::Release(PointerButton::Primary)] {
        input(&mut app, &to, 0.0, 0.0, action);
    }
    let heard = &app.world().resource::<Heard>().0;
    assert!(heard.contains(&("click", btn)) && heard.contains(&("dom-down", btn)), "{heard:?}");
    // the hit: the page point, and the ray's distance to the screen as its depth
    let r = &app.world().resource::<Routes>().0[&PointerId::Mouse];
    assert!(r.at.distance(Vec2::new(100.0, 50.0)) < 0.01, "{:?}", r.at);
    let hit = &app.world().resource::<bevy::picking::hover::HoverMap>()[&PointerId::Mouse][&btn];
    assert!((hit.depth - 10.0).abs() < 1e-3, "{}", hit.depth);
}

#[test]
fn a_scene_in_an_element_is_picked_through_its_camera() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default(), bevy::input::InputPlugin, bevy::picking::PickingPlugin, bevy::picking::InteractionPlugin, BridgePlugin::<()>::default(), PickPlugin))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .init_non_send::<Docs>()
        .init_resource::<raea_sys_page::PageClock>()
        .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    let window = app.world_mut().spawn((Window::default(), PrimaryWindow)).id();
    let cam = app.world_mut().spawn((Camera::default(), RenderTarget::Window(WindowRef::Primary))).id();
    let at = Page { size: UVec2::new(400, 300), scale: 1.0 };
    let page = app.world_mut().spawn(at).id();
    // the slot: a 200 x 100 content box at page (100, 50), past its 4 px border
    let html = r#"<body style="margin:0"><div id="slot" style="margin:46px 0 0 96px;border:4px solid;width:200px;height:100px"></div></body>"#;
    app.world_mut().non_send_mut::<Docs>().0.insert(page, blitz_html::HtmlDocument::from_html(html, raea_sys_page::config(at)).into_inner());
    let mut target = ComputedUiTargetCamera::default();
    target.field_mut("camera").unwrap().apply(&Some(cam));
    app.world_mut().spawn((PageView::new(page), ComputedNode { size: Vec2::new(400.0, 300.0), ..default() }, UiGlobalTransform::from_translation(Vec2::new(200.0, 150.0)), target));
    // a camera drawing to an image, 10 in front of the origin, 90 degrees high at the slot's 2:1 (the view 40 x 20
    // at z = 0), and a 2 x 2 x 2 cube at (10, 0, 0): the picture's UV (0.75, 0.5)
    let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
    use bevy::camera::CameraProjection;
    let mut camera = Camera::default();
    camera.computed.clip_from_view = PerspectiveProjection { fov: std::f32::consts::FRAC_PI_2, aspect_ratio: 2.0, near: 0.1, far: 100.0, ..default() }.get_clip_from_view();
    let scene_cam = app.world_mut().spawn((camera, RenderTarget::Image(image.clone().into()), GlobalTransform::from_translation(Vec3::Z * 10.0))).id();
    let cube = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Cuboid::new(2.0, 2.0, 2.0)));
    let cube = app.world_mut().spawn((PickableMesh, Mesh3d(cube), GlobalTransform::from_translation(Vec3::X * 10.0))).id();
    app.world_mut().spawn(PointerId::Mouse);
    app.update();
    let slot = twin(&mut app, page, "#slot");
    let node = app.world().get::<PageElement>(slot).unwrap().node;
    app.world_mut().resource_mut::<ElementImages>().0.insert((page, node), image);
    let to = NormalizedRenderTarget::Window(WindowRef::Primary.normalize(Some(window)).unwrap());
    // over the cube in the picture: the cube, through the scene's camera, above the slot
    input(&mut app, &to, 250.0, 100.0, PointerAction::Move { delta: Vec2::ZERO });
    let hover = &app.world().resource::<HoverMap>()[&PointerId::Mouse];
    let hit = hover.get(&cube).unwrap_or_else(|| panic!("{hover:?}"));
    assert!(!hover.contains_key(&slot), "{hover:?}");
    assert_eq!(hit.camera, scene_cam);
    // the ray at 45 degrees from the near plane (z 9.9) to the cube's face (z 1)
    assert!((hit.depth - 8.9 * std::f32::consts::SQRT_2).abs() < 1e-3, "{}", hit.depth);
    // beside it, the picture's empty: the slot
    input(&mut app, &to, 150.0, 100.0, PointerAction::Move { delta: Vec2::ZERO });
    let hover = &app.world().resource::<HoverMap>()[&PointerId::Mouse];
    assert!(hover.contains_key(&slot) && !hover.contains_key(&cube), "{hover:?}");
}
