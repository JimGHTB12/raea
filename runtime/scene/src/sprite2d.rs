//! 2D parts' rows (`<Sprite>`, `<Shape2D>` in sdk/render's 3d/sprite.tsx): a `sprite` or `shape2d` row each, read by
//! raea-sys-sprite. The picture, sheet and shape come from the first frame; the place, the sheet's cell, the colour and
//! the opacity are keyed per frame.

use super::*;

/// A 2D part's place (composition px from the top left, y down; radians clockwise; scale; draw order), its colour and
/// its opacity, each keyed.
fn place2d(p: &mut Program, row: &mut BTreeMap<String, V>, id: &str, its: &[Item], color: &str) {
    for (prop, col, d) in [("x", "x", 0.0), ("y", "y", 0.0), ("rotation", "rot", 0.0), ("z", "z", 0.0), ("opacity", "opacity", 1.0)] {
        column(p, row, id, col, &its.iter().map(|i| n(&i.props, prop, d)).collect::<Vec<_>>());
    }
    // `scale`: one number, or [x, y]
    let scale = |i: &Item, k: usize| match i.props.get("scale") {
        Some(Value::Array(a)) => a.get(k).and_then(Value::as_f64).unwrap_or(1.0),
        Some(v) => v.as_f64().unwrap_or(1.0),
        None => 1.0,
    };
    for (k, col) in ["scale_x", "scale_y"].iter().enumerate() {
        column(p, row, id, col, &its.iter().map(|i| scale(i, k)).collect::<Vec<_>>());
    }
    tint(p, row, id, its, color);
}

/// Sprites: the picture, its sheet and size from the first frame; place, cell, tint and opacity keyed.
pub(super) fn sprites(tracks: &[Track], p: &mut Program) {
    for (k, (_, track)) in tracks.iter().enumerate() {
        let its = filled(track);
        let id = format!("sprite{k}");
        let mut row = BTreeMap::new();
        row.insert("id".into(), V::S(id.clone()));
        let first = &its[0].props;
        row.insert("src".into(), V::S(src_path(first.get("src").and_then(Value::as_str).unwrap_or(""))));
        for (prop, col) in [("width", "w"), ("height", "h")] {
            if let Some(v) = num(first, prop) {
                row.insert(col.into(), V::N(v));
            }
        }
        sheet(p, &mut row, &id, &its);
        for (prop, col) in [("flipX", "flip_x"), ("flipY", "flip_y")] {
            if first.get(prop) == Some(&Value::Bool(true)) {
                row.insert(col.into(), V::N(1.0));
            }
        }
        place2d(p, &mut row, &id, &its, "#ffffff");
        visible(p, &mut row, &id, track);
        p.spawns.push(("sprite".into(), vec![row]));
    }
}

/// Flat shapes: the shape and its size from the first frame; place, colour and opacity keyed.
pub(super) fn shapes(tracks: &[Track], p: &mut Program) {
    for (k, (_, track)) in tracks.iter().enumerate() {
        let its = filled(track);
        let id = format!("shape2d{k}");
        let mut row = BTreeMap::new();
        row.insert("id".into(), V::S(id.clone()));
        let first = &its[0].props;
        row.insert("shape".into(), V::S(first.get("shape").and_then(Value::as_str).unwrap_or("rect").to_string()));
        let size = first.get("size").and_then(Value::as_array);
        let at = |i: usize| size.and_then(|a| a.get(i)).and_then(Value::as_f64).unwrap_or(100.0);
        row.insert("w".into(), V::N(at(0)));
        row.insert("h".into(), V::N(at(1)));
        row.insert("sides".into(), V::N(n(first, "sides", 5.0)));
        place2d(p, &mut row, &id, &its, "#ffffff");
        visible(p, &mut row, &id, track);
        p.spawns.push(("shape2d".into(), vec![row]));
    }
}
