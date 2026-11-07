//! A field's spec (the row's `grass` JSON, as raea/3d's <Grass> sends it) and its tiles' blades.
//!
//! The grids are `grid[0]` x `grid[1]` values over the area, row by row from its min z, the first and last of each row
//! on the area's edges: `height` (the ground's y), `density` (clumps per square metre), `patch` (which of `tints`),
//! and `size` (a multiple of each clump's scale; 1 without it). Between grid points a value is the bilinear mix of its
//! four, a patch the nearest's.

use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use serde_json::Value as J;

use super::Blades;

/// One step of detail: tiles nearer than `to` metres draw this share of their clumps (`keep`), each with its first
/// `blades` blades in `segments` segments, `widen` times as wide.
#[derive(Clone, Debug, PartialEq)]
pub struct Lod {
    pub to: f32,
    pub keep: f32,
    pub blades: u32,
    pub segments: u32,
    pub widen: f32,
}

#[derive(Clone, Debug)]
pub struct Spec {
    /// min x, min z, max x, max z
    pub area: [f32; 4],
    pub nx: usize,
    pub nz: usize,
    pub height: Vec<f32>,
    pub density: Vec<f32>,
    pub patch: Vec<u32>,
    pub size: Vec<f32>,
    /// each patch's root and tip colours (linear)
    pub tints: Vec<([f32; 3], [f32; 3])>,
    pub highlight: [f32; 3],
    pub highlight_mix: f32,
    pub blades: u32,
    pub blade_height: f32,
    pub width: [f32; 2],
    pub spread: f32,
    pub lean: [f32; 2],
    pub scale: [f32; 2],
    pub seed: u32,
    pub tile: f32,
    pub lod: Vec<Lod>,
}

fn nums(j: &J, k: &str) -> Result<Vec<f32>, String> {
    match j.get(k) {
        Some(J::Array(a)) => a.iter().map(|v| v.as_f64().map(|x| x as f32).ok_or_else(|| format!("grass: {k} is numbers"))).collect(),
        _ => Err(format!("grass: no {k}")),
    }
}

use raea_sys_frame::json_f32 as num;

fn pair(j: &J, k: &str, d: [f32; 2]) -> [f32; 2] {
    match nums(j, k) {
        Ok(v) if v.len() == 2 => [v[0], v[1]],
        _ => d,
    }
}

/// "#rrggbb" as linear rgb
pub fn linear(hex: &str) -> Option<[f32; 3]> {
    let s = hex.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let c = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    let l = Color::srgb_u8(c(0)?, c(2)?, c(4)?).to_linear();
    Some([l.red, l.green, l.blue])
}

impl Spec {
    pub fn from_json(j: &J) -> Result<Spec, String> {
        let area = nums(j, "area")?;
        let grid = nums(j, "grid")?;
        if area.len() != 4 || !(area[2] > area[0] && area[3] > area[1]) {
            return Err("grass: area is [min x, min z, max x, max z]".into());
        }
        if grid.len() != 2 || grid[0] < 2.0 || grid[1] < 2.0 {
            return Err("grass: grid is [columns, rows], 2 or more each".into());
        }
        let (nx, nz) = (grid[0] as usize, grid[1] as usize);
        let height = nums(j, "height")?;
        let density = nums(j, "density")?;
        let patch: Vec<u32> = nums(j, "patch").unwrap_or_else(|_| vec![0.0; nx * nz]).into_iter().map(|p| p.max(0.0) as u32).collect();
        let size = nums(j, "size").unwrap_or_else(|_| vec![1.0; nx * nz]);
        for (k, n) in [("height", height.len()), ("density", density.len()), ("patch", patch.len()), ("size", size.len())] {
            if n != nx * nz {
                return Err(format!("grass: {k} has {n} values, the grid {nx} x {nz}"));
            }
        }
        let tints: Vec<([f32; 3], [f32; 3])> = match j.get("tints") {
            Some(J::Array(a)) if !a.is_empty() => a
                .iter()
                .map(|t| {
                    let c = |i: usize| t.get(i).and_then(J::as_str).and_then(linear).ok_or("grass: tints are [root, tip] \"#rrggbb\" pairs".to_string());
                    Ok((c(0)?, c(1)?))
                })
                .collect::<Result<_, String>>()?,
            _ => vec![(linear("#3a5e1e").unwrap(), linear("#7ea535").unwrap())],
        };
        let lod = match j.get("lod") {
            Some(J::Array(a)) if !a.is_empty() => a
                .iter()
                .map(|l| Lod { to: num(l, "to", f32::MAX), keep: num(l, "keep", 1.0).clamp(0.0, 1.0), blades: num(l, "blades", 14.0).max(1.0) as u32, segments: num(l, "segments", 3.0).clamp(1.0, 6.0) as u32, widen: num(l, "widen", 1.0).max(0.1) })
                .collect(),
            _ => default_lod(),
        };
        Ok(Spec {
            area: [area[0], area[1], area[2], area[3]],
            nx,
            nz,
            height,
            density,
            patch,
            size,
            tints,
            highlight: j.get("highlight").and_then(J::as_str).and_then(linear).unwrap_or(linear("#f2e7a0").unwrap()),
            highlight_mix: num(j, "highlightMix", 0.22),
            blades: num(j, "blades", 14.0).max(1.0) as u32,
            blade_height: num(j, "bladeHeight", 0.3),
            width: pair(j, "width", [0.034, 0.05]),
            spread: num(j, "spread", 0.06),
            lean: pair(j, "lean", [0.2, 0.7]),
            scale: pair(j, "scale", [0.85, 1.35]),
            seed: num(j, "seed", 1.0) as u32,
            tile: num(j, "tile", 4.0).max(0.5),
            lod,
        })
    }

