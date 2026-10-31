//! Splat files read: 3DGS .ply (with spherical harmonics), antimatter15 .splat and Niantic .spz, into one cloud, cached
//! by path and placed in the world by a `.ply.json` beside it.

use super::*;

/// One splat file, read and placed by its sidecar's transform: what the GPU draws and what the sort reads.
pub(crate) struct Cloud {
    pub(crate) n: usize,
    /// per splat: centre xyz + opacity, scale xyz + 0, rotation xyzw, colour rgb + 0 (the shader's `Gauss`)
    pub(crate) gauss: Vec<[f32; 16]>,
    /// the rest of the spherical harmonics, f16 pairs, `sh_words` per splat
    pub(crate) sh: Vec<u32>,
    pub(crate) sh_deg: u32,
    pub(crate) sh_words: u32,
    /// the world's rotation of the file (its transpose turns view directions back into the file's)
    pub(crate) rot: Mat3,
    pub(crate) center: Vec3,
    pub(crate) radius: f32,
}

impl Cloud {
    pub(crate) fn pos(&self, i: usize) -> Vec3 {
        let g = &self.gauss[i];
        Vec3::new(g[0], g[1], g[2])
    }
}

pub(crate) static CLOUDS: Mutex<Option<HashMap<PathBuf, Arc<Cloud>>>> = Mutex::new(None);

/// A splat file's cloud, read the first time it's asked for (a file is read once per process, whatever uses it).
pub(crate) fn cloud(path: &Path) -> Result<Arc<Cloud>, String> {
    let mut g = CLOUDS.lock().unwrap();
    let map = g.get_or_insert_with(HashMap::new);
    if let Some(c) = map.get(path) {
        return Ok(c.clone());
    }
    let c = Arc::new(read_cloud(path)?);
    map.insert(path.to_path_buf(), c.clone());
    Ok(c)
}

/// Gaussians as a file has them, before placing: centre, linear scale, rotation (w x y z, normalised), opacity 0..1,
/// degree-0 colour (0.5 + C0 dc), and the rest of the harmonics as floats, coefficient-major (k * 3 + channel).
pub(crate) struct Raw {
    pub(crate) pos: Vec<[f32; 3]>,
    pub(crate) scale: Vec<[f32; 3]>,
    pub(crate) rot: Vec<[f32; 4]>,
    pub(crate) opacity: Vec<f32>,
    pub(crate) color: Vec<[f32; 3]>,
    pub(crate) deg: u32,
    pub(crate) rest: Vec<f32>,
}

pub(crate) const SH_C0: f32 = 0.282_094_8;

pub(crate) fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

pub(crate) fn read_cloud(path: &Path) -> Result<Cloud, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("splat {}: {e}", path.display()))?;
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let raw = match ext.as_str() {
        "ply" => read_ply(&bytes),
        "splat" => read_splat(&bytes),
        #[cfg(feature = "native")]
        "spz" => read_spz(&bytes),
        #[cfg(not(feature = "native"))]
        "spz" => Err(".spz needs the native engine".to_string()),
        _ => Err("a .ply (3D gaussian splatting), .splat or .spz file".to_string()),
    }
    .map_err(|e| format!("splat {}: {e}", path.display()))?;
    // the sidecar's placing (raea splat train writes it): a rotation, a uniform scale and an offset
    let side = path.with_file_name(format!("{}.json", path.file_name().and_then(|f| f.to_str()).unwrap_or("")));
    let m = match std::fs::read_to_string(&side) {
        Ok(s) => {
            let v: serde_json::Value = serde_json::from_str(&s).map_err(|e| format!("{}: {e}", side.display()))?;
            match v.get("transform").and_then(|t| t.as_array()) {
                Some(a) if a.len() == 16 => {
                    let f: Vec<f32> = a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect();
                    Mat4::from_cols_slice(&f)
                }
                _ => Mat4::IDENTITY,
            }
        }
        Err(_) => Mat4::IDENTITY,
    };
    Ok(place(raw, m))
}

