//! raea-rune-bevy: a Rune tree as a Bevy layer (docs/targets.feature T6c-e; T6b's spike, projects/rune
//! spikes/rune-in-bevy). Bevy owns the window, the device and the frame; a Rune tree is an Image in it.
//!
//! - A layer is an entity with a [`RuneLayer`]: the Image it draws into and the colour it clears to. Its tree (a Rune
//!   `Runtime`, which isn't `Send`) lives in [`Trees`], a non-send resource, under the entity (`World::insert_rune`).
//! - Each frame, in `PostUpdate`, every tree runs a frame (its poll, its view if state changed, its springs) and its
//!   `Renderer`, made on Bevy's `RenderDevice`, draws it straight into the Image's GpuImage view and submits on Bevy's
//!   `RenderQueue`. The render world then samples it: the same queue orders the two, so a frame shows what Rune drew
//!   in that frame. No copy, no second device.
//! - The render world publishes each layer's view after it prepares the Image ([`Views`]); a layer's first frame,
//!   before its Image is on the GPU, draws nothing.
//! - It needs Bevy's pipelined rendering off (raea's engine runs without it): with a render thread the render world
//!   would sample the Image while the next frame draws into it. The plugin refuses an app that has it.
//! - Anything can show the Image: a UI `ImageNode` over a camera (a native app's window, a game's HUD), a 3D material,
//!   a render's encoder. Input reaches a tree through [`input`]: an overlay's from the window, or any `Input` sent.

pub mod input;
pub mod keys;
pub mod readback;
mod tree;

pub use input::{Input, RuneOverlay};
pub use tree::{AnyTree, Tree, draw as draw_runtime};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::{Render, RenderApp, RenderSystems};

/// The format a layer's Image has, and its tree's Renderer draws: sRGB RGBA8, premultiplied (Rune's own offscreen one).
pub const FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

/// A Rune tree drawn into `image` each frame. Its tree is in [`Trees`] under this entity.
#[derive(Component, Clone)]
pub struct RuneLayer {
    pub image: Handle<Image>,
    /// what the Image holds where the tree paints nothing (premultiplied linear RGBA); transparent by default
    pub clear: [f32; 4],
    /// a fixed step a frame in seconds (a render's or a check's virtual clock); none: Bevy's `Time` delta
    pub step: Option<f64>,
}

/// The layers' trees, by entity (a non-send resource: Rune's Runtime isn't `Send`).
#[derive(Default)]
pub struct Trees(pub HashMap<Entity, Box<dyn AnyTree>>);

/// Each layer's GpuImage view, as the render world prepared it, for the main world to draw into.
#[derive(Resource, Clone, Default)]
pub struct Views(Arc<Mutex<ViewMap>>);

#[derive(Default)]
struct ViewMap {
    /// the Images the layers draw into, which the render world looks up
    wanted: Vec<AssetId<Image>>,
    views: HashMap<AssetId<Image>, wgpu::TextureView>,
}

impl Views {
    fn get(&self, id: AssetId<Image>) -> Option<wgpu::TextureView> {
        self.0.lock().ok()?.views.get(&id).cloned()
    }
}

/// Rune layers in an App.
pub struct RuneLayerPlugin;

impl Plugin for RuneLayerPlugin {
    fn build(&self, app: &mut App) {
        assert!(
            !app.is_plugin_added::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>(),
            "raea-rune-bevy: RuneLayerPlugin needs Bevy's pipelined rendering off (DefaultPlugins.build().disable::<PipelinedRenderingPlugin>()): \
             a render thread would sample a layer's Image while the next frame draws into it"
        );
        let views = Views::default();
        app.init_non_send::<Trees>().insert_resource(views.clone()).add_systems(PreUpdate, input::forward.after(bevy::input::InputSystems)).add_systems(PostUpdate, (input::overlay_size, draw).chain());
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render.insert_resource(views).add_systems(Render, publish.in_set(RenderSystems::PrepareAssets).after(bevy::render::render_asset::prepare_assets::<GpuImage>));
        }
    }
}

