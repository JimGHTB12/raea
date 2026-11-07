//! One solid's measures: volume, area, box, centre of mass (from its triangles, by the divergence theorem), pieces and
//! their volumes, genus, cavities (shells closed inside a piece), triangles. A Manifold is closed by construction, so a
//! solid that isn't empty is watertight.

use raea_core_shape::Manifold;
use serde_json::{Value, json};

pub fn r6(x: f64) -> f64 {
    if !x.is_finite() {
        return 0.0;
    }
    let m = 1e6;
    (x * m).round() / m
}

#[derive(Clone, Debug)]
pub struct Measured {
    pub empty: bool,
    pub volume: f64,
    pub area: f64,
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub center_of_mass: [f64; 3],
    pub pieces: usize,
    /// largest first
    pub piece_volumes: Vec<f64>,
    pub genus: i32,
    /// shells enclosed in a piece (a hollow sealed inside)
    pub cavities: usize,
    pub tris: usize,
}

pub fn measure(m: &Manifold) -> Measured {
    if m.is_empty() {
        return Measured { empty: true, volume: 0.0, area: 0.0, min: [0.0; 3], max: [0.0; 3], center_of_mass: [0.0; 3], pieces: 0, piece_volumes: Vec::new(), genus: 0, cavities: 0, tris: 0 };
    }
    let b = m.bounding_box();
    let (v, t) = raea_core_shape::to_mesh(m);
    let p = |i: u32| vertex(&v, i);
    let (mut vol, mut c) = (0.0, [0.0; 3]);
    for tri in t.chunks(3) {
        let (a, bb, d) = (p(tri[0]), p(tri[1]), p(tri[2]));
        let six = a[0] * (bb[1] * d[2] - bb[2] * d[1]) - a[1] * (bb[0] * d[2] - bb[2] * d[0]) + a[2] * (bb[0] * d[1] - bb[1] * d[0]);
        vol += six;
        for k in 0..3 {
            c[k] += six * (a[k] + bb[k] + d[k]);
        }
    }
    let com = if vol.abs() > 1e-18 { [c[0] / (4.0 * vol), c[1] / (4.0 * vol), c[2] / (4.0 * vol)] } else { [0.0; 3] };
    // decompose gives a sealed hollow as its own, inside-out shell: a cavity, not a piece
    let all: Vec<f64> = m.decompose().iter().map(|x| r6(signed_volume(x))).collect();
    let cavities = all.iter().filter(|v| **v < 0.0).count();
    let mut piece_volumes: Vec<f64> = all.into_iter().filter(|v| *v >= 0.0).collect();
    piece_volumes.sort_by(|a, b| b.total_cmp(a));
    Measured {
        empty: false,
        volume: m.volume(),
        area: m.surface_area(),
        min: [b.min.x, b.min.y, b.min.z],
        max: [b.max.x, b.max.y, b.max.z],
        center_of_mass: com,
        pieces: piece_volumes.len(),
        cavities,
        piece_volumes,
        genus: m.genus(),
        tris: m.num_tri(),
    }
}

/// Volume from the triangles' winding (divergence theorem): an inside-out shell, a cavity on its own, is negative.
fn signed_volume(m: &Manifold) -> f64 {
    let (v, t) = raea_core_shape::to_mesh(m);
    let p = |i: u32| vertex(&v, i);
    t.chunks(3).map(|tri| {
        let (a, b, d) = (p(tri[0]), p(tri[1]), p(tri[2]));
        a[0] * (b[1] * d[2] - b[2] * d[1]) - a[1] * (b[0] * d[2] - b[2] * d[0]) + a[2] * (b[0] * d[1] - b[1] * d[0])
    }).sum::<f64>() / 6.0
}

impl Measured {
    pub fn size(&self) -> [f64; 3] {
        [self.max[0] - self.min[0], self.max[1] - self.min[1], self.max[2] - self.min[2]]
    }

    pub fn json(&self) -> Value {
        let s = self.size();
        let r3 = |a: [f64; 3]| a.map(r6);
        let c = [(self.min[0] + self.max[0]) / 2.0, (self.min[1] + self.max[1]) / 2.0, (self.min[2] + self.max[2]) / 2.0];
        json!({"empty": self.empty, "volume": r6(self.volume), "area": r6(self.area),
            "box": {"min": r3(self.min), "max": r3(self.max), "size": r3(s), "center": r3(c)},
            "centerOfMass": r3(self.center_of_mass), "pieces": self.pieces, "genus": self.genus, "cavities": self.cavities, "tris": self.tris, "watertight": !self.empty})
    }
}

/// Vertex `i` of a flat [x, y, z, ...] position list.
fn vertex(v: &[f32], i: u32) -> [f64; 3] {
    let i = i as usize * 3;
    [v[i] as f64, v[i + 1] as f64, v[i + 2] as f64]
}
