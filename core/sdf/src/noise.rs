//! Seeded value noise in 3D and its fractal sum: the same seed gives the same surface, every run.

fn hash(x: i32, y: i32, z: i32, seed: u32) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x8da6b343) ^ (y as u32).wrapping_mul(0xd8163841) ^ (z as u32).wrapping_mul(0xcb1ab31f) ^ seed.wrapping_mul(0x9e3779b9);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1e995);
    h ^= h >> 15;
    (h as f64 / u32::MAX as f64) * 2.0 - 1.0
}

/// Smooth value noise in [-1, 1].
pub fn noise(p: [f64; 3], seed: u32) -> f64 {
    let i = [p[0].floor(), p[1].floor(), p[2].floor()];
    let f = [p[0] - i[0], p[1] - i[1], p[2] - i[2]];
    let s = f.map(|t| t * t * (3.0 - 2.0 * t));
    let (x, y, z) = (i[0] as i32, i[1] as i32, i[2] as i32);
    let mut acc = 0.0;
    for c in 0..8 {
        let (dx, dy, dz) = (c & 1, (c >> 1) & 1, (c >> 2) & 1);
        let w = (if dx == 1 { s[0] } else { 1.0 - s[0] }) * (if dy == 1 { s[1] } else { 1.0 - s[1] }) * (if dz == 1 { s[2] } else { 1.0 - s[2] });
        acc += w * hash(x + dx, y + dy, z + dz, seed);
    }
    acc
}

/// Fractal noise: `octaves` layers, each twice the frequency and half the amplitude, summed to about [-1, 1].
pub fn fbm(p: [f64; 3], octaves: u32, seed: u32) -> f64 {
    let (mut sum, mut amp, mut norm, mut q) = (0.0, 1.0, 0.0, p);
    for o in 0..octaves.clamp(1, 8) {
        sum += amp * noise(q, seed.wrapping_add(o * 1013));
        norm += amp;
        amp *= 0.5;
        q = [q[0] * 2.03 + 17.1, q[1] * 2.03 - 4.7, q[2] * 2.03 + 9.3];
    }
    sum / norm
}
