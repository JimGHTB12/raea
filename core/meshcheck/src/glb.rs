//! A glTF binary read for measuring: every triangle of every mesh the default scene draws, in world space (node
//! transforms applied down the tree), and the file's JSON kept so a fixed copy can be written without touching its
//! buffers, materials or textures.

use serde_json::Value;

pub struct Glb {
    pub json: Value,
    /// the whole file, for rewriting with a new JSON chunk
    pub bytes: Vec<u8>,
    /// where the BIN chunk's data starts and ends in `bytes`
    bin: (usize, usize),
}

/// World-space triangles: positions flat (x y z) and indices by threes, as one welded-free list per primitive.
pub struct World {
    pub verts: Vec<[f64; 3]>,
    pub tris: Vec<[u32; 3]>,
    /// primitives drawn as something other than triangles, skipped
    pub skipped: usize,
}

type M4 = [f64; 16];
const IDENTITY: M4 = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];

fn mul(a: &M4, b: &M4) -> M4 {
    let mut o = [0.0; 16];
    for c in 0..4 {
        for r in 0..4 {
            o[c * 4 + r] = (0..4).map(|k| a[k * 4 + r] * b[c * 4 + k]).sum();
        }
    }
    o
}

fn apply(m: &M4, p: [f64; 3]) -> [f64; 3] {
    [m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12], m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13], m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14]]
}

/// A node's local matrix: its `matrix` (column-major), else translation, rotation (x y z w) and scale.
fn local(n: &Value) -> M4 {
    if let Some(m) = n["matrix"].as_array().filter(|m| m.len() == 16) {
        let mut o = [0.0; 16];
        for (i, v) in m.iter().enumerate() {
            o[i] = v.as_f64().unwrap_or(0.0);
        }
        return o;
    }
    let g = |k: &str, d: &[f64]| -> Vec<f64> { serde_json::from_value(n[k].clone()).unwrap_or_else(|_| d.to_vec()) };
    let t = g("translation", &[0.0, 0.0, 0.0]);
    let q = g("rotation", &[0.0, 0.0, 0.0, 1.0]);
    let s = g("scale", &[1.0, 1.0, 1.0]);
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let r = [
        1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y + z * w), 2.0 * (x * z - y * w), 0.0,
        2.0 * (x * y - z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z + x * w), 0.0,
        2.0 * (x * z + y * w), 2.0 * (y * z - x * w), 1.0 - 2.0 * (x * x + y * y), 0.0,
        0.0, 0.0, 0.0, 1.0,
    ];
    let mut o = r;
    for c in 0..3 {
        for k in 0..3 {
            o[c * 4 + k] *= s[c];
        }
    }
    o[12] = t[0];
    o[13] = t[1];
    o[14] = t[2];
    o
}

impl Glb {
    pub fn read(path: &str) -> Result<Glb, String> {
        let b = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        Glb::parse(b).map_err(|e| format!("{path}: {e}"))
    }