    /// tiles across and deep
    pub fn tiles(&self) -> (u32, u32) {
        (((self.area[2] - self.area[0]) / self.tile).ceil() as u32, ((self.area[3] - self.area[1]) / self.tile).ceil() as u32)
    }

    fn tile_box(&self, ix: u32, iz: u32) -> [f32; 4] {
        let x0 = self.area[0] + ix as f32 * self.tile;
        let z0 = self.area[1] + iz as f32 * self.tile;
        [x0, z0, (x0 + self.tile).min(self.area[2]), (z0 + self.tile).min(self.area[3])]
    }

    /// a grid's value at (x, z), bilinear
    fn at(&self, g: &[f32], x: f32, z: f32) -> f32 {
        let fx = ((x - self.area[0]) / (self.area[2] - self.area[0]) * (self.nx - 1) as f32).clamp(0.0, (self.nx - 1) as f32);
        let fz = ((z - self.area[1]) / (self.area[3] - self.area[1]) * (self.nz - 1) as f32).clamp(0.0, (self.nz - 1) as f32);
        let (i, k) = ((fx as usize).min(self.nx - 2), (fz as usize).min(self.nz - 2));
        let (u, v) = (fx - i as f32, fz - k as f32);
        let g = |a: usize, b: usize| g[b * self.nx + a];
        (g(i, k) * (1.0 - u) + g(i + 1, k) * u) * (1.0 - v) + (g(i, k + 1) * (1.0 - u) + g(i + 1, k + 1) * u) * v
    }

    pub fn ground(&self, x: f32, z: f32) -> f32 {
        self.at(&self.height, x, z)
    }

    /// the ground's normal at (x, z)
    pub fn normal(&self, x: f32, z: f32) -> Vec3 {
        let e = 0.5 * ((self.area[2] - self.area[0]) / (self.nx - 1) as f32).min((self.area[3] - self.area[1]) / (self.nz - 1) as f32);
        let dx = self.ground(x + e, z) - self.ground(x - e, z);
        let dz = self.ground(x, z + e) - self.ground(x, z - e);
        Vec3::new(-dx, 2.0 * e, -dz).normalize_or(Vec3::Y)
    }

    fn patch_at(&self, x: f32, z: f32) -> usize {
        let i = (((x - self.area[0]) / (self.area[2] - self.area[0]) * (self.nx - 1) as f32).round() as usize).min(self.nx - 1);
        let k = (((z - self.area[1]) / (self.area[3] - self.area[1]) * (self.nz - 1) as f32).round() as usize).min(self.nz - 1);
        (self.patch[k * self.nx + i] as usize).min(self.tints.len() - 1)
    }

    /// how far the eye is from a tile (its nearest point, at the ground's height there)
    pub fn distance(&self, ix: u32, iz: u32, eye: Vec3) -> f32 {
        let b = self.tile_box(ix, iz);
        let (x, z) = (eye.x.clamp(b[0], b[2]), eye.z.clamp(b[1], b[3]));
        eye.distance(Vec3::new(x, self.ground(x, z), z))
    }

    /// the step of detail for a tile this far away
    pub fn lod_at(&self, d: f32) -> usize {
        self.lod.iter().position(|l| d < l.to).unwrap_or(self.lod.len() - 1)
    }

