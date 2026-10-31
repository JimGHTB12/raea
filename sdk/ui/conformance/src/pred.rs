//! Predicates over `ui` trees ({frame, window: {w,h}, nodes: [{id, role, name, rect, visible, hovered, focused,
//! pressed, selected, value, scroll?, text?}]}). A predicate holds or says what the tree showed instead.
//!
//! On nodes (selected by `id`, a glob where * matches any run of characters, narrowed by `where = {field = value}`):
//!   visible/hovered/focused/pressed/selected/value/name/role/text = V      the field equals V
//!   exists = bool                                                         some node matches (or none)
//!   name_contains/value_contains/text_contains = "s"
//!   field = "a.b" with eq/ne/gt/ge/lt/le = N, contains = "s", eq_field = "c.d" (another field of the node),
//!     changed/unchanged = true, increased_by/decreased_by = N (against the same node in snapshot `since`, default
//!     "last": the tree just before the last input), eq_snap = {snap, id, field} (a field of a node in a snapshot)
//!   quant = "any" (default) | "all" | "none", or count = N (exactly N matching nodes satisfy it)
//! A plain id (no *) that matches nothing stands for a node that isn't there: visible, hovered, focused, pressed and
//! selected false, everything else null.
//!
//! Over the whole tree (`check = ...`):
//!   no_overlap  {ids?}       no two visible interactive nodes intersect (nested ones, one inside the other, may)
//!   min_size    {ids?, min}  every visible interactive node is at least min pt (default 24) both ways
//!   on_screen   {ids?}       every visible interactive node is on screen (scroll content: at least partly)
//!   text_width  {}           every visible node with text has a rect wider and taller than 0
//!   window      {w?, h?, unchanged_since?}
//!   inside      {id, within, axis?} every visible node matching id lies inside `within`'s rect (axis x, y or both)
//! Combinators: all = [..], any = [..], not = {..}.
//!
//! Interactive: the roles in INTERACTIVE. Scroll content: a node that intersects a scroller's rect (a node with a
//! `scroll` field) without containing it; for overlaps it counts as its rect cut to that scroller.

use std::collections::HashMap;

use serde_json::{json, Value};

mod check;
pub(crate) use check::*;
#[cfg(test)]
mod tests;

pub const INTERACTIVE: &[&str] = &[
    "button", "link", "textbox", "searchbox", "tab", "checkbox", "radio", "switch", "slider", "menuitem", "option", "combobox",
    "listitem", "treeitem", "row", "card",
];

pub type Snaps = HashMap<String, Value>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn of(n: &Value) -> Option<Rect> {
        let r = &n["rect"];
        Some(Rect { x: r["x"].as_f64()?, y: r["y"].as_f64()?, w: r["w"].as_f64()?, h: r["h"].as_f64()? })
    }
    fn r(&self) -> f64 {
        self.x + self.w
    }
    fn b(&self) -> f64 {
        self.y + self.h
    }
    /// The overlap, if it has area beyond `tol` both ways.
    pub fn cut(&self, o: &Rect, tol: f64) -> Option<Rect> {
        let (x, y, r, b) = (self.x.max(o.x), self.y.max(o.y), self.r().min(o.r()), self.b().min(o.b()));
        (r - x > tol && b - y > tol).then_some(Rect { x, y, w: r - x, h: b - y })
    }
    pub fn contains(&self, o: &Rect, tol: f64) -> bool {
        o.x >= self.x - tol && o.y >= self.y - tol && o.r() <= self.r() + tol && o.b() <= self.b() + tol
    }
    fn show(&self) -> String {
        format!("{:.0},{:.0} {:.0}x{:.0}", self.x, self.y, self.w, self.h)
    }
}

pub fn glob(pat: &str, s: &str) -> bool {
    let parts: Vec<&str> = pat.split('*').collect();
    if parts.len() == 1 {
        return pat == s;
    }
    let mut rest = s;
    for (i, p) in parts.iter().enumerate() {
        if i == 0 {
            match rest.strip_prefix(p) {
                Some(r) => rest = r,
                None => return false,
            }
        } else if i == parts.len() - 1 {
            return rest.len() >= p.len() && rest.ends_with(p);
        } else {
            match rest.find(p) {
                Some(k) => rest = &rest[k + p.len()..],
                None => return false,
            }
        }
    }
    true
}

pub fn nodes(tree: &Value) -> &[Value] {
    tree["nodes"].as_array().map(|v| v.as_slice()).unwrap_or(&[])
}

pub fn node<'a>(tree: &'a Value, id: &str) -> Option<&'a Value> {
    nodes(tree).iter().find(|n| n["id"].as_str() == Some(id))
}

pub fn get<'a>(v: &'a Value, path: &str) -> &'a Value {
    path.split('.').fold(v, |v, k| &v[k])
}

