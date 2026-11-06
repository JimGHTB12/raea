//! raea-runtime-edit: text geometry and the selection's paint for sdk/ui's Range, Selection and contenteditable (docs/ts-native.feature
//! P14e), as synchronous host calls answered from Blitz's inline layout as the JS wrote it:
//! - `text.rects {spans: [[id, from, to], ...]}`: each text's client rects for its UTF-16 range [from, to) (a
//!   collapsed one: the caret's zero-width rect), as Range.getClientRects;
//! - `caret.at {x, y}`: the text position at a point, `{id, offset}` (a text's wire id and UTF-16 offset, or an
//!   element's with a child index), as caretPositionFromPoint;
//! - `selection.set {anchor, focus, caret} | null`: Blitz's selection set to it (painted as Blitz paints a drag's), and
//!   the caret drawn where it's collapsed in a focused editable.
//!
//! Blitz keeps text positions as (inline root, byte offset in its layout's text); each text node's bytes there are
//! recorded as its layout is built (TextLayout::text_nodes), so a DOM offset maps to the layout's and back by walking
//! the node's own characters against the layout's (collapsed white space taken out).

use bevy::prelude::*;
use blitz_dom::node::NodeData;
use blitz_dom::{BaseDocument, NodeId};
use serde_json::{Value, json};

use raea_runtime_dom::host::PageSync;
use raea_sys_page::ops::{self, Nodes};

/// Registers `text.rects`, `caret.at` and `selection.set`.
pub fn register() {
    raea_runtime_js::calls::on_sync_ctx("text.rects", rects);
    raea_runtime_js::calls::on_sync_ctx("caret.at", caret_at);
    raea_runtime_js::calls::on_sync_ctx("selection.set", selection_set);
}

type R<T> = Result<T, String>;

/// The page's document committed and laid out as the JS wrote it (as style.computed does).
fn ready(p: &mut PageSync) -> R<()> {
    let at = {
        let w = p.world();
        w.resource::<raea_sys_page::PageClock>().0.unwrap_or_else(|| w.resource::<Time>().elapsed_secs_f64())
    };
    p.commit();
    p.doc().ok_or("the page has no document")?.resolve(at);
    Ok(())
}

/// The inline root a node's text is laid out in: up from it to the first that is one.
fn inline_root(doc: &BaseDocument, mut n: NodeId) -> Option<NodeId> {
    loop {
        let node = doc.get_node(n)?;
        if node.flags.is_inline_root() {
            return Some(n);
        }
        n = node.parent?;
    }
}

/// A text node's content.
fn content(doc: &BaseDocument, t: NodeId) -> Option<&str> {
    match &doc.get_node(t)?.data {
        NodeData::Text(d) => Some(&d.content),
        _ => None,
    }
}

/// Where a text node's bytes are in its inline root's layout text: (root, start, end).
fn span_of(doc: &BaseDocument, t: NodeId) -> Option<(NodeId, usize, usize)> {
    let root = inline_root(doc, doc.get_node(t)?.parent?)?;
    let lay = doc.get_node(root)?.element_data()?.inline_layout_data.as_ref()?;
    lay.text_nodes.iter().find(|e| e.0 == t).map(|&(_, a, b)| (root, a, b))
}

/// A UTF-16 offset in a text node's content to a byte offset in the layout's text: the node's characters walked
/// against the layout's from its start, a collapsed white-space character taking no layout byte.
fn to_layout(content: &str, lay: &str, start: usize, end: usize, off16: usize) -> usize {
    let mut b = start;
    let mut u = 0;
    let mut lay_chars = lay.get(start..end).unwrap_or("").chars();
    let mut next = lay_chars.next();
    for c in content.chars() {
        if u >= off16 {
            break;
        }
        u += c.len_utf16();
        match next {
            Some(l) if l == c || (l.is_whitespace() && c.is_whitespace()) => {
                b += l.len_utf8();
                next = lay_chars.next();
            }
            // collapsed away
            _ if c.is_whitespace() => {}
            // a transformed character (text-transform): one for one
            Some(l) => {
                b += l.len_utf8();
                next = lay_chars.next();
            }
            None => {}
        }
    }
    b.min(end)
}