pub(crate) fn place(raw: Raw, m: Mat4) -> Cloud {
    let (s3, q, t) = m.to_scale_rotation_translation();
    let s = (s3.x.abs() + s3.y.abs() + s3.z.abs()) / 3.0;
    let rot = Mat3::from_quat(q);
    let deg = raw.deg;
    let k = ((deg + 1) * (deg + 1) - 1) as usize;
    let sh_words = ((k * 3) as u32).div_ceil(2);
    let mut gauss = Vec::with_capacity(raw.pos.len());
    let mut sh = Vec::with_capacity(raw.pos.len() * sh_words as usize);
    for i in 0..raw.pos.len() {
        // nothing a pixel would show
        if raw.opacity[i] < 1.0 / 255.0 {
            continue;
        }
        let p = q * Vec3::from_array(raw.pos[i]) * s + t;
        let [w, x, y, z] = raw.rot[i];
        let r = (q * Quat::from_xyzw(x, y, z, w)).normalize();
        let sc = Vec3::from_array(raw.scale[i]) * s;
        let c = raw.color[i];
        gauss.push([p.x, p.y, p.z, raw.opacity[i], sc.x, sc.y, sc.z, 0.0, r.x, r.y, r.z, r.w, c[0], c[1], c[2], 0.0]);
        let rest = &raw.rest[i * k * 3..(i + 1) * k * 3];
        for w in 0..sh_words as usize {
            let a = rest.get(w * 2).copied().unwrap_or(0.0);
            let b = rest.get(w * 2 + 1).copied().unwrap_or(0.0);
            sh.push(f16_bits(a) as u32 | (f16_bits(b) as u32) << 16);
        }
    }
    if sh.is_empty() {
        sh.push(0);
    }
    let n = gauss.len();
    // a robust centre and size: the median on each axis, and how far 80% of the splats lie from it
    let (center, radius) = if n == 0 {
        (Vec3::ZERO, 1.0)
    } else {
        let med = |a: usize| {
            let mut v: Vec<f32> = gauss.iter().map(|g| g[a]).collect();
            let mid = v.len() / 2;
            *v.select_nth_unstable_by(mid, f32::total_cmp).1
        };
        let c = Vec3::new(med(0), med(1), med(2));
        let mut d: Vec<f32> = gauss.iter().map(|g| (Vec3::new(g[0], g[1], g[2]) - c).length()).collect();
        let k80 = (d.len() * 8 / 10).min(d.len() - 1);
        let r = *d.select_nth_unstable_by(k80, f32::total_cmp).1;
        (c, r.max(1e-3))
    };
    Cloud { n, gauss, sh, sh_deg: deg, sh_words, rot, center, radius }
}

/// f32 to IEEE half (round to nearest even), as WGSL's unpack2x16float reads it.
pub(crate) fn f16_bits(f: f32) -> u16 {
    let x = f.to_bits();
    let sign = ((x >> 16) & 0x8000) as u16;
    let e = ((x >> 23) & 0xff) as i32;
    let m = x & 0x7f_ffff;
    if e == 0xff {
        return sign | 0x7c00 | if m != 0 { 0x200 } else { 0 };
    }
    let e2 = e - 127 + 15;
    if e2 >= 0x1f {
        return sign | 0x7c00;
    }
    if e2 <= 0 {
        if e2 < -10 {
            return sign;
        }
        let m = m | 0x80_0000;
        let shift = (14 - e2) as u32;
        let half = 1u32 << (shift - 1);
        let mut r = m >> shift;
        let rem = m & ((1 << shift) - 1);
        if rem > half || (rem == half && (r & 1) == 1) {
            r += 1;
        }
        return sign | r as u16;
    }
    let mut r = ((e2 as u32) << 10) | (m >> 13);
    let rem = m & 0x1fff;
    if rem > 0x1000 || (rem == 0x1000 && (r & 1) == 1) {
        r += 1;
    }
    sign | r as u16
}

