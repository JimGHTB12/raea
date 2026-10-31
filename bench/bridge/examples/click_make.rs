//! B3 H3's live check (docs/blitz-bevy.feature; bench/bridge/run.sh --pixels): a host page in a plain App's live
//! play, clicked through Bevy's picking. The page's script makes `#made` when its button is clicked (on its
//! pointerup, as sdk/ui makes onClick); a Bevy observer on `dom::Click` finds `#made`'s twin in that click's own
//! frame, and a second click, on `#made`, reaches that twin.
//!
//!   cargo run --release -p raea-bridge-bench --example click_make
use bevy::camera::{NormalizedRenderTarget, RenderTarget};
use bevy::ecs::system::SystemState;
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::ExitCondition;
use raea_runtime_twins::bridge::dom;
use raea_runtime_input::pick::{PageView, PickPlugin};
use raea_sys_page::PageImage; use raea_runtime_twins::PageQuery;

const W: u32 = 400;
const H: u32 = 300;

/// The page's script: the wire's ops by hand (sdk/ui's wire.ts sends the same), its button heard on pointerup.
const BUNDLE: &str = r#"
const flush = (ops) => __ui_flush(JSON.stringify({ v: 1, ops }));
let made = 0;
globalThis.__ui_deliver = (json) => {
  const m = JSON.parse(json);
  if (m.t !== "event" || m.id !== 2 || m.event !== "pointerup" || made) return;
  made = 3;
  flush([
    { op: "create", id: 3, kind: "div", props: { id: "made", style: { position: "absolute", left: "0px", top: "200px", width: "120px", height: "40px", background: "rgb(0, 160, 0)" } } },
    { op: "append", parent: 1, child: 3 },
  ]);
};
flush([
  { op: "create", id: 1, kind: "div", props: { style: { position: "relative", width: "400px", height: "300px", background: "rgb(255, 255, 255)" } } },
  { op: "create", id: 2, kind: "div", props: { id: "go", style: { width: "120px", height: "40px", background: "rgb(0, 0, 200)" } } },
  { op: "listen", id: 2, events: ["pointerup"] },
  { op: "append", parent: 1, child: 2 },
  { op: "append", parent: 0, child: 1 },
]);
"#;

#[derive(Resource)]
struct ThePage(Entity);

/// Each click a twin heard first (its target): the target, and `#made`'s twin as the observer found it.
#[derive(Resource, Default)]
struct Heard(Vec<(Entity, Option<Entity>)>);

#[derive(Component)]
struct Shows;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
    )
    .add_plugins((raea_page::PagePlugin, raea_runtime_dom::host::HostPlugin, raea_runtime_dom::host::HostStepPlugin, PickPlugin))
    .init_resource::<Heard>()
    .add_systems(Update, show)
    .add_observer(|e: On<dom::Click>, page: Res<ThePage>, q: PageQuery, mut heard: ResMut<Heard>| {
        if e.entity == e.target {
            heard.0.push((e.target, q.by_id(page.0, "made")));
        }
    });
    let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(W, H, TextureFormat::Rgba8UnormSrgb, None));
    let target = RenderTarget::Image(image.into());
    let to = target.normalize(None).expect("the image's target");
    let cam = app.world_mut().spawn((Camera2d, target)).id();
    let page = raea_runtime_dom::host::mount(app.world_mut(), W, H, BUNDLE, &[], None).unwrap_or_else(|e| {
        eprintln!("click_make: mount: {e}");
        std::process::exit(1);
    });
    app.insert_resource(ThePage(page));
    app.world_mut().spawn((Shows, PageView::new(page), Node { position_type: PositionType::Absolute, width: px(W), height: px(H), ..default() }, UiTargetCamera(cam), bevy::picking::Pickable::IGNORE));
    app.finish();
    app.cleanup();
    for _ in 0..10 {
        app.update();
    }
    let twin = |app: &mut App, id: &str| {
        let mut q: SystemState<PageQuery> = SystemState::new(app.world_mut());
        q.get(app.world()).ok().and_then(|q| q.by_id(page, id))
    };
    let go = twin(&mut app, "go");
    let before = twin(&mut app, "made");
    click(&mut app, &to, 60.0, 20.0);
    let first = std::mem::take(&mut app.world_mut().resource_mut::<Heard>().0);
    let made = twin(&mut app, "made");
    click(&mut app, &to, 60.0, 220.0);
    let second = std::mem::take(&mut app.world_mut().resource_mut::<Heard>().0);

    let checks = [
        ("no #made before the click", go.is_some() && before.is_none()),
        ("the click reached #go's twin, and #made's twin was there in that event", go.is_some() && made.is_some() && first == vec![(go.unwrap(), made)]),
        ("a click on #made reached its twin", made.is_some() && second.first().map(|h| h.0) == made),
    ];
    let mut ok = true;
    for (what, good) in checks {
        ok &= good;
        println!("click_make: {what}{}", if good { "" } else { "  <- wrong" });
    }
    if !ok {
        println!("click_make: #go {go:?}, #made {made:?}; first click heard {first:?}, second {second:?}");
    }
    println!("click_make: {}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}

/// A press and release at page (x, y), each its own frame, as a mouse's are.
fn click(app: &mut App, to: &NormalizedRenderTarget, x: f32, y: f32) {
    for action in [PointerAction::Move { delta: Vec2::ZERO }, PointerAction::Press(PointerButton::Primary), PointerAction::Release(PointerButton::Primary)] {
        app.world_mut().write_message(PointerInput::new(PointerId::Mouse, Location { target: to.clone(), position: Vec2::new(x, y) }, action));
        app.update();
    }
    app.update();
}

/// The page's image on the node that shows it, once raea-page made it.
fn show(images: Query<&PageImage>, page: Res<ThePage>, shows: Query<Entity, (With<Shows>, Without<ImageNode>)>, mut commands: Commands) {
    let Ok(img) = images.get(page.0) else { return };
    for e in &shows {
        commands.entity(e).insert(ImageNode::new(img.0.clone()));
    }
}
