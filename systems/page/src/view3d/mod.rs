//! The host page's `view3d` (props.rs: its camera and parts from its props): a box a camera of its own renders its
//! `mesh3d` and `light3d` children into, on a render layer no other camera sees. The camera's image is the box's size
//! in device pixels; the element holds a placeholder picture of that size, which Vello draws the image's texture in
//! place of, copied on the GPU each frame (`ImageViews`, H7's views: no readback).

use std::collections::HashMap;
use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::camera::{ClearColorConfig, RenderTarget, visibility::RenderLayers};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use blitz_dom::BaseDocument;
use blitz_dom::node::{ImageData, RasterImageData, SpecialElementData};
use serde_json::{Map, Value};

use crate::ops::Nodes;

pub mod props;

/// The render layers views are given: past raea-runtime-js's own, which count up from 1 (RenderLayers::layer takes
/// layers below 64).
const FIRST_LAYER: usize = 32;

/// One view's camera, image and parts.
pub struct View {
    camera: Entity,
    layer: usize,
    image: Handle<Image>,
    /// the placeholder the element holds: Vello's key for the image's texture
    placeholder: vello::peniko::ImageData,
    props: Map<String, Value>,
    /// each part's props as last made, and its entity
    parts: HashMap<u64, (Map<String, Value>, Entity)>,
}

/// Each root's views by their wire id.
pub type Views = HashMap<u64, View>;

/// The images Vello draws in place of placeholders this frame (extracted to the render world by lib.rs), and whether
/// each is premultiplied (a camera's picture with a see-through clear, made straight on the GPU first, as Vello takes
/// an element's picture straight).
#[derive(Resource, Default, Clone)]
pub struct ImageViews(pub Vec<(vello::peniko::ImageData, Handle<Image>, bool)>);

/// The views brought up to the document: cameras made for new view3d elements and dropped with gone ones, each
/// image sized to its box at `scale`, each part made again when its props change.
pub fn sync(world: &mut World, doc: &mut BaseDocument, nodes: &Nodes, views: &mut Views, scale: f32) {
    let gone: Vec<u64> = views.keys().filter(|id| !nodes.by_id.contains_key(id)).copied().collect();
    for id in gone {
        if let Some(v) = views.remove(&id) {
            despawn(world, v.camera);
            for (_, e) in v.parts.into_values() {
                despawn(world, e);
            }
        }
    }
    let ids: Vec<u64> = nodes.by_id.keys().filter(|id| nodes.kind(**id) == Some("view3d")).copied().collect();
    for id in ids {
        let Some(&n) = nodes.by_id.get(&id) else { continue };
        let props = nodes.props.get(&id).cloned().unwrap_or_default();
        // its box in device pixels (an unlaid box: one pixel, until layout gives it a size)
        let size = doc.get_node(n).map(|node| {
            let l = node.final_layout();
            (l.content_box_width() * scale, l.content_box_height() * scale)
        });
        let px = size.map_or(UVec2::ONE, |(w, h)| UVec2::new(w.round().max(1.0) as u32, h.round().max(1.0) as u32));
        if !views.contains_key(&id) {
            let layer = FIRST_LAYER + world.resource_mut::<NextLayer>().take() % 31;
            let image = target(&mut world.resource_mut::<Assets<Image>>(), px);
            let clear = ClearColorConfig::Custom(props::clear_color(&props));
            let camera = world
                .spawn((
                    Camera3d::default(),
                    Camera { order: -100, clear_color: clear, ..default() },
                    RenderTarget::Image(image.clone().into()),
                    props::projection(&props),
                    props::eye(&props),
                    Tonemapping::None,
                    RenderLayers::layer(layer),
                    Msaa::Sample4,
                ))
                .id();
            let placeholder = placeholder(doc, n, px);
            views.insert(id, View { camera, layer, image, placeholder, props: props.clone(), parts: HashMap::new() });
        }
        let v = views.get_mut(&id).unwrap();
        if v.props != props {
            v.props = props.clone();
            if let Ok(mut em) = world.get_entity_mut(v.camera) {
                if let Some(mut c) = em.get_mut::<Camera>() {
                    c.clear_color = ClearColorConfig::Custom(props::clear_color(&props));
                }
                em.insert((props::projection(&props), props::eye(&props)));
            }
        }
        // the box resized: the camera's image and the placeholder follow
        if (v.placeholder.width, v.placeholder.height) != (px.x, px.y) {
            let image = target(&mut world.resource_mut::<Assets<Image>>(), px);
            if let Ok(mut em) = world.get_entity_mut(v.camera) {
                em.insert(RenderTarget::Image(image.clone().into()));
            }
            v.image = image;
            v.placeholder = placeholder(doc, n, px);
        }
        // its parts: the element's children that are mesh3d or light3d
        let kids: Vec<(u64, String)> = doc
            .get_node(n)
            .map(|node| node.children.iter().filter_map(|c| nodes.id_of(*c)).filter_map(|k| Some((k, nodes.kind(k)?.to_string()))).collect())
            .unwrap_or_default();
        v.parts.retain(|k, (_, e)| {
            let keep = kids.iter().any(|(id, _)| id == k);
            if !keep && let Ok(em) = world.get_entity_mut(*e) {
                em.despawn();
            }
            keep
        });
        for (k, kind) in kids {
            let p = nodes.props.get(&k).cloned().unwrap_or_default();
            if v.parts.get(&k).is_some_and(|(old, _)| *old == p) {
                continue;
            }
            let Some(made) = props::spawn_part(world, &kind, &p, v.layer, Some(v.camera)) else { continue };
            if let Some((_, old)) = v.parts.insert(k, (p, made)) {
                despawn(world, old);
            }
        }
    }
    // what Vello draws in place of the placeholders this frame
    let list: Vec<_> = views.values().map(|v| (v.placeholder.clone(), v.image.clone(), false)).collect();
    world.resource_mut::<ImageViews>().0.extend(list);
}