/// A 3DGS PLY: binary little endian vertices with x y z, f_dc_0..2, f_rest_*, opacity (logit), scale_0..2 (log),
/// rot_0..3 (w x y z). Properties may come in any order, as float, double or uchar.
pub(crate) fn read_ply(b: &[u8]) -> Result<Raw, String> {
    let end = b.windows(11).position(|w| w == b"end_header\n").ok_or("no end_header: not a PLY")? + 11;
    let head = std::str::from_utf8(&b[..end]).map_err(|_| "a PLY header that isn't text")?;
    if !head.contains("binary_little_endian") {
        return Err("only binary_little_endian PLY files".into());
    }
    let mut n = 0usize;
    let mut props: Vec<(String, usize)> = Vec::new();
    let mut in_vertex = false;
    let mut skip_before = 0usize;
    for line in head.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        match w.as_slice() {
            ["element", "vertex", c] => {
                n = c.parse().map_err(|_| "a bad vertex count")?;
                in_vertex = true;
            }
            ["element", ..] => {
                if !in_vertex && n == 0 {
                    // an element before the vertices: not handled (none of the 3DGS writers put one there)
                    skip_before += 1;
                }
                in_vertex = false;
            }
            ["property", ty, name] if in_vertex => {
                let size = match *ty {
                    "float" | "float32" | "int" | "uint" | "int32" | "uint32" => 4,
                    "double" | "float64" => 8,
                    "uchar" | "uint8" | "char" | "int8" => 1,
                    "short" | "ushort" | "int16" | "uint16" => 2,
                    _ => return Err(format!("a property type I can't read: {ty}")),
                };
                props.push((format!("{ty}:{name}"), size));
            }
            _ => {}
        }
    }
    if skip_before > 0 {
        return Err("an element before the vertices".into());
    }
    let stride: usize = props.iter().map(|p| p.1).sum();
    let need = end + n * stride;
    if b.len() < need {
        return Err(format!("{n} vertices of {stride} bytes need {need} bytes, the file has {}", b.len()));
    }
    let mut offs: HashMap<&str, (usize, &str)> = HashMap::new();
    let mut o = 0;
    for (tn, size) in &props {
        let (ty, name) = tn.split_once(':').unwrap();
        offs.insert(name, (o, ty));
        o += size;
    }
    let read = |i: usize, name: &str| -> Option<f32> {
        let (off, ty) = *offs.get(name)?;
        let at = end + i * stride + off;
        Some(match ty {
            "double" | "float64" => f64::from_le_bytes(b[at..at + 8].try_into().unwrap()) as f32,
            "uchar" | "uint8" => b[at] as f32 / 255.0,
            "float" | "float32" => f32::from_le_bytes(b[at..at + 4].try_into().unwrap()),
            _ => 0.0,
        })
    };
    for need in ["x", "y", "z"] {
        if !offs.contains_key(need) {
            return Err(format!("no {need} property"));
        }
    }
    let gs = offs.contains_key("scale_0") && offs.contains_key("rot_0") && offs.contains_key("opacity");
    if !gs {
        return Err("not a gaussian splat PLY (no scale_0, rot_0 or opacity; a plain point cloud?)".into());
    }
    let rest_n = (0..).take_while(|k| offs.contains_key(format!("f_rest_{k}").as_str())).count();
    let per = rest_n / 3;
    let deg = match per {
        0 => 0,
        1..=3 => 1,
        4..=8 => 2,
        _ => 3,
    };
    let k = ((deg + 1) * (deg + 1) - 1) as usize;
    let rest_names: Vec<String> = (0..rest_n).map(|k| format!("f_rest_{k}")).collect();
    let mut r = Raw { pos: Vec::with_capacity(n), scale: Vec::with_capacity(n), rot: Vec::with_capacity(n), opacity: Vec::with_capacity(n), color: Vec::with_capacity(n), deg, rest: Vec::with_capacity(n * k * 3) };
    let has_dc = offs.contains_key("f_dc_0");
    for i in 0..n {
        let g = |s: &str| read(i, s).unwrap_or(0.0);
        r.pos.push([g("x"), g("y"), g("z")]);
        r.scale.push([g("scale_0").exp(), g("scale_1").exp(), g("scale_2").exp()]);
        let q = Vec4::new(g("rot_0"), g("rot_1"), g("rot_2"), g("rot_3"));
        let q = if q.length_squared() > 0.0 { q.normalize() } else { Vec4::X };
        r.rot.push(q.to_array());
        r.opacity.push(sigmoid(g("opacity")));
        r.color.push(if has_dc {
            [0.5 + SH_C0 * g("f_dc_0"), 0.5 + SH_C0 * g("f_dc_1"), 0.5 + SH_C0 * g("f_dc_2")]
        } else {
            [g("red"), g("green"), g("blue")]
        });
        // the file's are channel-major (all of red's coefficients, then green's, then blue's)
        for kk in 0..k {
            for c in 0..3 {
                r.rest.push(if kk < per { read(i, &rest_names[c * per + kk]).unwrap_or(0.0) } else { 0.0 });
            }
        }
    }
    Ok(r)
}

