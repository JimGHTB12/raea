//! A distance field sampled on a grid and meshed by marching tetrahedra (see `mesh`). The grid is padded so the surface never meets its border,
//! which keeps the mesh closed. Blocks far from the surface are skipped from a coarse pass (the field changes by at
//! most about its distance, so a block whose corners are all far is all outside or all inside).

use crate::node::V;

pub struct Grid {
    pub min: V,
    pub h: f64,
    pub n: [usize; 3],
}

impl Grid {
    fn at(&self, i: usize, j: usize, k: usize) -> V {
        [self.min[0] + i as f64 * self.h, self.min[1] + j as f64 * self.h, self.min[2] + k as f64 * self.h]
    }
    fn idx(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.n[0] * (j + self.n[1] * k)
    }
}

/// The field sampled on the grid's points, across threads, skipping far blocks.
pub fn sample(g: &Grid, f: &(dyn Fn(V) -> f64 + Sync)) -> Vec<f32> {
    const B: usize = 4;
    let [nx, ny, nz] = g.n;
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 12);
    // the coarse pass: every B-th point
    let (cx, cy, cz) = ((nx - 1) / B + 2, (ny - 1) / B + 2, (nz - 1) / B + 2);
    let coarse: Vec<f64> = par(cz, threads, |k| {
        let mut row = Vec::with_capacity(cx * cy);
        for j in 0..cy {
            for i in 0..cx {
                row.push(f(g.at(i * B, j * B, k * B)));
            }
        }
        row
    });
    let reach = g.h * B as f64 * 3f64.sqrt() * 1.5;
    let fine: Vec<f32> = par(nz, threads, |k| {
        let mut out = vec![0f32; nx * ny];
        let bk = k / B;
        for j in 0..ny {
            let bj = j / B;
            for i in 0..nx {
                let bi = i / B;
                let c = |a: usize, b: usize, cc: usize| coarse[(bi + a).min(cx - 1) + cx * ((bj + b).min(cy - 1) + cy * (bk + cc).min(cz - 1))];
                let corners = [c(0, 0, 0), c(1, 0, 0), c(0, 1, 0), c(1, 1, 0), c(0, 0, 1), c(1, 0, 1), c(0, 1, 1), c(1, 1, 1)];
                let all_out = corners.iter().all(|&d| d > reach);
                let all_in = corners.iter().all(|&d| d < -reach);
                out[i + nx * j] = if all_out {
                    reach as f32
                } else if all_in {
                    -reach as f32
                } else {
                    f(g.at(i, j, k)) as f32
                };
            }
        }
        out
    });
    // the border is outside, so the mesh closes
    let mut v = fine;
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                if i == 0 || j == 0 || k == 0 || i == nx - 1 || j == ny - 1 || k == nz - 1 {
                    let x = g.idx(i, j, k);
                    v[x] = v[x].abs().max(g.h as f32);
                }
            }
        }
    }
    v
}

