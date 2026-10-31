//! The bridge between a page and the world (docs/blitz-bevy.feature B3; names from docs/sdk.feature S4). Every
//! element of every page has a Bevy entity, its twin, carrying the element's box (H1); Bevy systems change elements
//! through [`PageCommands`], applied before restyle in the same frame (H2); and the page's work each frame runs in
//! [`PageSystems`]' sets, in H8's order, for other plugins to order their systems against. Every DOM event is also a
//! Bevy event on the twins of its path, after React has had it (H3: [`dom`]). On a React page, each property has one
//! owner (H9): what React set is React's and what a system set first is Bevy's, and a write to the other's is refused
//! and reported as a [`PageConflict`], never a silent fight.
//!
//! A twin is keyed by what names its element for good: a React page's op id (host.rs's roots; Blitz reuses a dropped
//! node's slot, so a node id can come back as another element), an HTML page's node id with its tag (its nodes are
//! rarely dropped; a reused slot with another tag is a new twin).

use std::collections::{BTreeMap, HashMap};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use blitz_dom::{BaseDocument, Document, EventHandler, NodeId};
use blitz_traits::events::{DomEvent, DomEventData, EventState};

use raea_sys_page::{Docs, Page, PageSystems, Rendered};

mod claims;
pub use claims::*;
#[cfg(test)]
mod tests;

/// A twin: the page entity its element is in and the element's node (H1).
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct PageElement {
    pub page: Entity,
    pub node: NodeId,
    /// a React page's op id for the element (what keys the twin), None in an HTML page
    pub op: Option<u64>,
}

/// The element's border box in its page's CSS px, as last laid out; written only when it moves or resizes.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct ElementBox(pub Rect);

/// A plain App's pages (no host Root): each element's node to the id its author knows it by (React's, a ref's
/// `current.id`), which its twin's `op` carries. raea-runtime-react's WorldPagePlugin writes it as it syncs a page.
#[derive(Resource, Default)]
pub struct TwinOps(pub HashMap<Entity, HashMap<NodeId, u64>>);

/// Every page's twins, by key and by node.
#[derive(Resource, Default)]
pub struct Twins {
    pages: HashMap<Entity, PageTwins>,
}

#[derive(Default)]
struct PageTwins {
    by_key: HashMap<Key, Entity>,
    by_node: HashMap<NodeId, Entity>,
    /// each twin's components as the twins step last wrote them: a twin found otherwise was written by a system (H9)
    written: HashMap<Entity, (PageElement, ElementBox)>,
    /// the step's buffers, kept from frame to frame so a frame where nothing changed allocates nothing: the elements
    /// now, the tree walk's stack, and the key map being built (swapped with `by_key`)
    now: Vec<(Key, NodeId)>,
    stack: Vec<NodeId>,
    next: HashMap<Key, Entity>,
}

/// A twin's key: an op id, or a node id and its tag (an atom: a clone allocates nothing).
#[derive(Clone, PartialEq, Eq, Hash)]
enum Key {
    Op(u64),
    Node(NodeId, blitz_dom::LocalName),
}

impl Twins {
    /// The twin of `node` in `page`.
    pub fn of(&self, page: Entity, node: NodeId) -> Option<Entity> {
        self.pages.get(&page)?.by_node.get(&node).copied()
    }
}

/// Finding twins by id, class or selector (H1): `PageQuery` reads the documents, so it's for systems on the main
/// thread (the documents are a non-send resource).
#[derive(SystemParam)]
pub struct PageQuery<'w> {
    docs: NonSend<'w, Docs>,
    twins: Res<'w, Twins>,
}

impl PageQuery<'_> {
    /// The twin of the first element in `page` that `selector` matches.
    pub fn one(&self, page: Entity, selector: &str) -> Option<Entity> {
        let node = self.docs.0.get(&page)?.query_selector(selector).ok()??;
        self.twins.of(page, node)
    }

    /// The twins of every element in `page` that `selector` matches, in document order; none for a bad selector.
    pub fn all(&self, page: Entity, selector: &str) -> Vec<Entity> {
        let Some(doc) = self.docs.0.get(&page) else { return vec![] };
        doc.query_selector_all(selector).map(|ns| ns.iter().filter_map(|n| self.twins.of(page, *n)).collect()).unwrap_or_default()
    }

    /// The twin of the element whose id is `id`.
    pub fn by_id(&self, page: Entity, id: &str) -> Option<Entity> {
        self.one(page, &format!("#{}", css_ident(id)))
    }

    /// The twins of the elements with the class `class`.
    pub fn by_class(&self, page: Entity, class: &str) -> Vec<Entity> {
        self.all(page, &format!(".{}", css_ident(class)))
    }
}

