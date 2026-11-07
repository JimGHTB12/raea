//! A root's JS run: timers, commits applied to the document, focus, and the drain of host calls, asks and posts.

use super::*;

/// The timers due now, one call each, as raea-runtime-js's host runs them (a timer's microtasks before the next timer's).
pub(crate) fn timers(js: &mut dyn JsEngine) {
    let now = raea_runtime_js::js::now_ms();
    for _ in 0..1000 {
        match js.call("__ui_timers", &[Arg::Num(now)]) {
            Some(Ok(n)) if n == 0.0 => {}
            Some(Err(e)) => {
                eprintln!("raea-page: js: a timer threw: {e}");
                break;
            }
            _ => break,
        }
    }
}

/// What takes a page's ops before its document does, and hands back the document's: raea-rune-slots', whose Rune
/// layers draw what's under their slots (T6c). None: every op is the document's.
pub type Split = Box<dyn FnMut(Entity, Vec<wire::Op>) -> Vec<wire::Op>>;
thread_local!(static SPLIT: std::cell::RefCell<Option<Split>> = const { std::cell::RefCell::new(None) });

/// `f` as every root's [`Split`] on this thread (the page host's).
pub fn set_split(f: Split) {
    SPLIT.with_borrow_mut(|s| *s = Some(f));
}

/// Page `page`'s op batches applied to its document.
pub(crate) fn commit(page: Entity, doc: &mut BaseDocument, nodes: &mut Nodes, batches: Vec<String>) {
    for b in batches {
        match wire::decode(&b) {
            Ok(o) => ops::apply(doc, nodes, SPLIT.with_borrow_mut(|s| match s.as_mut() { Some(f) => f(page, o), None => o })),
            Err(e) => eprintln!("raea-page: {e}"),
        }
    }
}

/// `batches` committed to `doc`; a focus the commit moved (an autofocus) is told to the root (`queue`), as a focus
/// event its element hears.
pub(crate) fn commit_focus(page: Entity, doc: &mut BaseDocument, nodes: &mut Nodes, batches: Vec<String>, queue: Option<&mut Vec<Value>>) {
    let before = doc.get_focussed_node_id();
    commit(page, doc, nodes, batches);
    if let Some(q) = queue
        && let Some(n) = doc.get_focussed_node_id().filter(|n| Some(*n) != before)
        && let Some(id) = nodes.id_of(n).filter(|id| nodes.hears(*id, Ev::Focus))
    {
        q.push(wire::event(id, Ev::Focus, Value::Null));
    }
}

