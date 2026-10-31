//! B3 H7 from TSX (docs/sdk.feature S4a; bench/bridge/run.sh --pixels): bench/bridge/canvas.tsx has a layered
//! <Canvas3D> inside a positioned 200 x 150 box at (100, 50) on a red page. Its 3D is drawn at the box (world.rs gives
//! the slot's image to SizedImages, which sizes it to the box): green where the 3D draws nothing, the blue cube in the
//! middle and square on screen, red outside.
//!
//!   raea bundle bench/bridge/canvas.tsx --composition-bundle -o canvas.js
//!   cargo run --release -p raea-bridge-bench --example canvas_box -- canvas.js [OUT.png]
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::ExitCondition;
use raea_runtime_react::scene::ScenePlugin;
use raea_runtime_page::WorldPagePlugin;
use raea_runtime_react::ReactPlugin;
use raea_render::engine::raea_plugins::{RaeaPlugins, Scene3dPlugin};
use std::path::PathBuf;

const W: u32 = 640;
const H: u32 = 360;

#[derive(Resource, Default)]
struct Shot(Option<Image>);

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(js) = args.next() else {
        eprintln!("usage: canvas_box CANVAS.js [OUT.png]");
        std::process::exit(2);
    };
    let out = args.next();
    let js = PathBuf::from(js).canonicalize().expect("the bundle");
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: true, ..default() })
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
    );
    let target = app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(W, H, TextureFormat::Rgba8UnormSrgb, None));
    let dir = js.parent().unwrap().to_path_buf();
    app.add_plugins((RaeaPlugins, Scene3dPlugin, ReactPlugin::new(js.clone()), ScenePlugin { target: RenderTarget::Image(target.clone().into()), size: (W, H), dir: dir.clone(), frame: Some(0) }))
        .add_plugins((raea_page::PagePlugin, WorldPagePlugin { size: (W, H), dir, fps: 30.0 }))
        .init_resource::<Shot>();
    app.finish();
    app.cleanup();
    // React commits on its own thread: update until the page is over the scene's camera, then a few frames more
    let mut seen = None;
    for i in 0..600 {
        app.update();
        let shown = app.world_mut().query_filtered::<(), (With<Camera>, With<raea_runtime_twins::overlay::PageOverlay>)>().iter(app.world()).count() > 0;
        if shown && seen.is_none() {
            seen = Some(i);
        }
        if seen.is_some_and(|s| i >= s + 8) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    if seen.is_none() {
        eprintln!("canvas_box: the page was never shown");
        std::process::exit(1);
    }
    app.world_mut().spawn(Screenshot::image(target)).observe(|e: On<ScreenshotCaptured>, mut shot: ResMut<Shot>| shot.0 = Some(e.image.clone()));
    for _ in 0..60 {
        app.update();
        if app.world().resource::<Shot>().0.is_some() {
            break;
        }
    }
    let Some(shot) = app.world_mut().resource_mut::<Shot>().0.take() else {
        eprintln!("canvas_box: no frame");
        std::process::exit(1);
    };
    if let Some(out) = &out {
        shot.clone().try_into_dynamic().unwrap().to_rgb8().save(out).unwrap();
    }
    let px = |x: u32, y: u32| -> [u8; 3] {
        let c = shot.get_color_at(x, y).unwrap().to_srgba();
        [(c.red * 255.0).round() as u8, (c.green * 255.0).round() as u8, (c.blue * 255.0).round() as u8]
    };
    let checks = [
        ("the box where the 3D draws nothing, its background", px(110, 60), [0, 255, 0]),
        // the unlit blue cube as the camera's tonemapping leaves it
        ("the box's middle, the cube", px(200, 125), [0, 54, 212]),
        ("the page outside it", px(500, 300), [255, 0, 0]),
    ];
    let mut ok = true;
    for (what, got, want) in checks {
        let near = got.iter().zip(want).all(|(g, w)| (*g as i32 - w as i32).abs() <= 8);
        ok &= near;
        println!("canvas_box: {what}: {got:?} (want {want:?}){}", if near { "" } else { "  <- wrong" });
    }
    // the cube's extent across and down through the box's middle: square when the 3D is drawn at the box's aspect
    let blue = |x: u32, y: u32| { let p = px(x, y); p[2] > 200 && p[0] < 60 && p[1] < 60 };
    let across = (100..300).filter(|&x| blue(x, 125)).count() as f32;
    let down = (50..200).filter(|&y| blue(200, y)).count() as f32;
    let square = down > 0.0 && (across / down - 1.0).abs() <= 0.05;
    ok &= square;
    println!("canvas_box: the cube {across} px across and {down} down (want square){}", if square { "" } else { "  <- wrong" });
    println!("canvas_box: {}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}
