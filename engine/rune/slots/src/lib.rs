//! raea-rune-slots: Rune layers in a Blitz page (docs/targets.feature T6c, per layer). An element marked
//! `data-raea-renderer="rune"` (sdk/rune's `<RuneLayer>`) stays Blitz's, a slot; what's under it is a Rune page's
//! (raea-rune-page), drawn by Rune on Bevy's device into an Image (raea-rune-bevy) that the slot shows (raea-page's
//! ElementImages, as the stage's `<img>` shows its picture).
//!
//! - Ops: the page host's commits pass through raea-runtime-dom's split, where raea-rune-layer's Router takes each
//!   layer's ops out of the batch; a page whose batches never make a slot isn't routed at all.
//! - Size: each frame a layer's viewport is its slot's content box in CSS px (its points), at the page's scale.
//! - Input: a pointer over a slot is a pointer over its layer, at the same point in the slot's content box (the
//!   twin's picking events, [`input`]); the events the Rune page's handlers queue go to the root's JS as Blitz's do.
//! - Said, not dropped ([`Slots::report`]): a move between the page and a layer, and that a layer's elements have no
//!   geometry in the page's layout snapshot (getBoundingClientRect on one is zero).

mod input;
mod layer;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use bevy::prelude::*;
use raea_core_wire::Op;
use raea_rune_bevy::{RuneLayer, RuneLayerPlugin, SpawnRune, Trees};
use raea_rune_layer::{Router, creates_slot};
use raea_runtime_dom::host::Roots;
use raea_sys_page::view3d::ElementImages;
use raea_sys_page::{Docs, Page};

/// What the page host's commits routed since the last sync.
#[derive(Default)]
struct Inbox {
    routers: HashMap<Entity, Router>,
    ops: Vec<(Entity, u64, Vec<Op>)>,
    gone: Vec<(Entity, u64)>,
    refused: Vec<String>,
}

/// One layer: its RuneLayer entity, and the ops its tree takes at its next frame.
pub struct Slot {
    pub entity: Entity,
    ops: Rc<RefCell<Vec<Op>>>,
    node: Option<blitz_dom::NodeId>,
}

/// The layers, by page and slot's wire id (a non-send resource).
#[derive(Default)]
pub struct Slots {
    inbox: Rc<RefCell<Inbox>>,
    pub layers: HashMap<(Entity, u64), Slot>,
    /// what the layers' Rune pages queued for the JS, by page
    outbox: Rc<RefCell<Vec<(Entity, String)>>>,
    /// what couldn't be done as asked, each said once
    pub report: Vec<String>,
    /// the layer a held press captured the pointer to (input.rs)
    held: Option<(Entity, u64)>,
}

/// Rune layers in the App's Blitz pages.
pub struct RuneSlotsPlugin;

impl Plugin for RuneSlotsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<RuneLayerPlugin>() {
            app.add_plugins(RuneLayerPlugin);
        }
        let slots = Slots::default();
        let inbox = slots.inbox.clone();
        raea_runtime_dom::host::set_split(Box::new(move |page, ops| split(&inbox, page, ops)));
        app.init_resource::<ElementImages>().init_resource::<input::Pending>().insert_non_send(slots).add_systems(Update, sync);
        input::observe(app);
    }
}

/// A batch of `page`'s ops: the page's back, each layer's kept for the next sync.
fn split(inbox: &RefCell<Inbox>, page: Entity, ops: Vec<Op>) -> Vec<Op> {
    let mut i = inbox.borrow_mut();
    if !i.routers.get(&page).is_some_and(Router::has_layers) && !ops.iter().any(creates_slot) {
        return ops;
    }
    let out = i.routers.entry(page).or_default().route(ops);
    i.ops.extend(out.layers.into_iter().map(|(slot, ops)| (page, slot, ops)));
    i.gone.extend(out.gone.into_iter().map(|s| (page, s)));
    i.refused.extend(out.refused);
    out.page
}