/// The other way: a byte offset in the layout's text (within the node's bytes) to a UTF-16 offset in its content.
fn to_dom(content: &str, lay: &str, start: usize, end: usize, byte: usize) -> usize {
    let mut b = start;
    let mut u = 0;
    let mut lay_chars = lay.get(start..end).unwrap_or("").chars();
    let mut next = lay_chars.next();
    for c in content.chars() {
        if b >= byte {
            break;
        }
        match next {
            Some(l) if l == c || (l.is_whitespace() && c.is_whitespace()) || !c.is_whitespace() => {
                b += l.len_utf8();
                next = lay_chars.next();
            }
            _ => {}
        }
        u += c.len_utf16();
    }
    u
}

/// A text node's UTF-16 offset as (inline root, byte).
fn text_point(doc: &BaseDocument, t: NodeId, off16: usize) -> Option<(NodeId, usize)> {
    let (root, a, b) = span_of(doc, t)?;
    let lay = &doc.get_node(root)?.element_data()?.inline_layout_data.as_ref()?.text;
    Some((root, to_layout(content(doc, t)?, lay, a, b, off16)))
}

/// The first (or last) text node in `n`'s subtree that has a place in a layout.
fn edge_text(doc: &BaseDocument, n: NodeId, last: bool) -> Option<NodeId> {
    let node = doc.get_node(n)?;
    if matches!(node.data, NodeData::Text(_)) {
        return span_of(doc, n).map(|_| n);
    }
    let kids: Vec<NodeId> = if last { node.children.iter().rev().copied().collect() } else { node.children.iter().copied().collect() };
    kids.into_iter().find_map(|k| edge_text(doc, k, last))
}

/// The first inline root at or under `n` (an empty block's: its line, with no text in it).
fn root_under(doc: &BaseDocument, n: NodeId) -> Option<NodeId> {
    let node = doc.get_node(n)?;
    if node.flags.is_inline_root() {
        return Some(n);
    }
    node.children.iter().find_map(|&k| root_under(doc, k))
}

/// A boundary point from sdk/ui (`{id, offset}` for a text or an element's own string, `{id, before}` for a point
/// among an element's children) as (inline root, byte).
fn point(doc: &BaseDocument, nodes: &Nodes, v: &Value) -> Option<(NodeId, usize)> {
    let el = ops::node(nodes, v["id"].as_u64()?)?;
    let node = doc.get_node(el)?;
    if let Some(off) = v["offset"].as_u64() {
        let off = off as usize;
        // a text, or the element's own string (its one text child)
        let t = if matches!(node.data, NodeData::Text(_)) { el } else { *node.children.iter().find(|&&k| matches!(doc.get_node(k).map(|n| &n.data), Some(NodeData::Text(_))))? };
        return text_point(doc, t, off);
    }
    // before a child: the first text from it on (else the end of the last before it); at the end: the last text
    let before = v["before"].as_u64().and_then(|b| ops::node(nodes, b));
    let kids = &node.children;
    let at = before.and_then(|b| kids.iter().position(|&k| k == b)).unwrap_or(kids.len());
    if let Some(t) = kids[at..].iter().find_map(|&k| edge_text(doc, k, false)) {
        // only within the same inline root as the point's element (a later paragraph's text is a different line)
        let (root, a, _) = span_of(doc, t)?;
        if inline_root(doc, el) == Some(root) || before.is_some_and(|b| doc.get_node(b).is_some_and(|n| !n.flags.is_inline_root())) {
            return Some((root, a));
        }
    }
    if let Some(t) = kids[..at].iter().rev().find_map(|&k| edge_text(doc, k, true)) {
        let (root, _, b) = span_of(doc, t)?;
        return Some((root, b));
    }
    let r = before.and_then(|b| root_under(doc, b)).or_else(|| inline_root(doc, el)).or_else(|| root_under(doc, el))?;
    Some((r, 0))
}

