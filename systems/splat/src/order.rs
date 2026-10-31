//! Each frame's order and motion: the cloud packed for the GPU, a Hilbert curve for locality, the row's effects on
//! each splat's centre (twist, wave, explode, disperse with curl noise), and the stable depth sort far to near.

use super::*;

pub(crate) fn bytes_of(g: &[[f32; 16]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(g.len() * 64);
    for s in g {
        for v in s {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    out
}

pub(crate) fn words_of(w: &[u32]) -> Vec<u8> {
    w.iter().flat_map(|v| v.to_le_bytes()).collect()
}

// ---- morph: each splat of the source given one of the target's

/// The position along a 3D Hilbert curve of a point in the unit cube (10 bits an axis; Skilling's transform).
pub(crate) fn hilbert(p: Vec3) -> u32 {
    let b = 10u32;
    let mut x = [(p.x.clamp(0.0, 1.0) * 1023.0) as u32, (p.y.clamp(0.0, 1.0) * 1023.0) as u32, (p.z.clamp(0.0, 1.0) * 1023.0) as u32];
    let m = 1u32 << (b - 1);
    // inverse undo
    let mut q = m;
    while q > 1 {
        let pm = q - 1;
        for i in 0..3 {
            if x[i] & q != 0 {
                x[0] ^= pm;
            } else {
                let t = (x[0] ^ x[i]) & pm;
                x[0] ^= t;
                x[i] ^= t;
            }
        }
        q >>= 1;
    }
    // Gray encode
    for i in 1..3 {
        x[i] ^= x[i - 1];
    }
    let mut t = 0;
    let mut q = m;
    while q > 1 {
        if x[2] & q != 0 {
            t ^= q - 1;
        }
        q >>= 1;
    }
    for v in x.iter_mut() {
        *v ^= t;
    }
    // interleave, most significant bit first
    let mut h = 0u32;
    for bit in (0..b).rev() {
        for v in &x {
            h = (h << 1) | ((v >> bit) & 1);
        }
    }
    h
}

/// Each cloud's splats in Hilbert order within its own bounds (so like parts meet: a head finds a head).
pub(crate) fn curve_order(c: &Cloud) -> Vec<u32> {
    let mut lo = Vec3::splat(f32::MAX);
    let mut hi = Vec3::splat(f32::MIN);
    for i in 0..c.n {
        lo = lo.min(c.pos(i));
        hi = hi.max(c.pos(i));
    }
    let size = (hi - lo).max(Vec3::splat(1e-6));
    let mut keyed: Vec<(u32, u32)> = (0..c.n).map(|i| (hilbert((c.pos(i) - lo) / size), i as u32)).collect();
    keyed.sort_unstable();
    keyed.into_iter().map(|(_, i)| i).collect()
}

/// The target's splat each source splat morphs into: the source's and target's Hilbert orders matched by rank (a
/// cheap stand-in for optimal transport that keeps neighbours together), fitted to the source's centre and size, as
/// the shader's `target_splats` reads them.
pub(crate) fn matched(a: &Cloud, b: &Cloud, crop: f32) -> Vec<[f32; 16]> {
    let mut out = vec![[0f32; 16]; a.n];
    // the target within `crop` metres of its centre (all of it at 0), its centre and size measured again
    let sub;
    let b = if crop > 0.0 {
        let keep: Vec<usize> = (0..b.n).filter(|&j| (b.pos(j) - b.center).length() <= crop).collect();
        let raw = Raw {
            pos: keep.iter().map(|&j| b.pos(j).to_array()).collect(),
            scale: keep.iter().map(|&j| [b.gauss[j][4], b.gauss[j][5], b.gauss[j][6]]).collect(),
            rot: keep.iter().map(|&j| [b.gauss[j][11], b.gauss[j][8], b.gauss[j][9], b.gauss[j][10]]).collect(),
            opacity: keep.iter().map(|&j| b.gauss[j][3]).collect(),
            color: keep.iter().map(|&j| [b.gauss[j][12], b.gauss[j][13], b.gauss[j][14]]).collect(),
            deg: 0,
            rest: Vec::new(),
        };
        sub = place(raw, Mat4::IDENTITY);
        &sub
    } else {
        b
    };
    if b.n == 0 {
        return out;
    }
    let oa = curve_order(a);
    let ob = curve_order(b);
    for (rank, &i) in oa.iter().enumerate() {
        let j = ob[(rank * b.n / a.n.max(1)).min(b.n - 1)] as usize;
        // the target fitted to the source: its centre on the source's, its size the source's, so one shape becomes
        // the other where it stands whatever the two files' scales
        let mut g = b.gauss[j];
        let k = a.radius / b.radius.max(1e-6);
        for ax in 0..3 {
            g[ax] = a.center[ax] + (g[ax] - b.center[ax]) * k;
            g[4 + ax] *= k;
        }
        out[i as usize] = g;
    }
    out
}

// ---- the frame's effects on a splat's centre (splat.wesl's `animate`, its moving steps)

pub(crate) fn pcg(v: u32) -> u32 {
    let s = v.wrapping_mul(747796405).wrapping_add(2891336453);
    let w = ((s >> ((s >> 28) + 4)) ^ s).wrapping_mul(277803737);
    (w >> 22) ^ w
}

pub(crate) fn hash(seed: f32, i: u32, k: u32) -> f32 {
    (pcg(i ^ pcg(k.wrapping_add(seed.to_bits()))) >> 8) as f32 / 16777216.0
}

pub(crate) fn potential(x: Vec3) -> Vec3 {
    Vec3::new(
        (x.y * 1.7 + 1.3).sin() + (x.z * 2.3 + 0.4).cos() + 0.5 * (x.y * 3.9 + x.z * 1.1 + 2.0).sin(),
        (x.z * 1.9 + 2.9).sin() + (x.x * 2.1 + 1.7).cos() + 0.5 * (x.z * 4.1 + x.x * 1.3 + 0.7).sin(),
        (x.x * 1.5 + 0.2).sin() + (x.y * 2.7 + 2.2).cos() + 0.5 * (x.x * 3.7 + x.y * 1.2 + 1.5).sin(),
    )
}

pub(crate) fn curl(x: Vec3) -> Vec3 {
    let e = 0.05;
    let (dx, dy, dz) = (Vec3::X * e, Vec3::Y * e, Vec3::Z * e);
    let (px0, px1) = (potential(x - dx), potential(x + dx));
    let (py0, py1) = (potential(x - dy), potential(x + dy));
    let (pz0, pz1) = (potential(x - dz), potential(x + dz));
    Vec3::new((py1.z - py0.z) - (pz1.y - pz0.y), (pz1.x - pz0.x) - (px1.z - px0.z), (px1.y - px0.y) - (py1.x - py0.x)) / (2.0 * e)
}

pub(crate) fn smooth01(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Whether the row's effects move any splat (else the file's centres are where they draw).
pub(crate) fn moves(p: &SplatParams) -> bool {
    p.morph > 0.0 || p.twist != 0.0 || p.wave_amp != 0.0 || p.explode > 0.0 || p.disperse > 0.0
}

pub(crate) fn anim_center(p: &SplatParams, i: u32, home: Vec3, target: Option<&[f32; 16]>) -> Vec3 {
    let r = p.radius.max(1e-4);
    let mut pos = home;
    if p.morph > 0.0 {
        if let Some(t) = target {
            let m = smooth01(p.morph);
            let lift = (std::f32::consts::PI * m).sin() * 0.35 * r;
            pos = pos.lerp(Vec3::new(t[0], t[1], t[2]), m) + lift * (Vec3::new(hash(p.seed, i, 11), hash(p.seed, i, 12), hash(p.seed, i, 13)) - 0.5) * 2.0;
        }
    }
    if p.twist != 0.0 {
        let ang = p.twist * (pos.y - p.center.y);
        let (s, c) = ang.sin_cos();
        let d = pos - p.center;
        pos = p.center + Vec3::new(c * d.x + s * d.z, d.y, -s * d.x + c * d.z);
    }
    if p.wave_amp != 0.0 {
        let d = (pos - p.center).xz().length();
        pos.y += p.wave_amp * (std::f32::consts::TAU * d / p.wave_len.max(1e-3) - p.wave_phase).sin();
    }
    if p.explode > 0.0 {
        let e = p.explode;
        let jit = Vec3::new(hash(p.seed, i, 21), hash(p.seed, i, 22), hash(p.seed, i, 23)) - 0.5;
        let dir = (pos - p.center + jit * 0.6 * r + Vec3::new(0.0, 1e-4, 0.0)).normalize();
        pos += dir * e * r * 2.0 * (0.4 + hash(p.seed, i, 24));
        pos.y -= 1.2 * r * e * e;
    }
    if p.disperse > 0.0 {
        let w = (p.wind + Vec3::new(0.0, 0.0, 1e-6)).normalize();
        let o = if p.disperse_mode == 0 { ((pos - p.center).dot(w) / r + 1.0) * 0.5 } else { (pos - p.center).length() / (1.5 * r) };
        let o = (o * 0.8 + hash(p.seed, i, 51) * 0.2).clamp(0.0, 1.0);
        if p.disperse_mode == 0 {
            let local = ((p.disperse - 0.55 * o) / 0.45).clamp(0.0, 1.0);
            pos += w * local * local * r * 3.0 + curl(pos * (1.4 / r) + Vec3::splat(p.seed)) * (p.curl * r * 0.25 * local);
            pos.y += local * local * r * 0.4;
        } else {
            let local = smooth01((p.disperse - 0.5 * o) / 0.5);
            let (u, v) = (hash(p.seed, i, 31), hash(p.seed, i, 32));
            let rr = hash(p.seed, i, 33).powf(0.333) * if p.spread > 0.0 { p.spread } else { r * 1.6 };
            let th = std::f32::consts::TAU * u;
            let ph = (2.0 * v - 1.0).acos();
            let q = p.center + rr * Vec3::new(ph.sin() * th.cos(), ph.cos(), ph.sin() * th.sin());
            let swirl = curl(q * (1.4 / r) + Vec3::splat(p.seed)) * (p.curl * r * 0.12);
            pos = pos.lerp(q + swirl, local);
        }
    }
    pos
}

// ---- the sort

/// A float's bits made to sort as the float does (as an unsigned integer).
pub(crate) fn ordered(f: f32) -> u32 {
    let b = f.to_bits();
    if b & 0x8000_0000 != 0 { !b } else { b | 0x8000_0000 }
}

/// The splats far to near from the camera, ties by index: a stable LSD radix sort (four 8-bit passes) of each splat's
/// depth where the frame's effects put it. `keys` and `tmp` are scratch kept between frames.
pub(crate) fn sort(c: &Cloud, target: Option<&[[f32; 16]]>, p: &SplatParams, model: Mat4, cam: Vec3, fwd: Vec3, out: &mut Vec<u32>, keys: &mut Vec<u32>) {
    let n = c.n;
    keys.resize(n, 0);
    let mv = moves(p);
    // the camera into the splats' own space, so each depth is one dot product
    let inv = model.inverse();
    let cam_l = inv.transform_point3(cam);
    let fwd_l = model.transpose().transform_vector3(fwd);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 8);
    let chunk = n.div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for (t, ks) in keys.chunks_mut(chunk).enumerate() {
            s.spawn(move || {
                for (j, k) in ks.iter_mut().enumerate() {
                    let i = t * chunk + j;
                    let home = c.pos(i);
                    let pos = if mv { anim_center(p, i as u32, home, target.map(|t| &t[i])) } else { home };
                    // far first: the larger the depth, the smaller the key
                    *k = !ordered((pos - cam_l).dot(fwd_l));
                }
            });
        }
    });
    let mut a: Vec<u32> = (0..n as u32).collect();
    let mut b = vec![0u32; n];
    for pass in 0..4 {
        let shift = pass * 8;
        let mut count = [0usize; 257];
        for &i in &a {
            count[((keys[i as usize] >> shift) & 0xff) as usize + 1] += 1;
        }
        for d in 0..256 {
            count[d + 1] += count[d];
        }
        for &i in &a {
            let d = ((keys[i as usize] >> shift) & 0xff) as usize;
            b[count[d]] = i;
            count[d] += 1;
        }
        std::mem::swap(&mut a, &mut b);
    }
    *out = a;
}

// ---- the row
