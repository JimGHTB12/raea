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
