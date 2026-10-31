//! Solids to and from triangle meshes and mesh files (OBJ, glTF binary).

use manifold_rust::manifold::Manifold;

/// A solid from triangles (positions flat x y z, indices by threes); merged by position so it closes.
pub fn from_mesh(verts: &[f32], tris: &[u32]) -> Result<Manifold, String> {
    let mut mesh = manifold_rust::types::MeshGL::default();
    mesh.num_prop = 3;
    mesh.vert_properties = verts.to_vec();
    mesh.tri_verts = tris.to_vec();
    mesh.merge();
    let m = Manifold::from_mesh_gl(&mesh);
    if m.is_empty() {
        return Err("the mesh isn't a closed solid (a manifold)".into());
    }
    Ok(m)
}

/// A solid's triangles: positions flat (x y z) and indices by threes.
pub fn to_mesh(m: &Manifold) -> (Vec<f32>, Vec<u32>) {
    let g = m.get_mesh_gl(-1);
    let np = g.num_prop as usize;
    let verts: Vec<f32> = g.vert_properties.chunks(np).flat_map(|c| [c[0], c[1], c[2]]).collect();
    (verts, g.tri_verts.clone())
}

/// Wavefront OBJ: v lines and f lines (polygons fanned, "i/t/n" indices, negatives from the end).
pub fn read_obj(path: &str) -> Result<(Vec<f32>, Vec<u32>), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let (mut v, mut t) = (Vec::new(), Vec::new());
    for line in text.lines() {
        let mut w = line.split_whitespace();
        match w.next() {
            Some("v") => {
                for _ in 0..3 {
                    v.push(w.next().and_then(|x| x.parse::<f32>().ok()).ok_or_else(|| format!("{path}: a bad v line"))?);
                }
            }
            Some("f") => {
                let n = (v.len() / 3) as i64;
                let idx: Vec<u32> = w.map(|x| x.split('/').next().unwrap_or("").parse::<i64>().map(|i| if i < 0 { (n + i) as u32 } else { (i - 1) as u32 })).collect::<Result<_, _>>().map_err(|_| format!("{path}: a bad f line"))?;
                for k in 1..idx.len().saturating_sub(1) {
                    t.extend([idx[0], idx[k], idx[k + 1]]);
                }
            }
            _ => {}
        }
    }
    Ok((v, t))
}

/// glTF binary: the first mesh's primitives (POSITION and indices), node transforms not applied.
pub fn read_glb(path: &str) -> Result<(Vec<f32>, Vec<u32>), String> {
    let b = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    if b.len() < 20 || &b[0..4] != b"glTF" {
        return Err(format!("{path}: not a GLB"));
    }
    let u32le = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as usize;
    let jl = u32le(12);
    let json: serde_json::Value = serde_json::from_slice(&b[20..20 + jl]).map_err(|e| format!("{path}: {e}"))?;
    let bin_at = 20 + jl;
    let bin = if b.len() > bin_at + 8 { &b[bin_at + 8..bin_at + 8 + u32le(bin_at)] } else { &[][..] };
    let acc = |i: usize| -> Result<(Vec<u8>, usize, u64, usize), String> {
        let a = &json["accessors"][i];
        let view = &json["bufferViews"][a["bufferView"].as_u64().ok_or("accessor without a view")? as usize];
        let off = view["byteOffset"].as_u64().unwrap_or(0) as usize + a["byteOffset"].as_u64().unwrap_or(0) as usize;
        let count = a["count"].as_u64().unwrap_or(0) as usize;
        let ct = a["componentType"].as_u64().unwrap_or(0);
        let n = match a["type"].as_str().unwrap_or("") { "VEC3" => 3, "SCALAR" => 1, _ => 0 };
        let size = match ct { 5126 | 5125 => 4, 5123 => 2, 5121 => 1, _ => 4 };
        let stride = view["byteStride"].as_u64().unwrap_or(0) as usize;
        let stride = if stride == 0 { size * n } else { stride };
        let mut out = Vec::with_capacity(count * size * n);
        for k in 0..count {
            let at = off + k * stride;
            out.extend_from_slice(bin.get(at..at + size * n).ok_or("an accessor past the buffer")?);
        }
        Ok((out, count, ct, n))
    };
    let (mut v, mut t) = (Vec::new(), Vec::new());
    for prim in json["meshes"][0]["primitives"].as_array().ok_or_else(|| format!("{path}: no mesh"))? {
        let base = (v.len() / 3) as u32;
        let (p, _, _, _) = acc(prim["attributes"]["POSITION"].as_u64().ok_or("no POSITION")? as usize)?;
        v.extend(p.chunks(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])));
        let (ix, count, ct, _) = acc(prim["indices"].as_u64().ok_or("no indices")? as usize)?;
        for k in 0..count {
            let i = match ct {
                5125 => u32::from_le_bytes([ix[k * 4], ix[k * 4 + 1], ix[k * 4 + 2], ix[k * 4 + 3]]),
                5123 => u16::from_le_bytes([ix[k * 2], ix[k * 2 + 1]]) as u32,
                _ => ix[k] as u32,
            };
            t.push(base + i);
        }
    }
    Ok((v, t))
}

/// A solid written as OBJ (format "obj") or glTF binary ("glb").
pub fn export(m: &Manifold, path: &str, format: &str) -> Result<usize, String> {
    let (v, t) = to_mesh(m);
    let bytes = match format {
        "obj" => {
            let mut s = String::from("# raea solid\n");
            for c in v.chunks(3) {
                s += &format!("v {} {} {}\n", c[0], c[1], c[2]);
            }
            for c in t.chunks(3) {
                s += &format!("f {} {} {}\n", c[0] + 1, c[1] + 1, c[2] + 1);
            }
            s.into_bytes()
        }
        "glb" => {
            let mut bin = Vec::new();
            for x in &v {
                bin.extend_from_slice(&x.to_le_bytes());
            }
            let ilen = bin.len();
            for i in &t {
                bin.extend_from_slice(&i.to_le_bytes());
            }
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for c in v.chunks(3) {
                for k in 0..3 {
                    lo[k] = lo[k].min(c[k]);
                    hi[k] = hi[k].max(c[k]);
                }
            }
            let json = serde_json::json!({
                "asset": {"version": "2.0", "generator": "raea"},
                "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
                "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 1}]}],
                "buffers": [{"byteLength": bin.len()}],
                "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": ilen, "target": 34962}, {"buffer": 0, "byteOffset": ilen, "byteLength": bin.len() - ilen, "target": 34963}],
                "accessors": [
                    {"bufferView": 0, "componentType": 5126, "count": v.len() / 3, "type": "VEC3", "min": lo, "max": hi},
                    {"bufferView": 1, "componentType": 5125, "count": t.len(), "type": "SCALAR"}
                ]
            });
            let mut js = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
            while js.len() % 4 != 0 {
                js.push(b' ');
            }
            while bin.len() % 4 != 0 {
                bin.push(0);
            }
            let total = 12 + 8 + js.len() + 8 + bin.len();
            let mut out = Vec::with_capacity(total);
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
        f => return Err(format!("export: {f}? (obj, glb)")),
    };
    std::fs::write(path, &bytes).map_err(|e| format!("{path}: {e}"))?;
    Ok(bytes.len())
}
