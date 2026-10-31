//! B3 H4's bench (docs/blitz-bevy.feature; bench/bridge/run.sh): a plain Bevy App sharing a resource with its
//! composition's React (shared.rs; @raea/app's useResource and setResource). The composition (bench/bridge/shared.tsx)
//! sets the score to 100 when it mounts and shows the World's value; a system counts up from 100. It passes when
//! React's write reached the World and React's text shows the World's values as they change.
//!
//!   raea bundle bench/bridge/shared.tsx --composition-bundle -o shared.js
//!   cargo run --release -p raea-bridge-bench --example shared_state -- shared.js
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::ExitCondition;
use raea_runtime_react::scene::ScenePlugin;
use raea_runtime_react::shared::ShareExt;
use raea_runtime_react::{ReactPlugin, ReactText};
use std::path::PathBuf;

#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
#[type_path = "bench"]
struct Score {
    points: u32,
}

/// Counting up, once React has set the score.
fn count(mut score: ResMut<Score>) {
    if score.points >= 100 {
        score.points += 1;
    }
}

fn main() {
    let Some(js) = std::env::args().nth(1) else {
        eprintln!("usage: shared_state SHARED.js");
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
        .init_resource::<Score>()
        .share_resource::<Score>()
        .add_systems(Update, count);
    app.finish();
    app.cleanup();
    // what React showed, frame by frame, beside the World's value then
    let mut seen: Vec<(u32, u32)> = Vec::new();
    for _ in 0..600 {
        app.update();
        let world = app.world().resource::<Score>().points;
        let shown = app.world_mut().query::<&ReactText>().iter(app.world()).find_map(|t| t.0.strip_prefix("score ").map(str::to_string));
        if let Some(n) = shown.and_then(|s| s.parse::<u32>().ok()) {
            if seen.last().map(|s| s.0) != Some(n) {
                seen.push((n, world));
            }
        }
        if seen.iter().filter(|s| s.0 >= 100).count() >= 4 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(4));
    }
    // React's write reached the World (the count starts at 100), and React showed the World's values (its first, 0,
    // then the count's), in order, never ahead of it
    let after: Vec<_> = seen.iter().filter(|s| s.0 >= 100).collect();
    let ok = after.len() >= 4 && after.windows(2).all(|w| w[1].0 > w[0].0) && seen.iter().all(|(shown, world)| shown <= world);
    println!("shared_state: {} (React showed, World had: {seen:?})", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}
