//! A TSX root on a Blitz page (docs/blitz-bevy.feature B5c, and B3's H3: the page's events heard by the app). The
//! space's JS speaks the wire unchanged (raea-core-wire: op batches in; element events, host events and
//! answers out), its elements are a Blitz document's (ops.rs), and Blitz does with input what a browser does: hit
//! testing, hover, the press's focus, text fields' editing and IME, scrolling, each as a default action that runs
//! after the element's handlers unless one prevented it. The handlers run inside Blitz's event dispatch: an event goes
//! to the nearest element up from its target that listens for it (as raea-runtime-js's host sends it), its commit is applied
//! to the document there and then, and its `prevented` is Blitz's prevent_default, so the default action sees what the
//! handler did. What needs the World (host calls, asks, watches) is drained after the event, as raea-runtime-js's host drains
//! after each delivery.
//!
//! The JS engines, the prelude, host calls and space actions are raea-runtime-js's own (raea_runtime_js::js, raea_runtime_js::calls), so a
//! bundle can't tell which page it's on.

use std::collections::HashMap;

use bevy::prelude::*;
use raea_runtime_js::calls::{HostCalls, SpaceActions};
use raea_runtime_js::js::{Arg, JsEngine, Shared};
use raea_core_wire::{self as wire, Ev};
use blitz_dom::{BaseDocument, Document, EventDriver, EventHandler, NodeId};
use blitz_traits::events::{BlitzPointerEvent, BlitzWheelDelta, DomEvent, DomEventData, EventState, MouseEventButton, UiEvent};
use serde_json::{Value, json};

use raea_sys_page::Docs;
use raea_sys_page::ops::{self, Nodes};

mod frames;
pub use frames::*;
mod commit;
pub use commit::*;
mod step;
pub use step::*;
mod bridge;
pub use bridge::*;
mod elements;
pub use elements::*;

type R<T> = Result<T, String>;

/// One root: its page (the entity holding its document and Image), its JS and the ids its ops named.
pub struct Root {
    pub page: Entity,
    js: Box<dyn JsEngine>,
    out: Shared,
    pub nodes: Nodes,
    /// host events and answers for its next step
    queue: Vec<Value>,
    /// it asked for a `frame` host event every step
    ticking: bool,
    /// the press count the app's input says (a double click's 2), for pointerdown's `detail`
    pub clicks: u32,
    /// what it captured the pointer to (setPointerCapture): its moves and its release go there while a press is held
    capture: Option<u64>,
    /// what was hovered when a drag started (dnd.ts's `hoverFreeze`): it stays hovered while the drag lasts, as Chrome
    /// sends the page no mouse events then
    hover_frozen: Option<Vec<NodeId>>,
    /// its video and audio elements' decoders (media.rs)
    medias: crate::media::Medias,
    /// its view3d elements' cameras and parts (view3d.rs)
    views: raea_sys_page::view3d::Views,
    /// document.fonts waiting (load, ready): the drains since it asked, answered once no @font-face face is loading
    fonts_wait: Option<u32>,
    /// its presses as raea-runtime-js's host sends a bevy_ui root's (bevy_ui_presses): one whose handler moved the focus
    /// keeps it, and a right press is its contextmenu alone
    bevy_ui: bool,
    /// its videos' frames are placed by the caller (place_videos_itself), not drawn from their streams
    placed_videos: bool,
    /// the <iframe> whose document this root runs, when it's a frame's (frames below)
    frame: Option<Frame>,
    /// its iframes' loaded documents mount_frames has looked at (the iframe, its URL), scripted or not: each once
    frames_seen: std::collections::HashSet<(NodeId, String)>,
    /// a context call committed the JS's writes before the drain did (PageSync::commit)
    synced: bool,
}

/// What a root's JS calls lend its context host calls (raea_runtime_js::calls::on_sync_ctx: style.computed's resolve, a
/// canvas's pixels): the World, the root's page and ids, and what its JS has sent so far, for the call it's inside of.
pub struct PageSync {
    world: *mut World,
    pub page: Entity,
    /// the page's document while an input's dispatch holds it (it's out of Docs then)
    doc: Option<*mut BaseDocument>,
    nodes: *mut Nodes,
    pub out: Shared,
    /// the root's queue (a focus its commit moved is told there) and whether a commit was made, for its step's
    /// twins; none while an input's dispatch holds the document (it commits after each delivery itself)
    queue: Option<*mut Vec<Value>>,
    synced: Option<*mut bool>,
}

// (the pointers are what a JS call borrowed, alive for it alone: with_sync_ctx's handlers get them only inside it)
impl PageSync {
    pub fn world(&mut self) -> &mut World {
        unsafe { &mut *self.world }
    }

    /// The page's document, wherever it is now.
    pub fn doc(&mut self) -> Option<&mut BaseDocument> {
        match self.doc {
            Some(d) => Some(unsafe { &mut *d }),
            None => unsafe { &mut *self.world }.non_send_mut::<Docs>().into_inner().0.get_mut(&self.page),
        }
    }

    /// The ids the root's ops named, and the document they're in.
    pub fn nodes(&mut self) -> &mut Nodes {
        unsafe { &mut *self.nodes }
    }

    /// What the JS has flushed so far, committed to the page now, as the next drain would: an autofocus it moved is
    /// told to the root, and its step syncs the twins. For a call that answers from the page as the JS wrote it
    /// (style.computed).
    pub fn commit(&mut self) {
        let batches = std::mem::take(&mut self.out.borrow_mut().flushed);
        if batches.is_empty() {
            return;
        }
        let (nodes, queue) = (self.nodes, self.queue);
        let Some(doc) = self.doc() else { return };
        commit_focus(doc, unsafe { &mut *nodes }, batches, queue.map(|q| unsafe { &mut *q }));
        if let Some(c) = self.synced {
            unsafe { *c = true };
        }
    }
}

impl Root {
    /// `f` on its JS, the page lent to the context calls it makes (PageSync).
    fn js<T>(&mut self, world: &mut World, f: impl FnOnce(&mut dyn JsEngine) -> T) -> T {
        let mut ctx = PageSync { world, page: self.page, doc: None, nodes: &mut self.nodes, out: self.out.clone(), queue: Some(&mut self.queue), synced: Some(&mut self.synced) };
        raea_runtime_js::calls::with_sync_ctx(&mut ctx, || f(self.js.as_mut()))
    }
}
