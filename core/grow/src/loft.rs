//! A loft: closed outlines at heights, skinned into one closed solid. A vase, a bottle, a lamp base, a handle's grip, a
//! boat's hull, a column with a swelling (entasis): each outline is resampled to the same count by arc length, starting
//! from the same angle, so neighbouring outlines join without twisting, and the ends are capped by ear clipping.
//!
//! Text form (what `loft("...")` takes in a shape expression): sections by ';', each "y: outline" where the outline is
//! `circle r`, `ellipse rx rz`, `rect w d`, `ngon n r`, or points `x z, x z, ...` (any winding).

use crate::mesh::Mesh;

pub type Section = (f64, Vec<[f64; 2]>);

pub fn parse(text: &str) -> Result<Vec<Section>, String> {
    let mut out = Vec::new();
    for part in text.split(';').map(str::trim).filter(|s| !s.is_empty()) {
        let (y, shape) = part.split_once(':').ok_or_else(|| format!("loft: a section is \"y: outline\", got {part:?}"))?;
        let y: f64 = y.trim().parse().map_err(|_| format!("loft: {y:?} isn't a height"))?;
        let w: Vec<&str> = shape.split_whitespace().collect();
        let nums = |from: usize, n: usize| -> Result<Vec<f64>, String> {
            let v: Vec<f64> = w[from..].iter().map(|x| x.parse::<f64>().map_err(|_| format!("loft: bad number {x:?}"))).collect::<Result<_, _>>()?;
            if v.len() != n || v.iter().any(|x| *x <= 0.0) {
                return Err(format!("loft: {} takes {n} numbers above 0", w[0]));
            }
            Ok(v)
        };
        let ring = |n: usize, rx: f64, rz: f64| (0..n).map(|i| { let a = i as f64 / n as f64 * std::f64::consts::TAU; [rx * a.cos(), rz * a.sin()] }).collect::<Vec<_>>();
        let pts = match w.first().copied() {
            Some("circle") => { let v = nums(1, 1)?; ring(64, v[0], v[0]) }
            Some("ellipse") => { let v = nums(1, 2)?; ring(64, v[0], v[1]) }
            Some("ngon") => { let v = nums(1, 2)?; ring(v[0].round().max(3.0) as usize, v[1], v[1]) }
            Some("rect") => { let v = nums(1, 2)?; let (x, z) = (v[0] / 2.0, v[1] / 2.0); vec![[x, z], [-x, z], [-x, -z], [x, -z]] }
            _ => {
                let mut p = Vec::new();
                for pt in shape.split(',') {
                    let xz: Vec<f64> = pt.split_whitespace().map(|v| v.parse::<f64>().map_err(|_| format!("loft: bad number {v:?}"))).collect::<Result<_, _>>()?;
                    if xz.len() != 2 {
                        return Err(format!("loft: a point is \"x z\", got {pt:?}"));
                    }
                    p.push([xz[0], xz[1]]);
                }
                p
            }
        };
        if pts.len() < 3 {
            return Err("loft: an outline needs 3 points".into());
        }
        out.push((y, pts));
    }
    if out.len() < 2 {
        return Err("loft: 2 or more sections, \"y: outline; y: outline\"".into());
    }
    if out.windows(2).any(|w| w[1].0 <= w[0].0) {
        return Err("loft: sections go up: each y above the one before".into());
    }
    Ok(out)
}

fn area(p: &[[f64; 2]]) -> f64 {
    (0..p.len()).map(|i| { let (a, b) = (p[i], p[(i + 1) % p.len()]); a[0] * b[1] - b[0] * a[1] }).sum::<f64>() / 2.0
}

