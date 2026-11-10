use serde_json::json;

use super::*;

/// A GLB of boxes (min, max) under one node moved by `at` and scaled by `s`, with a speck if asked.
fn glb(boxes: &[([f64; 3], [f64; 3])], at: [f64; 3], s: f64) -> Vec<u8> {
    let (mut v, mut t): (Vec<f32>, Vec<u32>) = (Vec::new(), Vec::new());
    for (lo, hi) in boxes {
        let base = (v.len() / 3) as u32;
        for i in 0..8 {
            let p = [if i & 1 == 0 { lo[0] } else { hi[0] }, if i & 2 == 0 { lo[1] } else { hi[1] }, if i & 4 == 0 { lo[2] } else { hi[2] }];
            v.extend(p.iter().map(|x| *x as f32));
        }
        for f in [[0, 2, 3, 1], [4, 5, 7, 6], [0, 1, 5, 4], [2, 6, 7, 3], [0, 4, 6, 2], [1, 3, 7, 5]] {
            t.extend([f[0], f[1], f[2], f[0], f[2], f[3]].map(|k| base + k));
        }
    }
    let mut bin: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
    let il = bin.len();
    bin.extend(t.iter().flat_map(|x| x.to_le_bytes()));
    let j = json!({
        "asset": {"version": "2.0", "generator": "test"}, "scene": 0, "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "translation": at, "scale": [s, s, s]}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 1}]}],
        "buffers": [{"byteLength": bin.len()}],
        "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": il}, {"buffer": 0, "byteOffset": il, "byteLength": bin.len() - il}],
        "accessors": [{"bufferView": 0, "componentType": 5126, "count": v.len() / 3, "type": "VEC3"}, {"bufferView": 1, "componentType": 5125, "count": t.len(), "type": "SCALAR"}],
    });
    let mut js = serde_json::to_vec(&j).unwrap();
    while js.len() % 4 != 0 {
        js.push(b' ');
    }
    let total = 20 + js.len() + 8 + bin.len();
    let mut out = Vec::new();
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(js.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&js);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin);
    out
}

fn level(r: &Value, check: &str) -> String {
    r["found"].as_array().unwrap().iter().find(|f| f["check"] == check).map(|f| f["level"].as_str().unwrap().to_string()).unwrap_or_default()
}