/// antimatter15's .splat: 32 bytes a splat (centre f32 x3, scale f32 x3, rgba u8, rotation u8 x4 as w x y z).
pub(crate) fn read_splat(b: &[u8]) -> Result<Raw, String> {
    if b.len() % 32 != 0 {
        return Err("a .splat file is 32 bytes a splat".into());
    }
    let n = b.len() / 32;
    let mut r = Raw { pos: Vec::with_capacity(n), scale: Vec::with_capacity(n), rot: Vec::with_capacity(n), opacity: Vec::with_capacity(n), color: Vec::with_capacity(n), deg: 0, rest: Vec::new() };
    let f = |at: usize| f32::from_le_bytes(b[at..at + 4].try_into().unwrap());
    for i in 0..n {
        let o = i * 32;
        r.pos.push([f(o), f(o + 4), f(o + 8)]);
        r.scale.push([f(o + 12), f(o + 16), f(o + 20)]);
        r.color.push([b[o + 24] as f32 / 255.0, b[o + 25] as f32 / 255.0, b[o + 26] as f32 / 255.0]);
        r.opacity.push(b[o + 27] as f32 / 255.0);
        let q = Vec4::new((b[o + 28] as f32 - 128.0) / 128.0, (b[o + 29] as f32 - 128.0) / 128.0, (b[o + 30] as f32 - 128.0) / 128.0, (b[o + 31] as f32 - 128.0) / 128.0);
        r.rot.push(if q.length_squared() > 0.0 { q.normalize().to_array() } else { [1.0, 0.0, 0.0, 0.0] });
    }
    Ok(r)
}