    /// A tile's clumps at step `lod`: scattered as raea/3d's scatter() does (candidates over the tile at its densest,
    /// each kept by the density where it falls), every draw in the same order whatever the step, so a step's clumps
    /// are the nearer step's with some left out.
    pub fn tile_mesh(&self, ix: u32, iz: u32, lod: usize) -> Option<(Mesh, Aabb)> {
        let l = &self.lod[lod.min(self.lod.len() - 1)];
        let b = self.tile_box(ix, iz);
        let (w, d) = (b[2] - b[0], b[3] - b[1]);
        let mut most = 0.0f32;
        for i in 0..=4 {
            for k in 0..=4 {
                most = most.max(self.at(&self.density, b[0] + w * i as f32 / 4.0, b[1] + d * k as f32 / 4.0));
            }
        }
        // the grid's own points inside the tile too, so a narrow peak isn't missed
        for k in 0..self.nz {
            for i in 0..self.nx {
                let x = self.area[0] + (self.area[2] - self.area[0]) * i as f32 / (self.nx - 1) as f32;
                let z = self.area[1] + (self.area[3] - self.area[1]) * k as f32 / (self.nz - 1) as f32;
                if x >= b[0] && x <= b[2] && z >= b[1] && z <= b[3] {
                    most = most.max(self.density[k * self.nx + i]);
                }
            }
        }
        let mut out = Blades::new();
        if most <= 0.0 {
            return None;
        }
        let mut r = Rng::new(self.seed ^ ix.wrapping_mul(73_856_093) ^ iz.wrapping_mul(19_349_663) ^ 0x9e37_79b9);
        let want = most * w * d;
        let n = want.floor() as u32 + u32::from(r.next() < want.fract());
        for _ in 0..n {
            let (x, z) = (b[0] + r.next() * w, b[1] + r.next() * d);
            let accept = r.next() * most < self.at(&self.density, x, z);
            let keep = r.next();
            let s = (self.scale[0] + r.next() * (self.scale[1] - self.scale[0])) * self.at(&self.size, x, z);
            let seed = (r.next() * u32::MAX as f32) as u32;
            if !accept || keep >= l.keep {
                continue;
            }
            self.clump(&mut out, l, Vec3::new(x, self.ground(x, z), z), s, seed);
        }
        out.mesh(self.blade_height * self.scale[1] * 1.5)
    }

    /// One clump's blades (raea-site-lab's clump(): tapered, leaning, root to tip in the patch's greens, the tip's
    /// last point a warm highlight), tilted to the slope at its root.
    fn clump(&self, out: &mut Blades, l: &Lod, at: Vec3, s: f32, seed: u32) {
        let mut r = Rng::new(seed);
        let n = self.normal(at.x, at.z);
        let tilt = Quat::from_rotation_arc(Vec3::Y, n);
        let (root, tip) = self.tints[self.patch_at(at.x, at.z)];
        let mix = |a: [f32; 3], b: [f32; 3], t: f32| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, 1.0];
        let crown = mix(tip, self.highlight, self.highlight_mix);
        let phase = r.next();
        for k in 0..self.blades {
            let a = r.next() * std::f32::consts::TAU;
            let dd = r.next().sqrt() * self.spread;
            let ang = r.next() * std::f32::consts::TAU;
            let h = self.blade_height * (0.55 + r.next() * 0.75);
            let w = self.width[0] + r.next() * (self.width[1] - self.width[0]);
            let lean = self.lean[0] + r.next() * (self.lean[1] - self.lean[0]);
            if k >= l.blades {
                continue;
            }
            let (x, z) = (a.cos() * dd, a.sin() * dd);
            let (ca, sa) = (ang.cos(), ang.sin());
            let o = out.pos.len() as u32;
            let seg = l.segments;
            for si in 0..=seg {
                let t = si as f32 / seg as f32;
                let bend = lean * t * t * h;
                let hw = (w * (1.0 - t) / 2.0 + 0.0008) * l.widen;
                let (cx, cy, cz) = (x + ca * bend, h * t, z + sa * bend);
                let c = if si == seg { crown } else { mix(root, tip, t) };
                for side in [-1.0f32, 1.0] {
                    let p = at + tilt * (Vec3::new(cx - sa * hw * side, cy, cz + ca * hw * side) * s);
                    out.pos.push(p.to_array());
                    out.nrm.push(n.to_array());
                    out.col.push(c);
                    out.uv.push([(side + 1.0) / 2.0, t]);
                    // how high above its root (m): what the wind bends it by
                    out.uv1.push([cy * s, phase]);
                }
                if si < seg {
                    let q = o + si * 2;
                    out.idx.extend_from_slice(&[q, q + 1, q + 2, q + 1, q + 3, q + 2]);
                }
            }
        }
    }
}

fn default_lod() -> Vec<Lod> {
    vec![
        Lod { to: 5.0, keep: 1.0, blades: 14, segments: 3, widen: 1.0 },
        Lod { to: 14.0, keep: 0.5, blades: 8, segments: 1, widen: 1.4 },
        Lod { to: f32::MAX, keep: 0.18, blades: 5, segments: 1, widen: 2.0 },
    ]
}

/// mulberry32, as raea-site-lab's rng(): a draw in [0, 1)
pub(crate) struct Rng(u32);

impl Rng {
    pub(crate) fn new(seed: u32) -> Rng {
        Rng(seed)
    }
    pub(crate) fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        ((t ^ (t >> 14)) as f64 / 4_294_967_296.0) as f32
    }
}