/// The outline as `n` points evenly spaced along it, counter-clockwise (in x-z), starting where it crosses +x from
/// its centre.
fn resample(p: &[[f64; 2]], n: usize) -> Vec<[f64; 2]> {
    let mut p = p.to_vec();
    if area(&p) < 0.0 {
        p.reverse();
    }
    let c = [p.iter().map(|q| q[0]).sum::<f64>() / p.len() as f64, p.iter().map(|q| q[1]).sum::<f64>() / p.len() as f64];
    // start at the point nearest angle 0 from the centre
    let start = (0..p.len()).min_by(|&a, &b| {
        let ang = |q: [f64; 2]| (q[1] - c[1]).atan2(q[0] - c[0]).abs();
        ang(p[a]).total_cmp(&ang(p[b]))
    }).unwrap();
    p.rotate_left(start);
    let seg: Vec<f64> = (0..p.len()).map(|i| { let (a, b) = (p[i], p[(i + 1) % p.len()]); ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt() }).collect();
    let total: f64 = seg.iter().sum();
    let mut out = Vec::with_capacity(n);
    let (mut i, mut walked) = (0usize, 0.0);
    for k in 0..n {
        let want = k as f64 / n as f64 * total;
        while walked + seg[i] < want && i < p.len() - 1 {
            walked += seg[i];
            i += 1;
        }
        let t = if seg[i] > 0.0 { (want - walked) / seg[i] } else { 0.0 };
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        out.push([a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]);
    }
    out
}

/// Ear clipping: the polygon's triangles as index triples (counter-clockwise input).
fn ears(p: &[[f64; 2]]) -> Vec<[usize; 3]> {
    let cross = |a: [f64; 2], b: [f64; 2], c: [f64; 2]| (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    let mut idx: Vec<usize> = (0..p.len()).collect();
    let mut out = Vec::new();
    let mut guard = 0;
    while idx.len() > 3 && guard < p.len() * p.len() {
        guard += 1;
        let n = idx.len();
        let mut cut = None;
        for k in 0..n {
            let (a, b, c) = (idx[(k + n - 1) % n], idx[k], idx[(k + 1) % n]);
            if cross(p[a], p[b], p[c]) <= 1e-14 {
                continue;
            }
            let inside = idx.iter().any(|&o| o != a && o != b && o != c && cross(p[a], p[b], p[o]) > 0.0 && cross(p[b], p[c], p[o]) > 0.0 && cross(p[c], p[a], p[o]) > 0.0);
            if !inside {
                cut = Some(k);
                out.push([a, b, c]);
                break;
            }
        }
        match cut {
            Some(k) => { idx.remove(k); }
            // degenerate: fan what is left
            None => break,
        }
    }
    for k in 1..idx.len().saturating_sub(1) {
        out.push([idx[0], idx[k], idx[k + 1]]);
    }
    out
}

/// One closed solid through the sections, `n` points around each (0: the most any section has, at least 24).
pub fn loft(sections: &[Section], n: usize) -> Mesh {
    let n = if n >= 3 { n } else { sections.iter().map(|s| s.1.len()).max().unwrap_or(24).max(24) };
    let rings: Vec<Vec<[f64; 2]>> = sections.iter().map(|s| resample(&s.1, n)).collect();
    let mut m = Mesh::default();
    let ids: Vec<Vec<u32>> = rings.iter().zip(sections).map(|(r, s)| r.iter().map(|q| m.vert([q[0], s.0, q[1]])).collect()).collect();
    // counter-clockwise in (x, z) seen from +y is clockwise (x right, z down the page): sides wound for that
    for w in ids.windows(2) {
        for s in 0..n {
            let (a, b) = (w[0][s], w[0][(s + 1) % n]);
            let (c, d) = (w[1][s], w[1][(s + 1) % n]);
            m.tri(a, c, d);
            m.tri(a, d, b);
        }
    }
    let (bot, top) = (&ids[0], &ids[ids.len() - 1]);
    for e in ears(&rings[0]) {
        m.tri(bot[e[0]], bot[e[1]], bot[e[2]]);
    }
    for e in ears(&rings[rings.len() - 1]) {
        m.tri(top[e[0]], top[e[2]], top[e[1]]);
    }
    m
}
