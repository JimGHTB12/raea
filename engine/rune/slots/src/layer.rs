//! A layer's tree: a Rune page (raea-rune-page) in a raea-rune-bevy Tree, taking its routed ops (and its slot's size
//! as its breakpoints' width) before each frame and handing what its handlers queued for the JS to the plugin after it.

use std::cell::RefCell;
use std::rc::Rc;

use bevy::prelude::Entity;
use raea_core_wire::Op;
use raea_rune_bevy::Tree;
use raea_rune_page::page::{Out, Page, view};
use rune::{Env, Runtime};

/// A layer of page `page` at `scale` (sized to its slot by the next sync), its messages for the JS into `outbox`; and
/// the ops it takes at its next frame.
pub(crate) fn make(page: Entity, scale: f32, outbox: Rc<RefCell<Vec<(Entity, String)>>>) -> (Tree<Page>, Rc<RefCell<Vec<Op>>>) {
    let viewport = (1.0, 1.0);
    let text = rune::text::TextSystem::new(raea_rune_media::fonts::registered(), ("Inter", "Inter"));
    let mut state = Page::default();
    state.viewport = viewport;
    let mut tree = Tree::new(Runtime::with_text(state, view, viewport, Env::new(false, scale), text), scale);
    let pending = Rc::new(RefCell::new(Vec::new()));
    let ops = pending.clone();
    tree.before_frame = Some(Box::new(move |rt: &mut Runtime<Page>| {
        let ops = std::mem::take(&mut *ops.borrow_mut());
        // the slot's size, the page's breakpoints' width (Page::resized would tell the root's JS the window resized)
        let resized = rt.state.viewport != rt.viewport;
        if ops.is_empty() && !resized {
            return;
        }
        let (mut r, viewport) = (Ok(()), rt.viewport);
        rt.update(|p, _| {
            p.viewport = viewport;
            r = p.apply_ops(&ops);
        });
        if let Err(e) = r {
            eprintln!("raea-rune-slots: a layer's ops: {e}");
        }
    }));
    tree.after_frame = Some(Box::new(move |rt: &mut Runtime<Page>| {
        let m = raea_rune_page::layout::measure(&rt.elements(), &rt.state.boxes.borrow(), rt.viewport);
        rt.state.measured(m);
        rt.state.media_news();
        // its element events and host messages for `__ui_deliver`; a layout snapshot is the page's (Blitz's) alone
        let mut out = outbox.borrow_mut();
        out.extend(rt.state.take_out().into_iter().filter_map(|o| match o {
            Out::Deliver(m) => Some((page, m)),
            Out::Layout(_) => None,
        }));
    }));
    (tree, pending)
}
