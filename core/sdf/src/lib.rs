//! Organic solids as signed distance fields (docs/handoff/solids.md, "Growth for organic shapes"; the 11-06 research:
//! a plain union of spheres and cylinders tops out at tubes-and-balls, smooth unions are what make a creature, a
//! limb or a branch read as one thing). A field is a tree of primitives (sphere, box, round cone, tube, torus,
//! ellipsoid), blends (union, smooth union, subtract, intersect, each smooth with a reach k), and modifiers (move,
//! rotate, scale, mirror, round, shell, displace by fractal noise). `mesh` samples it on a grid and makes a closed
//! triangle mesh by marching tetrahedra, which raea-core-shape turns into a Manifold solid (`sdf("...", step)`), so a
//! blended creature can still be cut, unioned and measured like any solid.
//!
//! Frame: the scene's (y up), metres; a cylinder stands along y and a torus lies flat in x-z.

mod many;
mod nets;
mod node;
mod noise;
mod parse;

pub use node::{Node, Rot, V, dot, len, smin};
pub use noise::{fbm, noise};
pub use parse::parse;

/// How fine a field is meshed: its cell size (metres), from the size it spans when not given.
pub fn step_for(node: &Node, step: Option<f64>) -> f64 {
    let (lo, hi) = node.bounds();
    let span = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(hi[2] - lo[2]).max(1e-6);
    step.filter(|s| *s > 0.0).unwrap_or(span / 96.0)
}

/// The most cells a field may be meshed in (about 330 a side): beyond it the step is too fine for the size.
pub const MAX_CELLS: usize = 36_000_000;

/// A closed mesh of the field's surface: positions flat (x y z) and triangles by threes, wound outward.
pub fn mesh(node: &Node, step: Option<f64>) -> Result<(Vec<f32>, Vec<u32>), String> {
    let h = step_for(node, step);
    let (lo, hi) = node.bounds();
    let pad = 2.0 * h;
    let min = [lo[0] - pad, lo[1] - pad, lo[2] - pad];
    let n = [((hi[0] - lo[0] + 2.0 * pad) / h).ceil() as usize + 1, ((hi[1] - lo[1] + 2.0 * pad) / h).ceil() as usize + 1, ((hi[2] - lo[2] + 2.0 * pad) / h).ceil() as usize + 1];
    let cells = n[0] * n[1] * n[2];
    if cells > MAX_CELLS {
        return Err(format!("sdf: a step of {h} m over {:.2} x {:.2} x {:.2} m is {cells} cells (most {MAX_CELLS}): use a coarser step, or mesh the parts apart", hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]));
    }
    let g = nets::Grid { min, h, n };
    // a union of many parts reads only the parts near each sample
    let band = 4.0 * h;
    let many = match node {
        Node::Union(c) if c.len() > 24 => Some(many::Many::new(c.clone(), 0.0, band)),
        Node::SmoothUnion(k, c) if c.len() > 24 => Some(many::Many::new(c.clone(), *k, band)),
        _ => None,
    };
    let v = match &many {
        Some(m) => nets::sample(&g, &|p| m.d(p)),
        None => nets::sample(&g, &|p| node.d(p)),
    };
    let (verts, tris) = nets::mesh(&g, &v);
    if tris.is_empty() {
        return Err("sdf: the field has no surface (everything outside, or a step coarser than its parts)".into());
    }
    Ok((verts, tris))
}

/// `parse` then `mesh`.
pub fn mesh_text(text: &str, step: Option<f64>) -> Result<(Vec<f32>, Vec<u32>), String> {
    mesh(&parse(text)?, step)
}

#[cfg(test)]
mod tests;
