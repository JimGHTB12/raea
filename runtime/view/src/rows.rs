//! A 2D row (title, text, box, caption, image) as the page reads it: its id, kind, props and span.

use std::collections::BTreeMap;

use raea_core_program::eval::Tup;
use raea_core_program::ir::V;

/// The kinds drawn as 2D rows by the page (rows2d.rs) rather than spawned in the 3D.
pub const KINDS_2D: &[&str] = &["image"];

/// A 2D row (title, text, box, caption, image) as a raea row: its id ("kind:id"), kind, props and span.
/// A row with a `parent` (the id of another 2D row) sits inside it; raea times a child from its parent's start.
pub fn row2d(t: &Tup, all: &[Tup]) -> (String, serde_json::Value) {
    let vals = &t.vals;
    let parent = match vals.get("parent") {
        Some(V::S(p)) if !p.is_empty() => all.iter().find(|r| &r.id == p && is_2d(&r.kind) && r.id != t.id),
        _ => None,
    };
    let at = parent.and_then(|p| p.vals.get("from")).map(|f| f.num()).unwrap_or(0.0);
    let mut props = serde_json::Map::new();
    for (c, v) in vals {
        if matches!(c.as_str(), "from" | "to" | "class" | "parent") {
            continue;
        }
        // an image's src is a content hash: the bytes live in <edit dir>/assets
        if let (true, V::S(src)) = (c == "src", v) {
            if let Some(h) = src.strip_prefix("asset:") {
                props.insert("src".into(), serde_json::json!({"asset": format!("assets/{h}.png")}));
                continue;
            }
            // a picture's path as written (`image logo {src: "assets/logo.png"}`) was dropped and the row drew nothing
            if t.kind == "image" && !src.starts_with("media:") && !src.is_empty() {
                props.insert("src".into(), serde_json::json!({"asset": src}));
                continue;
            }
        }
        props.insert(prop_of(c).to_string(), json_of(v));
    }
    let id = format!("{}:{}", t.kind, t.id);
    let mut row = serde_json::json!({"id": id, "kind": if t.kind == "caption" { "text" } else { t.kind.as_str() }, "props": props});
    if let Some(f) = vals.get("from") {
        row["from"] = serde_json::json!((f.num() - at) as i64);
    }
    if let Some(to) = vals.get("to") {
        row["to"] = serde_json::json!((to.num() - at) as i64);
    }
    if let Some(p) = parent {
        row["parent"] = serde_json::json!(format!("{}:{}", p.kind, p.id));
    }
    (id, row)
}

pub fn is_2d(kind: &str) -> bool {
    KINDS_2D.contains(&kind)
}

/// SQL column -> raea prop, for 2D kinds.
pub fn prop_of(col: &str) -> &str {
    match col {
        "top" => "y",
        "left" => "x",
        "width" => "w",
        "height" => "h",
        other => other,
    }
}

pub fn json_of(v: &V) -> serde_json::Value {
    match v {
        V::N(n) => serde_json::json!(n),
        V::S(s) => serde_json::json!(s),
    }
}

pub use raea_sys_frame::Base2d;

/// A 2D row's props at a frame: its base under the frame's values (phase 5: the columns feed the DOM node directly).
pub fn props_2d(base: &Base2d, vals: &BTreeMap<String, V>) -> serde_json::Map<String, serde_json::Value> {
    let mut p = base.0.clone();
    for (k, x) in vals {
        if !matches!(k.as_str(), "from" | "to" | "class" | "src") {
            p.insert(prop_of(k).to_string(), json_of(x));
        }
    }
    p
}