/// An Image a layer of `size` pixels draws into: a render target Bevy samples, on the GPU only.
pub fn image(images: &mut Assets<Image>, (w, h): (u32, u32)) -> Handle<Image> {
    let mut image = Image::new_target_texture(w.max(1), h.max(1), FORMAT, None);
    image.texture_descriptor.usage |= TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    images.add(image)
}

/// Spawning a layer: the entity with its [`RuneLayer`] and its tree in [`Trees`].
pub trait SpawnRune {
    /// `tree` drawn into a new Image of its size in pixels; the layer's entity, to show the Image or send it input
    fn spawn_rune(&mut self, tree: impl AnyTree + 'static, clear: [f32; 4]) -> Entity;
    /// `tree` as the tree of `entity`'s layer, in place of the one it had
    fn insert_rune(&mut self, entity: Entity, tree: impl AnyTree + 'static);
}

impl SpawnRune for World {
    fn spawn_rune(&mut self, tree: impl AnyTree + 'static, clear: [f32; 4]) -> Entity {
        let handle = image(&mut self.resource_mut::<Assets<Image>>(), tree.pixels());
        let e = self.spawn(RuneLayer { image: handle, clear, step: None }).id();
        self.insert_rune(e, tree);
        e
    }

    fn insert_rune(&mut self, entity: Entity, tree: impl AnyTree + 'static) {
        self.non_send_mut::<Trees>().0.insert(entity, Box::new(tree));
    }
}

/// PostUpdate: each layer's tree runs a frame and draws into its Image on Bevy's device and queue.
fn draw(world: &mut World) {
    let dt = world.resource::<Time>().delta_secs_f64();
    let (Some(device), Some(queue)) = (world.get_resource::<RenderDevice>(), world.get_resource::<RenderQueue>()) else { return };
    let (device, queue) = (device.wgpu_device().clone(), (**queue).clone());
    let views = world.resource::<Views>().clone();
    let mut layers: Vec<(Entity, RuneLayer)> = world.query::<(Entity, &RuneLayer)>().iter(world).map(|(e, l)| (e, l.clone())).collect();
    layers.sort_by_key(|(e, _)| *e);
    // a tree resized this frame (an overlay's window) gets an Image of its new size
    for (e, layer) in &mut layers {
        let px = world.non_send::<Trees>().0.get(e).map(|t| t.pixels());
        let size = world.resource::<Assets<Image>>().get(&layer.image).map(|i| (i.width(), i.height()));
        if let (Some(px), Some(size)) = (px, size)
            && px != size
        {
            world.resource_mut::<Assets<Image>>().get_mut(&layer.image).expect("the layer's Image").resize(bevy::render::render_resource::Extent3d { width: px.0.max(1), height: px.1.max(1), depth_or_array_layers: 1 });
            if let Ok(mut views) = views.0.lock() {
                views.views.remove(&layer.image.id());
            }
        }
    }
    if let Ok(mut v) = views.0.lock() {
        v.wanted = layers.iter().map(|(_, l)| l.image.id()).collect();
    }
    let mut trees = world.non_send_mut::<Trees>();
    trees.0.retain(|e, _| layers.iter().any(|(l, _)| l == e));
    for (e, layer) in &layers {
        let Some(tree) = trees.0.get_mut(e) else { continue };
        tree.step(layer.step.unwrap_or(dt));
        if let Some(view) = views.get(layer.image.id()) {
            tree.draw(&device, &queue, &view, layer.clear);
        }
    }
}

/// Render: the views of the Images the layers draw into, once they're on the GPU.
fn publish(views: Res<Views>, images: Res<RenderAssets<GpuImage>>) {
    let Ok(mut v) = views.0.lock() else { return };
    let wanted = std::mem::take(&mut v.wanted);
    for id in &wanted {
        if let Some(gpu) = images.get(*id) {
            v.views.insert(*id, (*gpu.texture_view).clone());
        }
    }
    v.views.retain(|id, _| wanted.contains(id));
    v.wanted = wanted;
}