fn say(slots: &mut Slots, what: String) {
    if !slots.report.contains(&what) {
        eprintln!("raea-rune-slots: {what}");
        slots.report.push(what);
    }
}

/// Update: routed ops to their layers (a layer made by its first), gone slots' layers dropped, each layer sized to
/// its slot and shown there, pointer input in, and what the layers' pages queued out to the JS.
fn sync(world: &mut World) {
    let (ops, gone, refused) = {
        let s = world.non_send::<Slots>();
        let mut i = s.inbox.borrow_mut();
        (std::mem::take(&mut i.ops), std::mem::take(&mut i.gone), std::mem::take(&mut i.refused))
    };
    let mut slots = world.remove_non_send::<Slots>().expect("Slots");
    for r in refused {
        say(&mut slots, r);
    }
    for key in gone {
        if slots.held == Some(key) {
            slots.held = None;
        }
        if let Some(s) = slots.layers.remove(&key) {
            world.despawn(s.entity);
            if let Some(n) = s.node {
                world.resource_mut::<ElementImages>().0.remove(&(key.0, n));
            }
        }
    }
    for (page, slot, ops) in ops {
        if !slots.layers.contains_key(&(page, slot)) {
            let scale = world.get::<Page>(page).map_or(1.0, |p| p.scale);
            let (tree, pending) = layer::make(page, scale, slots.outbox.clone());
            let entity = world.spawn_rune(tree, [0.0; 4]);
            slots.layers.insert((page, slot), Slot { entity, ops: pending, node: None });
            say(&mut slots, "a Rune layer's elements aren't in the page's layout snapshot (their getBoundingClientRect is zero)".into());
        }
        slots.layers[&(page, slot)].ops.borrow_mut().extend(ops);
    }
    place(world, &mut slots);
    input::apply(world, &mut slots);
    let out = std::mem::take(&mut *slots.outbox.borrow_mut());
    world.insert_non_send(slots);
    for (page, msg) in out {
        match serde_json::from_str(&msg) {
            Ok(v) => raea_runtime_dom::host::deliver_to(world, page, v),
            Err(e) => eprintln!("raea-rune-slots: a layer's message that isn't JSON: {e}"),
        }
    }
}

/// Each layer at its slot: the slot's node found, the tree's viewport its content box, its Image shown there.
fn place(world: &mut World, slots: &mut Slots) {
    for ((page, id), slot) in slots.layers.iter_mut() {
        let node = world.get_non_send::<Roots>().and_then(|r| r.0.iter().find(|r| r.page == *page).and_then(|r| r.nodes.by_id.get(id).copied()));
        let size = node.and_then(|n| world.non_send::<Docs>().0.get(page)?.get_node(n).map(|n| content(n).1));
        let scale = world.get::<Page>(*page).map_or(1.0, |p| p.scale);
        if let Some(size) = size {
            let mut trees = world.non_send_mut::<Trees>();
            if let Some(t) = trees.0.get_mut(&slot.entity)
                && t.viewport() != ((size.x, size.y), scale)
            {
                t.resize((size.x.max(1.0), size.y.max(1.0)), scale);
            }
        }
        let Some(image) = world.get::<RuneLayer>(slot.entity).map(|l| l.image.clone()) else { continue };
        let mut shows = world.resource_mut::<ElementImages>();
        if let Some(old) = slot.node.filter(|o| Some(*o) != node) {
            shows.0.remove(&(*page, old));
        }
        if let Some(n) = node {
            shows.0.insert((*page, n), image);
        }
        slot.node = node;
    }
}

/// A node's content box: its top left in page CSS px, and its size.
pub(crate) fn content(node: &blitz_dom::Node) -> (Vec2, Vec2) {
    let l = node.final_layout();
    let abs = node.absolute_position(0.0, 0.0);
    let min = Vec2::new(abs.x + l.border.left + l.padding.left, abs.y + l.border.top + l.padding.top);
    (min, Vec2::new(l.content_box_width(), l.content_box_height()))
}
