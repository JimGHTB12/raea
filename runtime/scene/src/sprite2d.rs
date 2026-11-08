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
    // `quat`: Bevy's rotation as given, in place of `rot` (q_x q_y q_z q_w, as a 3D part's)
    if its.iter().any(|i| i.props.get("quat").is_some()) {
        let q = |i: &Item, k: usize| i.props.get("quat").and_then(|q| q.get(k)).and_then(Value::as_f64).unwrap_or(if k == 3 { 1.0 } else { 0.0 });
        for (k, col) in ["q_x", "q_y", "q_z", "q_w"].iter().enumerate() {
            column(p, row, id, col, &its.iter().map(|i| q(i, k)).collect::<Vec<_>>());
        }
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
        // keyed: a size that changes shows more or fewer tiles of a tiled one (Bevy's custom_size, 2d/sprite_tile)
        for (prop, col) in [("width", "w"), ("height", "h"), ("tileStretch", "tile_stretch")] {
            if num(first, prop).is_some() {
                column(p, &mut row, &id, col, &its.iter().map(|i| n(&i.props, prop, 1.0)).collect::<Vec<_>>());
            }
        }
        sheet(p, &mut row, &id, &its);
        // the picture's sampler (Bevy's load_with_settings), as JSON for raea-sys-sampler
        if let Some(s) = first.get("sampler") {
            row.insert("texture_sampler".into(), V::S(s.to_string()));
        }
        // Bevy's SpriteImageMode as given (systems/sprite reads it)
        if let Some(m) = first.get("image_mode") {
            row.insert("image_mode".into(), V::S(m.to_string()));
        }
        for (prop, col) in [("flipX", "flip_x"), ("flipY", "flip_y"), ("tileX", "tile_x"), ("tileY", "tile_y")] {
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
        // its ColorMaterial's texture (a path, or { path, sampler }) and AlphaMode2d ("opaque", "blend", or "mask" with its cutoff)
        let tex = first.get("texture");
        if let Some(t) = tex.and_then(|t| t.as_str().or(t.get("path").and_then(Value::as_str))) { row.insert("texture".into(), V::S(src_path(t))); }
        // as JSON: the texture's sampler (raea-sys-sampler's), the uv_transform (an Affine2) and the mesh's Mesh::ATTRIBUTE_COLOR
        for (col, v) in [("texture_sampler", tex.and_then(|t| t.get("sampler"))), ("uv_transform", first.get("uv_transform")), ("vertex_colors", first.get("vertex_colors"))] { if let Some(v) = v { row.insert(col.into(), V::S(v.to_string())); } }
        match first.get("alpha_mode") {
            Some(Value::String(a)) => { row.insert("alpha".into(), V::S(a.to_lowercase())); }
            Some(Value::Object(m)) => { row.insert("alpha".into(), V::S("mask".into())); row.insert("alpha_cutoff".into(), V::N(n(m, "Mask", 0.5))); }
            _ => {}
        }
        place2d(p, &mut row, &id, &its, "#ffffff");
        visible(p, &mut row, &id, track);
        p.spawns.push(("shape2d".into(), vec![row]));
    }
}
