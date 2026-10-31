//! B3 H5's 3D planes in a plain Bevy App (docs/blitz-bevy.feature; bench/bridge/run.sh --pixels). Two cases on a
//! white 400 x 400 page:
//! - sorting (WPT transform3d-sorting-001): in a preserve-3d box, a lime plane at translateZ(10px) comes first in the
//!   DOM and a red one at translateZ(-10px), moved over it, second. The lime is nearer, so it's what shows.
//! - behind the viewer: under `perspective: 200px` (its origin the box's centre, page (200, 250)), a blue 200 x 400
//!   plane centred there at translateZ(150px) rotateX(60deg) has its part more than 58px below its centre past the
//!   viewer (z above 200px). The rest is drawn, page y 160 down: its point 80px above the centre lands at (200, 183).
//!   The part behind, unclipped, would land above the page's y 88; at (200, 60) it's white.
//! - crossing (as WPT's transform3d-sorting tests with intersecting planes): in a preserve-3d box under
//!   `perspective: 500px`, a lime plane turned rotateY(40deg) and a red one rotateY(-40deg) about the same axis cross
//!   there. Each one's near half is in front of the other's far half: lime shows on the left, red on the right. Their
//!   centres are at the same depth, so ordered whole the later one (red) covers both.
//!
//!   cargo run --release -p raea-bridge-bench --example planes -- [OUT.png]
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::ExitCondition;

const W: u32 = 400;
const H: u32 = 400;

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
        { "op": "create", "id": 1, "kind": "div", "props": { "style": { "width": "400px", "height": "400px", "background": "rgb(255, 255, 255)", "position": "relative" } } },
        // sorting
        { "op": "create", "id": 2, "kind": "div", "props": { "style": { "position": "absolute", "left": "0px", "top": "0px", "width": "100px", "height": "100px", "transformStyle": "preserve-3d" } } },
        { "op": "create", "id": 3, "kind": "div", "props": { "style": { "width": "100px", "height": "100px", "background": "rgb(0, 255, 0)", "transform": "translateZ(10px)" } } },
        { "op": "create", "id": 4, "kind": "div", "props": { "style": { "width": "100px", "height": "100px", "background": "rgb(255, 0, 0)", "transform": "translateZ(-10px) translateY(-100px)" } } },
        { "op": "append", "parent": 2, "child": 3 },
        { "op": "append", "parent": 2, "child": 4 },
        // behind the viewer
        { "op": "create", "id": 5, "kind": "div", "props": { "style": { "position": "absolute", "left": "0px", "top": "100px", "width": "400px", "height": "300px", "perspective": "200px" } } },
        { "op": "create", "id": 6, "kind": "div", "props": { "style": { "position": "absolute", "left": "100px", "top": "-50px", "width": "200px", "height": "400px", "background": "rgb(0, 0, 255)", "transform": "translateZ(150px) rotateX(60deg)" } } },
        { "op": "append", "parent": 5, "child": 6 },
        // crossing
        { "op": "create", "id": 7, "kind": "div", "props": { "style": { "position": "absolute", "left": "200px", "top": "0px", "width": "200px", "height": "100px", "perspective": "500px", "transformStyle": "preserve-3d" } } },
        { "op": "create", "id": 8, "kind": "div", "props": { "style": { "position": "absolute", "left": "0px", "top": "0px", "width": "200px", "height": "100px", "background": "rgb(0, 255, 0)", "transform": "rotateY(40deg)" } } },
        { "op": "create", "id": 9, "kind": "div", "props": { "style": { "position": "absolute", "left": "0px", "top": "0px", "width": "200px", "height": "100px", "background": "rgb(255, 0, 0)", "transform": "rotateY(-40deg)" } } },
        { "op": "append", "parent": 7, "child": 8 },
        { "op": "append", "parent": 7, "child": 9 },
        { "op": "append", "parent": 1, "child": 7 },
        { "op": "append", "parent": 1, "child": 2 },
        { "op": "append", "parent": 1, "child": 5 },
        { "op": "append", "parent": 0, "child": 1 },
    ]))
    .unwrap();
    raea_sys_page::ops::apply(app.world_mut().non_send_mut::<raea_sys_page::Docs>().0.get_mut(&page).unwrap(), &mut nodes, ops);
    app.world_mut().spawn((Camera2d, Camera { order: 1, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() }, RenderTarget::Image(target.clone().into()), raea_runtime_twins::overlay::PageOverlay { page }));
    app.finish();
    app.cleanup();
    for _ in 0..30 {
        app.update();
    }
    app.world_mut().spawn(Screenshot::image(target)).observe(|e: On<ScreenshotCaptured>, mut shot: ResMut<Shot>| shot.0 = Some(e.image.clone()));
    for _ in 0..60 {
        app.update();
        if app.world().resource::<Shot>().0.is_some() {
            break;
        }
    }
    let Some(shot) = app.world_mut().resource_mut::<Shot>().0.take() else {
        eprintln!("planes: no frame");
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
        ("the nearer plane over the farther, though first in the DOM", px(50, 50), [0, 255, 0]),
        ("a plane's part in front of the viewer, its rest behind", px(200, 183), [0, 0, 255]),
        ("above it, where its part behind the viewer would land", px(200, 60), [255, 255, 255]),
        ("crossing planes: the lime one's near half, on the left", px(250, 50), [0, 255, 0]),
        ("crossing planes: the red one's near half, on the right", px(350, 50), [255, 0, 0]),
    ];
    let mut ok = true;
    for (what, got, want) in checks {
        let near = got.iter().zip(want).all(|(g, w)| (*g as i32 - w as i32).abs() <= 8);
        ok &= near;
        println!("planes: {what}: {got:?} (want {want:?}){}", if near { "" } else { "  <- wrong" });
    }
    println!("planes: {}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}
