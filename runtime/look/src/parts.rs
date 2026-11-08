//! Row helpers a part of any kind shares: shown on some frames, a sprite sheet's cell, a keyed colour.

use super::*;

/// A part shown only on some frames: `visible` keyed (0 on the frames it's missing from)
pub fn visible(p: &mut Program, row: &mut BTreeMap<String, V>, id: &str, track: &[Option<Item>]) {
    if track.iter().any(Option::is_none) {
        let on: Vec<f64> = track.iter().map(|t| t.is_some() as u8 as f64).collect();
        column(p, row, id, "visible", &on);
    }
}

/// A sprite sheet (`grid` [columns, rows], from the first frame): the cell `frame` shows, keyed (an <ImagePlane>'s,
/// a <Sprite>'s)
pub fn sheet(p: &mut Program, row: &mut BTreeMap<String, V>, id: &str, its: &[Item]) {
    if let Some(Value::Array(g)) = its[0].props.get("grid") {
        if let [c, r] = g.as_slice() {
            row.insert("sheet_cols".into(), V::N(c.as_f64().unwrap_or(1.0)));
            row.insert("sheet_rows".into(), V::N(r.as_f64().unwrap_or(1.0)));
            column(p, row, id, "sheet_frame", &its.iter().map(|i| n(&i.props, "frame", 0.0).floor()).collect::<Vec<_>>());
        }
    }
}

/// `color` keyed as col_r, col_g, col_b (`default` where a frame has none)
pub fn tint(p: &mut Program, row: &mut BTreeMap<String, V>, id: &str, its: &[Item], default: &str) {
    let colors: Vec<[f64; 3]> = its.iter().map(|i| rgb(&hex(i.props.get("color")).unwrap_or(default.into()))).collect();
    for (c, name) in ["col_r", "col_g", "col_b"].iter().enumerate() {
        exact(p, row, id, name, &colors.iter().map(|v| v[c]).collect::<Vec<_>>());
    }
}

/// A vector prop keyed as one column per component (a camera's `up` as up_x/y/z, a 2D part's `quat` as q_x..q_w),
/// written only when some frame sets it; a frame without it (or with the wrong length) takes `d`
pub fn keyed_vec(p: &mut Program, row: &mut BTreeMap<String, V>, id: &str, its: &[Item], key: &str, names: &[&str], d: &[f64]) {
    if !its.iter().any(|i| i.props.contains_key(key)) {
        return;
    }
    for (k, name) in names.iter().enumerate() {
        let at = |i: &Item| i.props.get(key).and_then(Value::as_array).filter(|a| a.len() == names.len()).and_then(|a| a[k].as_f64());
        column(p, row, id, name, &its.iter().map(|i| at(i).unwrap_or(d[k])).collect::<Vec<_>>());
    }
}

/// A <Material>'s uniforms and storage buffers over the frames (docs/control.feature FC2a), sent as keyed
/// `own:uniforms` component rows from each frame they change, which raea-sys-shader's own_live.rs packs into the
/// part's Material; none when they never change (the spec the part is made with holds them). `spec` finds an item's
/// Material spec: a mesh's material's `shader`, a 2D shape's own. Called after the part's row is pushed.
pub fn own_uniforms(p: &mut Program, id: &str, its: &[Item], spec: impl Fn(&Item) -> Option<&Value>) {
    let each: Vec<(usize, Value)> = its.iter().enumerate().filter_map(|(f, it)| {
        let s = spec(it).filter(|s| s.get("own").is_some())?;
        let v = serde_json::json!({ "uniforms": s.get("uniforms").cloned().unwrap_or(Value::Null), "buffers": s.get("buffers").cloned().unwrap_or(Value::Null) });
        Some((f, serde_json::json!({ "type": "own:uniforms", "value": v })))
    }).collect();
    if each.windows(2).all(|w| w[0].1 == w[1].1) {
        return;
    }
    // the part's spec carries its track's hash: parts share a Material (as Bevy's share one handle) only when their
    // spec and every frame's values are the same (raea-sys-shader own.rs)
    let mut h = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&each.iter().map(|(f, v)| format!("{f}{v}")).collect::<String>(), &mut h);
    let track = format!("{:016x}", std::hash::Hasher::finish(&h));
    let row = p.spawns.iter_mut().rev().flat_map(|(_, rows)| rows.iter_mut()).find(|r| r.get("id") == Some(&V::S(id.to_string())));
    if let Some(V::S(s)) = row.and_then(|r| r.get_mut("shader")) {
        if let Ok(Value::Object(mut o)) = serde_json::from_str::<Value>(s) {
            o.insert("track".into(), Value::String(track));
            *s = Value::Object(o).to_string();
        }
    }
    keyed(p, "component", Some(id), "o", each.iter().map(|(f, v)| (*f, v)));
}