/// Elements showing Bevy Images, by page and node (a plain App's page: a layered Canvas3D's slot, a camera's target in
/// an element, H7): each holds a placeholder the image's size, and Vello draws the image's texture in its place each
/// frame, as a view3d box's camera image is drawn. The image is taken as a camera leaves it, premultiplied, so a
/// see-through clear and the 3D's antialiased edges go over what's under the element as raea's render lays them.
/// Whoever adds an entry keeps it while the element should show it.
#[derive(Resource, Default)]
pub struct ElementImages(pub HashMap<(Entity, blitz_dom::NodeId), Handle<Image>>);

/// ElementImages' placeholders made (again when an image or its size changes) and listed in ImageViews for this frame,
/// last frame's taken out first (host.rs clears the list at its step; a plain App has no host).
pub fn element_images(mut docs: NonSendMut<crate::Docs>, shows: Res<ElementImages>, images: Res<Assets<Image>>, mut views: ResMut<ImageViews>, mut held: Local<HashMap<(Entity, blitz_dom::NodeId), (vello::peniko::ImageData, Handle<Image>)>>) {
    if held.is_empty() && shows.0.is_empty() {
        return;
    }
    views.0.retain(|(p, _, _)| !held.values().any(|(q, _)| q.data.id() == p.data.id()));
    held.retain(|k, _| shows.0.contains_key(k));
    for (key, image) in &shows.0 {
        let Some(px) = images.get(image).map(Image::size) else { continue };
        if held.get(key).is_some_and(|(p, h)| (p.width, p.height) == (px.x, px.y) && h == image) {
            continue;
        }
        let Some(doc) = docs.0.get_mut(&key.0) else { continue };
        held.insert(*key, (placeholder(doc, key.1, px), image.clone()));
    }
    views.0.extend(held.values().map(|(p, h)| (p.clone(), h.clone(), true)));
}

/// ElementImages entries whose image follows its element (H7): the element's size sets the camera target's. Each
/// frame, before the placeholders, the image is resized in place to the element's content box in device pixels (the
/// page's scale) as last laid out, so a camera drawing into it draws at the element's size and aspect (Bevy's
/// camera_system takes an image target's new size). Whoever adds an entry keeps it while the image should follow.
#[derive(Resource, Default)]
pub struct SizedImages(pub std::collections::HashSet<(Entity, blitz_dom::NodeId)>);

/// SizedImages' images resized to their elements' boxes, only when the box's device size changed.
pub fn size_to_boxes(docs: NonSend<crate::Docs>, pages: Query<&crate::Page>, sized: Res<SizedImages>, shows: Res<ElementImages>, mut images: ResMut<Assets<Image>>) {
    for key in &sized.0 {
        let (Some(image), Some(doc), Ok(page)) = (shows.0.get(key), docs.0.get(&key.0), pages.get(key.0)) else { continue };
        let Some(node) = doc.get_node(key.1) else { continue };
        let l = node.final_layout();
        let px = UVec2::new((l.content_box_width() * page.scale).round().max(1.0) as u32, (l.content_box_height() * page.scale).round().max(1.0) as u32);
        if images.get(image).is_some_and(|i| i.size() != px)
            && let Some(mut i) = images.get_mut(image)
        {
            i.resize(Extent3d { width: px.x, height: px.y, ..default() });
        }
    }
}

/// The render layers handed out so far.
#[derive(Resource, Default)]
pub struct NextLayer(usize);

impl NextLayer {
    fn take(&mut self) -> usize {
        self.0 += 1;
        self.0
    }
}

fn despawn(world: &mut World, e: Entity) {
    if let Ok(em) = world.get_entity_mut(e) {
        em.despawn();
    }
}

/// A camera's render target `px` in size: sRGB, as Vello reads an image's bytes.
fn target(images: &mut Assets<Image>, px: UVec2) -> Handle<Image> {
    let mut image = Image::new_fill(Extent3d { width: px.x, height: px.y, ..default() }, TextureDimension::D2, &[0, 0, 0, 0], TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT;
    images.add(image)
}

/// The element's placeholder picture `px` in size, set as its image: what Vello draws the camera's image over.
fn placeholder(doc: &mut BaseDocument, n: blitz_dom::NodeId, px: UVec2) -> vello::peniko::ImageData {
    let blob = vello::peniko::Blob::new(Arc::new(vec![0u8; (px.x * px.y * 4) as usize]));
    if let Some(node) = doc.get_node_mut(n)
        && let Some(el) = node.element_data_mut()
    {
        el.special_data = SpecialElementData::Image(Box::new(ImageData::Raster(RasterImageData { width: px.x, height: px.y, data: blob.clone() })));
        node.insert_damage(crate::ALL_DAMAGE);
        node.mark_ancestors_dirty();
    }
    vello::peniko::ImageData { data: blob, format: vello::peniko::ImageFormat::Rgba8, alpha_type: vello::peniko::ImageAlphaType::Alpha, width: px.x, height: px.y }
}
