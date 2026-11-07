//! Generators for what is too numerous or too organic to place by hand (the 11-06 research: what Blender users get
//! from Geometry Nodes and Sapling, and what an agent can't fake with a few primitives): trees grown by space
//! colonization, lofts through outlines, terrain slabs, boulders, and phyllotaxis rosettes. Each returns closed
//! triangle pieces, which raea-core-shape turns into a solid (`tree("...")`, `loft("...")`, `terrain("...")`,
//! `boulder("...")`, `rosette("...")`), so the result can be measured, cut and placed like any other solid.
//!
//! Frame: the scene's (y up), metres; everything stands on y 0 at the origin.

pub mod loft;
pub mod mesh;
pub mod rosette;
pub mod terrain;
pub mod tree;
pub mod tube;

pub use mesh::Mesh;

/// A tree from its settings text; `part` is "wood", "leaves" or "all". The wood is one tube a branch (closed pieces
/// overlapping at the forks), the leaves thin closed cards: `tree_pieces` gives them apart, to be unioned into one
/// solid (raea-core-shape does), and this gives them as one mesh.
pub fn tree(text: &str, part: &str) -> Result<Mesh, String> {
    let mut m = Mesh::default();
    for p in tree_pieces(text, part)? {
        m.append(&p);
    }
    Ok(m)
}

/// A tree's closed pieces: a tube a branch, then a card a leaf.
pub fn tree_pieces(text: &str, part: &str) -> Result<Vec<Mesh>, String> {
    let t = tree::Tree::parse(text)?;
    let s = tree::grow(&t);
    let out = match part {
        "wood" => tree::wood(&t, &s),
        "leaves" => vec![tree::leaves(&t, &s)],
        "all" | "" => [tree::wood(&t, &s), vec![tree::leaves(&t, &s)]].concat(),
        o => return Err(format!("tree: part {o:?} isn't wood, leaves or all")),
    };
    if out.iter().all(|m| m.t.is_empty()) {
        return Err("tree: nothing grew (a bare tree's leaves, or too few points)".into());
    }
    Ok(out)
}

/// A loft from its sections text (see `loft`), `n` points around (0: chosen).
pub fn loft(text: &str, n: usize) -> Result<Mesh, String> {
    Ok(loft::loft(&loft::parse(text)?, n))
}

pub use rosette::rosette;
pub use terrain::{boulder, terrain};

#[cfg(test)]
mod tests;
