//! `raea media lod MODEL.glb [--level 1] [--texture 1024] -o OUT.glb`: a model at a lower level of detail
//! (docs/preview.feature PV3c). Each level halves the triangles of every mesh (meshoptimizer's simplifier, on the
//! mesh's own vertices, so skins, morph targets, UVs and materials are untouched: only the index lists are replaced)
//! and its embedded pictures are made at most `--texture` px on their longer side. What nothing uses any more leaves
//! the file. A .glb with its buffer inside it only: a .gltf's separate files are left to their own tools.

use std::path::Path;

use serde_json::json;

use super::*;

const GLB: u32 = 0x4654_6c67;
const JSON: u32 = 0x4e4f_534a;
const BIN: u32 = 0x004e_4942;

/// The file's JSON and its binary chunk.
fn read(path: &Path) -> R<(Value, Vec<u8>)> {
    let b = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let u32_at = |i: usize| b.get(i..i + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()));
    if u32_at(0) != Some(GLB) {
        return Err(format!("{}: not a .glb (raea media lod reads a binary glTF with its buffer inside it)", path.display()));
    }
    let (mut at, mut doc, mut bin) = (12usize, None, Vec::new());
    while let (Some(len), Some(kind)) = (u32_at(at), u32_at(at + 4)) {
        let body = b.get(at + 8..at + 8 + len as usize).ok_or("the glb's chunk runs past its end")?;
        match kind {
            JSON => doc = Some(serde_json::from_slice::<Value>(body).map_err(|e| format!("the glb's JSON: {e}"))?),
            BIN if bin.is_empty() => bin = body.to_vec(),
            _ => {}
        }
        at += 8 + len as usize;
    }
    Ok((doc.ok_or("the glb has no JSON chunk")?, bin))
}

fn write(path: &Path, doc: &Value, bin: &[u8]) -> R<()> {
    let mut text = serde_json::to_vec(doc).map_err(|e| e.to_string())?;
    while text.len() % 4 != 0 {
        text.push(b' ');
    }
    let mut bin = bin.to_vec();
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    let mut out = Vec::with_capacity(28 + text.len() + bin.len());
    for v in [GLB, 2, (28 + text.len() + bin.len()) as u32, text.len() as u32, JSON] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&text);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(&BIN.to_le_bytes());
    out.extend_from_slice(&bin);
    std::fs::write(path, out).map_err(|e| format!("{}: {e}", path.display()))
}

fn n(v: &Value, k: &str) -> Option<usize> {
    Some(v.get(k)?.as_u64()? as usize)
}

/// An accessor's bytes in the buffer: where they start, the step between elements, how many.
fn span(doc: &Value, acc: usize) -> Option<(usize, usize, usize, u64)> {
    let a = doc["accessors"].get(acc)?;
    if a.get("sparse").is_some() {
        return None;
    }
    let view = doc["bufferViews"].get(n(a, "bufferView")?)?;
    if n(view, "buffer").unwrap_or(0) != 0 {
        return None;
    }
    let ty = a["componentType"].as_u64()?;
    let size = match ty { 5120 | 5121 => 1, 5122 | 5123 => 2, _ => 4 } * match a["type"].as_str()? { "SCALAR" => 1, "VEC2" => 2, "VEC3" => 3, _ => 4 };
    Some((n(view, "byteOffset").unwrap_or(0) + n(a, "byteOffset").unwrap_or(0), n(view, "byteStride").filter(|s| *s > 0).unwrap_or(size), n(a, "count")?, ty))
}