/// Rows 0..n made by `f` on `threads` threads, joined in order.
fn par<T: Send, F: Fn(usize) -> Vec<T> + Sync>(n: usize, threads: usize, f: F) -> Vec<T> {
    let per = n.div_ceil(threads.max(1));
    std::thread::scope(|s| {
        let hs: Vec<_> = (0..threads).map(|t| {
            let f = &f;
            s.spawn(move || (t * per..((t + 1) * per).min(n)).flat_map(f).collect::<Vec<T>>())
        }).collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}

/// The mesh: positions flat (x y z) and triangles by threes, wound outward. Marching tetrahedra: each cell cut into
/// six tetrahedra along its main diagonal (the same cut in every cell, so neighbours share their faces' splits), and
/// the field taken as linear inside each: the surface of a piecewise-linear field over tetrahedra is always a closed
/// 2-manifold, which Manifold needs (surface nets join two sheets at a vertex where a thin part pinches).
pub fn mesh(g: &Grid, v: &[f32]) -> (Vec<f32>, Vec<u32>) {
    let [nx, ny, nz] = g.n;
    // no sample exactly on the surface: a zero would put a vertex on a grid point, shared by sheets
    let val = |i: usize| { let x = v[i]; if x == 0.0 { 1e-9 } else { x } };
    let mut verts: Vec<f32> = Vec::new();
    let mut at: std::collections::HashMap<(u32, u32), u32> = std::collections::HashMap::new();
    let mut tris: Vec<u32> = Vec::new();
    const TETS: [[usize; 4]; 6] = [[0, 1, 3, 7], [0, 1, 5, 7], [0, 2, 3, 7], [0, 2, 6, 7], [0, 4, 5, 7], [0, 4, 6, 7]];
    let pos = |i: usize| { let x = i % nx; let y = (i / nx) % ny; let z = i / (nx * ny); g.at(x, y, z) };
    let mut vert = |a: usize, b: usize, verts: &mut Vec<f32>| -> u32 {
        let key = (a.min(b) as u32, a.max(b) as u32);
        if let Some(&k) = at.get(&key) {
            return k;
        }
        let (fa, fb) = (val(a) as f64, val(b) as f64);
        let t = fa / (fa - fb);
        let (pa, pb) = (pos(a), pos(b));
        let k = (verts.len() / 3) as u32;
        for q in 0..3 {
            verts.push((pa[q] + t * (pb[q] - pa[q])) as f32);
        }
        at.insert(key, k);
        k
    };
    for k in 0..nz - 1 {
        for j in 0..ny - 1 {
            for i in 0..nx - 1 {
                let c = |b: usize| g.idx(i + (b & 1), j + ((b >> 1) & 1), k + ((b >> 2) & 1));
                let corners = [c(0), c(1), c(2), c(3), c(4), c(5), c(6), c(7)];
                let inside = corners.iter().filter(|&&x| val(x) < 0.0).count();
                if inside == 0 || inside == 8 {
                    continue;
                }
                for tet in TETS {
                    // corners by their place in the cell (bits x, y, z), split inside and outside
                    let (ins, outs): (Vec<usize>, Vec<usize>) = tet.iter().partition(|&&b| val(corners[b]) < 0.0);
                    let g = |b: usize| corners[b];
                    let mut emit = |tri: [u32; 3], keep: bool| if keep { tris.extend(tri) } else { tris.extend([tri[0], tri[2], tri[1]]) };
                    // winding from the sign of the corners' determinant, exact on the cell's integer offsets (a
                    // triangle's own normal flips on slivers, which breaks the halfedge pairing Manifold checks)
                    match ins.len() {
                        1 => {
                            let a = ins[0];
                            let t = [vert(g(a), g(outs[0]), &mut verts), vert(g(a), g(outs[1]), &mut verts), vert(g(a), g(outs[2]), &mut verts)];
                            emit(t, det(outs[0], outs[1], outs[2], a) > 0);
                        }
                        3 => {
                            let o = outs[0];
                            let t = [vert(g(ins[0]), g(o), &mut verts), vert(g(ins[1]), g(o), &mut verts), vert(g(ins[2]), g(o), &mut verts)];
                            emit(t, det(ins[0], ins[1], ins[2], o) < 0);
                        }
                        2 => {
                            let (a, b) = (ins[0], ins[1]);
                            let (c0, d0) = (outs[0], outs[1]);
                            let q = [vert(g(a), g(c0), &mut verts), vert(g(a), g(d0), &mut verts), vert(g(b), g(d0), &mut verts), vert(g(b), g(c0), &mut verts)];
                            let keep = det(b, c0, d0, a) > 0;
                            emit([q[0], q[1], q[2]], keep);
                            emit([q[0], q[2], q[3]], keep);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    (verts, tris)
}

/// det(p - o, q - o, r - o) over the cell's corners as integer offsets (bit 0 x, bit 1 y, bit 2 z).
fn det(p: usize, q: usize, r: usize, o: usize) -> i32 {
    let off = |b: usize, o: usize| [(b & 1) as i32 - (o & 1) as i32, ((b >> 1) & 1) as i32 - ((o >> 1) & 1) as i32, ((b >> 2) & 1) as i32 - ((o >> 2) & 1) as i32];
    let (u, v, w) = (off(p, o), off(q, o), off(r, o));
    u[0] * (v[1] * w[2] - v[2] * w[1]) - u[1] * (v[0] * w[2] - v[2] * w[0]) + u[2] * (v[0] * w[1] - v[1] * w[0])
}
