//! `raea solid export FILE.ts -o OUT.glb [--set k=v,...] [--part NAME] [--colour #RRGGBB]` (docs/blender.feature BL1):
//! a solid written as one .glb, metres, y up, so a Blender step can take it (bake.py for real UVs and baked maps, a
//! cloth to fall on). The solid stays the source: the .glb is made again from it whenever it changes. An empty solid,
//! or one solid info finds an error in, is refused with the findings. The file is read back and measured against the
//! solid (volume, pieces, box, within 0.5%), and a licence sidecar is written beside it.

use std::path::Path;

use raea_core_shape::Manifold;
use raea_core_solidinfo::{Measured, measure};
use raea_runtime_bundle::flag;
use serde_json::{Value, json};

/// How far the read-back may differ, as a share of the solid's own value.
const TOLERANCE: f64 = 0.005;

/// `#RRGGBB` as linear RGBA.
fn linear(hex: &str) -> Result<[f64; 4], String> {
    let h = hex.trim_start_matches('#');
    let c = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or(""), 16).map_err(|_| format!("--colour takes #RRGGBB, got {hex:?}"));
    let lin = |v: u8| {
        let s = v as f64 / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    if h.len() != 6 {
        return Err(format!("--colour takes #RRGGBB, got {hex:?}"));
    }
    Ok([lin(c(0)?), lin(c(2)?), lin(c(4)?), 1.0])
}

/// The read-back's differences from the solid, each over the tolerance: none when they match.
pub(crate) fn differences(a: &Measured, b: &Measured) -> Vec<String> {
    let mut d = Vec::new();
    let off = |x: f64, y: f64, of: f64| (x - y).abs() > TOLERANCE * of.abs().max(1e-9);
    if off(a.volume, b.volume, a.volume) {
        d.push(format!("volume {} m³ read back as {}", a.volume, b.volume));
    }
    if a.pieces != b.pieces {
        d.push(format!("{} pieces read back as {}", a.pieces, b.pieces));
    }
    let span = (0..3).map(|k| a.max[k] - a.min[k]).fold(0.0, f64::max);
    if (0..3).any(|k| off(a.min[k], b.min[k], span) || off(a.max[k], b.max[k], span)) {
        d.push(format!("box {:?}..{:?} read back as {:?}..{:?}", a.min, a.max, b.min, b.max));
    }
    d
}

/// Solid info's findings at one level, as it prints them: all of them, or one part's alone.
fn said(info: &Value, level: &str, part: Option<&str>) -> Vec<String> {
    let mine = |f: &&Value| f["level"] == level && part.is_none_or(|p| f["part"].as_str() == Some(p));
    info["found"].as_array().into_iter().flatten().filter(mine).map(|f| format!("{level:<8} {}", f["says"].as_str().unwrap_or(""))).collect()
}

pub fn export(file: &Path, parts: &[(String, String)], info: &Value, args: &[String]) -> Result<String, String> {
    let out = flag(args, "-o").filter(|o| o.ends_with(".glb")).ok_or("raea solid export FILE.ts -o OUT.glb: give the .glb to write")?;
    let part = flag(args, "--part");
    let chosen: Vec<&(String, String)> = parts.iter().filter(|(n, _)| part.as_deref().is_none_or(|p| p == n)).collect();
    if chosen.is_empty() {
        return Err(format!("solid export: no part {} (the file's parts: {})", part.unwrap_or_default(), parts.iter().map(|p| p.0.as_str()).collect::<Vec<_>>().join(", ")));
    }
    // refused on solid info's errors: the whole's, or the part's alone when one is asked for
    let errors = said(info, "error", part.as_deref());
    if !errors.is_empty() {
        return Err(format!("solid export: refused, solid info finds errors:\n{}", errors.join("\n")));
    }
    let solids = chosen.iter().map(|(_, e)| raea_core_shape::solid(e)).collect::<Result<Vec<Manifold>, String>>()?;
    let m = solids.into_iter().reduce(|a, b| a.union(&b)).ok_or("solid export: no part to write")?;
    let mine = measure(&m);
    if mine.empty {
        return Err("solid export: refused, the solid is empty".into());
    }
    if let Some(dir) = Path::new(&out).parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    raea_core_shape::export(&m, &out, "glb")?;
    // its name, and a material when a colour is given (bake.py multiplies its AO into it)
    let g = raea_core_meshcheck::Glb::read(&out)?;
    let mut j = g.json.clone();
    let name = part.clone().unwrap_or_else(|| file.file_stem().and_then(|s| s.to_str()).unwrap_or("solid").to_string());
    j["nodes"][0]["name"] = json!(name);
    j["meshes"][0]["name"] = json!(name);
    j["asset"]["generator"] = json!("raea solid export");
    if let Some(c) = flag(args, "--colour") {
        j["materials"] = json!([{"name": name, "pbrMetallicRoughness": {"baseColorFactor": linear(&c)?, "metallicFactor": 0.0, "roughnessFactor": 0.8}}]);
        j["meshes"][0]["primitives"][0]["material"] = json!(0);
    }
    std::fs::write(&out, g.with_json(&j)?).map_err(|e| format!("{out}: {e}"))?;
    // read back: the file a scene and Blender will load, measured against the solid
    let (v, t) = raea_core_shape::read_glb(&out)?;
    let back = measure(&raea_core_shape::from_mesh(&v, &t)?);
    let diff = differences(&mine, &back);
    if !diff.is_empty() {
        return Err(format!("solid export: {out} doesn't read back as the solid: {}", diff.join("; ")));
    }
    let side = json!({"source": format!("raea solid export {}{}", file.display(), part.as_ref().map(|p| format!(" --part {p}")).unwrap_or_default()), "license": "built in code for this project", "credit": ""});
    std::fs::write(format!("{out}.json"), serde_json::to_string_pretty(&side).unwrap_or_default()).map_err(|e| e.to_string())?;
    let r = |x: f64| (x * 1e5).round() / 1e5;
    let mut t = vec![format!(
        "APPLIED  {out}: {} triangles, {} m³, {} x {} x {} m, {} piece(s); read back the same (within {}%); licence in {out}.json",
        mine.tris, r(mine.volume), r(mine.max[0] - mine.min[0]), r(mine.max[1] - mine.min[1]), r(mine.max[2] - mine.min[2]), mine.pieces, TOLERANCE * 100.0
    )];
    t.extend(said(info, "warning", None));
    Ok(t.join("\n"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_solid_reads_back_as_itself() {
        let m = raea_core_shape::solid("minus(box(0.4, 0.3, 0.2), cylinder(0.05, 0.4))").unwrap();
        let dir = std::env::temp_dir().join(format!("raea-solid-export-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("x.glb");
        raea_core_shape::export(&m, out.to_str().unwrap(), "glb").unwrap();
        let (v, t) = raea_core_shape::read_glb(out.to_str().unwrap()).unwrap();
        let back = raea_core_solidinfo::measure(&raea_core_shape::from_mesh(&v, &t).unwrap());
        let mine = raea_core_solidinfo::measure(&m);
        assert!(super::differences(&mine, &back).is_empty(), "{:?}", super::differences(&mine, &back));
        // a mismatch is named
        let mut off = raea_core_solidinfo::measure(&m);
        off.volume *= 1.1;
        off.pieces = 2;
        assert_eq!(super::differences(&off, &back).len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(super::linear("#ffffff").unwrap(), [1.0, 1.0, 1.0, 1.0]);
        assert!(super::linear("red").is_err());
    }
}
