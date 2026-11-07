//! Trees grown by space colonization (Runions et al., 2007): attraction points scattered inside a crown envelope, and
//! branches that grow a step at a time toward the points near them, which are used up as branches reach them. The
//! crown's shape is the input, so a tree is art-directed by shape (a dome for an oak, a cone for a pine, a column for
//! a cypress) and the growth fills it: the twigs come from the rule, never from hand placement, so a crown can't be
//! faked with a few blobs. Radii follow the pipe model (a fork's children's cross-sections add up to about its
//! parent's: Leonardo's rule), so limbs thin as they go out. Leaves are cards on the outer twigs.

use std::collections::HashMap;

use crate::mesh::{Mesh, Rng, add, cross, len, mul, norm, sub};
use crate::tube::tube;

#[derive(Clone, Debug)]
pub struct Tree {
    /// the whole tree's height, metres
    pub height: f64,
    /// the clear trunk below the crown, as a share of the height
    pub trunk: f64,
    /// the crown's width as a share of its own height (1: as wide as tall)
    pub width: f64,
    /// dome, sphere, cone, column, umbrella, vase
    pub crown: String,
    /// attraction points: more is a denser, twiggier crown
    pub points: usize,
    /// the growth step, as a share of the height (twig length)
    pub step: f64,
    /// the trunk's radius at the ground, metres (0: a fortieth of the height)
    pub base: f64,
    /// the outermost twigs' radius, metres
    pub tip: f64,
    /// how strongly branches turn up (positive) or droop (negative) as they grow
    pub tropism: f64,
    /// leaves per outer twig node (0: a bare tree)
    pub leaves: usize,
    /// a leaf's length, metres
    pub leaf: f64,
    /// sides of each branch's tube
    pub sides: usize,
    pub seed: u64,
}

impl Default for Tree {
    fn default() -> Tree {
        Tree { height: 6.0, trunk: 0.35, width: 0.9, crown: "dome".into(), points: 1200, step: 0.03, base: 0.0, tip: 0.006, tropism: 0.15, leaves: 3, leaf: 0.09, sides: 7, seed: 1 }
    }
}

impl Tree {
    /// `height=8 crown=cone points=2000 seed=4`: settings by name, the rest default.
    pub fn parse(text: &str) -> Result<Tree, String> {
        let mut t = Tree::default();
        for kv in text.split_whitespace() {
            let (k, v) = kv.split_once('=').ok_or_else(|| format!("tree: {kv:?} isn't name=value"))?;
            let num = || v.parse::<f64>().map_err(|_| format!("tree: {k} takes a number, got {v:?}"));
            match k {
                "height" => t.height = num()?.max(0.05),
                "trunk" => t.trunk = num()?.clamp(0.0, 0.9),
                "width" => t.width = num()?.max(0.05),
                "crown" => t.crown = v.to_string(),
                "points" => t.points = num()?.clamp(10.0, 20000.0) as usize,
                "step" => t.step = num()?.clamp(0.005, 0.2),
                "base" => t.base = num()?.max(0.0),
                "tip" => t.tip = num()?.max(0.0005),
                "tropism" => t.tropism = num()?.clamp(-1.0, 1.0),
                "leaves" => t.leaves = num()?.clamp(0.0, 12.0) as usize,
                "leaf" => t.leaf = num()?.max(0.005),
                "sides" => t.sides = num()?.clamp(3.0, 24.0) as usize,
                "seed" => t.seed = num()? as u64,
                o => return Err(format!("tree: no setting {o}: height, trunk, width, crown, points, step, base, tip, tropism, leaves, leaf, sides, seed")),
            }
        }
        if !["dome", "sphere", "cone", "column", "umbrella", "vase"].contains(&t.crown.as_str()) {
            return Err(format!("tree: crown {} isn't one of dome, sphere, cone, column, umbrella, vase", t.crown));
        }
        Ok(t)
    }