/// What the root's JS handed over since it was last drained: its commits applied, its calls run, its emits acted on.
/// Whether it committed (a context call's commit since counts).
pub(crate) fn drain(world: &mut World, r: &mut Root) -> bool {
    let (batches, requests, emits) = {
        let mut o = r.out.borrow_mut();
        (std::mem::take(&mut o.flushed), std::mem::take(&mut o.requests), std::mem::take(&mut o.emits))
    };
    let committed = !batches.is_empty() || std::mem::take(&mut r.synced);
    if !batches.is_empty()
        && let Some(doc) = world.non_send_mut::<Docs>().0.get_mut(&r.page)
    {
        commit_focus(r.page, doc, &mut r.nodes, batches, Some(&mut r.queue));
    }
    // a scroller a commit moved (its scrollTop set, here or in an event's delivery): its scroll event, as Chrome
    // sends one after
    if !r.nodes.scrolled.is_empty()
        && let Some(doc) = world.non_send::<Docs>().0.get(&r.page)
    {
        for id in std::mem::take(&mut r.nodes.scrolled) {
            let Some(node) = ops::node(&r.nodes, id).and_then(|n| doc.get_node(n)) else { continue };
            if r.nodes.hears(id, Ev::Scroll) {
                let at = node.scroll_offset();
                r.queue.push(wire::event(id, Ev::Scroll, json!({"scrollTop": at.y, "scrollLeft": at.x, "scrollHeight": node.scroll_height(), "scrollWidth": node.scroll_width(), "clientHeight": node.client_height(), "clientWidth": node.client_width()})));
            }
        }
    }
    // video and audio: sources opened, seeks asked, and what their decoders told (media.rs)
    if let Some(doc) = world.non_send_mut::<Docs>().0.get_mut(&r.page) {
        let mut told = vec![];
        crate::media::sync(doc, &r.nodes, &mut r.medias, &mut told, !r.placed_videos);
        r.queue.extend(told.into_iter().map(|v| wire::named("media", v)));
    }
    // view3d boxes: cameras, images and parts kept to the document (its doc taken out while the world is used)
    if r.nodes.has_kind("view3d") || !r.views.is_empty() {
        let scale = world.get::<raea_sys_page::Page>(r.page).map_or(1.0, |p| p.scale);
        if let Some(mut doc) = world.non_send_mut::<Docs>().0.remove(&r.page) {
            raea_sys_page::view3d::sync(world, &mut doc, &r.nodes, &mut r.views, scale);
            world.non_send_mut::<Docs>().0.insert(r.page, doc);
        }
    }
    // an img's source in or failed: raea-runtime-js's media event (sdk/ui's dom layer fires `load` or `error` at it)
    if let Some(doc) = world.non_send_mut::<Docs>().0.get_mut(&r.page)
        && !doc.image_loads.is_empty()
    {
        for (n, size) in std::mem::take(&mut doc.image_loads) {
            let Some(id) = r.nodes.id_of(n) else { continue };
            r.queue.push(wire::named("media", match size {
                Some((w, h)) => json!({"id": id, "type": "load", "w": w, "h": h}),
                None => json!({"id": id, "type": "error"}),
            }));
        }
    }
    // document.fonts: answered a drain after it asked at the soonest (the sheet its commit brought has started its
    // @font-face fetches by then), once none is still loading
    if let Some(n) = r.fonts_wait {
        let loading = world.non_send::<Docs>().0.get(&r.page).is_some_and(|d| d.has_pending_fonts());
        if n > 0 && !loading {
            r.queue.push(wire::named("fonts.ready", Value::Null));
            r.fonts_wait = None;
        } else {
            r.fonts_wait = Some(n + 1);
        }
    }
    for (id, name, v) in requests {
        // an opaque frame reaches nothing of the app's (B5c iframes 6)
        if r.frame.as_ref().is_some_and(|f| f.opaque) {
            r.queue.push(wire::answer(id, Err(format!("SecurityError: a sandboxed frame can't call the host ({name})"))));
            continue;
        }
        raea_runtime_js::calls::call(world, r.page, id, &name, serde_json::from_str(&v).unwrap_or(Value::Null));
    }
    for (name, v) in emits {
        let v: Value = serde_json::from_str(&v).unwrap_or(Value::Null);
        match name.as_str() {
            // every Blitz root's events are a browser's; a `prevented` outside an event's delivery has nothing to stop
            "dom" | "prevented" => {}
            "ticks" => r.ticking = v.as_bool().unwrap_or(false),
            "fonts.wait" => r.fonts_wait = r.fonts_wait.or(Some(0)),
            "blur" => {
                if let Some(doc) = world.non_send_mut::<Docs>().0.get_mut(&r.page) {
                    doc.clear_focus();
                }
            }
            // an element by its wire id (a ref's focus) or its data-testid
            "focus" => {
                let mut docs = world.non_send_mut::<Docs>();
                let Some(doc) = docs.0.get_mut(&r.page) else { continue };
                let node = v.as_u64().and_then(|id| ops::node(&r.nodes, id)).or_else(|| {
                    let t = v.as_str()?;
                    doc.query_selector(&format!("[data-testid=\"{}\"]", t.replace('"', "\\\""))).ok().flatten()
                });
                if let Some(n) = node {
                    doc.set_focus_to(n);
                }
            }
            "capture" => r.capture = v.as_u64(),
            // body's or html's pointer-events (a modal's `body { pointer-events: none }`): the root's, which the page's
            // base sheet gives `[raea-inert]`
            "rootPointerEvents" => {
                let mut docs = world.non_send_mut::<Docs>();
                let Some(doc) = docs.0.get_mut(&r.page) else { continue };
                let mut m = doc.mutate();
                if v.as_str() == Some("none") {
                    m.set_attribute(r.nodes.root, ops::qual("raea-inert"), "");
                } else {
                    m.clear_attribute(r.nodes.root, ops::qual("raea-inert"));
                }
            }
            // an HTML drag event dnd.ts fired, for the twins of its path (S4a), delivered after the input's step
            "drag" => {
                let path = v["path"].as_array().map(|p| p.iter().filter_map(|w| w.as_u64().and_then(|w| ops::node(&r.nodes, w))).collect()).unwrap_or_default();
                let data = raea_runtime_twins::bridge::dom::DragData {
                    x: v["x"].as_f64().unwrap_or(0.0) as f32,
                    y: v["y"].as_f64().unwrap_or(0.0) as f32,
                    drop_effect: v["dropEffect"].as_str().unwrap_or("none").to_string(),
                    effect_allowed: v["effectAllowed"].as_str().unwrap_or("uninitialized").to_string(),
                    types: serde_json::from_value(v["types"].clone()).unwrap_or_default(),
                };
                let fired = raea_runtime_twins::bridge::Fired::drag(path, v["type"].as_str().unwrap_or(""), data, v["prevented"] == Value::Bool(true), v["stopped"] == Value::Bool(true));
                match world.get_non_send_mut::<Drags>() {
                    Some(mut d) => d.0.push((r.page, fired)),
                    None => world.insert_non_send(Drags(vec![(r.page, fired)])),
                }
            }
            "hoverFreeze" => {
                let docs = world.non_send::<Docs>();
                r.hover_frozen = (v == Value::Bool(true))
                    .then(|| docs.0.get(&r.page).map(|d| d.get_hover_node_id().map(|h| d.node_chain(h)).unwrap_or_default()))
                    .flatten();
            }
            // a frame's message to its parent's window, and the parent's to a frame's (contentWindow.postMessage)
            // a frame's message to its parent's window (the parent is http://localhost, as @raea/ui's pages are: a
            // target origin that isn't it, or "*", drops it as a browser does)
            "frame.post" => {
                if let Some(f) = &r.frame
                    && matches!(v["origin"].as_str(), Some("*" | "http://localhost" | "/"))
                {
                    let origin = if f.opaque { "null" } else { "http://localhost" }.to_string();
                    world.get_resource_or_insert_with(FramePosts::default).0.push(Post::ToParent { parent: f.parent, wid: f.wid, data: v["data"].to_string(), origin });
                }
            }
            // the parent's message to a frame's window (contentWindow.postMessage)
            "frame.postTo" => {
                if let Some(wid) = v["frame"].as_u64() {
                    let target = v["origin"].as_str().unwrap_or("*").to_string();
                    world.get_resource_or_insert_with(FramePosts::default).0.push(Post::ToFrame { parent: r.page, wid, data: v["data"].to_string(), target });
                }
            }
            // what reaches the app (watches, actions, asks) a frame's root doesn't
            _ if r.frame.is_some() => {}
            "watch" => raea_runtime_js::world::watch(world, r.page, &v),
            "unwatch" => raea_runtime_js::world::unwatch(world, r.page, &v),
            "actions" => world.resource_mut::<SpaceActions>().set(r.page, v),
            "actionDone" => world.resource_mut::<SpaceActions>().done(&v),
            _ => world.resource_mut::<PageAsks>().0.push((r.page, name, v)),
        }
    }
    committed
}

/// HTML drag events the pages' scripts fired, each with its page, waiting for their twins (step.rs's input)
#[derive(Default)]
pub(crate) struct Drags(pub Vec<(Entity, raea_runtime_twins::bridge::Fired)>);

/// A host event (resize, scan, agent...) for every root at its next step.
pub fn post(world: &mut World, name: &str, value: Value) {
    if let Some(mut roots) = world.get_non_send_mut::<Roots>() {
        for r in &mut roots.0 {
            r.queue.push(wire::named(name, value.clone()));
        }
    }
}

/// A host event for one root at its next step.
pub fn post_to(world: &mut World, page: Entity, name: &str, value: Value) {
    deliver_to(world, page, wire::named(name, value));
}

/// A message for one root's `__ui_deliver` at its next step (an element's event a Rune layer heard).
pub fn deliver_to(world: &mut World, page: Entity, msg: Value) {
    if let Some(r) = world.get_non_send_mut::<Roots>().as_mut().and_then(|r| r.get_mut(page)) {
        r.queue.push(msg);
    }
}
