//! docs/sdk.feature S2d's bench (bench/bridge/run.sh --pixels): /ecs's typed query on the host's World (@raea/app's
//! HostWorld) in a plain App. The component is registered for reflection only, never shared (no share_component); the
//! composition (bench/bridge/hostquery.tsx) queries it by name outside React's render, sets it to 100 through a write
//! binding once, and shows what the query answers; a system counts up from 100. It passes when the write reached the
//! World and the text shows the World's values as they change, in order, never ahead of it.
//!
//!   raea bundle bench/bridge/hostquery.tsx --composition-bundle -o hostquery.js
//!   cargo run --release -p raea-bridge-bench --example host_query -- hostquery.js
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::ExitCondition;
use raea_runtime_react::scene::ScenePlugin;
use raea_runtime_react::{ReactPlugin, ReactText};
use std::path::PathBuf;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
#[type_path = "bench"]
struct Hq {
    n: u32,
}

/// Counting up, once React has set it.
fn count(mut q: Query<&mut Hq>) {
    for mut h in &mut q {
        if h.n >= 100 {
            h.n += 1;
        }
    }
}

fn main() {
    let Some(js) = std::env::args().nth(1) else {
        eprintln!("usage: host_query HOSTQUERY.js");
        std::process::exit(2);
    };
    let js = PathBuf::from(js).canonicalize().expect("the bundle");
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
    );
    let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(320, 180, TextureFormat::Rgba8UnormSrgb, None));
    app.add_plugins((ReactPlugin::new(js.clone()), ScenePlugin { target: RenderTarget::Image(image.into()), size: (320, 180), dir: js.parent().unwrap().to_path_buf(), frame: None }))
        .register_type::<Hq>()
        .add_systems(Update, count);
    let hq = app.world_mut().spawn((Name::new("hq"), Hq::default())).id();
    app.finish();
    app.cleanup();
    // what React showed, beside the World's value then
    let mut seen: Vec<(u32, u32)> = Vec::new();
    for _ in 0..600 {
        app.update();
        let world = app.world().get::<Hq>(hq).map_or(0, |h| h.n);
        let shown = app.world_mut().query::<&ReactText>().iter(app.world()).find_map(|t| t.0.strip_prefix("hq ").map(str::to_string));
        if let Some(n) = shown.and_then(|s| s.parse::<u32>().ok()) {
            if seen.last().map(|s| s.0) != Some(n) {
                seen.push((n, world));
            }
        }
        if seen.iter().filter(|s| s.0 > 100).count() >= 4 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(4));
    }
    // the write binding reached the World (the count starts at 100), and the query answered the World's values (0
    // first, then the count's), in order, never ahead of it
    let after: Vec<_> = seen.iter().filter(|s| s.0 >= 100).collect();
    let ok = after.len() >= 4 && after.windows(2).all(|w| w[1].0 > w[0].0) && seen.iter().all(|(shown, world)| shown <= world);
    let texts: Vec<String> = app.world_mut().query::<&ReactText>().iter(app.world()).map(|t| t.0.clone()).collect();
    println!("host_query: {} (shown, World had: {seen:?}; the page's text at the end: {texts:?})", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}
