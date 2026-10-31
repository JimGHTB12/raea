//! glTF: nodes cut out of a .glb as their own file (a body's parts), and their triangles.

use std::path::Path;

use rapier3d::glamx::glam;
use serde_json::{Value, json};

use super::*;

pub(crate) struct Glb {
    j: Value,
    bin: Vec<u8>,
}

impl Glb {
    pub(crate) fn read(p: &Path) -> R<Glb> {
        let b = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
        if b.len() < 20 || &b[0..4] != b"glTF" {
            return Err(format!("{}: not a .glb", p.display()));
        }
        let jl = u32::from_le_bytes(b[12..16].try_into().unwrap()) as usize;
        let j: Value = serde_json::from_slice(&b[20..20 + jl]).map_err(|e| format!("{}: {e}", p.display()))?;
        let mut bin = Vec::new();
        let at = 20 + jl;
        if b.len() >= at + 8 {
            let bl = u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
            bin = b[at + 8..at + 8 + bl].to_vec();
        }
        Ok(Glb { j, bin })
    }

    fn arr(&self, k: &str) -> Vec<Value> {
        self.j.get(k).and_then(Value::as_array).cloned().unwrap_or_default()
    }

    /// Each node's world matrix (column-major), parents first.
    fn world(&self) -> Vec<glam::Mat4> {
        let nodes = self.arr("nodes");
        let mut w = vec![glam::Mat4::IDENTITY; nodes.len()];
        let local = |n: &Value| -> glam::Mat4 {
            if let Some(m) = n.get("matrix").and_then(Value::as_array) {
                let a: Vec<f32> = m.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect();
                if a.len() == 16 {
                    return glam::Mat4::from_cols_slice(&a);
                }
            }
            let v = |k: &str, d: &[f32]| -> Vec<f32> { n.get(k).and_then(Value::as_array).map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect()).unwrap_or(d.to_vec()) };
            let (t, r, s) = (v("translation", &[0.0; 3]), v("rotation", &[0.0, 0.0, 0.0, 1.0]), v("scale", &[1.0; 3]));
            glam::Mat4::from_scale_rotation_translation(glam::Vec3::new(s[0], s[1], s[2]), glam::Quat::from_xyzw(r[0], r[1], r[2], r[3]), glam::Vec3::new(t[0], t[1], t[2]))
        };
        let scenes = self.arr("scenes");
        let roots: Vec<usize> = scenes.first().and_then(|s| s.get("nodes")).and_then(Value::as_array).map(|a| a.iter().filter_map(|x| x.as_u64().map(|x| x as usize)).collect()).unwrap_or_else(|| (0..nodes.len()).collect());
        let mut stack: Vec<(usize, glam::Mat4)> = roots.into_iter().map(|r| (r, glam::Mat4::IDENTITY)).collect();
        while let Some((i, parent)) = stack.pop() {
            let Some(n) = nodes.get(i) else { continue };
            w[i] = parent * local(n);
            for c in n.get("children").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_u64) {
                stack.push((c as usize, w[i]));
            }
        }
        w
    }

    pub(crate) fn nodes_named(&self, names: &[String]) -> R<Vec<usize>> {
        let nodes = self.arr("nodes");
        let mut out = Vec::new();
        for name in names {
            let i = nodes.iter().position(|n| n.get("name").and_then(Value::as_str) == Some(name)).ok_or(format!("no node named {name:?}"))?;
            out.push(i);
        }
        Ok(out)
    }

    fn view(&self, accessor: usize) -> R<(Vec<u8>, usize, String, String, usize)> {
        let a = self.arr("accessors").get(accessor).cloned().ok_or("no such accessor")?;
        let count = a["count"].as_u64().unwrap_or(0) as usize;
        let ty = a["type"].as_str().unwrap_or("").to_string();
        let ct = a["componentType"].as_u64().unwrap_or(0);
        let comp = match ct { 5126 | 5125 => 4, 5123 | 5122 => 2, _ => 1 };
        let n = match ty.as_str() { "SCALAR" => 1, "VEC2" => 2, "VEC3" => 3, "VEC4" => 4, "MAT4" => 16, _ => 1 };
        let bv = self.arr("bufferViews").get(a["bufferView"].as_u64().unwrap_or(0) as usize).cloned().ok_or("no bufferView")?;
        let off = bv["byteOffset"].as_u64().unwrap_or(0) as usize + a["byteOffset"].as_u64().unwrap_or(0) as usize;
        let stride = bv["byteStride"].as_u64().map(|s| s as usize).unwrap_or(comp * n);
        let mut out = Vec::with_capacity(count * comp * n);
        for i in 0..count {
            let s = off + i * stride;
            out.extend_from_slice(self.bin.get(s..s + comp * n).ok_or("accessor past the buffer")?);
        }
        Ok((out, count, ty, ct.to_string(), comp * n))
    }

    /// The nodes' triangles in the world: positions and indices.
    pub(crate) fn triangles(&self, nodes: &[usize]) -> R<(Vec<glam::Vec3>, Vec<[u32; 3]>)> {
        let w = self.world();
        let all = self.arr("nodes");
        let meshes = self.arr("meshes");
        let (mut pos, mut tri) = (Vec::new(), Vec::new());
        for &i in nodes {
            let Some(m) = all[i].get("mesh").and_then(Value::as_u64) else { continue };
            for prim in meshes[m as usize]["primitives"].as_array().into_iter().flatten() {
                let base = pos.len() as u32;
                let (b, count, _, _, _) = self.view(prim["attributes"]["POSITION"].as_u64().ok_or("a primitive without positions")? as usize)?;
                for k in 0..count {
                    let f = |o: usize| f32::from_le_bytes(b[k * 12 + o..k * 12 + o + 4].try_into().unwrap());
                    pos.push(w[i].transform_point3(glam::Vec3::new(f(0), f(4), f(8))));
                }
                let idx: Vec<u32> = match prim.get("indices").and_then(Value::as_u64) {
                    Some(ia) => {
                        let (b, count, _, ct, _) = self.view(ia as usize)?;
                        (0..count).map(|k| match ct.as_str() {
                            "5125" => u32::from_le_bytes(b[k * 4..k * 4 + 4].try_into().unwrap()),
                            "5123" => u16::from_le_bytes(b[k * 2..k * 2 + 2].try_into().unwrap()) as u32,
                            _ => b[k] as u32,
                        }).collect()
                    }
                    None => (0..count as u32).collect(),
                };
                for t in idx.chunks_exact(3) {
                    tri.push([base + t[0], base + t[1], base + t[2]]);
                }
            }
        }
        Ok((pos, tri))
    }

    /// A new .glb holding only these nodes (each with its world matrix, so it stands where it stood), their meshes,
    /// materials and pictures; `material` replaces every material (a glitch's echo or missing texture).
    fn subset(&self, nodes: &[usize], material: Option<&Value>) -> R<Vec<u8>> {
        let w = self.world();
        let all = self.arr("nodes");
        let (accessors, views, meshes) = (self.arr("accessors"), self.arr("bufferViews"), self.arr("meshes"));
        let (materials, textures, images, samplers) = (self.arr("materials"), self.arr("textures"), self.arr("images"), self.arr("samplers"));
        let mut bin: Vec<u8> = Vec::new();
        let mut new_views: Vec<Value> = Vec::new();
        let mut view_map = std::collections::HashMap::new();
        let mut copy_view = |v: usize, bin: &mut Vec<u8>, new_views: &mut Vec<Value>| -> usize {
            *view_map.entry(v).or_insert_with(|| {
                let bv = &views[v];
                let off = bv["byteOffset"].as_u64().unwrap_or(0) as usize;
                let len = bv["byteLength"].as_u64().unwrap_or(0) as usize;
                while bin.len() % 4 != 0 {
                    bin.push(0);
                }
                let at = bin.len();
                bin.extend_from_slice(&self.bin[off..off + len]);
                let mut nv = bv.clone();
                nv["buffer"] = json!(0);
                nv["byteOffset"] = json!(at);
                new_views.push(nv);
                new_views.len() - 1
            })
        };
        let mut acc_map = std::collections::HashMap::new();
        let mut new_acc: Vec<Value> = Vec::new();
        let mut mat_map = std::collections::HashMap::new();
        let mut new_mats: Vec<Value> = Vec::new();
        let (mut tex_map, mut img_map, mut smp_map) = (std::collections::HashMap::new(), std::collections::HashMap::new(), std::collections::HashMap::new());
        let (mut new_tex, mut new_img, mut new_smp): (Vec<Value>, Vec<Value>, Vec<Value>) = (Vec::new(), Vec::new(), Vec::new());
        let mut new_meshes = Vec::new();
        let mut new_nodes = Vec::new();
        let override_mat = material.map(|m| {
            new_mats.push(m.clone());
            0usize
        });
        for &i in nodes {
            let n = &all[i];
            let mut nn = json!({"name": n.get("name").cloned().unwrap_or(Value::Null), "matrix": w[i].to_cols_array()});
            if let Some(m) = n.get("mesh").and_then(Value::as_u64) {
                let mut mesh = meshes[m as usize].clone();
                for prim in mesh["primitives"].as_array_mut().into_iter().flatten() {
                    let mut remap = |a: u64, bin: &mut Vec<u8>, new_views: &mut Vec<Value>| -> u64 {
                        *acc_map.entry(a).or_insert_with(|| {
                            let mut acc = accessors[a as usize].clone();
                            if let Some(v) = acc.get("bufferView").and_then(Value::as_u64) {
                                acc["bufferView"] = json!(copy_view(v as usize, bin, new_views));
                            }
                            new_acc.push(acc);
                            (new_acc.len() - 1) as u64
                        })
                    };
                    if let Some(attrs) = prim["attributes"].as_object_mut() {
                        for v in attrs.values_mut() {
                            *v = json!(remap(v.as_u64().unwrap_or(0), &mut bin, &mut new_views));
                        }
                    }
                    if let Some(ix) = prim.get("indices").and_then(Value::as_u64) {
                        prim["indices"] = json!(remap(ix, &mut bin, &mut new_views));
                    }
                    // morph targets aren't carried
                    if let Some(o) = prim.as_object_mut() {
                        o.remove("targets");
                    }
                    let mi = prim.get("material").and_then(Value::as_u64);
                    let nm = match (override_mat, mi) {
                        (Some(o), _) => Some(o),
                        (None, Some(mi)) => Some(*mat_map.entry(mi).or_insert_with(|| {
                            let mut mat = materials[mi as usize].clone();
                            // every texture reference in the material, wherever it sits (pbr, normal, extensions)
                            fn walk(v: &mut Value, f: &mut dyn FnMut(u64) -> u64) {
                                match v {
                                    Value::Object(o) => {
                                        for (k, x) in o.iter_mut() {
                                            if k == "index" && x.is_u64() {
                                                *x = json!(f(x.as_u64().unwrap()));
                                            } else {
                                                walk(x, f);
                                            }
                                        }
                                    }
                                    Value::Array(a) => a.iter_mut().for_each(|x| walk(x, f)),
                                    _ => {}
                                }
                            }
                            walk(&mut mat, &mut |t| {
                                *tex_map.entry(t).or_insert_with(|| {
                                    let mut tex = textures[t as usize].clone();
                                    if let Some(s) = tex.get("source").and_then(Value::as_u64) {
                                        let ni = *img_map.entry(s).or_insert_with(|| {
                                            let mut img = images[s as usize].clone();
                                            if let Some(v) = img.get("bufferView").and_then(Value::as_u64) {
                                                img["bufferView"] = json!(copy_view(v as usize, &mut bin, &mut new_views));
                                            }
                                            new_img.push(img);
                                            (new_img.len() - 1) as u64
                                        });
                                        tex["source"] = json!(ni);
                                    }
                                    if let Some(s) = tex.get("sampler").and_then(Value::as_u64) {
                                        let ns = *smp_map.entry(s).or_insert_with(|| {
                                            new_smp.push(samplers[s as usize].clone());
                                            (new_smp.len() - 1) as u64
                                        });
                                        tex["sampler"] = json!(ns);
                                    }
                                    new_tex.push(tex);
                                    (new_tex.len() - 1) as u64
                                })
                            });
                            new_mats.push(mat);
                            new_mats.len() - 1
                        })),
                        (None, None) => None,
                    };
                    if let Some(nm) = nm {
                        prim["material"] = json!(nm);
                    }
                }
                new_meshes.push(mesh);
                nn["mesh"] = json!(new_meshes.len() - 1);
            }
            new_nodes.push(nn);
        }
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let mut j = json!({
            "asset": {"version": "2.0", "generator": "raea physics.part"},
            "scene": 0, "scenes": [{"nodes": (0..new_nodes.len()).collect::<Vec<_>>()}],
            "nodes": new_nodes, "meshes": new_meshes, "accessors": new_acc, "bufferViews": new_views,
            "buffers": [{"byteLength": bin.len()}], "materials": new_mats,
        });
        if !new_tex.is_empty() {
            j["textures"] = json!(new_tex);
            j["images"] = json!(new_img);
        }
        if !new_smp.is_empty() {
            j["samplers"] = json!(new_smp);
        }
        let mut used: Vec<Value> = self.j.get("extensionsUsed").and_then(Value::as_array).cloned().unwrap_or_default();
        if material.is_some_and(|m| m.pointer("/extensions/KHR_materials_unlit").is_some()) && !used.contains(&json!("KHR_materials_unlit")) {
            used.push(json!("KHR_materials_unlit"));
        }
        if !used.is_empty() {
            j["extensionsUsed"] = json!(used);
        }
        Ok(glb_bytes(&j, &bin))
    }
}

