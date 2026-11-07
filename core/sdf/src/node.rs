//! The distance field's tree: primitives, blends and modifiers, each with its bounds and its distance at a point.

use crate::noise::fbm;

pub type V = [f64; 3];

fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn len(a: V) -> f64 {
    dot(a, a).sqrt()
}

/// A rotation as a 3x3 matrix (rows), from x, y, z degrees applied in that order (as Manifold's rotate).
#[derive(Clone, Copy, Debug)]
pub struct Rot(pub [[f64; 3]; 3]);

impl Rot {
    pub fn degrees(x: f64, y: f64, z: f64) -> Rot {
        let (sx, cx) = x.to_radians().sin_cos();
        let (sy, cy) = y.to_radians().sin_cos();
        let (sz, cz) = z.to_radians().sin_cos();
        let rx = [[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]];
        let ry = [[cy, 0.0, sy], [0.0, 1.0, 0.0], [-sy, 0.0, cy]];
        let rz = [[cz, -sz, 0.0], [sz, cz, 0.0], [0.0, 0.0, 1.0]];
        let m = |a: [[f64; 3]; 3], b: [[f64; 3]; 3]| {
            let mut o = [[0.0; 3]; 3];
            for (i, row) in o.iter_mut().enumerate() {
                for (j, v) in row.iter_mut().enumerate() {
                    *v = (0..3).map(|k| a[i][k] * b[k][j]).sum();
                }
            }
            o
        };
        Rot(m(rz, m(ry, rx)))
    }
    pub fn apply(&self, p: V) -> V {
        let m = &self.0;
        [dot(m[0], p), dot(m[1], p), dot(m[2], p)]
    }
    /// the inverse (the transpose, for a rotation)
    pub fn back(&self, p: V) -> V {
        let m = &self.0;
        [m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2], m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2], m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2]]
    }
}

#[derive(Clone, Debug)]
pub enum Node {
    Sphere(f64),
    /// width, height, depth, edge radius
    Box(V, f64),
    /// a cylinder along y: radius, height
    Cylinder(f64, f64),
    /// a ring in the x-z plane (lying flat): major, minor radius
    Torus(f64, f64),
    /// radii along x, y, z
    Ellipsoid(V),
    /// a round cone from a (radius ra) to b (radius rb): a capsule when ra = rb, a limb or a branch when they differ
    Cone(V, V, f64, f64),
    /// a polyline of round cones, each point with its radius: a tube, a tentacle, a root, a stem
    Tube(Vec<(V, f64)>),
    Union(Vec<Node>),
    /// blended joins: k is how far the blend reaches (metres)
    SmoothUnion(f64, Vec<Node>),
    Subtract(Box<Node>, Box<Node>),
    SmoothSubtract(f64, Box<Node>, Box<Node>),
    Intersect(Vec<Node>),
    SmoothIntersect(f64, Vec<Node>),
    Move(Box<Node>, V),
    Rotate(Box<Node>, Rot),
    Scale(Box<Node>, f64),
    /// the x < 0 half mirrored from the x > 0 half: a face, a body, a vase's symmetry
    MirrorX(Box<Node>),
    /// grown by r (rounded): every edge rounded by r
    Round(Box<Node>, f64),
    /// a skin of thickness t (hollow, open nowhere)
    Shell(Box<Node>, f64),
    /// the surface moved by fractal noise: amplitude (m), frequency (per m), octaves, seed
    Displace(Box<Node>, f64, f64, u32, u32),
}