/// `s` escaped as a CSS identifier (CSS.escape's rules, enough for ids and classes).
fn css_ident(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        match c {
            'a'..='z' | 'A'..='Z' | '_' | '-' => out.push(c),
            '0'..='9' if i > 0 => out.push(c),
            c if !c.is_ascii() => out.push(c),
            c => out.push_str(&format!("\\{:x} ", c as u32)),
        }
    }
    out
}

/// One queued change to an element (H2).
#[derive(Clone, Debug)]
pub enum PageOp {
    SetStyle(String, String),
    RemoveStyle(String),
    SetAttribute(String, String),
    RemoveAttribute(String),
    AddClass(String),
    RemoveClass(String),
    SetText(String),
}

/// The changes queued this frame, by twin.
#[derive(Resource, Default)]
pub struct PageQueue(Vec<(Entity, PageOp)>);

/// Bevy to page (H2): a system queues changes to elements by their twins, as with Commands; they're applied in
/// [`PageSystems::Commands`], before restyle in the same frame.
#[derive(SystemParam)]
pub struct PageCommands<'w> {
    queue: ResMut<'w, PageQueue>,
}

impl PageCommands<'_> {
    pub fn set_style(&mut self, twin: Entity, property: &str, value: &str) {
        self.queue.0.push((twin, PageOp::SetStyle(property.into(), value.into())));
    }
    pub fn remove_style(&mut self, twin: Entity, property: &str) {
        self.queue.0.push((twin, PageOp::RemoveStyle(property.into())));
    }
    pub fn set_attribute(&mut self, twin: Entity, name: &str, value: &str) {
        self.queue.0.push((twin, PageOp::SetAttribute(name.into(), value.into())));
    }
    pub fn remove_attribute(&mut self, twin: Entity, name: &str) {
        self.queue.0.push((twin, PageOp::RemoveAttribute(name.into())));
    }
    pub fn add_class(&mut self, twin: Entity, class: &str) {
        self.queue.0.push((twin, PageOp::AddClass(class.into())));
    }
    pub fn remove_class(&mut self, twin: Entity, class: &str) {
        self.queue.0.push((twin, PageOp::RemoveClass(class.into())));
    }
    /// The element's children replaced by one text node.
    pub fn set_text(&mut self, twin: Entity, text: &str) {
        self.queue.0.push((twin, PageOp::SetText(text.into())));
    }
}

pub struct BridgePlugin<R: OpIds = ()>(std::marker::PhantomData<fn() -> R>);

impl<R: OpIds> Default for BridgePlugin<R> {
    fn default() -> Self {
        BridgePlugin(std::marker::PhantomData)
    }
}

/// Each React root's op ids by its page (raea-runtime-dom's Roots), so a twin keeps its element's op id; `()` for an
/// App whose pages are only HTML.
pub trait OpIds: 'static {
    fn op_ids(&self, page: Entity) -> Option<&HashMap<u64, NodeId>>;
}

impl OpIds for () {
    fn op_ids(&self, _: Entity) -> Option<&HashMap<u64, NodeId>> {
        None
    }
}

impl<R: OpIds> Plugin for BridgePlugin<R> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Twins>()
            .init_resource::<TwinOps>()
            .init_resource::<PageQueue>()
            .init_resource::<Claims>()
            .add_message::<PageConflict>()
            .configure_sets(PreUpdate, PageSystems::Step.after(PageSystems::Events).after(bevy::picking::PickingSystems::Last))
            .configure_sets(PostUpdate, (PageSystems::Commands, PageSystems::Restyle, PageSystems::Layout, PageSystems::Twins, PageSystems::Paint).chain())
            .add_systems(PostUpdate, (apply_commands.in_set(PageSystems::Commands), twins::<R>.in_set(PageSystems::Twins)));
    }
}

