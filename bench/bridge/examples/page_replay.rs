//! B3 H10's page half (docs/blitz-bevy.feature; bench/bridge/run.sh --pixels): a page in a plain App played live with
//! a mouse, its input logged as `raea play --input-log` logs it, then rendered from that log, the same pixels.
//!
//!   page_replay play LOG DIR     the mouse scripted into Bevy's PointerInput as a window's would be, the pointer
//!                                recorded each frame (raea_runtime_input::replay::InputRecord) to LOG, frames to DIR/play-F.png
//!   page_replay render LOG DIR   the pointer from LOG (InputReplay, before picking), frames to DIR/render-F.png, each
//!                                compared with play's; PASS when every one is the same bytes, the button showed
//!                                its :hover, :active and plain looks at the frames it should, and the field the keys
//!                                typed into it after a click
use bevy::camera::{NormalizedRenderTarget, RenderTarget};
use bevy::picking::PickingSystems;
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKeyCode};
use bevy::window::{ExitCondition, WindowEvent};
use raea_runtime_input::pick::{PageView, PickPlugin};
use raea_runtime_input::replay::{InputFrame, InputRecord, InputReplay, ReplayPlugin};
use std::collections::BTreeMap;

const W: u32 = 400;
const H: u32 = 300;
const FPS: f64 = 30.0;
const FRAMES: i64 = 40;
/// the frames shot, each with the button's colour then: hovered, pressed, released over it, the pointer gone off it,
/// clicked into the field, and after "hi" was typed into it
const SHOTS: [(i64, [u8; 3]); 6] = [(8, [0, 160, 0]), (12, [200, 0, 0]), (16, [0, 160, 0]), (22, [0, 0, 200]), (27, [0, 0, 200]), (36, [0, 0, 200])];
/// the field's box, [x0, y0, x1, y1]
const FIELD: [u32; 4] = [20, 100, 220, 130];
const CSS: &str = "#b { background: rgb(0, 0, 200); transition: none } #b:hover { background: rgb(0, 160, 0) } #b:active { background: rgb(200, 0, 0) }";

#[derive(Resource, Default)]
struct Shots(BTreeMap<i64, Image>);

