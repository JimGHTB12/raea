//! The page stepped each update: input in, focus moved and tabbed out of frames.

use super::*;

/// One frame of every root (raea-runtime-js's host's step): the frame number set, its timers run, then its host events and
/// answers delivered in order, each one's commit applied before the next.
pub fn step(world: &mut World) {
    let Some(mut roots) = world.remove_non_send::<Roots>() else { return };
    // the 3D views' images are listed again by this step's drains (view3d.rs)
    if let Some(mut v) = world.get_resource_mut::<raea_sys_page::view3d::ImageViews>() {
        v.0.clear();
    }
    let frame = world.get_resource::<raea_runtime_js::UiFrame>().map_or(0, |f| f.0);
    // what came across the roots' edge: answers to their calls, runs of their actions, watches whose answers changed
    let answers = world.get_resource::<HostCalls>().map(|c| c.answers()).unwrap_or_default();
    let asked = world.get_resource_mut::<SpaceActions>().map(|mut a| a.take_asked()).unwrap_or_default();
    let watched = raea_runtime_js::world::changed(world);
    for (page, m) in answers.into_iter().chain(asked).chain(watched) {
        if let Some(r) = roots.get_mut(page) {
            r.queue.push(m);
        }
    }
    let posts = world.get_resource_mut::<FramePosts>().map(|mut p| std::mem::take(&mut p.0)).unwrap_or_default();
    let mut committed = false;
    for r in &mut roots.0 {
        // a frame's root runs on its document, swapped out of its iframe for the step
        if let Some(f) = &r.frame
            && !swap_frame(world, r.page, f)
        {
            continue;
        }
        for p in &posts {
            let js = match (p, &r.frame) {
                (Post::ToParent { parent, wid, data, origin }, None) if *parent == r.page => format!("__raea_frames.deliver({wid}, {data}, {})", json!(origin)),
                // to an opaque frame only "*" reaches it; to another, "*" or its origin
                (Post::ToFrame { parent, wid, data, target }, Some(f)) if *parent == f.parent && *wid == f.wid && (target == "*" || (!f.opaque && target == "http://localhost")) => {
                    format!("__raea_frame.deliver({data}, \"http://localhost\")")
                }
                _ => continue,
            };
            if let Err(e) = r.js(world, |j| j.eval(&js, "message.js")) {
                eprintln!("raea-page: js: a message listener threw: {e}");
            }
        }
        if r.ticking {
            r.queue.push(wire::named("frame", json!(frame)));
        }
        let _ = r.js.call("__ui_setFrame", &[Arg::Num(frame as f64)]);
        r.js(world, timers);
        committed |= drain(world, r);
        // (a delivery can queue more: its answers come next frame, as raea-runtime-js's do)
        for m in std::mem::take(&mut r.queue) {
            if let Some(Err(e)) = r.js(world, |js| js.call("__ui_deliver", &[Arg::Str(&m.to_string())])) {
                eprintln!("raea-page: js: an event threw: {e}");
            }
            committed |= drain(world, r);
        }
        r.js(world, |js| js.idle());
        if let Some(f) = &r.frame {
            swap_frame(world, r.page, f);
        }
    }
    if raea_sys_page::iframes() {
        mount_frames(world, &mut roots);
    }
    world.insert_non_send(roots);
    // what the roots committed has its twins now, for this frame's game systems (H8), as an event's commit does (H3)
    if committed {
        raea_runtime_twins::bridge::sync_twins::<Roots>(world);
    }
}