fn glb_bytes(j: &Value, bin: &[u8]) -> Vec<u8> {
    let mut js = j.to_string().into_bytes();
    while js.len() % 4 != 0 {
        js.push(b' ');
    }
    let total = 12 + 8 + js.len() + 8 + bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend(b"glTF");
    out.extend(2u32.to_le_bytes());
    out.extend((total as u32).to_le_bytes());
    out.extend((js.len() as u32).to_le_bytes());
    out.extend(b"JSON");
    out.extend(&js);
    out.extend((bin.len() as u32).to_le_bytes());
    out.extend(b"BIN\0");
    out.extend(bin);
    out
}

/// The material a part's variant asks for: `{missing: true}` (the magenta and black checker of a texture that didn't
/// load), or `{color, emissive, opacity, unlit}`.
fn variant_material(m: &Value, bin_images: &mut Vec<Vec<u8>>) -> Value {
    let rgb = |k: &str, d: [f64; 3]| -> [f64; 3] {
        m.get(k).and_then(Value::as_str).map(|h| {
            let h = h.trim_start_matches('#');
            let b = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("ff"), 16).unwrap_or(255) as f64 / 255.0;
            // sRGB to linear (glTF's factors are linear)
            [b(0), b(2), b(4)].map(|c| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) })
        }).unwrap_or(d)
    };
    if m.get("missing").and_then(Value::as_bool) == Some(true) {
        bin_images.push(checker_png());
        return json!({"name": "missing", "pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0.0, "roughnessFactor": 0.6},
            "emissiveFactor": [0.25, 0.0, 0.25]});
    }
    let c = rgb("color", [1.0, 1.0, 1.0]);
    let op = m.get("opacity").and_then(Value::as_f64).unwrap_or(1.0);
    let mut mat = json!({"name": "variant", "pbrMetallicRoughness": {"baseColorFactor": [c[0], c[1], c[2], op], "metallicFactor": 0.0, "roughnessFactor": m.get("roughness").and_then(Value::as_f64).unwrap_or(0.5)},
        "emissiveFactor": rgb("emissive", [0.0, 0.0, 0.0]), "doubleSided": true});
    if op < 1.0 {
        mat["alphaMode"] = json!("BLEND");
    }
    if m.get("unlit").and_then(Value::as_bool) == Some(true) {
        mat["extensions"] = json!({"KHR_materials_unlit": {}});
    }
    mat
}

