//! Pointer input into a layer: the picking events on its slot's twin (raea-runtime-twins: every element's entity),
//! queued as they're triggered and given to the layer's tree at the sync, at the same point in the slot's content box
//! (its CSS px are the layer's points). A wheel's lines are 40 points each, as raea-rune-bevy's overlay takes them.

use bevy::input::mouse::MouseScrollUnit;
use bevy::picking::events::{PointerMove, PointerOut, PointerPress, PointerRelease, PointerScroll};
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use raea_rune_bevy::{Input, Trees};
use raea_runtime_twins::bridge::PageElement;
use raea_sys_page::Docs;

use crate::Slots;

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Move,
    Down(PointerButton),
    Up(PointerButton),
    Leave,
    /// a wheel's vertical points, positive moving the content down
    Wheel(f32),
}

/// Picking events on twins since the last sync: the twin, what happened, and where in its page (CSS px).
#[derive(Resource, Default)]
pub(crate) struct Pending(Vec<(Entity, Kind, Option<Vec2>)>);

/// `E` on any entity queued as what `read` makes of it.
fn on<E: EntityEvent>(app: &mut App, read: fn(&E) -> (Kind, Option<Vec3>)) {
    app.add_observer(move |e: On<E>, mut c: Commands| {
        let (twin, (kind, at)) = (e.event_target(), read(&e));
        c.queue(move |w: &mut World| w.resource_mut::<Pending>().0.push((twin, kind, at.map(|p| p.truncate()))));
    });
}

pub(crate) fn observe(app: &mut App) {
    on::<PointerMove>(app, |e| (Kind::Move, e.hit.position));
    // a press and a release are alike but for what they are
    macro_rules! button {
        ($e:ty, $kind:path) => {
            on::<$e>(app, |e| ($kind(e.button), e.hit.position))
        };
    }
    button!(PointerPress, Kind::Down);
    button!(PointerRelease, Kind::Up);
    on::<PointerOut>(app, |_| (Kind::Leave, None));
    on::<PointerScroll>(app, |e| {
        let lines = matches!(e.unit, MouseScrollUnit::Line);
        (Kind::Wheel(if lines { e.y * 40.0 } else { e.y }), e.hit.position)
    });
}

/// The pending events on slots' twins, each to its layer's tree. A press on a layer captures the pointer, as
/// setPointerCapture does on the page: until its release, that page's moves and the release go to the layer wherever
/// they land, at their points in the slot's box, so a press dragged off the layer still ends there.
pub(crate) fn apply(world: &mut World, slots: &mut Slots) {
    let pending = std::mem::take(&mut world.resource_mut::<Pending>().0);
    for (twin, kind, at) in pending {
        let Some(el) = world.get::<PageElement>(twin).copied() else { continue };
        let held = slots.held.filter(|(page, _)| *page == el.page && matches!(kind, Kind::Move | Kind::Up(_) | Kind::Leave));
        // picking's events bubble: otherwise only the slot's own twin is the layer's (its parents' are the page's)
        let Some(key) = held.or_else(|| el.op.map(|id| (el.page, id)).filter(|k| slots.layers.contains_key(k))) else { continue };
        let Some(slot) = slots.layers.get(&key) else { continue };
        let min = slot.node.and_then(|n| world.non_send::<Docs>().0.get(&key.0)?.get_node(n).map(|n| crate::content(n).0));
        let p = at.zip(min).map(|(at, min)| at - min);
        let ev = match (kind, p) {
            // a held press keeps the pointer: leaving the slot isn't leaving the layer
            (Kind::Leave, _) if held.is_some() => continue,
            (Kind::Leave, _) => Input::Leave,
            (Kind::Move, Some(p)) => Input::Move(p.x, p.y),
            (Kind::Down(PointerButton::Secondary), Some(p)) => Input::SecondaryDown(p.x, p.y),
            (Kind::Up(PointerButton::Secondary), Some(p)) => Input::SecondaryUp(p.x, p.y),
            (Kind::Down(PointerButton::Primary), Some(p)) => Input::Down(p.x, p.y),
            (Kind::Up(PointerButton::Primary), Some(p)) => Input::Up(p.x, p.y),
            (Kind::Wheel(dy), Some(p)) => Input::Wheel(p.x, p.y, dy),
            _ => continue,
        };
        match kind {
            Kind::Down(_) => slots.held = Some(key),
            Kind::Up(_) => slots.held = None,
            _ => {}
        }
        if let Some(t) = world.non_send_mut::<Trees>().0.get_mut(&slot.entity) {
            t.input(ev);
        }
    }
}