/// Each page's elements now, by key, into `out`: a React page's from its ops' ids, an HTML page's by walking its tree.
fn elements(doc: &BaseDocument, ops: Option<&HashMap<u64, NodeId>>, out: &mut Vec<(Key, NodeId)>, stack: &mut Vec<NodeId>) {
    out.clear();
    if let Some(ops) = ops {
        out.extend(ops.iter().filter(|(_, n)| doc.get_node(**n).is_some_and(|n| n.is_element())).map(|(id, n)| (Key::Op(*id), *n)));
        return;
    }
    stack.clear();
    stack.push(doc.root_node().id);
    while let Some(id) = stack.pop() {
        let Some(n) = doc.get_node(id) else { continue };
        if let Some(el) = n.element_data() {
            out.push((Key::Node(id, el.name.local.clone()), id));
        }
        stack.extend(n.children.iter().rev());
    }
}

/// Twins spawned for new elements and despawned with theirs (and with their page), and each one's box written when it
/// changed (H1). A twin's components are the page's (H9): one a system wrote is put back, named.
fn twins<R: OpIds>(
    mut commands: Commands,
    docs: NonSend<Docs>,
    roots: Option<NonSend<R>>,
    plain: Res<TwinOps>,
    pages: Query<(), With<Page>>,
    mut index: ResMut<Twins>,
    mut twins: Query<(&mut PageElement, &mut ElementBox)>,
    parents: Query<Option<&ChildOf>, With<PageElement>>,
    mut conflicts: MessageWriter<PageConflict>,
) {
    let mut report = |twin: Entity, property: &str| {
        error!("raea-page: a system wrote {property} on {twin}, which is the page's; the page's value stays (docs/blitz-bevy.feature B3 H9)");
        conflicts.write(PageConflict { twin, property: property.into(), owner: Owner::Page });
    };
    let index = &mut *index;
    // pages gone: their twins with them
    index.pages.retain(|page, t| {
        let keep = docs.0.contains_key(page) && pages.contains(*page);
        if !keep {
            for e in t.by_key.values() {
                commands.entity(*e).try_despawn();
            }
        }
        keep
    });
    for (page, doc) in &docs.0 {
        if !pages.contains(*page) {
            continue;
        }
        let ops = roots.as_ref().and_then(|r| r.op_ids(*page));
        let t = index.pages.entry(*page).or_default();
        let mut now = std::mem::take(&mut t.now);
        elements(doc, ops, &mut now, &mut t.stack);
        let mut by_key = std::mem::take(&mut t.next);
        by_key.clear();
        t.by_node.clear();
        for (key, node) in now.drain(..) {
            let at = doc.get_client_bounding_rect(node).map(|b| ElementBox(Rect::new(b.x as f32, b.y as f32, (b.x + b.width) as f32, (b.y + b.height) as f32))).unwrap_or_default();
            let op = if let Key::Op(id) = key { Some(id) } else { plain.0.get(page).and_then(|m| m.get(&node)).copied() };
            let el_now = PageElement { page: *page, node, op };
            let e = match t.by_key.remove(&key) {
                Some(e) => {
                    if let Ok((mut el, mut b)) = twins.get_mut(e) {
                        if let Some((was_el, was_b)) = t.written.get(&e) {
                            if *el != *was_el {
                                report(e, "PageElement");
                            }
                            if *b != *was_b {
                                report(e, "ElementBox");
                            }
                        }
                        el.set_if_neq(el_now);
                        b.set_if_neq(at);
                    }
                    e
                }
                None => commands.spawn((el_now, at)).id(),
            };
            t.written.insert(e, (el_now, at));
            by_key.insert(key, e);
            t.by_node.insert(node, e);
        }
        // the twins' hierarchy is the elements': each a child of its nearest element's twin up the tree, the topmost
        // the page's, so Bevy's own bubbling (picking's, ChildOf) runs the DOM's path. Set before the gone are
        // despawned, so a moved element's twin isn't despawned with its old parent's.
        for (node, e) in &t.by_node {
            let mut up = doc.get_node(*node).and_then(|n| n.parent);
            let parent = loop {
                match up {
                    Some(p) => match t.by_node.get(&p) {
                        Some(pe) => break *pe,
                        None => up = doc.get_node(p).and_then(|n| n.parent),
                    },
                    None => break *page,
                }
            };
            if parents.get(*e).ok().flatten().map(|c| c.parent()) != Some(parent) {
                commands.entity(*e).try_insert(ChildOf(parent));
            }
        }
        // what's left: elements gone
        for e in t.by_key.values() {
            commands.entity(*e).try_despawn();
            t.written.remove(e);
        }
        t.next = std::mem::replace(&mut t.by_key, by_key);
        t.now = now;
    }
}