/// A smooth minimum (polynomial, Inigo Quilez's): equal to min far from the join, rounded within k of it.
pub fn smin(a: f64, b: f64, k: f64) -> f64 {
    if k <= 0.0 {
        return a.min(b);
    }
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

fn cone(p: V, a: V, b: V, ra: f64, rb: f64) -> f64 {
    // a round cone between two spheres (IQ's exact form)
    let ba = sub(b, a);
    let l2 = dot(ba, ba);
    if l2 < 1e-18 {
        return len(sub(p, a)) - ra.max(rb);
    }
    let rr = ra - rb;
    let a2 = l2 - rr * rr;
    let il2 = 1.0 / l2;
    let pa = sub(p, a);
    let y = dot(pa, ba);
    let z = y - l2;
    let xv = [pa[0] * l2 - ba[0] * y, pa[1] * l2 - ba[1] * y, pa[2] * l2 - ba[2] * y];
    let x2 = dot(xv, xv);
    let y2 = y * y * l2;
    let z2 = z * z * l2;
    let k = rr.signum() * rr * rr * x2;
    if z.signum() * a2 * z2 > k {
        return (x2 + z2).sqrt() * il2 - rb;
    }
    if y.signum() * a2 * y2 < k {
        return (x2 + y2).sqrt() * il2 - ra;
    }
    ((x2 * a2 * il2).sqrt() + y * rr) * il2 - ra
}

impl Node {
    /// The distance from p to the surface (negative inside). Exact for primitives, a bound for blends.
    pub fn d(&self, p: V) -> f64 {
        match self {
            Node::Sphere(r) => len(p) - r,
            Node::Box(s, r) => {
                let q = [p[0].abs() - s[0] / 2.0 + r, p[1].abs() - s[1] / 2.0 + r, p[2].abs() - s[2] / 2.0 + r];
                len([q[0].max(0.0), q[1].max(0.0), q[2].max(0.0)]) + q[0].max(q[1]).max(q[2]).min(0.0) - r
            }
            Node::Cylinder(r, h) => {
                let d = [(p[0] * p[0] + p[2] * p[2]).sqrt() - r, p[1].abs() - h / 2.0];
                d[0].max(d[1]).min(0.0) + (d[0].max(0.0).powi(2) + d[1].max(0.0).powi(2)).sqrt()
            }
            Node::Torus(big, small) => {
                let q = (p[0] * p[0] + p[2] * p[2]).sqrt() - big;
                (q * q + p[1] * p[1]).sqrt() - small
            }
            Node::Ellipsoid(r) => {
                let k0 = len([p[0] / r[0], p[1] / r[1], p[2] / r[2]]);
                let k1 = len([p[0] / (r[0] * r[0]), p[1] / (r[1] * r[1]), p[2] / (r[2] * r[2])]);
                if k1 < 1e-12 { -r[0].min(r[1]).min(r[2]) } else { k0 * (k0 - 1.0) / k1 }
            }
            Node::Cone(a, b, ra, rb) => cone(p, *a, *b, *ra, *rb),
            Node::Tube(pts) => pts.windows(2).map(|w| cone(p, w[0].0, w[1].0, w[0].1, w[1].1)).fold(f64::MAX, f64::min),
            Node::Union(c) => c.iter().map(|n| n.d(p)).fold(f64::MAX, f64::min),
            Node::SmoothUnion(k, c) => c.iter().map(|n| n.d(p)).reduce(|a, b| smin(a, b, *k)).unwrap_or(f64::MAX),
            Node::Subtract(a, b) => a.d(p).max(-b.d(p)),
            Node::SmoothSubtract(k, a, b) => -smin(-a.d(p), b.d(p), *k),
            Node::Intersect(c) => c.iter().map(|n| n.d(p)).fold(f64::MIN, f64::max),
            Node::SmoothIntersect(k, c) => c.iter().map(|n| n.d(p)).reduce(|a, b| -smin(-a, -b, *k)).unwrap_or(f64::MAX),
            Node::Move(n, t) => n.d(sub(p, *t)),
            Node::Rotate(n, r) => n.d(r.back(p)),
            Node::Scale(n, s) => n.d([p[0] / s, p[1] / s, p[2] / s]) * s,
            Node::MirrorX(n) => n.d([p[0].abs(), p[1], p[2]]),
            Node::Round(n, r) => n.d(p) - r,
            Node::Shell(n, t) => n.d(p).abs() - t,
            Node::Displace(n, amp, freq, oct, seed) => n.d(p) + amp * fbm([p[0] * freq, p[1] * freq, p[2] * freq], *oct, *seed),
        }
    }

    /// A box holding the surface: (min, max).
    pub fn bounds(&self) -> (V, V) {
        let cube = |r: f64| ([-r, -r, -r], [r, r, r]);
        let grow = |(lo, hi): (V, V), g: f64| ([lo[0] - g, lo[1] - g, lo[2] - g], [hi[0] + g, hi[1] + g, hi[2] + g]);
        let join = |v: &[Node]| v.iter().map(|n| n.bounds()).reduce(|(a, b), (c, d)| ([a[0].min(c[0]), a[1].min(c[1]), a[2].min(c[2])], [b[0].max(d[0]), b[1].max(d[1]), b[2].max(d[2])])).unwrap_or(([0.0; 3], [0.0; 3]));
        match self {
            Node::Sphere(r) => cube(*r),
            Node::Box(s, _) => ([-s[0] / 2.0, -s[1] / 2.0, -s[2] / 2.0], [s[0] / 2.0, s[1] / 2.0, s[2] / 2.0]),
            Node::Cylinder(r, h) => ([-r, -h / 2.0, -r], [*r, h / 2.0, *r]),
            Node::Torus(a, b) => ([-a - b, -b, -a - b], [a + b, *b, a + b]),
            Node::Ellipsoid(r) => ([-r[0], -r[1], -r[2]], *r),
            Node::Cone(a, b, ra, rb) => {
                let r = ra.max(*rb);
                ([a[0].min(b[0]) - r, a[1].min(b[1]) - r, a[2].min(b[2]) - r], [a[0].max(b[0]) + r, a[1].max(b[1]) + r, a[2].max(b[2]) + r])
            }
            Node::Tube(pts) => {
                let mut lo = [f64::MAX; 3];
                let mut hi = [f64::MIN; 3];
                for (q, r) in pts {
                    for k in 0..3 {
                        lo[k] = lo[k].min(q[k] - r);
                        hi[k] = hi[k].max(q[k] + r);
                    }
                }
                (lo, hi)
            }
            Node::Union(c) | Node::Intersect(c) | Node::SmoothIntersect(_, c) => join(c),
            Node::SmoothUnion(k, c) => grow(join(c), k * 0.25),
            Node::Subtract(a, _) | Node::SmoothSubtract(_, a, _) => a.bounds(),
            Node::Move(n, t) => {
                let (lo, hi) = n.bounds();
                ([lo[0] + t[0], lo[1] + t[1], lo[2] + t[2]], [hi[0] + t[0], hi[1] + t[1], hi[2] + t[2]])
            }
            Node::Rotate(n, r) => {
                let (lo, hi) = n.bounds();
                let mut a = [f64::MAX; 3];
                let mut b = [f64::MIN; 3];
                for i in 0..8 {
                    let c = r.apply([if i & 1 == 0 { lo[0] } else { hi[0] }, if i & 2 == 0 { lo[1] } else { hi[1] }, if i & 4 == 0 { lo[2] } else { hi[2] }]);
                    for k in 0..3 {
                        a[k] = a[k].min(c[k]);
                        b[k] = b[k].max(c[k]);
                    }
                }
                (a, b)
            }
            Node::Scale(n, s) => {
                let (lo, hi) = n.bounds();
                ([lo[0] * s, lo[1] * s, lo[2] * s], [hi[0] * s, hi[1] * s, hi[2] * s])
            }
            Node::MirrorX(n) => {
                let (lo, hi) = n.bounds();
                let x = lo[0].abs().max(hi[0].abs());
                ([-x, lo[1], lo[2]], [x, hi[1], hi[2]])
            }
            Node::Round(n, r) => grow(n.bounds(), *r),
            Node::Shell(n, t) => grow(n.bounds(), *t),
            Node::Displace(n, amp, ..) => grow(n.bounds(), amp.abs()),
        }
    }
}
