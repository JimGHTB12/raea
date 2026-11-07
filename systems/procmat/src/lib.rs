//! Procedural surfaces: the standard material with a pattern made on the GPU and use written over it (the 11-06 research:
//! hayashimon's one grime group over every surface is the biggest realism step that needs no geometry, and a flat
//! default colour is what gives a scene away). Patterns: noise, wood, marble, stone, brick, tiles, fabric, brushed and
//! plaster, in two colours with a roughness each, a gap colour for brick and tile joints, and a bump from the
//! pattern's height. Use: `dust` on what faces up, `dirt` in gaps, hollows, undersides and along the floor line,
//! `edges` worn lighter. The pattern is taken in the mesh's own space, so it rides with a moving part.
//!
//! A mesh row's columns (from a material's `procedural` prop, `columns`): proc (the pattern's name), proc_a, proc_b,
//! proc_gap_color, proc_dust_color, proc_dirt_color (hex), proc_scale (metres a feature, 0.25), proc_rough_a/b (0.6,
//! 0.8), proc_bump (0.5), proc_seed, proc_tile_x/y (a brick or tile's size, 0.215 x 0.065 for brick, 0.3 x 0.3 for
//! tiles), proc_gap (the joint's width, 0.01), proc_dust, proc_dirt, proc_edges (0..1, 0).

use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use raea_core_program::ir::V;
use raea_sys_frame::num;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub type ProcMaterial = ExtendedMaterial<StandardMaterial, Procedural>;

pub const PATTERNS: [&str; 9] = ["noise", "wood", "marble", "stone", "brick", "tiles", "fabric", "brushed", "plaster"];