/// Page to Bevy (H3): each DOM event as a Bevy EntityEvent of its own kind, as Bevy's picking events are, observed
/// with `On<dom::Click>` beside `On<Pointer<Click>>`. It's triggered on the twin of each element in its path, target
/// first (only the target's for events that don't bubble), after React's handlers ran; `target` is the target's twin,
/// `data` Blitz's payload, and `default_prevented` and `propagation_stopped` what the page did with it.
pub mod dom {
    use bevy::prelude::*;
    use blitz_traits::events::*;
    // the data's types, for a handler outside raea-page
    pub use blitz_traits::events::{BlitzFocusEvent, BlitzPointerEvent, BlitzPointerId, BlitzWheelDelta, MouseEventButton};
    pub use blitz_traits::NodeId;

    macro_rules! dom_events {
        ($($kind:ident($data:ty)),* $(,)?) => {
            $(
                #[derive(EntityEvent, Clone, Debug)]
                pub struct $kind {
                    pub entity: Entity,
                    pub target: Entity,
                    pub data: $data,
                    pub default_prevented: bool,
                    pub propagation_stopped: bool,
                }
            )*

            /// `data` triggered on `at` as its kind's event.
            pub(super) fn trigger(commands: &mut Commands, at: Entity, target: Entity, data: &DomEventData, default_prevented: bool, propagation_stopped: bool) {
                match data {
                    $(DomEventData::$kind(d) => commands.trigger($kind { entity: at, target, data: d.clone(), default_prevented, propagation_stopped }),)*
                }
            }
        };
    }

    dom_events! {
        PointerMove(BlitzPointerEvent), PointerDown(BlitzPointerEvent), PointerUp(BlitzPointerEvent),
        PointerCancel(BlitzPointerEvent), PointerEnter(BlitzPointerEvent), PointerLeave(BlitzPointerEvent),
        PointerOver(BlitzPointerEvent), PointerOut(BlitzPointerEvent),
        MouseMove(BlitzPointerEvent), MouseDown(BlitzPointerEvent), MouseUp(BlitzPointerEvent),
        MouseEnter(BlitzPointerEvent), MouseLeave(BlitzPointerEvent), MouseOver(BlitzPointerEvent), MouseOut(BlitzPointerEvent),
        TouchStart(BlitzPointerEvent), TouchMove(BlitzPointerEvent), TouchEnd(BlitzPointerEvent), TouchCancel(BlitzPointerEvent),
        Scroll(BlitzScrollEvent), Wheel(BlitzWheelEvent),
        Click(BlitzPointerEvent), ContextMenu(BlitzPointerEvent), DoubleClick(BlitzPointerEvent),
        KeyPress(BlitzKeyEvent), KeyDown(BlitzKeyEvent), KeyUp(BlitzKeyEvent), Input(BlitzInputEvent), Ime(BlitzImeEvent),
        Focus(BlitzFocusEvent), Blur(BlitzFocusEvent), FocusIn(BlitzFocusEvent), FocusOut(BlitzFocusEvent),
        AppleStandardKeybinding(blitz_traits::SmolStr),
    }

    /// An HTML drag's state as an event of it has it (@raea/ui's dnd.ts makes the drag from the pointer events, as
    /// Chrome does, so Blitz has none of its own): where the pointer is in the page, in CSS px, the effects agreed so
    /// far, and the types of the data dragged
    #[derive(Clone, Debug, Default)]
    pub struct DragData {
        pub x: f32,
        pub y: f32,
        pub drop_effect: String,
        pub effect_allowed: String,
        pub types: Vec<String>,
    }