/// A rect in an inline root's layout (its units, from the content box's start, past its block offset) in client px.
fn client_rect(doc: &BaseDocument, root: NodeId, r: &parley::BoundingBox) -> Option<[f64; 4]> {
    let node = doc.get_node(root)?;
    let lay = node.element_data()?.inline_layout_data.as_ref()?;
    let b = doc.get_client_bounding_rect(root)?;
    let l = node.final_layout();
    let s = lay.layout.scale() as f64;
    let x = b.x + (l.padding.left + l.border.left) as f64 + r.x0 / s;
    let y = b.y + (l.padding.top + l.border.top + lay.block_offset) as f64 + r.y0 / s;
    Some([x, y, (r.x1 - r.x0) / s, (r.y1 - r.y0) / s])
}

/// One text's rects for [from, to) (UTF-16): the selection's boxes, or the caret's zero-width one when collapsed.
fn text_rects(doc: &BaseDocument, t: NodeId, from: usize, to: usize) -> Option<Vec<[f64; 4]>> {
    let (root, a) = text_point(doc, t, from)?;
    let (_, b) = text_point(doc, t, to)?;
    let layout = &doc.get_node(root)?.element_data()?.inline_layout_data.as_ref()?.layout;
    use parley::{Affinity, Cursor, Selection};
    if a == b {
        let c = Cursor::from_byte_index(layout, a, Affinity::Downstream).geometry(layout, 0.0);
        return Some(vec![client_rect(doc, root, &c)?]);
    }
    let sel = Selection::new(Cursor::from_byte_index(layout, a, Affinity::Downstream), Cursor::from_byte_index(layout, b, Affinity::Upstream));
    Some(sel.geometry(layout).iter().filter_map(|(r, _)| client_rect(doc, root, r)).collect())
}

fn rects(ctx: &mut dyn std::any::Any, args: &Value) -> R<Value> {
    let p = ctx.downcast_mut::<PageSync>().ok_or("text.rects: not a page's call")?;
    ready(p)?;
    let spans: Vec<(u64, usize, usize)> = args["spans"].as_array().map(|a| a.iter().filter_map(|s| Some((s[0].as_u64()?, s[1].as_u64()? as usize, s[2].as_u64()? as usize))).collect()).unwrap_or_default();
    let ids: Vec<Option<NodeId>> = spans.iter().map(|s| ops::node(p.nodes(), s.0)).collect();
    let doc = p.doc().ok_or("text.rects: the page has no document")?;
    let out: Vec<Value> = spans.iter().zip(ids).map(|(&(_, a, b), t)| t.and_then(|t| text_rects(doc, t, a, b)).map_or(Value::Null, |rs| json!(rs))).collect();
    Ok(Value::Array(out))
}

/// The text node and UTF-16 offset at a layout position (at a seam, the text it ends: Chrome's upstream caret).
fn text_at(doc: &BaseDocument, root: NodeId, byte: usize) -> Option<(NodeId, usize)> {
    let lay = doc.get_node(root)?.element_data()?.inline_layout_data.as_ref()?;
    // the text it's inside, else the one it ends
    let holds = lay.text_nodes.iter().filter(|e| e.1 <= byte && byte <= e.2);
    let e = holds.min_by_key(|e| e.2 == byte)?;
    Some((e.0, to_dom(content(doc, e.0)?, &lay.text, e.1, e.2, byte)))
}

/// The inline roots in a subtree (a block's paragraphs).
fn roots_in(doc: &BaseDocument, n: NodeId, out: &mut Vec<NodeId>) {
    let Some(node) = doc.get_node(n) else { return };
    if node.flags.is_inline_root() {
        out.push(n);
        return;
    }
    for &k in node.children.iter() {
        roots_in(doc, k, out);
    }
}

