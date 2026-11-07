//! A swept tube along a path of points, each with its radius: rings carried along by parallel transport (no twist
//! flips at bends), capped at both ends, so each tube is one closed piece. A branch, a root, a cable, a stem.

use crate::mesh::{Mesh, add, cross, dot, mul, norm, sub};

/// One closed tube: `sides` around (6 or more), the path's points and radii.
pub fn tube(path: &[([f64; 3], f64)], sides: usize) -> Mesh {
    let mut m = Mesh::default();
    if path.len() < 2 {
        return m;
    }
    let sides = sides.max(3);
    // tangents: the mean of the segments meeting at each point
    let tan: Vec<[f64; 3]> = (0..path.len()).map(|i| {
        let a = path[i.saturating_sub(1)].0;
        let b = path[(i + 1).min(path.len() - 1)].0;
        norm(sub(b, a))
    }).collect();
    // a first normal across the first tangent, then carried along
    let t0 = tan[0];
    let helper = if t0[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
    let mut n = norm(cross(t0, helper));
    let mut rings: Vec<Vec<u32>> = Vec::with_capacity(path.len());
    for i in 0..path.len() {
        if i > 0 {
            // parallel transport: take out the part of n along the new tangent
            let t = tan[i];
            n = norm(sub(n, mul(t, dot(n, t))));
        }
        let b = cross(tan[i], n);
        let (c, r) = path[i];
        let ring: Vec<u32> = (0..sides).map(|s| {
            let a = s as f64 / sides as f64 * std::f64::consts::TAU;
            m.vert(add(c, add(mul(n, r * a.cos()), mul(b, r * a.sin()))))
        }).collect();
        rings.push(ring);
    }
    for w in rings.windows(2) {
        for s in 0..sides {
            let (a, b) = (w[0][s], w[0][(s + 1) % sides]);
            let (c, d) = (w[1][s], w[1][(s + 1) % sides]);
            m.tri(a, b, d);
            m.tri(a, d, c);
        }
    }
    // caps: a fan to each end's centre
    let start = m.vert(path[0].0);
    let end = m.vert(path[path.len() - 1].0);
    let (first, last) = (&rings[0], &rings[rings.len() - 1]);
    for s in 0..sides {
        m.tri(start, first[(s + 1) % sides], first[s]);
        m.tri(end, last[s], last[(s + 1) % sides]);
    }
    m
}