    macro_rules! drag_events {
        ($($kind:ident = $name:literal),* $(,)?) => {
            $(
                #[derive(EntityEvent, Clone, Debug)]
                pub struct $kind {
                    pub entity: Entity,
                    pub target: Entity,
                    pub data: DragData,
                    pub default_prevented: bool,
                    pub propagation_stopped: bool,
                }
            )*

            /// The DOM names of the drag events (dragstart ... dragend)
            pub const DRAG_KINDS: &[&str] = &[$($name),*];

            /// a drag event `name` triggered on `at` as its kind's event
            pub(super) fn trigger_drag(commands: &mut Commands, at: Entity, target: Entity, name: &str, data: &DragData, default_prevented: bool, propagation_stopped: bool) {
                match name {
                    $($name => commands.trigger($kind { entity: at, target, data: data.clone(), default_prevented, propagation_stopped }),)*
                    _ => {}
                }
            }
        };
    }

    drag_events! {
        DragStart = "dragstart", Drag = "drag", DragEnter = "dragenter", DragOver = "dragover", DragLeave = "dragleave",
        Drop = "drop", DragEnd = "dragend",
    }
}

/// One DOM event as the page left it: its path (target first), its data, and whether it was prevented or stopped.
pub struct Fired {
    path: Vec<NodeId>,
    data: Data,
    prevented: bool,
    stopped: bool,
}

/// A fired event's data: Blitz's, or an HTML drag's (by its DOM name, one of dom::DRAG_KINDS)
enum Data {
    Dom(DomEventData),
    Drag(String, dom::DragData),
}

impl Fired {
    /// An HTML drag event `name` as the page's script fired it, on `path` (target first)
    pub fn drag(path: Vec<NodeId>, name: &str, data: dom::DragData, prevented: bool, stopped: bool) -> Fired {
        Fired { path, data: Data::Drag(name.to_string(), data), prevented, stopped }
    }
}

/// A page's event handler with H3's tap: each event handed to `inner` (the host's: React's handlers and its commit),
/// then kept, as the page left it, for [`deliver`].
pub struct Forward<'a, H> {
    pub inner: H,
    pub fired: &'a mut Vec<Fired>,
}

impl<H: EventHandler> EventHandler for Forward<'_, H> {
    fn handle_event(&mut self, chain: &[NodeId], event: &mut DomEvent, doc: &mut dyn Document, state: &mut EventState) {
        self.inner.handle_event(chain, event, doc, state);
        let path = if event.bubbles && !chain.is_empty() { chain.to_vec() } else { vec![event.target] };
        self.fired.push(Fired { path, data: Data::Dom(event.data.clone()), prevented: state.is_cancelled(), stopped: state.propagation_is_stopped() });
    }
}

/// `page`'s fired events triggered on its twins, at once (the input's own frame): each on every twin up its path.
/// The twins are brought up to the page first, so an element the event's own commit made (or remade) has one, there
/// for the event's path and for whatever runs in the frame after it (its box comes at [`PageSystems::Twins`]).
/// The twins brought to the pages now, outside their set: before an event's delivery, after a host step's commits.
pub fn sync_twins<R: OpIds>(world: &mut World) {
    if let Err(e) = world.run_system_cached(twins::<R>) {
        warn!("raea-page: twins: {e}");
    }
}

pub fn deliver<R: OpIds>(world: &mut World, page: Entity, fired: Vec<Fired>) {
    if fired.is_empty() {
        return;
    }
    sync_twins::<R>(world);
    let Some(twins) = world.get_resource::<Twins>() else { return };
    let mut out = vec![];
    for f in fired {
        let path: Vec<Entity> = f.path.iter().filter_map(|n| twins.of(page, *n)).collect();
        let Some(target) = path.first().copied() else { continue };
        out.push((path, target, f));
    }
    let mut commands = world.commands();
    for (path, target, f) in &out {
        for at in path {
            match &f.data {
                Data::Dom(d) => dom::trigger(&mut commands, *at, *target, d, f.prevented, f.stopped),
                Data::Drag(name, d) => dom::trigger_drag(&mut commands, *at, *target, name, d, f.prevented, f.stopped),
            }
        }
    }
    world.flush();
}
