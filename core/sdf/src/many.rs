//! A union of many parts (a tree's thousand branches, a bush's leaves) answered from the few near the point: their
//! boxes hashed into a grid, so a sample reads only the parts whose reach covers its cell. Far from every part the
//! field is `far` (positive), which is all a mesher needs there.

use std::collections::HashMap;

use crate::node::{Node, V, smin};

pub struct Many {
    parts: Vec<Node>,
    k: f64,
    cell: f64,
    cells: HashMap<(i32, i32, i32), Vec<u32>>,
    far: f64,
}

impl Many {
    /// `band`: how far from the surface distances must be right (a few mesh cells).
    pub fn new(parts: Vec<Node>, k: f64, band: f64) -> Many {
        let reach = k + band;
        let boxes: Vec<(V, V)> = parts.iter().map(|p| p.bounds()).collect();
        // a cell about the size of a typical part, and never so small that one part fills thousands
        let mut sizes: Vec<f64> = boxes.iter().map(|(lo, hi)| (hi[0] - lo[0]).max(hi[1] - lo[1]).max(hi[2] - lo[2]) + 2.0 * reach).collect();
        sizes.sort_by(f64::total_cmp);
        let cell = sizes.get(sizes.len() / 2).copied().unwrap_or(1.0).max(1e-6);
        let mut cells: HashMap<(i32, i32, i32), Vec<u32>> = HashMap::new();
        for (i, (lo, hi)) in boxes.iter().enumerate() {
            let a = [((lo[0] - reach) / cell).floor() as i32, ((lo[1] - reach) / cell).floor() as i32, ((lo[2] - reach) / cell).floor() as i32];
            let b = [((hi[0] + reach) / cell).floor() as i32, ((hi[1] + reach) / cell).floor() as i32, ((hi[2] + reach) / cell).floor() as i32];
            for x in a[0]..=b[0] {
                for y in a[1]..=b[1] {
                    for z in a[2]..=b[2] {
                        cells.entry((x, y, z)).or_default().push(i as u32);
                    }
                }
            }
        }
        Many { parts, k, cell, cells, far: reach }
    }

    pub fn d(&self, p: V) -> f64 {
        let key = ((p[0] / self.cell).floor() as i32, (p[1] / self.cell).floor() as i32, (p[2] / self.cell).floor() as i32);
        let Some(near) = self.cells.get(&key) else { return self.far };
        let mut d = self.far;
        for &i in near {
            d = smin(d, self.parts[i as usize].d(p), self.k);
        }
        d
    }
}