    /// Whether a point (in the crown's unit space: y 0 at the crown's bottom, 1 at its top, x and z by its radius) is in
    /// the crown.
    fn inside(&self, x: f64, y: f64, z: f64) -> bool {
        let r = (x * x + z * z).sqrt();
        match self.crown.as_str() {
            "sphere" => r * r + (2.0 * y - 1.0).powi(2) <= 1.0,
            "dome" => y >= 0.0 && r * r + (y * 1.3 - 0.3).max(0.0).powi(2) / 1.0 <= 1.0 && y <= 1.0,
            "cone" => (0.0..=1.0).contains(&y) && r <= 1.0 - y,
            "column" => (0.0..=1.0).contains(&y) && r <= 0.55 * (1.0 - (2.0 * y - 1.0).powi(4)).sqrt(),
            "umbrella" => (0.55..=1.0).contains(&y) && r * r + ((y - 0.55) / 0.45 * 2.0 - 1.0).powi(2) <= 1.0,
            _ => (0.0..=1.0).contains(&y) && r <= 0.25 + 0.75 * y && r >= 0.15 * y,
        }
    }
}

/// The grown skeleton: each node's position, parent and radius.
pub struct Skeleton {
    pub pos: Vec<[f64; 3]>,
    pub parent: Vec<Option<usize>>,
    pub radius: Vec<f64>,
    pub children: Vec<Vec<usize>>,
}

pub fn grow(t: &Tree) -> Skeleton {
    let mut rng = Rng::new(t.seed);
    let h = t.height;
    let crown_y = h * t.trunk;
    let crown_h = h - crown_y;
    let crown_r = crown_h * t.width / 2.0;
    // attraction points inside the envelope
    let mut pts: Vec<[f64; 3]> = Vec::with_capacity(t.points);
    let mut tries = 0;
    while pts.len() < t.points && tries < t.points * 200 {
        tries += 1;
        let (x, y, z) = (rng.range(-1.0, 1.0), rng.f(), rng.range(-1.0, 1.0));
        if t.inside(x, y, z) {
            pts.push([x * crown_r, crown_y + y * crown_h, z * crown_r]);
        }
    }
    let step = h * t.step;
    let influence = step * 9.0;
    let kill = step * 1.6;
    let mut pos = vec![[0.0, 0.0, 0.0]];
    let mut parent: Vec<Option<usize>> = vec![None];
    // the trunk: straight up until the crown's points are in reach
    while pos[pos.len() - 1][1] < crown_y - influence * 0.5 && pos.len() < 4000 {
        let last = pos.len() - 1;
        let wobble = [rng.range(-0.04, 0.04), 1.0, rng.range(-0.04, 0.04)];
        pos.push(add(pos[last], mul(norm(wobble), step)));
        parent.push(Some(last));
    }
    let cell = influence;
    let key = |p: &[f64; 3]| p.map(|x| (x / cell).floor() as i32);
    let mut grid: HashMap<[i32; 3], Vec<usize>> = HashMap::new();
    for (i, p) in pos.iter().enumerate() {
        grid.entry(key(p)).or_default().push(i);
    }
    let up = [0.0, 1.0, 0.0];
    for _ in 0..400 {
        if pts.is_empty() {
            break;
        }
        // each point pulls on its nearest node within reach
        let mut pull: HashMap<usize, [f64; 3]> = HashMap::new();
        for p in &pts {
            let k = key(p);
            let mut best: Option<(usize, f64)> = None;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        for &n in grid.get(&[k[0] + dx, k[1] + dy, k[2] + dz]).into_iter().flatten() {
                            let d = len(sub(*p, pos[n]));
                            if d < influence && best.is_none_or(|b| d < b.1) {
                                best = Some((n, d));
                            }
                        }
                    }
                }
            }
            if let Some((n, _)) = best {
                let e = pull.entry(n).or_insert([0.0; 3]);
                *e = add(*e, norm(sub(*p, pos[n])));
            }
        }
        if pull.is_empty() {
            break;
        }
        let mut grew: Vec<(usize, [f64; 3])> = pull.into_iter().collect();
        grew.sort_by_key(|g| g.0);
        for (n, d) in grew {
            let dir = norm(add(norm(d), mul(up, t.tropism)));
            let p = add(pos[n], mul(dir, step));
            // two nodes in the same place stall growth: skip a step that lands on its parent's sibling
            if grid.get(&key(&p)).into_iter().flatten().any(|&o| len(sub(pos[o], p)) < step * 0.3) {
                continue;
            }
            let i = pos.len();
            pos.push(p);
            parent.push(Some(n));
            grid.entry(key(&p)).or_default().push(i);
        }
        // points a branch has reached are used up
        pts.retain(|p| {
            let k = key(p);
            !(-1..=1).any(|dx| (-1..=1).any(|dy| (-1..=1).any(|dz| grid.get(&[k[0] + dx, k[1] + dy, k[2] + dz]).into_iter().flatten().any(|&n| len(sub(*p, pos[n])) < kill))))
        });
    }
    let mut children = vec![Vec::new(); pos.len()];
    for (i, p) in parent.iter().enumerate() {
        if let Some(p) = p {
            children[*p].push(i);
        }
    }
    // the pipe model, from the tips in: r^2.5 adds up at every fork
    let mut radius = vec![t.tip; pos.len()];
    for i in (0..pos.len()).rev() {
        if !children[i].is_empty() {
            radius[i] = children[i].iter().map(|&c| radius[c].powf(2.5)).sum::<f64>().powf(1.0 / 2.5).max(t.tip);
        }
    }
    // the pipe model sets each limb's share; the trunk's size is the art's: tips stay at `tip`, the trunk meets `base`
    // (by default a fortieth of the height, as a broadleaf tree's), everything between mapped in proportion
    let base = if t.base > 0.0 { t.base } else { t.height / 40.0 };
    let r0 = radius[0];
    if r0 > t.tip {
        for r in &mut radius {
            *r = t.tip + (*r - t.tip) / (r0 - t.tip) * (base - t.tip).max(0.0);
        }
    }
    Skeleton { pos, parent, radius, children }
}