fn checker_png() -> Vec<u8> {
    let img = image::RgbaImage::from_fn(64, 64, |x, y| if ((x / 8) + (y / 8)) % 2 == 0 { image::Rgba([255, 0, 255, 255]) } else { image::Rgba([0, 0, 0, 255]) });
    let mut out = std::io::Cursor::new(Vec::new());
    let _ = image::DynamicImage::ImageRgba8(img).write_to(&mut out, image::ImageFormat::Png);
    out.into_inner()
}

/// `physics.part`: the nodes cut out of a .glb (and a variant's material), kept by key; their centre and bounds.
pub fn part(at: &Place, v: &Value) -> R<Value> {
    let src = v.get("src").and_then(Value::as_str).ok_or("physics.part: src, a .glb (staticFile(\"x.glb\"))")?;
    let nodes: Vec<String> = v.get("nodes").and_then(Value::as_array).ok_or("physics.part: nodes, the glTF nodes' names")?.iter().filter_map(|x| x.as_str().map(String::from)).collect();
    let path = (at.media)(src);
    let material = v.get("material").filter(|m| m.is_object()).cloned();
    let key = hex_sha(json!(["part", 2, file_id(&path)?, nodes, material]).to_string().as_bytes());
    let dir = &at.store;
    let out = dir.join(format!("{key}.glb"));
    let meta = dir.join(format!("{key}.glb.json"));
    if let Ok(b) = std::fs::read(&meta) {
        if out.exists() {
            if let Ok(m) = serde_json::from_slice::<Value>(&b) {
                return Ok(m);
            }
        }
    }
    let g = Glb::read(&path)?;
    let ix = g.nodes_named(&nodes).map_err(|e| format!("physics.part {src}: {e}"))?;
    let (pos, tri) = g.triangles(&ix)?;
    let (mut lo, mut hi) = (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN));
    for p in &pos {
        lo = lo.min(*p);
        hi = hi.max(*p);
    }
    let mut extra = Vec::new();
    let mat = material.as_ref().map(|m| variant_material(m, &mut extra));
    let mut bytes = g.subset(&ix, mat.as_ref())?;
    if let Some(png) = extra.pop() {
        // the checker as the file's one picture
        let gg = parse_glb(&bytes)?;
        let (mut j, mut bin) = gg;
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let at = bin.len();
        bin.extend(&png);
        let views = j["bufferViews"].as_array_mut().unwrap();
        views.push(json!({"buffer": 0, "byteOffset": at, "byteLength": png.len()}));
        let vi = views.len() - 1;
        j["images"] = json!([{"bufferView": vi, "mimeType": "image/png"}]);
        j["samplers"] = json!([{"magFilter": 9728, "minFilter": 9728}]);
        j["textures"] = json!([{"source": 0, "sampler": 0}]);
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        j["buffers"] = json!([{"byteLength": bin.len()}]);
        bytes = glb_bytes(&j, &bin);
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    std::fs::write(&out, &bytes).map_err(|e| format!("{}: {e}", out.display()))?;
    let m = json!({"file": out.display().to_string(), "centre": ((lo + hi) / 2.0).to_array(), "lo": lo.to_array(), "hi": hi.to_array(), "tris": tri.len()});
    std::fs::write(&meta, m.to_string()).map_err(|e| format!("{}: {e}", meta.display()))?;
    Ok(m)
}

fn parse_glb(b: &[u8]) -> R<(Value, Vec<u8>)> {
    let jl = u32::from_le_bytes(b[12..16].try_into().unwrap()) as usize;
    let j: Value = serde_json::from_slice(&b[20..20 + jl]).map_err(|e| e.to_string())?;
    let at = 20 + jl;
    let bl = u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
    Ok((j, b[at + 8..at + 8 + bl].to_vec()))
}
