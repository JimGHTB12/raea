//! B3 H5's world and surface modes (docs/blitz-bevy.feature; bench/bridge/run.sh --pixels): a page drawn as a quad in
//! the scene and on a cube's faces, in a plain Bevy App, checked by its pixels. The page is red with a blue box in its
//! top left quarter; the frame is read where the camera projects those parts of each layer.
//!
//!   cargo run --release -p raea-bridge-bench --example page_layers -- [OUT.png]
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::ExitCondition;
use raea_runtime_react::layers::{PageLayer, PageLayersPlugin};

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
    .add_plugins((raea_page::PagePlugin, PageLayersPlugin))
    .init_resource::<Shot>();
    let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(W, H, TextureFormat::Rgba8UnormSrgb, None));
    let (page, mut nodes) = raea_sys_page::open_doc(app.world_mut(), 400, 300, None);
    let ops: Vec<raea_core_wire::Op> = serde_json::from_value(serde_json::json!([
        { "op": "create", "id": 1, "kind": "div", "props": { "style": { "width": "400px", "height": "300px", "background": "rgb(255, 0, 0)" } } },
        { "op": "create", "id": 2, "kind": "div", "props": { "style": { "width": "200px", "height": "150px", "background": "rgb(0, 0, 255)" } } },
        { "op": "append", "parent": 1, "child": 2 },
        { "op": "append", "parent": 0, "child": 1 },
    ]))
    .unwrap();
    raea_sys_page::ops::apply(app.world_mut().non_send_mut::<raea_sys_page::Docs>().0.get_mut(&page).unwrap(), &mut nodes, ops);
    let cam = app.world_mut().spawn((Camera3d::default(), Camera { clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() }, RenderTarget::Image(image.clone().into()), bevy::core_pipeline::tonemapping::Tonemapping::None, Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y))).id();
    // world: a quad 2.4 wide, its page's aspect, left of centre; surface: a cube's faces, right, turned to show two
    let quad = app.world_mut().spawn((PageLayer::world(page, 2.4), Transform::from_xyz(-1.6, 0.0, 0.0))).id();
    let cube_mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Cuboid::new(1.6, 1.6, 1.6)));
    let cube_turn = Quat::from_rotation_y(0.5);
    let cube = app.world_mut().spawn((PageLayer::surface(page), Mesh3d(cube_mesh), Transform::from_xyz(1.6, 0.0, 0.0).with_rotation(cube_turn))).id();
    app.finish();
    app.cleanup();
    for _ in 0..30 {
        app.update();
    }
    app.world_mut().spawn(Screenshot::image(image)).observe(|e: On<ScreenshotCaptured>, mut shot: ResMut<Shot>| shot.0 = Some(e.image.clone()));
    for _ in 0..60 {
        app.update();
        if app.world().resource::<Shot>().0.is_some() {
            break;
        }
    }
    let Some(shot) = app.world_mut().resource_mut::<Shot>().0.take() else {
        eprintln!("page_layers: no frame");
        std::process::exit(1);
    };
    if let Some(out) = &out {
        shot.clone().try_into_dynamic().unwrap().to_rgb8().save(out).unwrap();
    }
    // where each layer's page parts land on screen
    let (camera, cam_at) = app.world_mut().query::<(&Camera, &GlobalTransform)>().get(app.world(), cam).unwrap();
    let px = |world: Vec3| -> [u8; 3] {
        let p = camera.world_to_viewport(cam_at, world).unwrap();
        let c = shot.get_color_at(p.x as u32, p.y as u32).unwrap().to_srgba();
        [(c.red * 255.0).round() as u8, (c.green * 255.0).round() as u8, (c.blue * 255.0).round() as u8]
    };
    let quad_at = *app.world().get::<GlobalTransform>(quad).unwrap();
    let cube_at = *app.world().get::<GlobalTransform>(cube).unwrap();
    // the quad is 2.4 x 1.8 (a Rectangle's UV 0,0 at its top left): its top left quarter's middle (-0.6, 0.45), its
    // bottom right's (0.6, -0.45). The cube's front face (+Z) has its UV 0,0 at its bottom left (Bevy's Cuboid), so
    // the page is there as its UVs place it: the blue quarter at the face's bottom left, (-0.4, -0.4, 0.8)
    let checks = [
        ("world quad, blue box", px(quad_at.transform_point(Vec3::new(-0.6, 0.45, 0.0))), [0, 0, 255]),
        ("world quad, red", px(quad_at.transform_point(Vec3::new(0.6, -0.45, 0.0))), [255, 0, 0]),
        ("cube's front face, blue box", px(cube_at.transform_point(Vec3::new(-0.4, -0.4, 0.8))), [0, 0, 255]),
        ("cube's front face, red", px(cube_at.transform_point(Vec3::new(0.4, 0.4, 0.8))), [255, 0, 0]),
        ("the clear between them", px(Vec3::new(0.0, 0.0, 0.0)), [0, 0, 0]),
    ];
    let mut ok = true;
    for (what, got, want) in checks {
        let near = got.iter().zip(want).all(|(g, w)| (*g as i32 - w as i32).abs() <= 8);
        ok &= near;
        println!("page_layers: {what}: {got:?} (want {want:?}){}", if near { "" } else { "  <- wrong" });
    }
    println!("page_layers: {}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}
