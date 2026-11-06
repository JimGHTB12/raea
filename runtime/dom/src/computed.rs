//! getComputedStyle on the Blitz page (docs/blitz-bevy.feature B5d): `style.computed`, a synchronous host call (sdk/ui's
//! `wire.callSync`) answered by stylo from the page as its JS sees it. What the JS wrote so far is committed and the
//! document restyled and laid out, as a browser flushes style and layout for getComputedStyle, then each property's
//! resolved value is read off the element (blitz-dom's resolved_style_value: used values for sizes, margins, insets
//! and transforms). sdk/ui asks it for every wired element of an HTML page; its own cascade answers only where the host
//! has none (an unwired element, a call outside a step). `layout.now` is the same flush for a box script reads.

use bevy::prelude::*;
use serde_json::{Map, Value};

use crate::host::PageSync;
use raea_sys_page::ops;

/// Registers `style.computed`: `{id, names}` (a wire id, kebab-case names) to `{name: value}`; and `layout.now`.
pub fn register() {
    raea_runtime_js::calls::on_sync_ctx("style.computed", computed);
    raea_runtime_js::calls::on_sync_ctx("layout.now", layout_now);
}

/// `layout.now`: the page laid out as its JS wrote it so far, and its snapshot (what the next `__ui_layout` would
/// give), as a browser lays out when script reads a box it changed. sdk/ui asks when an element it placed has no box
/// yet: Excalidraw sizes its opacity slider's label from the input's offsetWidth in the effect after it mounts.
fn layout_now(ctx: &mut dyn std::any::Any, _args: &Value) -> Result<Value, String> {
    let p = ctx.downcast_mut::<PageSync>().ok_or("layout.now: not a page's call")?;
    let at = {
        let w = p.world();
        w.resource::<raea_sys_page::PageClock>().0.unwrap_or_else(|| w.resource::<Time>().elapsed_secs_f64())
    };
    p.commit();
    let nodes = p.nodes() as *const raea_sys_page::ops::Nodes;
    let doc = p.doc().ok_or("layout.now: the page has no document")?;
    doc.resolve(at);
    // (the ids and the document are separate borrows of what the JS call lent)
    Ok(Value::String(crate::host::snapshot(doc, unsafe { &*nodes })))
}

fn computed(ctx: &mut dyn std::any::Any, args: &Value) -> Result<Value, String> {
    let p = ctx.downcast_mut::<PageSync>().ok_or("style.computed: not a page's call")?;
    let id = args["id"].as_u64().ok_or("style.computed: no id")?;
    let names: Vec<&str> = args["names"].as_array().map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
    let at = {
        let w = p.world();
        w.resource::<raea_sys_page::PageClock>().0.unwrap_or_else(|| w.resource::<Time>().elapsed_secs_f64())
    };
    // what the JS wrote so far, in the document as drain would put it (its focus and twins seen to after)
    p.commit();
    let node = ops::node(p.nodes(), id).ok_or_else(|| format!("style.computed: no element {id}"))?;
    let doc = p.doc().ok_or("style.computed: the page has no document")?;
    doc.resolve(at);
    let mut out = Map::new();
    for n in names {
        out.insert(n.to_string(), Value::String(doc.resolved_style_value(node, n)));
    }
    Ok(Value::Object(out))
}
