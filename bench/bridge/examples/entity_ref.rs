//! B3 H1's React side (docs/blitz-bevy.feature; bench/bridge/run.sh): a plain App with the composition's page on Blitz
//! (PagePlugin, WorldPagePlugin), whose React reads an element's twin with @raea/app's useEntity(ref) and shows it
//! (bench/bridge/entity.tsx). It passes when the entity React shows is the twin PageQuery finds for #card.
//!
//!   raea bundle bench/bridge/entity.tsx --composition-bundle -o entity.js
//!   cargo run --release -p raea-bridge-bench --example entity_ref -- entity.js
use bevy::camera::RenderTarget;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::ExitCondition;
use raea_runtime_twins::{PageElement, PageQuery};
use raea_runtime_react::scene::ScenePlugin;
use raea_runtime_page::WorldPagePlugin;
use raea_runtime_react::{ReactPlugin, ReactText};
use raea_render::engine::raea_plugins::{RaeaPlugins, Scene3dPlugin};
use std::path::PathBuf;

fn main() {
    let Some(js) = std::env::args().nth(1) else {
        eprintln!("usage: entity_ref ENTITY.js");
        std::process::exit(2);
    };
    let js = PathBuf::from(js).canonicalize().expect("the bundle");
    let (w, h) = (320, 240);
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
    );
    let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_target_texture(w, h, TextureFormat::Rgba8UnormSrgb, None));
    let dir = js.parent().unwrap().to_path_buf();
    app.add_plugins((RaeaPlugins, Scene3dPlugin, ReactPlugin::new(js.clone()), ScenePlugin { target: RenderTarget::Image(image.into()), size: (w, h), dir: dir.clone(), frame: None }));
    app.add_plugins((raea_page::PagePlugin, WorldPagePlugin { size: (w, h), dir, fps: 30.0 }));
    app.finish();
    app.cleanup();
    let mut found: Option<(u64, Option<u64>)> = None;
    for _ in 0..600 {
        app.update();
        let shown = app.world_mut().query::<&ReactText>().iter(app.world()).find_map(|t| t.0.strip_prefix("twin ").and_then(|n| n.parse::<u64>().ok()));
        if let Some(n) = shown {
            let page = app.world_mut().query::<&PageElement>().iter(app.world()).next().map(|el| el.page);
            let mut q: SystemState<PageQuery> = SystemState::new(app.world_mut());
            let card = page.and_then(|p| q.get(app.world()).ok().and_then(|q| q.by_id(p, "card")));
            found = Some((n, card.map(Entity::to_bits)));
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(4));
    }
    if found.is_none() {
        let texts: Vec<String> = app.world_mut().query::<&ReactText>().iter(app.world()).map(|t| t.0.clone()).collect();
        let ops: Vec<Option<u64>> = app.world_mut().query::<&PageElement>().iter(app.world()).map(|el| el.op).collect();
        eprintln!("entity_ref: React's texts {texts:?}; twins' ops {ops:?}");
    }
    let ok = matches!(found, Some((n, Some(c))) if n == c);
    println!("entity_ref: {} (React showed, #card's twin: {found:?})", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}
