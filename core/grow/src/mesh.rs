//! Triangle meshes as the generators make them: closed pieces appended to one list, and a seeded random stream.

#[derive(Default, Clone, Debug)]
pub struct Mesh {
    /// positions flat (x y z)
    pub v: Vec<f32>,
    /// triangles by threes, wound outward
    pub t: Vec<u32>,
}

impl Mesh {
    pub fn vert(&mut self, p: [f64; 3]) -> u32 {
        let i = (self.v.len() / 3) as u32;
        self.v.extend([p[0] as f32, p[1] as f32, p[2] as f32]);
        i
    }
    pub fn tri(&mut self, a: u32, b: u32, c: u32) {
        self.t.extend([a, b, c]);
    }
    pub fn append(&mut self, o: &Mesh) {
        let base = (self.v.len() / 3) as u32;
        self.v.extend_from_slice(&o.v);
        self.t.extend(o.t.iter().map(|i| i + base));
    }
    pub fn triangles(&self) -> usize {
        self.t.len() / 3
    }
}

/// A seeded stream in [0, 1) (splitmix64): the same seed, the same tree.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9e3779b97f4a7c15) ^ 0xd1b54a32d192ed03)
    }
    pub fn f(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f()
    }
    /// a unit vector, uniform on the sphere
    pub fn dir(&mut self) -> [f64; 3] {
        let z = self.range(-1.0, 1.0);
        let a = self.range(0.0, std::f64::consts::TAU);
        let r = (1.0 - z * z).sqrt();
        [r * a.cos(), z, r * a.sin()]
    }
}

pub fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn mul(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub use raea_core_sdf::{dot, len};
pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
pub fn norm(a: [f64; 3]) -> [f64; 3] {
    let l = len(a);
    if l < 1e-12 { [0.0, 1.0, 0.0] } else { mul(a, 1.0 / l) }
}