/// Input for a root's page, as Blitz takes it (its pointer in page px): hit tested, dispatched to the elements'
/// listeners through the wire and then Blitz's default actions, before the next step.
pub fn input(world: &mut World, page: Entity, ev: UiEvent) {
    let Some(mut roots) = world.remove_non_send::<Roots>() else { return };
    let mut fired = None;
    // its scripted frames, and what lands on them
    let frames = frames_of(&roots, page);
    let mut forwards = vec![];
    // a frame's root takes input on its document, swapped out of its iframe
    let framed = roots.get_mut(page).and_then(|r| r.frame);
    // a Tab (true with Shift), which can move the focus into a frame or out of one
    let tab = match &ev {
        UiEvent::KeyDown(k) if k.key == keyboard_types::Key::Named(keyboard_types::NamedKey::Tab) => Some((k.modifiers.contains(keyboard_types::Modifiers::SHIFT), ev.clone())),
        _ => None,
    };
    let mut left = false;
    if framed.as_ref().is_some_and(|f| !swap_frame(world, page, f)) {
        world.insert_non_send(roots);
        return;
    }
    if let Some(r) = roots.get_mut(page) {
        let doc = world.non_send_mut::<Docs>().0.remove(&page);
        if let Some(mut doc) = doc {
            // the input's modality, for :focus-visible (a Tab's focus shows the ring, a press's doesn't)
            match &ev {
                UiEvent::KeyDown(_) => doc.focus_from_keyboard = true,
                UiEvent::PointerDown(_) => doc.focus_from_keyboard = false,
                _ => {}
            }
            // a captured pointer's moves and release go to what captured it (Blitz hit tests every move)
            // (the focus order walks the tree: only for a Tab in a frame)
            let (before, none) = (focus_of(&doc), tab.is_some() && framed.is_some() && doc.tab_order().is_empty());
            let captured = r.capture.and_then(|id| ops::node(&r.nodes, id));
            let bridge = Bridge { world: &mut *world, page, js: r.js.as_mut(), out: &r.out, nodes: &mut r.nodes, clicks: r.clicks, captured, bevy_ui: r.bevy_ui, frames: &frames, forwards: &mut forwards };
            // each event, as the page left it, also goes to its twins (bridge.rs, B3 H3)
            let mut events = vec![];
            EventDriver::new(&mut doc, raea_runtime_twins::bridge::Forward { inner: bridge, fired: &mut events }).handle_ui_event(ev);
            // a press in a frame focuses its iframe here, so the keys that follow are the frame's too
            for (_, n, e) in &forwards {
                if matches!(e, UiEvent::PointerDown(_)) && doc.get_focussed_node_id() != Some(*n) {
                    doc.set_focus_to(*n);
                }
            }
            if let Some((_, e)) = &tab {
                let after = focus_of(&doc);
                // a Tab onto a scripted frame goes on into it: its first element has the focus (Shift+Tab's, its
                // last), as in Chrome, where the iframe itself is no stop of its own
                if let Some(n) = after.filter(|n| Some(*n) != before && frames.contains_key(n)) {
                    forwards.push((frames[&n], n, e.clone()));
                }
                // past a frame's last element (or first), or into a frame with none: the focus goes on in its parent
                left = framed.is_some() && after.is_none() && (before.is_some() || none);
            }
            world.non_send_mut::<Docs>().0.insert(page, doc);
            fired = Some(events);
        }
    }
    // delivered with the roots in the World: the twins step keys a root's elements by their op ids
    if let Some(events) = fired {
        world.insert_non_send(roots);
        raea_runtime_twins::bridge::deliver::<Roots>(world, page, events);
        let Some(r) = world.remove_non_send::<Roots>() else { return };
        roots = r;
    }
    if let Some(r) = roots.get_mut(page) {
        drain(world, r);
    }
    if let Some(f) = &framed {
        swap_frame(world, page, f);
    }
    world.insert_non_send(roots);
    // the drags the input's handlers made, on their twins after the input's own events
    if let Some(super::commit::Drags(drags)) = world.remove_non_send::<super::commit::Drags>() {
        for (page, fired) in drags {
            raea_runtime_twins::bridge::deliver::<Roots>(world, page, vec![fired]);
        }
    }
    for (frame, _, e) in forwards {
        input(world, frame, e);
    }
    if left && let (Some(f), Some((back, e))) = (framed, tab) {
        tab_out(world, f.parent, f.node, back, &e);
    }
}

/// What has the focus, if anything: Blitz answers the root element for nothing.
pub(crate) fn focus_of(doc: &BaseDocument) -> Option<NodeId> {
    doc.get_focussed_node_id().filter(|n| Some(*n) != doc.try_root_element().map(|e| e.id))
}

/// A page's scripted <iframe>s, each its frame root's entity.
pub(crate) fn frames_of(roots: &Roots, page: Entity) -> HashMap<NodeId, Entity> {
    roots.0.iter().filter_map(|f| f.frame.as_ref().filter(|x| x.parent == page).map(|x| (x.node, f.page))).collect()
}

/// The focus leaving a frame by Tab (`back`: Shift+Tab): in its parent `page` it moves on from the iframe `from` to
/// the next element in the focus order (or the previous), which hears its focus; into the next frame if that's one,
/// and out past the parent's end to the parent's own parent if the parent is a frame too. The parent hears no key, as
/// in Chrome, where the Tab was the frame's.
pub(crate) fn tab_out(world: &mut World, page: Entity, from: NodeId, back: bool, e: &UiEvent) {
    let Some(mut roots) = world.remove_non_send::<Roots>() else { return };
    let frames = frames_of(&roots, page);
    let framed = roots.get_mut(page).and_then(|r| r.frame);
    if framed.as_ref().is_some_and(|f| !swap_frame(world, page, f)) {
        world.insert_non_send(roots);
        return;
    }
    let mut to = None;
    if let Some(r) = roots.get_mut(page)
        && let Some(doc) = world.non_send_mut::<Docs>().0.get_mut(&page)
    {
        doc.focus_from_keyboard = true;
        doc.set_focus_to(from);
        to = Some(if back { doc.focus_prev_node() } else { doc.focus_next_node() });
        if let Some(id) = to.flatten().filter(|n| !frames.contains_key(n)).and_then(|n| r.nodes.id_of(n)).filter(|id| r.nodes.hears(*id, Ev::Focus)) {
            r.queue.push(wire::event(id, Ev::Focus, Value::Null));
        }
    }
    if let Some(f) = &framed {
        swap_frame(world, page, f);
    }
    world.insert_non_send(roots);
    match (to, framed) {
        (Some(Some(n)), _) if frames.contains_key(&n) => input(world, frames[&n], e.clone()),
        (Some(None), Some(f)) => tab_out(world, f.parent, f.node, back, e),
        _ => {}
    }
}