#[derive(Resource)]
struct Counter(i64);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(mode), Some(log), Some(dir)) = (args.get(1), args.get(2), args.get(3)) else {
        eprintln!("usage: page_replay play|render LOG DIR");
        std::process::exit(2);
    };
    let play = mode == "play";
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: true, ..default() })
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
    )
    .add_plugins((raea_page::PagePlugin, PickPlugin, raea_runtime_input::keys::KeysPlugin, ReplayPlugin))
    .init_resource::<Shots>()
    .insert_resource(Counter(0))
    // the app's frame: one each update from 0, the log's index and the page's clock
    .add_systems(First, frame.before(PickingSystems::PostInput));
    let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(W, H, TextureFormat::Rgba8UnormSrgb, None));
    let target = RenderTarget::Image(image.clone().into());
    let to = target.normalize(None).expect("the image's target");
    let fonts = raea_sys_fonts::context(&raea_io_home::home().join("core/fonts"));
    let (page, mut nodes) = raea_sys_page::open_doc(app.world_mut(), W, H, Some(fonts));
    let ops: Vec<raea_core_wire::Op> = serde_json::from_value(serde_json::json!([
        { "op": "create", "id": 1, "kind": "div", "props": { "style": { "width": "400px", "height": "300px", "background": "rgb(255, 255, 255)" } } },
        { "op": "create", "id": 2, "kind": "div", "props": { "id": "b", "style": { "position": "absolute", "left": "20px", "top": "20px", "width": "120px", "height": "40px" } } },
        { "op": "create", "id": 3, "kind": "style", "props": {}, "text": CSS },
        { "op": "create", "id": 4, "kind": "input", "props": { "id": "field", "style": { "position": "absolute", "left": "20px", "top": "100px", "width": "200px", "height": "30px", "fontSize": "20px", "background": "rgb(255, 255, 255)", "color": "rgb(0, 0, 0)", "border": "0px" } } },
        { "op": "append", "parent": 1, "child": 3 },
        { "op": "append", "parent": 1, "child": 2 },
        { "op": "append", "parent": 1, "child": 4 },
        { "op": "append", "parent": 0, "child": 1 },
    ]))
    .unwrap();
    raea_sys_page::ops::apply(app.world_mut().non_send_mut::<raea_sys_page::Docs>().0.get_mut(&page).unwrap(), &mut nodes, ops);
    let cam = app.world_mut().spawn((Camera2d, Camera { clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() }, target, raea_runtime_twins::overlay::PageOverlay { page })).id();
    app.world_mut().spawn((PageView::new(page), Node { position_type: PositionType::Absolute, width: px(W), height: px(H), ..default() }, UiTargetCamera(cam), bevy::picking::Pickable::IGNORE));
    if play {
        app.insert_resource(InputRecord { target: to.clone(), log: Default::default() });
    } else {
        let text = std::fs::read_to_string(log).unwrap_or_else(|e| fail(&format!("{log}: {e}")));
        let log = raea_runtime_input::replay::parse(&text).unwrap_or_else(|e| fail(&e));
        app.insert_resource(InputReplay::new(to.clone(), log));
    }
    app.finish();
    app.cleanup();
    for f in 0..FRAMES {
        if play {
            mouse(&mut app, &to, f);
        }
        if SHOTS.iter().any(|s| s.0 == f) {
            app.world_mut().spawn(Screenshot::image(image.clone())).observe(move |e: On<ScreenshotCaptured>, mut shots: ResMut<Shots>| {
                shots.0.insert(f, e.image.clone());
            });
        }
        app.update();
    }
    for _ in 0..60 {
        if app.world().resource::<Shots>().0.len() == SHOTS.len() {
            break;
        }
        app.update();
    }
    let shots = std::mem::take(&mut app.world_mut().resource_mut::<Shots>().0);
    if shots.len() != SHOTS.len() {
        fail(&format!("{} of {} frames captured", shots.len(), SHOTS.len()));
    }
    let name = if play { "play" } else { "render" };
    for (f, shot) in &shots {
        shot.clone().try_into_dynamic().unwrap().to_rgba8().save(format!("{dir}/{name}-{f}.png")).unwrap_or_else(|e| fail(&format!("{dir}: {e}")));
    }
    if play {
        let log_json = raea_runtime_input::replay::to_json(&app.world().resource::<InputRecord>().log);
        std::fs::write(log, log_json).unwrap_or_else(|e| fail(&format!("{log}: {e}")));
        println!("page_replay: played {FRAMES} frames, the log in {log}");
        return;
    }
    let mut ok = true;
    for (f, want) in SHOTS {
        let shot = &shots[&f];
        let got = shot.get_color_at(80, 40).unwrap().to_srgba();
        let got = [(got.red * 255.0).round() as u8, (got.green * 255.0).round() as u8, (got.blue * 255.0).round() as u8];
        let looks = got.iter().zip(want).all(|(g, w)| (*g as i32 - w as i32).abs() <= 2);
        let played = image::open(format!("{dir}/play-{f}.png")).map(|i| i.to_rgba8());
        let rendered = shot.clone().try_into_dynamic().unwrap().to_rgba8();
        let differ = match &played {
            Ok(p) if p.dimensions() == rendered.dimensions() => p.pixels().zip(rendered.pixels()).filter(|(a, b)| a != b).count(),
            _ => usize::MAX,
        };
        ok &= looks && differ == 0;
        if f == 36 {
            // "hi" in the field: its box changed from frame 27's, focused and empty
            let before = shots[&27].clone().try_into_dynamic().unwrap().to_rgba8();
            let typed = (FIELD[1]..FIELD[3]).flat_map(|y| (FIELD[0]..FIELD[2]).map(move |x| (x, y))).filter(|(x, y)| before.get_pixel(*x, *y) != rendered.get_pixel(*x, *y)).count();
            ok &= typed > 20;
            println!("page_replay: frame {f}: the field typed into ({typed} px changed from frame 27){}", if typed > 20 { "" } else { "  <- wrong" });
        }
        println!("page_replay: frame {f}: the button {got:?} (want {want:?}), {} pixels differ from the play's{}", if differ == usize::MAX { "all".into() } else { differ.to_string() }, if looks && differ == 0 { "" } else { "  <- wrong" });
    }
    println!("page_replay: {}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}

fn frame(mut c: ResMut<Counter>, mut f: ResMut<InputFrame>, mut clock: ResMut<raea_sys_page::PageClock>) {
    f.0 = c.0;
    clock.0 = Some(c.0 as f64 / FPS);
    c.0 += 1;
}

/// The live mouse and keys: onto the page, over the button, pressed and held, released, off it; then a click into the
/// field and "hi" typed, each key's press and release as a window sends them.
fn mouse(app: &mut App, to: &NormalizedRenderTarget, f: i64) {
    let action = |x: f32, y: f32, a: PointerAction| PointerInput::new(PointerId::Mouse, Location { target: to.clone(), position: Vec2::new(x, y) }, a);
    let input = match f {
        3 => action(300.0, 250.0, PointerAction::Move { delta: Vec2::ZERO }),
        6 => action(80.0, 40.0, PointerAction::Move { delta: Vec2::new(-220.0, -210.0) }),
        10 => action(80.0, 40.0, PointerAction::Press(PointerButton::Primary)),
        14 => action(80.0, 40.0, PointerAction::Release(PointerButton::Primary)),
        18 => action(300.0, 250.0, PointerAction::Move { delta: Vec2::new(220.0, 210.0) }),
        24 => action(100.0, 115.0, PointerAction::Move { delta: Vec2::new(-200.0, -135.0) }),
        25 => action(100.0, 115.0, PointerAction::Press(PointerButton::Primary)),
        26 => action(100.0, 115.0, PointerAction::Release(PointerButton::Primary)),
        28..=31 => {
            let c = if f < 30 { "h" } else { "i" };
            let state = if f % 2 == 0 { ButtonState::Pressed } else { ButtonState::Released };
            let k = KeyboardInput { key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified), logical_key: Key::Character(c.into()), state, text: (state == ButtonState::Pressed).then(|| c.into()), repeat: false, window: Entity::PLACEHOLDER };
            app.world_mut().write_message(k.clone());
            app.world_mut().write_message(WindowEvent::KeyboardInput(k));
            return;
        }
        _ => return,
    };
    app.world_mut().write_message(input);
}

fn fail(e: &str) -> ! {
    eprintln!("page_replay: {e}");
    println!("page_replay: FAIL");
    std::process::exit(1);
}
