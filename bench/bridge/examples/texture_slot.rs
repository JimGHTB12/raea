//! B3 H5's texture elements in a plain Bevy App (docs/blitz-bevy.feature; bench/bridge/run.sh --pixels): the
//! composition (bench/bridge/texture.tsx) has `<img data-raea-texture={7}>` in a 200 x 150 box at (100, 50) on a red
//! page; the App renders a camera of its own (a green clear, a blue cube) into an image it lists as PageTextures 7.
//! The frame is read inside the box (green, and blue at the cube) and outside it (red).
//!
//!   raea bundle bench/bridge/texture.tsx --composition-bundle -o texture.js
//!   cargo run --release -p raea-bridge-bench --example texture_slot -- texture.js [OUT.png]
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::ExitCondition;
use raea_runtime_react::scene::ScenePlugin;
use raea_runtime_page::{PageTextures, WorldPagePlugin};
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
        eprintln!("usage: texture_slot TEXTURE.js [OUT.png]");
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
    // the App's own camera, on a render layer of its own (the composition's 3D doesn't see its cube), into texture 7
    let view = raea_sys_image::offscreen::target(&mut app.world_mut().resource_mut::<Assets<Image>>(), 200, 150);
    let layer = RenderLayers::layer(9);
    app.world_mut().spawn((Camera3d::default(), Camera { order: -5, clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 1.0, 0.0)), ..default() }, RenderTarget::Image(view.clone().into()), bevy::core_pipeline::tonemapping::Tonemapping::None, Transform::from_xyz(0.0, 0.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y), layer.clone()));
    let cube = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
    let blue = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial { base_color: Color::srgb(0.0, 0.0, 1.0), unlit: true, ..default() });
    app.world_mut().spawn((Mesh3d(cube), MeshMaterial3d(blue), layer));
    app.world_mut().resource_mut::<PageTextures>().0.insert(7, view);
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
        eprintln!("texture_slot: the page was never shown");
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
        eprintln!("texture_slot: no frame");
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
        ("the texture element, the camera's clear", px(110, 60), [0, 255, 0]),
        ("the texture element's middle, the cube", px(200, 125), [0, 0, 255]),
        ("the page outside it", px(500, 300), [255, 0, 0]),
    ];
    let mut ok = true;
    for (what, got, want) in checks {
        let near = got.iter().zip(want).all(|(g, w)| (*g as i32 - w as i32).abs() <= 8);
        ok &= near;
        println!("texture_slot: {what}: {got:?} (want {want:?}){}", if near { "" } else { "  <- wrong" });
    }
    println!("texture_slot: {}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}
