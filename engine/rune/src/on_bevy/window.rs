//! The app in a Bevy window (`run app.js --draw bevy`, T6c): Bevy owns the window, the device and the frame, and the
//! app's tree is a Rune layer over it (raea-rune-bevy), sized to the window and given its pointer and keys
//! (RuneOverlay). The page, its JS and its hooks are window.rs's, as in Rune's own shell: the tree's poll moves the JS
//! on each frame, its on_key takes the keys Rune's focused control didn't, and after each frame its layout goes to the
//! next poll. A UI image shows the layer over a 2D camera; a 3D scene goes under it on the same device.

use std::sync::Arc;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use raea_rune_bevy::{RuneLayer, RuneOverlay, SpawnRune, Tree};
use rune::{Env, Runtime};

use crate::page::{Page, view};

#[derive(Resource, Clone, Copy)]
struct Size((f32, f32));

/// `bundle` in a `size` window titled `title`, until it closes. Errors from the JS end the app and come back here.
pub fn run(bundle: String, title: &str, size: (f32, f32)) -> Result<(), String> {
    crate::window::begin(bundle, size)?;
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window { title: title.into(), resolution: WindowResolution::new(size.0 as u32, size.1 as u32), ..default() }),
        ..default()
    }));
    app.add_plugins(raea_rune_bevy::RuneLayerPlugin).insert_resource(Size(size)).add_systems(Startup, spawn).add_systems(Last, end_on_error);
    app.run();
    crate::window::error(false).map_or(Ok(()), Err)
}

/// The page's tree as a layer filling the window, shown by a UI image over a 2D camera.
fn spawn(world: &mut World) {
    let size = world.resource::<Size>().0;
    let scale = world.query::<&Window>().iter(world).next().map_or(1.0, |w| w.resolution.scale_factor());
    let mut page = Page::default();
    page.viewport = size;
    // decodes and fetches land on the next frame: Bevy's loop runs every frame, so the waker has nothing to wake
    crate::window::setup(&mut page, Arc::new(|| {}));
    let text = rune::text::TextSystem::new(raea_rune_media::fonts::registered(), ("Inter", "Inter"));
    let mut rt = Runtime::with_text(page, view, size, Env::new(false, scale), text);
    rt.poll = Some(crate::window::poll);
    rt.on_key = Some(crate::window::on_key);
    let mut tree = Tree::new(rt, scale);
    tree.after_frame = Some(Box::new(|rt: &mut Runtime<Page>| crate::window::drawn(rt)));
    let layer = world.spawn_rune(tree, [1.0, 1.0, 1.0, 1.0]);
    world.entity_mut(layer).insert(RuneOverlay::default());
    let image = world.get::<RuneLayer>(layer).expect("the layer").image.clone();
    world.spawn(Camera2d);
    world.spawn((Node { width: percent(100), height: percent(100), ..default() }, ImageNode::new(image)));
}

/// The JS failed: the app ends, as Rune's shell does on `tx.quit()`.
fn end_on_error(mut exit: MessageWriter<AppExit>) {
    if crate::window::error(true).is_some() {
        exit.write(AppExit::error());
    }
}