/// A point that's off every line moved onto the nearest one under what it hit: into that inline root's content box.
fn nearest_line(doc: &BaseDocument, x: f32, y: f32) -> Option<(f32, f32)> {
    let hit = doc.hit(x, y)?;
    let mut roots = Vec::new();
    roots_in(doc, hit.node_id, &mut roots);
    let (x, y) = (x as f64, y as f64);
    let boxes = roots.into_iter().filter_map(|r| {
        let b = doc.get_client_bounding_rect(r)?;
        let l = doc.get_node(r)?.final_layout();
        let (l0, t0) = (b.x + (l.padding.left + l.border.left) as f64, b.y + (l.padding.top + l.border.top) as f64);
        let (r0, b0) = (b.x + b.width - (l.padding.right + l.border.right) as f64, b.y + b.height - (l.padding.bottom + l.border.bottom) as f64);
        (r0 > l0 && b0 > t0).then_some((l0, t0, r0, b0))
    });
    let dist = |&(l, t, r, b): &(f64, f64, f64, f64)| (x - x.clamp(l, r)).powi(2) + (y - y.clamp(t, b)).powi(2);
    let (l, t, r, b) = boxes.min_by(|a, b| dist(a).total_cmp(&dist(b)))?;
    Some((x.clamp(l + 0.5, r - 0.5) as f32, y.clamp(t + 0.5, b - 0.5) as f32))
}

fn caret_at(ctx: &mut dyn std::any::Any, args: &Value) -> R<Value> {
    let p = ctx.downcast_mut::<PageSync>().ok_or("caret.at: not a page's call")?;
    ready(p)?;
    let (x, y) = (args["x"].as_f64().unwrap_or(0.0) as f32, args["y"].as_f64().unwrap_or(0.0) as f32);
    let found = {
        let doc = p.doc().ok_or("caret.at: the page has no document")?;
        // a point off any line (a block's padding, past a paragraph's end): clamped into the nearest line, as Chrome's
        // caretPositionFromPoint clamps it
        let pos = doc.find_text_position(x, y).or_else(|| {
            let (cx, cy) = nearest_line(doc, x, y)?;
            doc.find_text_position(cx, cy)
        });
        match pos {
            Some((root, byte)) => text_at(doc, root, byte),
            None => doc.hit(x, y).map(|h| (h.node_id, 0)),
        }
    };
    let Some((node, off)) = found else { return Ok(Value::Null) };
    let doc = p.doc().ok_or("caret.at: the page has no document")?;
    let is_text = matches!(doc.get_node(node).map(|n| &n.data), Some(NodeData::Text(_)));
    let parent = doc.get_node(node).and_then(|n| n.parent);
    let nodes = p.nodes();
    // a text the ops made, else its element (React's own string: the offset is in it), else the nearest element up
    if let Some(id) = nodes.id_of(node) {
        return Ok(json!({ "id": id, "offset": off }));
    }
    let mut up = if is_text { parent } else { Some(node) };
    let doc = p.doc().ok_or("caret.at: the page has no document")?;
    let mut chain = Vec::new();
    while let Some(n) = up {
        chain.push(n);
        up = doc.get_node(n).and_then(|x| x.parent);
    }
    let nodes = p.nodes();
    for n in chain {
        if let Some(id) = nodes.id_of(n) {
            return Ok(json!({ "id": id, "offset": if is_text { off } else { 0 } }));
        }
    }
    Ok(Value::Null)
}

fn selection_set(ctx: &mut dyn std::any::Any, args: &Value) -> R<Value> {
    let p = ctx.downcast_mut::<PageSync>().ok_or("selection.set: not a page's call")?;
    ready(p)?;
    let ends = if args.is_null() {
        None
    } else {
        let nodes: *const Nodes = p.nodes();
        let doc = p.doc().ok_or("selection.set: the page has no document")?;
        // (nodes and the document are apart: the ids' map is the root's, the document the page's)
        let nodes = unsafe { &*nodes };
        point(doc, nodes, &args["anchor"]).zip(point(doc, nodes, &args["focus"]))
    };
    let caret = args["caret"].as_bool().unwrap_or(false);
    let doc = p.doc().ok_or("selection.set: the page has no document")?;
    match ends {
        Some((a, f)) => {
            doc.set_text_selection(a.0, a.1, f.0, f.1);
            doc.caret = (caret && a == f).then_some(f);
        }
        None => {
            doc.clear_text_selection();
            doc.caret = None;
        }
    }
    Ok(Value::Null)
}
