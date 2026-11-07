//! The surface's health from its triangles welded by position: open edges (one face), edges three or more faces
//! share, triangles with no area, and the separate pieces, with the small loose ones named by where they are.

use std::collections::HashMap;

pub struct Topo {
    pub pieces: usize,
    /// pieces under 0.5% of the triangles: their centre and triangle count
    pub loose: Vec<([f64; 3], usize)>,
    pub open_edges: usize,
    pub shared_edges: usize,
    pub degenerate: usize,
}

fn find(p: &mut [u32], mut i: u32) -> u32 {
    while p[i as usize] != i {
        p[i as usize] = p[p[i as usize] as usize];
        i = p[i as usize];
    }
    i
}

pub fn measure(verts: &[[f64; 3]], tris: &[[u32; 3]], span: f64) -> Topo {
    // weld: positions within a millionth of the model's size are one vertex (UV seams split vertices that touch)
    let q = (span.max(1e-9) * 1e-6).max(1e-9);
    let key = |p: &[f64; 3]| ((p[0] / q).round() as i64, (p[1] / q).round() as i64, (p[2] / q).round() as i64);
    let mut ids: HashMap<(i64, i64, i64), u32> = HashMap::with_capacity(verts.len());
    let weld: Vec<u32> = verts.iter().map(|p| { let n = ids.len() as u32; *ids.entry(key(p)).or_insert(n) }).collect();
    let mut parent: Vec<u32> = (0..ids.len() as u32).collect();
    let mut edges: HashMap<(u32, u32), u32> = HashMap::with_capacity(tris.len() * 2);
    let mut degenerate = 0;
    for t in tris {
        let w = [weld[t[0] as usize], weld[t[1] as usize], weld[t[2] as usize]];
        if w[0] == w[1] || w[1] == w[2] || w[0] == w[2] {
            degenerate += 1;
            continue;
        }
        let (a, b, c) = (verts[t[0] as usize], verts[t[1] as usize], verts[t[2] as usize]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        if (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() < q * q {
            degenerate += 1;
        }
        for k in 0..3 {
            let (x, y) = (w[k], w[(k + 1) % 3]);
            *edges.entry((x.min(y), x.max(y))).or_insert(0) += 1;
            let (rx, ry) = (find(&mut parent, x), find(&mut parent, y));
            if rx != ry {
                parent[rx as usize] = ry;
            }
        }
    }
    let open_edges = edges.values().filter(|&&c| c == 1).count();
    let shared_edges = edges.values().filter(|&&c| c > 2).count();
    // pieces: triangles grouped by their welded root
    let mut by: HashMap<u32, (usize, [f64; 3])> = HashMap::new();
    for t in tris {
        let r = find(&mut parent, weld[t[0] as usize]);
        let e = by.entry(r).or_insert((0, [0.0; 3]));
        e.0 += 1;
        for k in 0..3 {
            e.1[k] += (verts[t[0] as usize][k] + verts[t[1] as usize][k] + verts[t[2] as usize][k]) / 3.0;
        }
    }
    let small = ((tris.len() as f64) * 0.005).max(1.0) as usize;
    let mut loose: Vec<([f64; 3], usize)> = by.values().filter(|(n, _)| *n < small && by.len() > 1).map(|(n, s)| ([s[0] / *n as f64, s[1] / *n as f64, s[2] / *n as f64], *n)).collect();
    loose.sort_by(|a, b| b.1.cmp(&a.1));
    loose.truncate(12);
    Topo { pieces: by.len(), loose, open_edges, shared_edges, degenerate }
}