fn absent(id: &str) -> Value {
    json!({"id": id, "visible": false, "hovered": false, "focused": false, "pressed": false, "selected": false, "current": false, "absent": true})
}

fn eq(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => (x - y).abs() < 1e-6,
        _ => a == b || (a.as_str().is_some() && a.as_str() == b.as_str()),
    }
}

/// The node's fields worth showing, short.
pub fn show(n: &Value) -> String {
    if n["absent"].as_bool() == Some(true) {
        return format!("{}: not in the tree", n["id"].as_str().unwrap_or("?"));
    }
    let mut s = vec![];
    for k in ["visible", "hovered", "focused", "pressed", "selected", "current"] {
        if n[k].as_bool() == Some(true) {
            s.push(k.to_string());
        }
    }
    if n["visible"].as_bool() == Some(false) {
        s.push("hidden".into());
    }
    if let Some(r) = Rect::of(n) {
        s.push(format!("rect {}", r.show()));
    }
    for k in ["name", "value", "text"] {
        if !n[k].is_null() && n[k] != json!("") {
            s.push(format!("{k} {}", crate::mcp::clip(&n[k].to_string(), 60)));
        }
    }
    if !n["scroll"].is_null() {
        s.push(format!("scroll {}/{}", n["scroll"]["top"], n["scroll"]["max"]));
    }
    format!("{}: {}", n["id"].as_str().unwrap_or("?"), s.join(", "))
}

fn selected<'a>(p: &Value, tree: &'a Value) -> Vec<Value> {
    let id = p["id"].as_str().unwrap_or("*");
    let mut out: Vec<Value> = nodes(tree)
        .iter()
        .filter(|n| glob(id, n["id"].as_str().unwrap_or("")))
        .filter(|n| p["where"].as_object().is_none_or(|w| w.iter().all(|(k, v)| eq(get(n, k), v))))
        .cloned()
        .collect();
    if out.is_empty() && !id.contains('*') && p["where"].is_null() {
        out.push(absent(id));
    }
    out
}

const BOOL_FIELDS: &[&str] = &["visible", "hovered", "focused", "pressed", "selected", "current", "value", "name", "role", "text"];