pub fn install(app: &mut App) {
    bevy::asset::embedded_asset!(app, "procmat.wesl");
    app.add_plugins(MaterialPlugin::<ProcMaterial>::default());
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct Procedural {
    #[uniform(100)]
    p: ProcParams,
}

#[derive(ShaderType, Clone, Copy, Default, PartialEq, Debug)]
pub struct ProcParams {
    pub a: Vec4,
    pub b: Vec4,
    pub gap_color: Vec4,
    pub dust_color: Vec4,
    pub dirt_color: Vec4,
    /// kind, scale, rough_a, rough_b
    pub k: Vec4,
    /// bump, seed, tile_x, tile_y
    pub t: Vec4,
    /// gap, dust, dirt, edges
    pub w: Vec4,
}

impl MaterialExtension for Procedural {
    fn fragment_shader() -> ShaderRef {
        "embedded://raea_sys_procmat/procmat.wesl".into()
    }
}

const NUMS: [(&str, &str); 9] = [("scale", "proc_scale"), ("bump", "proc_bump"), ("seed", "proc_seed"), ("gap", "proc_gap"), ("dust", "proc_dust"), ("dirt", "proc_dirt"), ("edges", "proc_edges"), ("roughA", "proc_rough_a"), ("roughB", "proc_rough_b")];
const COLOURS: [(&str, &str); 3] = [("gapColor", "proc_gap_color"), ("dustColor", "proc_dust_color"), ("dirtColor", "proc_dirt_color")];

/// A material's `procedural` prop as its row's columns (the first frame's)
pub fn columns(row: &mut BTreeMap<String, V>, props: &Map<String, Value>) {
    let Some(w) = props.get("procedural").and_then(Value::as_object) else { return };
    let Some(pattern) = w.get("pattern").and_then(Value::as_str) else { return };
    row.insert("proc".into(), V::S(pattern.into()));
    for (k, col) in NUMS {
        if let Some(n) = w.get(k).and_then(Value::as_f64) {
            row.insert(col.into(), V::N(n));
        }
    }
    for (k, col) in COLOURS {
        if let Some(s) = w.get(k).and_then(Value::as_str) {
            row.insert(col.into(), V::S(s.into()));
        }
    }
    if let Some(cs) = w.get("colors").and_then(Value::as_array) {
        for (i, col) in ["proc_a", "proc_b"].iter().enumerate() {
            if let Some(s) = cs.get(i).and_then(Value::as_str) {
                row.insert((*col).into(), V::S(s.into()));
            }
        }
    }
    if let Some(t) = w.get("tile").and_then(Value::as_array) {
        for (i, col) in ["proc_tile_x", "proc_tile_y"].iter().enumerate() {
            if let Some(n) = t.get(i).and_then(Value::as_f64) {
                row.insert((*col).into(), V::N(n));
            }
        }
    }
}

/// Each pattern's own colours, roughness, scale and tile when the prop doesn't give them: (a, b, gap, rough a, rough b, scale, tile)
fn defaults(kind: usize) -> ([f32; 3], [f32; 3], [f32; 3], f32, f32, f32, [f32; 2]) {
    match PATTERNS[kind] {
        "wood" => ([0.55, 0.36, 0.21], [0.36, 0.22, 0.12], [0.1, 0.07, 0.05], 0.55, 0.7, 0.25, [0.3, 0.3]),
        "marble" => ([0.92, 0.91, 0.89], [0.45, 0.45, 0.47], [0.3, 0.3, 0.3], 0.18, 0.25, 0.4, [0.3, 0.3]),
        "stone" => ([0.62, 0.61, 0.58], [0.45, 0.44, 0.42], [0.3, 0.3, 0.3], 0.85, 0.95, 0.3, [0.3, 0.3]),
        "brick" => ([0.6, 0.27, 0.18], [0.42, 0.18, 0.12], [0.62, 0.6, 0.55], 0.85, 0.95, 0.2, [0.215, 0.065]),
        "tiles" => ([0.9, 0.9, 0.88], [0.82, 0.83, 0.8], [0.55, 0.54, 0.5], 0.2, 0.3, 0.2, [0.3, 0.3]),
        "fabric" => ([0.35, 0.38, 0.45], [0.25, 0.28, 0.34], [0.2, 0.2, 0.2], 0.9, 0.95, 0.004, [0.3, 0.3]),
        "brushed" => ([0.72, 0.72, 0.74], [0.6, 0.6, 0.62], [0.3, 0.3, 0.3], 0.3, 0.45, 0.05, [0.3, 0.3]),
        "plaster" => ([0.88, 0.86, 0.82], [0.8, 0.78, 0.74], [0.5, 0.5, 0.5], 0.85, 0.95, 0.4, [0.3, 0.3]),
        _ => ([0.6, 0.6, 0.6], [0.4, 0.4, 0.4], [0.3, 0.3, 0.3], 0.6, 0.8, 0.25, [0.3, 0.3]),
    }
}

/// The procedural surface a mesh row asks for, or None (no `proc` column, or a pattern this doesn't know)
pub fn of(vals: &BTreeMap<String, V>) -> Option<Procedural> {
    let Some(V::S(name)) = vals.get("proc") else { return None };
    let kind = PATTERNS.iter().position(|p| *p == name.as_str())?;
    let (a, b, gap, ra, rb, scale, tile) = defaults(kind);
    let rgb = |col: &str, d: [f32; 3]| {
        let c = match vals.get(col) {
            Some(V::S(s)) => Srgba::hex(s.trim()).unwrap_or(Srgba::rgb(d[0], d[1], d[2])),
            _ => Srgba::rgb(d[0], d[1], d[2]),
        };
        let l = LinearRgba::from(c);
        Vec4::new(l.red, l.green, l.blue, 1.0)
    };
    let share = |col: &str, d: f32| num(vals, col, d).clamp(0.0, 1.0);
    // a positive number, else the pattern's own
    let pos = |col: &str, d: f32| Some(num(vals, col, d)).filter(|s| *s > 0.0).unwrap_or(d);
    Some(Procedural {
        p: ProcParams {
            a: rgb("proc_a", a),
            b: rgb("proc_b", b),
            gap_color: rgb("proc_gap_color", gap),
            dust_color: rgb("proc_dust_color", [0.62, 0.6, 0.55]),
            dirt_color: rgb("proc_dirt_color", [0.2, 0.17, 0.13]),
            k: Vec4::new(kind as f32, pos("proc_scale", scale), share("proc_rough_a", ra), share("proc_rough_b", rb)),
            t: Vec4::new(num(vals, "proc_bump", 0.5).max(0.0), num(vals, "proc_seed", 0.0), pos("proc_tile_x", tile[0]), pos("proc_tile_y", tile[1])),
            w: Vec4::new(Some(num(vals, "proc_gap", 0.01)).filter(|s| *s >= 0.0).unwrap_or(0.01), share("proc_dust", 0.0), share("proc_dirt", 0.0), share("proc_edges", 0.0)),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prop_becomes_columns_and_the_columns_a_surface() {
        let props: Map<String, Value> = serde_json::from_value(serde_json::json!({"procedural": {"pattern": "brick", "colors": ["#884433", "#663322"], "tile": [0.25, 0.08], "dust": 0.4, "gapColor": "#aaaaaa"}})).unwrap();
        let mut row = BTreeMap::new();
        columns(&mut row, &props);
        assert_eq!(row.get("proc"), Some(&V::S("brick".into())));
        assert_eq!(row.get("proc_tile_y"), Some(&V::N(0.08)));
        let p = of(&row).unwrap().p;
        assert_eq!(p.k.x, 4.0);
        assert_eq!((p.t.z, p.t.w), (0.25, 0.08));
        assert!((p.w.y - 0.4).abs() < 1e-6);
        // a pattern it doesn't know draws a plain surface
        row.insert("proc".into(), V::S("plaid".into()));
        assert!(of(&row).is_none());
        // each pattern has its own look by default
        let wood = of(&BTreeMap::from([("proc".to_string(), V::S("wood".into()))])).unwrap().p;
        assert!(wood.a.x > wood.a.z, "wood is warm");
    }
}