    pub fn parse(b: Vec<u8>) -> Result<Glb, String> {
        if b.len() < 20 || &b[0..4] != b"glTF" {
            return Err("not a GLB (a .gltf with loose files: export it as .glb first)".into());
        }
        let u = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) as usize;
        let jl = u(12);
        let json: Value = serde_json::from_slice(b.get(20..20 + jl).ok_or("a cut-off JSON chunk")?).map_err(|e| e.to_string())?;
        let at = 20 + jl;
        let bin = if b.len() >= at + 8 { (at + 8, (at + 8 + u(at)).min(b.len())) } else { (at, at) };
        Ok(Glb { json, bytes: b, bin })
    }

    fn bin(&self) -> &[u8] {
        &self.bytes[self.bin.0..self.bin.1]
    }

    /// An accessor's elements as f64s (`n` components each), reading floats and (normalized or not) integers.
    fn accessor(&self, i: usize) -> Result<(Vec<f64>, usize), String> {
        let a = &self.json["accessors"][i];
        let n = match a["type"].as_str().unwrap_or("") { "SCALAR" => 1, "VEC2" => 2, "VEC3" => 3, "VEC4" => 4, t => return Err(format!("accessor {i}: type {t}")) };
        let count = a["count"].as_u64().unwrap_or(0) as usize;
        let Some(view) = a["bufferView"].as_u64() else { return Ok((vec![0.0; count * n], n)) };
        let view = &self.json["bufferViews"][view as usize];
        if view["buffer"].as_u64().unwrap_or(0) != 0 {
            return Err("a buffer outside the file".into());
        }
        let ct = a["componentType"].as_u64().unwrap_or(5126);
        let size = match ct { 5120 | 5121 => 1, 5122 | 5123 => 2, _ => 4 };
        let off = view["byteOffset"].as_u64().unwrap_or(0) as usize + a["byteOffset"].as_u64().unwrap_or(0) as usize;
        let stride = match view["byteStride"].as_u64().unwrap_or(0) as usize { 0 => size * n, s => s };
        let norm = a["normalized"].as_bool().unwrap_or(false);
        let bin = self.bin();
        let mut out = Vec::with_capacity(count * n);
        for k in 0..count {
            for c in 0..n {
                let o = off + k * stride + c * size;
                let x = bin.get(o..o + size).ok_or_else(|| format!("accessor {i} runs past the buffer"))?;
                let v = match ct {
                    5126 => f32::from_le_bytes([x[0], x[1], x[2], x[3]]) as f64,
                    5125 => u32::from_le_bytes([x[0], x[1], x[2], x[3]]) as f64,
                    5123 => { let v = u16::from_le_bytes([x[0], x[1]]) as f64; if norm { v / 65535.0 } else { v } }
                    5122 => { let v = i16::from_le_bytes([x[0], x[1]]) as f64; if norm { (v / 32767.0).max(-1.0) } else { v } }
                    5121 => { let v = x[0] as f64; if norm { v / 255.0 } else { v } }
                    _ => { let v = x[0] as i8 as f64; if norm { (v / 127.0).max(-1.0) } else { v } }
                };
                out.push(v);
            }
        }
        Ok((out, n))
    }

    /// Every triangle the default scene draws, in world space.
    pub fn world(&self) -> Result<World, String> {
        let mut w = World { verts: Vec::new(), tris: Vec::new(), skipped: 0 };
        let scene = self.json["scene"].as_u64().unwrap_or(0) as usize;
        let roots: Vec<usize> = self.json["scenes"][scene]["nodes"].as_array().map(|a| a.iter().filter_map(|v| v.as_u64().map(|v| v as usize)).collect()).unwrap_or_default();
        let mut stack: Vec<(usize, M4, usize)> = roots.into_iter().map(|r| (r, IDENTITY, 0)).collect();
        while let Some((i, parent, depth)) = stack.pop() {
            if depth > 64 {
                return Err("the node tree is deeper than 64 (a cycle?)".into());
            }
            let node = &self.json["nodes"][i];
            let m = mul(&parent, &local(node));
            if let Some(mesh) = node["mesh"].as_u64() {
                for prim in self.json["meshes"][mesh as usize]["primitives"].as_array().into_iter().flatten() {
                    if prim["mode"].as_u64().unwrap_or(4) != 4 {
                        w.skipped += 1;
                        continue;
                    }
                    let Some(pos) = prim["attributes"]["POSITION"].as_u64() else { continue };
                    let (p, _) = self.accessor(pos as usize)?;
                    let base = w.verts.len() as u32;
                    w.verts.extend(p.chunks_exact(3).map(|c| apply(&m, [c[0], c[1], c[2]])));
                    let count = (p.len() / 3) as u32;
                    let idx: Vec<u32> = match prim["indices"].as_u64() {
                        Some(ix) => self.accessor(ix as usize)?.0.into_iter().map(|v| v as u32).collect(),
                        None => (0..count).collect(),
                    };
                    for t in idx.chunks_exact(3) {
                        if t.iter().all(|&k| k < count) {
                            w.tris.push([base + t[0], base + t[1], base + t[2]]);
                        }
                    }
                }
            }
            for c in node["children"].as_array().into_iter().flatten().filter_map(|c| c.as_u64()) {
                stack.push((c as usize, m, depth + 1));
            }
        }
        Ok(w)
    }

    /// The pixel size of each image the file holds (PNG or JPEG headers), and the ones it only names.
    pub fn images(&self) -> Vec<Option<(u32, u32)>> {
        let bin = self.bin();
        self.json["images"].as_array().into_iter().flatten().map(|img| {
            let view = img["bufferView"].as_u64()?;
            let v = &self.json["bufferViews"][view as usize];
            let off = v["byteOffset"].as_u64().unwrap_or(0) as usize;
            let len = v["byteLength"].as_u64().unwrap_or(0) as usize;
            image_size(bin.get(off..off + len)?)
        }).collect()
    }

    /// The file again with `json` as its JSON chunk; the binary chunk is copied byte for byte.
    pub fn with_json(&self, json: &Value) -> Result<Vec<u8>, String> {
        let mut js = serde_json::to_vec(json).map_err(|e| e.to_string())?;
        while js.len() % 4 != 0 {
            js.push(b' ');
        }
        let rest = &self.bytes[(20 + u32::from_le_bytes([self.bytes[12], self.bytes[13], self.bytes[14], self.bytes[15]]) as usize).min(self.bytes.len())..];
        let total = 20 + js.len() + rest.len();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(b"glTF");
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&(total as u32).to_le_bytes());
        out.extend_from_slice(&(js.len() as u32).to_le_bytes());
        out.extend_from_slice(b"JSON");
        out.extend_from_slice(&js);
        out.extend_from_slice(rest);
        Ok(out)
    }
}

/// A PNG's or a JPEG's width and height, from its header.
fn image_size(b: &[u8]) -> Option<(u32, u32)> {
    if b.len() > 24 && &b[1..4] == b"PNG" {
        return Some((u32::from_be_bytes([b[16], b[17], b[18], b[19]]), u32::from_be_bytes([b[20], b[21], b[22], b[23]])));
    }
    if b.len() > 4 && b[0] == 0xff && b[1] == 0xd8 {
        let mut i = 2;
        while i + 9 < b.len() {
            if b[i] != 0xff {
                return None;
            }
            let marker = b[i + 1];
            let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
            if matches!(marker, 0xc0..=0xc3 | 0xc5..=0xc7 | 0xc9..=0xcb | 0xcd..=0xcf) {
                return Some((u16::from_be_bytes([b[i + 7], b[i + 8]]) as u32, u16::from_be_bytes([b[i + 5], b[i + 6]]) as u32));
            }
            i += 2 + len;
        }
    }
    None
}
