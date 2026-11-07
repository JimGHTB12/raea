//! Ground and rock. `terrain` is a fractal heightfield as a closed slab (a top surface, flat sides and a bottom), so a
//! hill is a solid that can be cut, carved by a path's `minus`, and stood on. `boulder` is an ellipsoid roughened by
//! fractal noise through the distance field, so its lumps are part of one closed surface rather than bumps on a ball.

use raea_core_sdf::{Node, fbm};

use crate::mesh::Mesh;

fn kv(what: &str, text: &str, names: &[&str]) -> Result<Vec<(String, f64)>, String> {
    let mut out = Vec::new();
    for w in text.split_whitespace() {
        let (k, v) = w.split_once('=').ok_or_else(|| format!("{what}: {w:?} isn't name=value"))?;
        if !names.contains(&k) {
            return Err(format!("{what}: no setting {k}: {}", names.join(", ")));
        }
        out.push((k.to_string(), v.parse::<f64>().map_err(|_| format!("{what}: {k} takes a number, got {v:?}"))?));
    }
    Ok(out)
}

/// `size=20 depth=20 height=2 cells=128 freq=0.15 octaves=5 seed=1 base=0.5 flat=0`: a slab `size` wide in x and
/// `depth` in z, centred at the origin, its surface at y 0 plus up to `height` of noise (`freq` cycles a metre,
/// `octaves` layers), the bottom `base` below the lowest point. `flat` (0-1) flattens a disc in the middle (a place to
/// stand things) that wide as a share of the size.
pub fn terrain(text: &str) -> Result<Mesh, String> {
    let (mut w, mut d, mut h, mut cells, mut freq, mut oct, mut seed, mut base, mut flat) = (20.0, 0.0, 2.0, 128.0, 0.15, 5.0, 1.0, 0.5, 0.0);
    for (k, v) in kv("terrain", text, &["size", "depth", "height", "cells", "freq", "octaves", "seed", "base", "flat"])? {
        match k.as_str() {
            "size" => w = v,
            "depth" => d = v,
            "height" => h = v,
            "cells" => cells = v,
            "freq" => freq = v,
            "octaves" => oct = v,
            "seed" => seed = v,
            "base" => base = v,
            _ => flat = v,
        }
    }
    let d = if d > 0.0 { d } else { w };
    if w <= 0.0 || h < 0.0 || base <= 0.0 {
        return Err("terrain: size and base above 0, height 0 or more".into());
    }
    let n = (cells.round() as usize).clamp(2, 1024);
    let nz = ((n as f64 * d / w).round() as usize).clamp(2, 1024);
    let (oct, seed) = (oct.clamp(1.0, 8.0) as u32, seed as u32);
    let height = |x: f64, z: f64| {
        let mut y = h * (fbm([x * freq, 0.37, z * freq], oct, seed) * 0.5 + 0.5);
        if flat > 0.0 {
            let r = (x * x + z * z).sqrt() / (w * flat / 2.0);
            let t = ((r - 0.7) / 0.6).clamp(0.0, 1.0);
            let s = t * t * (3.0 - 2.0 * t);
            y *= s;
        }
        y
    };
    let mut m = Mesh::default();
    let at = |i: usize, k: usize| (-w / 2.0 + w * i as f64 / n as f64, -d / 2.0 + d * k as f64 / nz as f64);
    let mut top = vec![0u32; (n + 1) * (nz + 1)];
    let mut low = f64::MAX;
    for k in 0..=nz {
        for i in 0..=n {
            let (x, z) = at(i, k);
            let y = height(x, z);
            low = low.min(y);
            top[i + (n + 1) * k] = m.vert([x, y, z]);
        }
    }
    let floor = low - base;
    let mut bot = vec![0u32; (n + 1) * (nz + 1)];
    for k in 0..=nz {
        for i in 0..=n {
            let (x, z) = at(i, k);
            bot[i + (n + 1) * k] = m.vert([x, floor, z]);
        }
    }
    let id = |i: usize, k: usize| i + (n + 1) * k;
    for k in 0..nz {
        for i in 0..n {
            let (a, b, c, e) = (id(i, k), id(i + 1, k), id(i, k + 1), id(i + 1, k + 1));
            // the top faces +y, the bottom -y
            m.tri(top[a], top[c], top[e]);
            m.tri(top[a], top[e], top[b]);
            m.tri(bot[a], bot[e], bot[c]);
            m.tri(bot[a], bot[b], bot[e]);
        }
    }
    // the four sides, each a strip from the top's edge down to the bottom's
    let side = |edge: Vec<usize>, m: &mut Mesh| {
        for p in edge.windows(2) {
            let (a, b) = (p[0], p[1]);
            m.tri(top[a], bot[b], bot[a]);
            m.tri(top[a], top[b], bot[b]);
        }
    };
    side((0..=n).map(|i| id(i, 0)).collect(), &mut m);
    side((0..=nz).map(|k| id(n, k)).collect(), &mut m);
    side((0..=n).rev().map(|i| id(i, nz)).collect(), &mut m);
    side((0..=nz).rev().map(|k| id(0, k)).collect(), &mut m);
    Ok(m)
}

/// `size=1 squash=0.7 rough=0.15 freq=3 octaves=4 seed=1 step=0`: a rock about `size` across, `squash` as tall as wide,
/// lumped by noise `rough` (a share of the size) at `freq` cycles across it, resting on y 0.
pub fn boulder(text: &str) -> Result<Mesh, String> {
    let (mut size, mut squash, mut rough, mut freq, mut oct, mut seed, mut step) = (1.0, 0.7, 0.15, 3.0, 4.0, 1.0, 0.0);
    for (k, v) in kv("boulder", text, &["size", "squash", "rough", "freq", "octaves", "seed", "step"])? {
        match k.as_str() {
            "size" => size = v,
            "squash" => squash = v,
            "rough" => rough = v,
            "freq" => freq = v,
            "octaves" => oct = v,
            "seed" => seed = v,
            _ => step = v,
        }
    }
    if size <= 0.0 || squash <= 0.0 || rough < 0.0 {
        return Err("boulder: size and squash above 0, rough 0 or more".into());
    }
    let r = size / 2.0;
    // a little longer one way than the other, from the seed, so a field of them isn't all the same egg
    let stretch = 1.0 + 0.25 * fbm([seed * 1.7, 0.5, 0.0], 1, seed as u32).abs();
    let node = Node::Displace(Box::new(Node::Ellipsoid([r * stretch, r * squash, r / stretch])), rough * size, freq / size, oct as u32, seed as u32);
    let (v, t) = raea_core_sdf::mesh(&node, if step > 0.0 { Some(step) } else { Some(size / 64.0) })?;
    let mut m = Mesh { v, t };
    // rest it on the ground
    let low = m.v.chunks(3).map(|p| p[1]).fold(f32::MAX, f32::min);
    for p in m.v.chunks_mut(3) {
        p[1] -= low;
    }
    Ok(m)
}