/// The wood: each branch one tube from the fork it leaves to its tip, following its thickest child at every fork.
/// The wood: one closed tube a branch, each starting inside its parent (so they overlap at the forks: union them).
pub fn wood(t: &Tree, s: &Skeleton) -> Vec<Mesh> {
    let mut m = Vec::new();
    let mut starts = vec![0usize];
    while let Some(start) = starts.pop() {
        let mut path = Vec::new();
        if let Some(p) = s.parent[start] {
            // begin inside the parent, so the joint is covered
            path.push((s.pos[p], s.radius[start]));
        }
        let mut at = start;
        loop {
            path.push((s.pos[at], s.radius[at]));
            let kids = &s.children[at];
            if kids.is_empty() {
                break;
            }
            let main = *kids.iter().max_by(|a, b| s.radius[**a].total_cmp(&s.radius[**b])).unwrap();
            for &k in kids {
                if k != main {
                    starts.push(k);
                }
            }
            at = main;
        }
        if path.len() >= 2 {
            // thin twigs get fewer sides
            let r0 = path[0].1;
            let sides = if r0 < t.tip * 3.0 { 3.max(t.sides / 2) } else { t.sides };
            m.push(tube(&path, sides));
        }
    }
    m
}

/// The leaves: flat cards (closed and thin, so they stay a solid) at the outer twigs, facing out and up.
pub fn leaves(t: &Tree, s: &Skeleton) -> Mesh {
    let mut m = Mesh::default();
    if t.leaves == 0 {
        return m;
    }
    let mut rng = Rng::new(t.seed ^ 0x1eaf);
    let outer = t.tip * 2.5;
    for i in 0..s.pos.len() {
        if s.radius[i] > outer {
            continue;
        }
        for _ in 0..t.leaves {
            let base = s.pos[i];
            let along = norm(add(rng.dir(), [0.0, 0.3, 0.0]));
            let side = norm(cross(along, if along[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] }));
            let face = norm(cross(along, side));
            let l = t.leaf * rng.range(0.75, 1.25);
            let w = l * 0.45;
            let th = (l * 0.02).max(0.0005);
            let tipp = add(base, mul(along, l));
            let mid = add(base, mul(along, l * 0.45));
            // a flat diamond: base, two sides, tip, each face a pair of triangles, split by thickness
            let a = m.vert(base);
            let b = m.vert(add(mid, mul(side, w / 2.0)));
            let c = m.vert(tipp);
            let d = m.vert(add(mid, mul(side, -w / 2.0)));
            let up = m.vert(add(mid, mul(face, th)));
            let dn = m.vert(add(mid, mul(face, -th)));
            for (x, y) in [(a, b), (b, c), (c, d), (d, a)] {
                m.tri(x, y, up);
                m.tri(y, x, dn);
            }
        }
    }
    m
}