/// Niantic's .spz (versions 2 and 3): gzip of a 16-byte header and the splats' fields one after another, fixed
/// point. Decompressed by the system's gzip (native only), so raea-render takes no inflate crate for it.
#[cfg(feature = "native")]
pub(crate) fn read_spz(b: &[u8]) -> Result<Raw, String> {
    let d = {
        use std::io::Write;
        let mut child = std::process::Command::new("gzip").arg("-dc").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().map_err(|e| format!("gzip: {e}"))?;
        let mut stdin = child.stdin.take().unwrap();
        let input = b.to_vec();
        let w = std::thread::spawn(move || stdin.write_all(&input));
        let out = child.wait_with_output().map_err(|e| format!("gzip: {e}"))?;
        let _ = w.join();
        if !out.status.success() {
            return Err("not gzip: a .spz is gzipped".into());
        }
        out.stdout
    };
    if d.len() < 16 || &d[0..4] != b"NGSP" {
        return Err("no NGSP magic".into());
    }
    let u32at = |at: usize| u32::from_le_bytes(d[at..at + 4].try_into().unwrap());
    let ver = u32at(4);
    let n = u32at(8) as usize;
    let deg = d[12] as u32;
    let frac = d[13];
    if !(2..=3).contains(&ver) || deg > 3 {
        return Err(format!("spz version {ver}, degree {deg}: I read versions 2 and 3, degree up to 3"));
    }
    let k = ((deg + 1) * (deg + 1) - 1) as usize;
    let rot_bytes = if ver == 3 { 4 } else { 3 };
    let need = 16 + n * (9 + 1 + 3 + 3 + rot_bytes + k * 3);
    if d.len() < need {
        return Err(format!("{n} splats need {need} bytes, it has {}", d.len()));
    }
    // the fields one after another: positions (24-bit fixed point), alphas, colours, scales, rotations, harmonics
    let o_pos = 16;
    let o_a = o_pos + n * 9;
    let o_c = o_a + n;
    let o_s = o_c + n * 3;
    let o_r = o_s + n * 3;
    let o_sh = o_r + n * rot_bytes;
    let scale = 1.0 / (1u32 << frac) as f32;
    let mut r = Raw { pos: Vec::with_capacity(n), scale: Vec::with_capacity(n), rot: Vec::with_capacity(n), opacity: Vec::with_capacity(n), color: Vec::with_capacity(n), deg, rest: Vec::with_capacity(n * k * 3) };
    for i in 0..n {
        let mut p = [0f32; 3];
        for (a, v) in p.iter_mut().enumerate() {
            let at = o_pos + i * 9 + a * 3;
            let mut x = d[at] as i32 | (d[at + 1] as i32) << 8 | (d[at + 2] as i32) << 16;
            if x & 0x80_0000 != 0 {
                x |= -0x100_0000;
            }
            *v = x as f32 * scale;
        }
        r.pos.push(p);
        r.opacity.push(d[o_a + i] as f32 / 255.0);
        let c = |a: usize| ((d[o_c + i * 3 + a] as f32 / 255.0) - 0.5) / 0.15;
        r.color.push([0.5 + SH_C0 * c(0), 0.5 + SH_C0 * c(1), 0.5 + SH_C0 * c(2)]);
        let s = |a: usize| (d[o_s + i * 3 + a] as f32 / 16.0 - 10.0).exp();
        r.scale.push([s(0), s(1), s(2)]);
        let q = if ver == 3 {
            // smallest three: the largest component's index in the top two bits, the others 10 bits each
            let v = u32::from_le_bytes(d[o_r + i * 4..o_r + i * 4 + 4].try_into().unwrap());
            let largest = (v >> 30) as usize;
            let mut q = [0f32; 4];
            let mut sum = 0.0;
            let mut bits = v;
            for c in (0..4).rev() {
                if c == largest {
                    continue;
                }
                let mag = (bits & 0x1ff) as f32 / 511.0 * std::f32::consts::FRAC_1_SQRT_2;
                let neg = (bits >> 9) & 1 == 1;
                bits >>= 10;
                q[c] = if neg { -mag } else { mag };
                sum += q[c] * q[c];
            }
            q[largest] = (1.0 - sum).max(0.0).sqrt();
            // x y z w in the file
            Vec4::new(q[3], q[0], q[1], q[2])
        } else {
            let x = d[o_r + i * 3] as f32 / 127.5 - 1.0;
            let y = d[o_r + i * 3 + 1] as f32 / 127.5 - 1.0;
            let z = d[o_r + i * 3 + 2] as f32 / 127.5 - 1.0;
            Vec4::new((1.0 - x * x - y * y - z * z).max(0.0).sqrt(), x, y, z)
        };
        r.rot.push(q.normalize().to_array());
        for j in 0..k * 3 {
            r.rest.push((d[o_sh + i * k * 3 + j] as f32 - 128.0) / 128.0);
        }
    }
    Ok(r)
}
