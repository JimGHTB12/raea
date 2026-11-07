//! The element tree as an agent checks it (R4b): each wire element Rune drew, with its wire id, Rune key, role,
//! label, text and box, in paint order, and its look (look.rs, T3e).

use serde_json::{Value, json};

use raea_rune_page::session::Session;

/// The tree after the last frame.
pub fn tree(s: &Session) -> Value {
    let els = s.rt.elements();
    let ids: Vec<Option<u64>> = els.iter().map(|e| e.key.as_deref().and_then(|k| k.strip_prefix('w')).and_then(|k| k.parse().ok())).collect();
    let mut out = Vec::new();
    let looks = s.rt.state.looks.borrow();
    let live = s.rt.state.live(false);
    for (i, e) in els.iter().enumerate() {
        let Some(id) = ids[i] else { continue };
        if e.exiting {
            continue;
        }
        let n = s.rt.state.tree.get(id);
        // the text under it: its own text elements' strings, as Rune drew them
        let text: String = els.iter().enumerate().filter(|(j, k)| ids[*j].is_none() && nearest(&els, &ids, *j) == Some(i) && k.text.is_some()).filter_map(|(_, k)| k.text.clone()).collect::<Vec<_>>().join(" ");
        let mut v = json!({"id": id, "key": e.key, "kind": n.map(|n| n.kind.clone()), "rect": raea_rune_page::session::shown(e).map(|x| (x * 100.0).round() / 100.0)});
        if !e.role.is_empty() {
            v["role"] = json!(e.role);
        }
        if let Some(l) = &e.label {
            v["label"] = json!(l);
        }
        if let Some(t) = n.and_then(|n| n.props.get("data-testid")) {
            v["testid"] = t.clone();
        }
        if !text.is_empty() {
            v["text"] = json!(text);
        }
        if e.focused || live.focus == Some(id) {
            v["focused"] = json!(true);
            v["focusVisible"] = json!(live.focus_visible);
        }
        // where a transform (an offset, a scale, a turn) drew it, when that isn't its box: Rune's drawn rect, scaled and
        // turned, and its scale (T3c)
        if (0..4).any(|k| (e.rect[k] - e.target[k]).abs() > 0.005) {
            v["drawn"] = json!(e.rect.map(|x| (f64::from(x) * 100.0).round() / 100.0));
            v["scale"] = json!((f64::from(e.scale) * 1000.0).round() / 1000.0);
        }
        if e.opacity < 0.999 {
            v["opacity"] = json!((f64::from(e.opacity) * 1000.0).round() / 1000.0);
        }
        v["look"] = crate::look::of(e, looks.get(&id), live.focus == Some(id) && live.focus_visible);
        out.push(v);
    }
    Value::Array(out)
}

/// The nearest keyed element up from element `i` (its parent chain in Rune's elements).
fn nearest(els: &[rune::ElementInfo], ids: &[Option<u64>], i: usize) -> Option<usize> {
    let mut at = els[i].parent;
    while let Some(j) = at {
        if ids[j].is_some() {
            return Some(j);
        }
        at = els[j].parent;
    }
    None
}
