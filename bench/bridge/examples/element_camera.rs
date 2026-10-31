//! B3 H7 in a plain Bevy App (docs/blitz-bevy.feature; bench/bridge/run.sh --pixels): an element showing a camera's
//! texture, GPU to GPU (raea-page's ElementImages). The page is red with a 200 x 150 box at (100, 50); a camera of its
//! own renders a green-cleared scene with a blue cube into an image the box shows. The frame is read inside the box
//! (green, and blue at the cube) and outside it (red). Then the box widens to 300 x 150, and the camera's image with it
//! (SizedImages: the element's size sets the camera target's), so the cube stays square.
//!
//!   cargo run --release -p raea-bridge-bench --example element_camera -- [OUT.png]
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::ExitCondition;

const W: u32 = 640;
const H: u32 = 360;

#[derive(Resource, Default)]
struct Shot(Option<Image>);

fn main() {
    let out = std::env::args().nth(1);
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: true, ..default() })
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
    )
    .add_plugins(raea_page::PagePlugin)
    .init_resource::<Shot>();
    let target = app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(W, H, TextureFormat::Rgba8UnormSrgb, None));
    let (page, mut nodes) = raea_sys_page::open_doc(app.world_mut(), W, H, None);
    let ops: Vec<raea_core_wire::Op> = serde_json::from_value(serde_json::json!([
        { "op": "create", "id": 1, "kind": "div", "props": { "style": { "width": "640px", "height": "360px", "background": "rgb(255, 0, 0)", "position": "relative" } } },
        { "op": "create", "id": 2, "kind": "img", "props": { "style": { "position": "absolute", "left": "100px", "top": "50px", "width": "200px", "height": "150px" } } },
        { "op": "append", "parent": 1, "child": 2 },
        { "op": "append", "parent": 0, "child": 1 },
    ]))
    .unwrap();
    raea_sys_page::ops::apply(app.world_mut().non_send_mut::<raea_sys_page::Docs>().0.get_mut(&page).unwrap(), &mut nodes, ops);
    // the camera the box shows: its own image, a green clear and a blue cube in the middle
    let view = raea_sys_image::offscreen::target(&mut app.world_mut().resource_mut::<Assets<Image>>(), 200, 150);
    app.world_mut().spawn((Camera3d::default(), Camera { order: -1, clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 1.0, 0.0)), ..default() }, RenderTarget::Image(view.clone().into()), bevy::core_pipeline::tonemapping::Tonemapping::None, Transform::from_xyz(0.0, 0.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y)));
    let cube = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)));
    let blue = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial { base_color: Color::srgb(0.0, 0.0, 1.0), unlit: true, ..default() });
    app.world_mut().spawn((Mesh3d(cube), MeshMaterial3d(blue)));
    let n = raea_sys_page::ops::node(&nodes, 2).unwrap();
    app.world_mut().resource_mut::<raea_sys_page::view3d::ElementImages>().0.insert((page, n), view.clone());
    // the page over the target, on a camera of its own
    app.world_mut().spawn((Camera2d, Camera { order: 1, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() }, RenderTarget::Image(target.clone().into()), raea_runtime_twins::overlay::PageOverlay { page }));
    // the box's size sets the camera's: its image follows the element's content box
    app.world_mut().resource_mut::<raea_sys_page::view3d::SizedImages>().0.insert((page, n));
    app.finish();
    app.cleanup();
    let Some(shot) = shoot(&mut app, &target) else {
        eprintln!("element_camera: no frame");
        std::process::exit(1);
    };
    if let Some(out) = &out {
        shot.clone().try_into_dynamic().unwrap().to_rgb8().save(out).unwrap();
    }
    // the box widened to 300 x 150: the camera draws 300 x 150, so the cube stays square (about 52 px across, centred
    // at (250, 125)); a 200 x 150 picture stretched to the box would draw it 78 px wide, blue at 32 px right of centre
    let widen: Vec<raea_core_wire::Op> = serde_json::from_value(serde_json::json!([
        { "op": "update", "id": 2, "set": { "style": { "position": "absolute", "left": "100px", "top": "50px", "width": "300px", "height": "150px" } } },
    ]))
    .unwrap();
    raea_sys_page::ops::apply(app.world_mut().non_send_mut::<raea_sys_page::Docs>().0.get_mut(&page).unwrap(), &mut nodes, widen);
    let Some(wide) = shoot(&mut app, &target) else {
        eprintln!("element_camera: no frame after the resize");
        std::process::exit(1);
    };
    let size = app.world().resource::<Assets<Image>>().get(&view).map(Image::size);
    let checks = [
        ("the box, the camera's clear", px(&shot, 110, 60), [0, 255, 0]),
        ("the box's middle, the cube", px(&shot, 200, 125), [0, 0, 255]),
        ("the page outside the box", px(&shot, 500, 300), [255, 0, 0]),
        ("the widened box's right end, the clear", px(&wide, 390, 60), [0, 255, 0]),
        ("the widened box's middle, the cube", px(&wide, 250, 125), [0, 0, 255]),
        ("32 px right of the cube's middle, the clear (square, not stretched)", px(&wide, 282, 125), [0, 255, 0]),
    ];
    let mut ok = size == Some(UVec2::new(300, 150));
    println!("element_camera: the camera's image after the resize: {size:?} (want 300 x 150){}", if ok { "" } else { "  <- wrong" });
    for (what, got, want) in checks {
        let near = got.iter().zip(want).all(|(g, w)| (*g as i32 - w as i32).abs() <= 8);
        ok &= near;
        println!("element_camera: {what}: {got:?} (want {want:?}){}", if near { "" } else { "  <- wrong" });
    }
    println!("element_camera: {}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}

/// 30 frames run, then the target's frame read back
fn shoot(app: &mut App, target: &Handle<Image>) -> Option<Image> {
    for _ in 0..30 {
        app.update();
    }
    app.world_mut().spawn(Screenshot::image(target.clone())).observe(|e: On<ScreenshotCaptured>, mut shot: ResMut<Shot>| shot.0 = Some(e.image.clone()));
    for _ in 0..60 {
        app.update();
        if app.world().resource::<Shot>().0.is_some() {
            break;
        }
    }
    app.world_mut().resource_mut::<Shot>().0.take()
}

fn px(shot: &Image, x: u32, y: u32) -> [u8; 3] {
    let c = shot.get_color_at(x, y).unwrap().to_srgba();
    [(c.red * 255.0).round() as u8, (c.green * 255.0).round() as u8, (c.blue * 255.0).round() as u8]
}