/// A view of `bytes` added at the end of the buffer (aligned to 4), and its index.
fn push_view(doc: &mut Value, bin: &mut Vec<u8>, bytes: &[u8], target: Option<u64>) -> usize {
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    let mut view = json!({"buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len()});
    if let Some(t) = target {
        view["target"] = json!(t);
    }
    bin.extend_from_slice(bytes);
    let views = doc["bufferViews"].as_array_mut().unwrap();
    views.push(view);
    views.len() - 1
}

/// Every triangle list's indices simplified to `keep` of its triangles (never under 12): the triangles before and after.
fn simplify(doc: &mut Value, bin: &mut Vec<u8>, keep: f32) -> (usize, usize) {
    let (mut before, mut after) = (0, 0);
    // an index list several primitives share is simplified once (against the first one's positions), in its own
    // accessor, so the old indices are left with nothing using them and leave the file
    let mut done: std::collections::HashSet<usize> = Default::default();
    let prims: Vec<(usize, usize)> = doc["meshes"].as_array().into_iter().flatten().enumerate().flat_map(|(m, mesh)| (0..mesh["primitives"].as_array().map_or(0, Vec::len)).map(move |p| (m, p))).collect();
    for (m, p) in prims {
        let prim = &doc["meshes"][m]["primitives"][p];
        if prim.get("mode").and_then(Value::as_u64).unwrap_or(4) != 4 {
            continue;
        }
        let (Some(ia), Some(pa)) = (n(prim, "indices"), n(&prim["attributes"], "POSITION")) else { continue };
        if done.contains(&ia) {
            continue;
        }
        let (Some((io, is, ic, ity)), Some((po, ps, pc, 5126))) = (span(doc, ia), span(doc, pa)) else { continue };
        before += ic / 3;
        let read = |i: usize| -> u32 {
            let at = io + i * is;
            match ity {
                5121 => bin[at] as u32,
                5123 => u16::from_le_bytes([bin[at], bin[at + 1]]) as u32,
                _ => u32::from_le_bytes([bin[at], bin[at + 1], bin[at + 2], bin[at + 3]]),
            }
        };
        if io + ic.saturating_sub(1) * is + 1 > bin.len() || po + pc * ps > bin.len() + ps - 12 {
            continue;
        }
        let indices: Vec<u32> = (0..ic).map(read).collect();
        let target = (((ic / 3) as f32 * keep) as usize).max(12).min(ic / 3) * 3;
        // the positions as meshoptimizer reads them: a stride of bytes, three floats at its start
        let positions = &bin[po..(po + pc * ps).min(bin.len())];
        let positions: Vec<u8> = if positions.len() % ps == 0 { positions.to_vec() } else { (0..pc).flat_map(|v| bin[po + v * ps..po + v * ps + 12].to_vec()).collect() };
        let stride = if positions.len() == pc * 12 { 12 } else { ps };
        let Ok(adapter) = meshopt::VertexDataAdapter::new(&positions, stride, 0) else { continue };
        // a loose error bound: the count is what a level asks for, the shape is kept as far as it allows
        let out = meshopt::simplify(&indices, &adapter, target, 0.05, meshopt::SimplifyOptions::None, None);
        let out = if out.len() >= 3 && out.len() < indices.len() { out } else { indices };
        after += out.len() / 3;
        let bytes: Vec<u8> = out.iter().flat_map(|i| i.to_le_bytes()).collect();
        let view = push_view(doc, bin, &bytes, Some(34963));
        doc["accessors"][ia] = json!({"bufferView": view, "componentType": 5125, "count": out.len(), "type": "SCALAR"});
        done.insert(ia);
    }
    (before, after)
}

/// Each embedded PNG or JPEG longer than `max` px made that long, in its own format: how many were.
fn shrink_pictures(doc: &mut Value, bin: &mut Vec<u8>, max: u32) -> usize {
    let mut made = 0;
    for i in 0..doc["images"].as_array().map_or(0, Vec::len) {
        let img = &doc["images"][i];
        let (Some(view), Some(mime)) = (n(img, "bufferView"), img["mimeType"].as_str().map(str::to_string)) else { continue };
        let v = &doc["bufferViews"][view];
        let (at, len) = (n(v, "byteOffset").unwrap_or(0), n(v, "byteLength").unwrap_or(0));
        let Some(bytes) = bin.get(at..at + len) else { continue };
        let Ok(pic) = image::load_from_memory(bytes) else { continue };
        if pic.width().max(pic.height()) <= max {
            continue;
        }
        let k = max as f64 / pic.width().max(pic.height()) as f64;
        let small = pic.resize_exact(((pic.width() as f64 * k).round() as u32).max(1), ((pic.height() as f64 * k).round() as u32).max(1), image::imageops::FilterType::CatmullRom);
        let mut out = std::io::Cursor::new(Vec::new());
        let wrote = if mime == "image/jpeg" { image::DynamicImage::ImageRgb8(small.to_rgb8()).write_to(&mut out, image::ImageFormat::Jpeg) } else { small.write_to(&mut out, image::ImageFormat::Png) };
        if wrote.is_err() || (mime != "image/jpeg" && mime != "image/png") {
            continue;
        }
        let view = push_view(doc, bin, out.get_ref(), None);
        doc["images"][i]["bufferView"] = json!(view);
        made += 1;
    }
    made
}

/// The buffer with only the views something still uses, the views numbered again.
fn compact(doc: &mut Value, bin: &[u8]) -> Vec<u8> {
    let views = doc["bufferViews"].as_array().cloned().unwrap_or_default();
    let mut used = vec![false; views.len()];
    let mut mark = |v: &Value| {
        if let Some(i) = n(v, "bufferView").filter(|i| *i < used.len()) {
            used[i] = true;
        }
    };
    for a in doc["accessors"].as_array().into_iter().flatten() {
        mark(a);
        mark(&a["sparse"]["indices"]);
        mark(&a["sparse"]["values"]);
    }
    for i in doc["images"].as_array().into_iter().flatten() {
        mark(i);
    }
    // anything an extension points at by "bufferView" stays too
    fn walk(v: &Value, mark: &mut dyn FnMut(&Value)) {
        match v {
            Value::Object(m) => {
                mark(v);
                m.values().for_each(|x| walk(x, mark));
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, mark)),
            _ => {}
        }
    }
    walk(&doc["extensions"].clone(), &mut mark);
    for m in doc["meshes"].as_array().into_iter().flatten() {
        walk(&m["primitives"], &mut |v| if v.get("extensions").is_some() { walk(&v["extensions"], &mut |x| if let Some(i) = n(x, "bufferView").filter(|i| *i < used.len()) { used[i] = true }) });
    }
    let mut out = Vec::new();
    let mut to = vec![usize::MAX; views.len()];
    let mut kept = Vec::new();
    for (i, v) in views.iter().enumerate().filter(|(i, _)| used[*i]) {
        while out.len() % 4 != 0 {
            out.push(0);
        }
        let (at, len) = (n(v, "byteOffset").unwrap_or(0), n(v, "byteLength").unwrap_or(0));
        let mut v = v.clone();
        v["byteOffset"] = json!(out.len());
        out.extend_from_slice(bin.get(at..at + len).unwrap_or(&[]));
        to[i] = kept.len();
        kept.push(v);
    }
    fn renumber(v: &mut Value, to: &[usize]) {
        match v {
            Value::Object(m) => {
                if let Some(i) = m.get("bufferView").and_then(Value::as_u64).map(|i| i as usize).filter(|i| *i < to.len() && to[*i] != usize::MAX) {
                    m.insert("bufferView".into(), json!(to[i]));
                }
                m.values_mut().for_each(|x| renumber(x, to));
            }
            Value::Array(a) => a.iter_mut().for_each(|x| renumber(x, to)),
            _ => {}
        }
    }
    doc["bufferViews"] = Value::Array(kept);
    for k in ["accessors", "images", "meshes", "extensions"] {
        if doc.get(k).is_some() {
            renumber(&mut doc[k], &to);
        }
    }
    doc["buffers"] = json!([{"byteLength": out.len()}]);
    out
}

pub fn lod(path: &Path, level: u32, texture: u32, out: &Path) -> R<Value> {
    let (mut doc, mut bin) = read(path)?;
    if doc["buffers"].as_array().is_some_and(|b| b.len() > 1 || b.iter().any(|x| x.get("uri").is_some())) {
        return Err(format!("{}: its buffers are separate files; raea media lod reads a .glb with its buffer inside it", path.display()));
    }
    let (before, after) = simplify(&mut doc, &mut bin, 0.5f32.powi(level.max(1) as i32));
    let pictures = shrink_pictures(&mut doc, &mut bin, texture.max(16));
    let bin = compact(&mut doc, &bin);
    write(out, &doc, &bin)?;
    let bytes = |p: &Path| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    Ok(json!({"wrote": out, "level": level.max(1), "triangles": [before, after], "pictures_made_smaller": pictures, "bytes": [bytes(path), bytes(out)]}))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat grid of `n` by `n` squares as a .glb: positions and u16 indices, and a picture nothing shrinks.
    fn grid(n: u16) -> (Value, Vec<u8>) {
        let mut bin = Vec::new();
        for y in 0..=n {
            for x in 0..=n {
                for v in [x as f32, 0.0, y as f32] {
                    bin.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
        let verts = (n as usize + 1) * (n as usize + 1);
        let index_at = bin.len();
        let mut count = 0;
        for y in 0..n {
            for x in 0..n {
                let a = y * (n + 1) + x;
                for i in [a, a + 1, a + n + 1, a + 1, a + n + 2, a + n + 1] {
                    bin.extend_from_slice(&i.to_le_bytes());
                    count += 1;
                }
            }
        }
        let doc = json!({
            "asset": {"version": "2.0"},
            "buffers": [{"byteLength": bin.len()}],
            "bufferViews": [{"buffer": 0, "byteOffset": 0, "byteLength": verts * 12}, {"buffer": 0, "byteOffset": index_at, "byteLength": count * 2}],
            "accessors": [{"bufferView": 0, "componentType": 5126, "count": verts, "type": "VEC3"}, {"bufferView": 1, "componentType": 5123, "count": count, "type": "SCALAR"}],
            "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 1}]}],
            "nodes": [{"mesh": 0}], "scenes": [{"nodes": [0]}], "scene": 0,
        });
        (doc, bin)
    }

    #[test]
    fn a_level_halves_the_triangles_and_the_old_indices_leave_the_file() {
        let dir = std::env::temp_dir().join(format!("raea-lod-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (doc, bin) = grid(16);
        let (src, out) = (dir.join("grid.glb"), dir.join("grid.lod1.glb"));
        write(&src, &doc, &bin).unwrap();
        let said = lod(&src, 1, 1024, &out).unwrap();
        let (before, after) = (said["triangles"][0].as_u64().unwrap(), said["triangles"][1].as_u64().unwrap());
        assert_eq!(before, 512);
        assert!(after <= 256 && after >= 12, "{after} triangles");
        // read back: one index view (the new one), the positions as they were, every index a vertex of the mesh
        let (doc, bin) = read(&out).unwrap();
        assert_eq!(doc["bufferViews"].as_array().unwrap().len(), 2);
        let (io, _, ic, ty) = span(&doc, doc["meshes"][0]["primitives"][0]["indices"].as_u64().unwrap() as usize).unwrap();
        assert_eq!((ic as u64, ty), (after * 3, 5125));
        assert!((0..ic).all(|i| u32::from_le_bytes(bin[io + i * 4..io + i * 4 + 4].try_into().unwrap()) < 289));
        assert_eq!(doc["accessors"][0]["count"], 289);
        let two = lod(&src, 2, 1024, &dir.join("grid.lod2.glb")).unwrap();
        assert!(two["triangles"][1].as_u64().unwrap() <= 128);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_file_that_is_not_a_glb_is_refused() {
        let p = std::env::temp_dir().join(format!("raea-lod-not-{}.glb", std::process::id()));
        std::fs::write(&p, b"plain text").unwrap();
        assert!(lod(&p, 1, 1024, &p.with_extension("out.glb")).unwrap_err().contains("not a .glb"));
        std::fs::remove_file(&p).ok();
    }
}