#[test]
fn a_generated_prop_is_measured_and_fixed() {
    // a 2 x 4 x 1 box authored 10x too big, its origin at its centre, moved off to the side
    let g = Glb::parse(glb(&[([-1.0, -2.0, -0.5], [1.0, 2.0, 0.5])], [3.0, 1.0, 0.0], 1.0)).unwrap();
    let r = check(&g, 1000, &Want { h: Some(0.4), ..Default::default() }).unwrap();
    assert_eq!(r["measured"]["size"], json!([2.0, 4.0, 1.0]));
    assert_eq!((level(&r, "size"), level(&r, "pivot")), ("warning".into(), "warning".into()));
    assert!((r["fixed"]["scale"].as_f64().unwrap() - 0.1).abs() < 1e-9);
    // the fixed copy: 0.4 tall, standing on y 0, centred on the origin
    let t = r["fixed"]["translate"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>();
    let out = Glb::parse(fix(&g, 0.1, 0.0, [t[0], t[1], t[2]]).unwrap()).unwrap();
    let again = check(&out, 1000, &Want { h: Some(0.4), ..Default::default() }).unwrap();
    assert_eq!((level(&again, "size"), level(&again, "pivot")), ("ok".into(), "ok".into()), "{again:#}");
    let b = &again["measured"]["box"];
    assert!((b["min"][1].as_f64().unwrap()).abs() < 1e-6 && (b["max"][1].as_f64().unwrap() - 0.4).abs() < 1e-6);
    // the buffers are the same bytes: only the JSON changed
    assert_eq!(out.world().unwrap().tris.len(), 12);
}

#[test]
fn turning_swaps_width_and_depth_and_dims_catch_invented_depth() {
    // 1 wide, 1 tall, 0.2 deep, wanted 0.2 wide and 1 deep: turned 90 it fits; unturned the dims check says so
    let g = Glb::parse(glb(&[([-0.5, 0.0, -0.1], [0.5, 1.0, 0.1])], [0.0; 3], 1.0)).unwrap();
    let want = Want { h: Some(1.0), dims: Some([0.2, 1.0, 1.0]), ..Default::default() };
    assert_eq!(level(&check(&g, 10, &want).unwrap(), "dims"), "warning");
    let turned = check(&g, 10, &Want { turn: 90.0, ..want }).unwrap();
    assert_eq!(level(&turned, "dims"), "", "{turned:#}");
    let s = &turned["fixed"]["size"];
    assert!((s[0].as_f64().unwrap() - 0.2).abs() < 1e-6 && (s[2].as_f64().unwrap() - 1.0).abs() < 1e-6, "{s}");
}

#[test]
fn loose_specks_and_weight_are_found() {
    let mut boxes = vec![([-0.5, 0.0, -0.5], [0.5, 1.0, 0.5])];
    // a big prop of 300 boxes and one speck floating above it
    for i in 0..300 {
        let x = (i % 20) as f64 * 0.05 - 0.5;
        let z = (i / 20) as f64 * 0.05 - 0.5;
        boxes.push(([x, 1.01, z], [x + 0.04, 1.05, z + 0.04]));
    }
    let g = Glb::parse(glb(&boxes, [0.0; 3], 1.0)).unwrap();
    let r = check(&g, 10, &Want { faces: Some(1000), ..Default::default() }).unwrap();
    assert_eq!(level(&r, "triangles"), "warning");
    assert_eq!(r["measured"]["pieces"], 301);
    // a closed box has no open edges
    let one = check(&Glb::parse(glb(&boxes[..1], [0.0; 3], 1.0)).unwrap(), 10, &Want::default()).unwrap();
    assert_eq!((one["measured"]["openEdges"].as_u64(), one["measured"]["pieces"].as_u64()), (Some(0), Some(1)));
}

fn fix_of(r: &Value, check: &str) -> Value {
    r["found"].as_array().unwrap().iter().find(|f| f["check"] == check).map(|f| f["fix"].clone()).unwrap_or(Value::Null)
}

#[test]
fn each_fault_names_its_blender_script() {
    // a blob of 301 boxes seen from 30 m: over its budget (blender::budget(30) = 2 000 triangles: 3 612 here)
    let mut boxes = vec![([-0.5, 0.0, -0.5], [0.5, 1.0, 0.5])];
    for i in 0..300 {
        let x = (i % 20) as f64 * 0.05 - 0.5;
        boxes.push(([x, 1.01, 0.0], [x + 0.04, 1.05, 0.04]));
    }
    let g = Glb::parse(glb(&boxes, [0.0; 3], 1.0)).unwrap();
    let want = Want { seen: Some(30.0), file: Some("media/crate.glb".into()), ..Default::default() };
    let r = check(&g, 10, &want).unwrap();
    assert_eq!(fix_of(&r, "triangles")["run"], "raea blender run blender/decimate.py media/crate.glb media/crate-2k.glb 2000");
    assert!(r["found"].as_array().unwrap().iter().any(|f| f["says"].as_str().unwrap().contains("; fix: raea blender run blender/decimate.py")));
    // one closed box: the brief's moving parts, a skeleton and a closed surface asked of it
    let one = Glb::parse(glb(&boxes[..1], [0.0; 3], 1.0)).unwrap();
    let want = Want { parts: vec!["door".into(), "body".into()], rig: true, file: Some("m.glb".into()), ..Default::default() };
    let r = check(&one, 10, &want).unwrap();
    assert_eq!(fix_of(&r, "parts")["blender"], "split.py");
    assert!(fix_of(&r, "parts")["run"].as_str().unwrap().contains("door=X0,Y0,Z0,X1,Y1,Z1 body=X0,Y0,Z0,X1,Y1,Z1 --each"));
    assert_eq!(fix_of(&r, "skeleton")["blender"], "rig.py");
    assert_eq!(level(&r, "surface"), "", "a closed box has no surface finding");
    // an open box meant to be solid names repair.py; not meant to be solid, it's a note
    // the box with its last face's two triangles cut off
    let g = Glb::parse(glb(&boxes[..1], [0.0; 3], 1.0)).unwrap();
    let mut j = g.json.clone();
    j["accessors"][1]["count"] = json!(30);
    let holed = Glb::parse(g.with_json(&j).unwrap()).unwrap();
    let r = check(&holed, 10, &Want { solid: true, ..Default::default() }).unwrap();
    assert_eq!(fix_of(&r, "surface")["blender"], "repair.py");
    assert_eq!(level(&check(&holed, 10, &Want::default()).unwrap(), "surface"), "note");
}

#[test]
fn a_fix_the_fitted_copy_makes_is_said_as_what_to_run() {
    let says = |fix: &str| said_fix(fix).unwrap_or_default();
    assert_eq!(says(r#"{"origin":"bottom centre"}"#), "run this check again with -o FIXED.glb, which puts its origin at the bottom centre");
    assert!(says(r#"{"scale":0.5}"#).ends_with("scales it by 0.5") && says(r#"{"turn":90.0}"#).ends_with("turns it 90° about y"));
    // a Blender script's fix isn't the fitted copy's
    assert_eq!(said_fix(r#"{"blender":"repair.py"}"#), None);
}