/// Does one node satisfy the node conditions of `p`? Err says how it didn't.
fn node_holds(p: &Value, n: &Value, snaps: &Snaps) -> Result<(), String> {
    let id = n["id"].as_str().unwrap_or("?");
    if let Some(e) = p["exists"].as_bool() {
        let there = n["absent"].as_bool() != Some(true);
        if there != e {
            return Err(if e { format!("{id}: not in the tree") } else { format!("{id} is there") });
        }
    }
    for k in BOOL_FIELDS {
        if let Some(v) = p.get(*k) {
            if !eq(&n[*k], v) {
                return Err(format!("{id}.{k} is {} (wanted {v})", n[*k]));
            }
        }
    }
    // a state token (e.g. "playing"), as the node lists them in `state`
    if let Some(want) = p["state"].as_str() {
        let has = n["state"].as_array().is_some_and(|s| s.iter().any(|t| t.as_str() == Some(want)));
        if !has {
            return Err(format!("{id}.state is {} (wanted it to have {want:?})", n["state"]));
        }
    }
    for (k, f) in [("name_contains", "name"), ("value_contains", "value"), ("text_contains", "text")] {
        if let Some(want) = p[k].as_str() {
            let have = n[f].as_str().map(str::to_string).unwrap_or_else(|| n[f].to_string());
            if !have.contains(want) {
                return Err(format!("{id}.{f} is {} (wanted it to contain {want:?})", n[f]));
            }
        }
    }
    let Some(field) = p["field"].as_str() else { return Ok(()) };
    let have = get(n, field);
    let hv = have.as_f64();
    let need_num = |op: &str, want: &Value| -> Result<(f64, f64), String> {
        match (hv, want.as_f64()) {
            (Some(h), Some(w)) => Ok((h, w)),
            _ => Err(format!("{id}.{field} is {have} ({op} needs a number)")),
        }
    };
    for (op, f) in [("gt", (|h: f64, w: f64| h > w) as fn(f64, f64) -> bool), ("ge", |h, w| h >= w - 1e-6), ("lt", |h, w| h < w), ("le", |h, w| h <= w + 1e-6)] {
        if let Some(want) = p.get(op) {
            let (h, w) = need_num(op, want)?;
            if !f(h, w) {
                return Err(format!("{id}.{field} is {h} (wanted {op} {w})"));
            }
        }
    }
    if let Some(want) = p.get("eq") {
        if !eq(have, want) {
            return Err(format!("{id}.{field} is {have} (wanted {want})"));
        }
    }
    if let Some(want) = p.get("ne") {
        if eq(have, want) {
            return Err(format!("{id}.{field} is {have} (wanted anything else)"));
        }
    }
    if let Some(want) = p["contains"].as_str() {
        if !have.as_str().unwrap_or("").contains(want) {
            return Err(format!("{id}.{field} is {have} (wanted it to contain {want:?})"));
        }
    }
    if let Some(other) = p["eq_field"].as_str() {
        if !eq(have, get(n, other)) {
            return Err(format!("{id}.{field} is {have}, {other} is {}", get(n, other)));
        }
    }
    if let Some(r) = p.get("eq_snap") {
        let snap = r["snap"].as_str().unwrap_or("last");
        let tree = snaps.get(snap).ok_or(format!("no snapshot {snap:?}"))?;
        let rid = r["id"].as_str().unwrap_or(id);
        let rn = node(tree, rid).cloned().unwrap_or_else(|| absent(rid));
        let want = get(&rn, r["field"].as_str().unwrap_or(field));
        if want.is_null() || !eq(have, want) {
            return Err(format!("{id}.{field} is {have} (wanted {rid}'s in {snap}: {want})"));
        }
    }
    let since = p["since"].as_str().unwrap_or("last");
    let rel = ["changed", "unchanged", "increased_by", "decreased_by"].iter().any(|k| p.get(*k).is_some());
    if rel {
        let tree = snaps.get(since).ok_or(format!("no snapshot {since:?}"))?;
        let before_n = node(tree, id).cloned().unwrap_or_else(|| absent(id));
        let before = get(&before_n, field);
        if p["changed"].as_bool() == Some(true) && eq(have, before) {
            return Err(format!("{id}.{field} is still {have} (as at {since})"));
        }
        if p["unchanged"].as_bool() == Some(true) && !eq(have, before) {
            return Err(format!("{id}.{field} went from {before} to {have} (since {since})"));
        }
        for (op, sign) in [("increased_by", 1.0), ("decreased_by", -1.0)] {
            if let Some(want) = p[op].as_f64() {
                let (h, b) = match (hv, before.as_f64()) {
                    (Some(h), Some(b)) => (h, b),
                    _ => return Err(format!("{id}.{field} went from {before} to {have} ({op} needs numbers)")),
                };
                // a scroll can't go past its ends: the distance left counts as the whole distance
                let mut want = want;
                if field == "scroll.top" {
                    let room = if sign > 0.0 { before_n["scroll"]["max"].as_f64().unwrap_or(f64::MAX) - b } else { b };
                    want = want.min(room.max(0.0));
                }
                if (h - b) * sign < want - 1e-6 {
                    return Err(format!("{id}.{field} went from {b} to {h} (wanted {} by at least {want})", if sign > 0.0 { "up" } else { "down" }));
                }
            }
        }
    }
    Ok(())
}

fn interactive(n: &Value) -> bool {
    n["visible"].as_bool() == Some(true) && INTERACTIVE.contains(&n["role"].as_str().unwrap_or(""))
}

fn pick<'a>(tree: &'a Value, p: &Value) -> Vec<&'a Value> {
    match p["ids"].as_array() {
        Some(ids) => nodes(tree)
            .iter()
            .filter(|n| n["visible"].as_bool() == Some(true))
            .filter(|n| ids.iter().filter_map(|i| i.as_str()).any(|i| glob(i, n["id"].as_str().unwrap_or(""))))
            .collect(),
        None => nodes(tree).iter().filter(|n| interactive(n)).collect(),
    }
}

/// The scroller a node is content of (the smallest that intersects it without being inside it), if any.
fn scroller_of(tree: &Value, n: &Value, r: &Rect) -> Option<Rect> {
    nodes(tree)
        .iter()
        .filter(|s| !s["scroll"].is_null() && s["visible"].as_bool() != Some(false) && s["id"] != n["id"])
        .filter_map(Rect::of)
        .filter(|s| s.cut(r, 0.0).is_some() && !r.contains(s, 0.5))
        .min_by(|a, b| (a.w * a.h).total_cmp(&(b.w * b.h)))
}

/// `r` inside `outer` (1 pt of slack) along x, y or both.
fn within_axis(outer: &Rect, r: &Rect, axis: &str) -> bool {
    let x = r.x >= outer.x - 1.0 && r.r() <= outer.r() + 1.0;
    let y = r.y >= outer.y - 1.0 && r.b() <= outer.b() + 1.0;
    match axis {
        "x" => x,
        "y" => y,
        _ => x && y,
    }
}

fn window(tree: &Value) -> Rect {
    Rect { x: 0.0, y: 0.0, w: tree["window"]["w"].as_f64().unwrap_or(0.0), h: tree["window"]["h"].as_f64().unwrap_or(0.0) }
}
