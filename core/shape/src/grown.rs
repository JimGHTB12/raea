//! Shapes from the field and the generators (raea-core-sdf, raea-core-grow), as solids:
//!
//!   sdf("smooth_union(0.05, sphere(0.3), move(capsule(0,0,0, 0,0.6,0, 0.1), 0, 0.2, 0))" [, step])
//!                                    a distance field (blended organic shapes), meshed at `step` metres
//!   tree("height=6 crown=dome" [, "wood"|"leaves"|"all"])   a tree grown by space colonization
//!   loft("0: circle 0.3; 0.4: circle 0.18; 1: rect 0.2 0.2" [, n])   outlines at heights, skinned
//!   terrain("size=20 height=2 seed=3")  boulder("size=1 seed=2")  rosette("leaves=21 length=0.3")
//!
//! The settings are one string of name=value words (the strings take no escapes, so they hold no quotes).

use manifold_rust::manifold::Manifold;
use manifold_rust::types::OpType;

use crate::mesh::from_mesh;
use crate::parse::Arg;

fn strs(a: &[Arg]) -> Vec<&str> {
    a.iter().filter_map(|x| if let Arg::Str(s) = x { Some(s.as_str()) } else { None }).collect()
}

fn num(a: &[Arg]) -> Option<f64> {
    a.iter().find_map(|x| if let Arg::Num(n) = x { Some(*n) } else { None })
}

/// The call when it is one of these shapes, else None.
pub(crate) fn call(f: &str, a: &[Arg]) -> Option<Result<Manifold, String>> {
    if !matches!(f, "sdf" | "tree" | "loft" | "terrain" | "boulder" | "rosette") {
        return None;
    }
    let s = strs(a);
    let text = s.first().copied().unwrap_or("");
    Some((|| {
        let (v, t) = match f {
            "sdf" => {
                if text.is_empty() {
                    return Err("sdf(\"field\" [, step]): give the field, e.g. sdf(\"sphere(0.5)\")".to_string());
                }
                raea_core_sdf::mesh_text(text, num(a))?
            }
            "tree" => {
                // each branch its own solid, unioned at the forks into one body
                let pieces = raea_core_grow::tree_pieces(text, s.get(1).copied().unwrap_or("all"))?;
                let solids = pieces.iter().filter(|m| !m.t.is_empty()).map(piece).collect::<Result<Vec<_>, _>>().map_err(|e| format!("tree: {e}"))?;
                return Ok(Manifold::batch_boolean(&solids, OpType::Add));
            }
            "loft" => { let m = raea_core_grow::loft(text, num(a).unwrap_or(0.0).max(0.0) as usize)?; (m.v, m.t) }
            "terrain" => { let m = raea_core_grow::terrain(text)?; (m.v, m.t) }
            "boulder" => { let m = raea_core_grow::boulder(text)?; (m.v, m.t) }
            _ => { let m = raea_core_grow::rosette(text)?; (m.v, m.t) }
        };
        from_mesh(&v, &t).map_err(|e| format!("{f}: {e}"))
    })())
}

#[cfg(test)]
mod tests {
    use crate::solid;

    #[test]
    fn grown_shapes_are_solids_that_cut_and_measure() {
        let ball = solid("sdf(\"sphere(0.5)\", 0.02)").unwrap();
        let want = 4.0 / 3.0 * std::f64::consts::PI * 0.125;
        assert!((ball.volume() - want).abs() / want < 0.02, "{}", ball.volume());
        // a blended field is a solid like any other: cut a box out of it
        let cut = solid("minus(sdf(\"smooth_union(0.1, sphere(0.3), move(sphere(0.3), 0.4, 0, 0))\", 0.02), box(2, 2, 2))");
        assert!(cut.is_err(), "the box takes all of it");
        let vase = solid("loft(\"0: circle 0.2; 0.3: circle 0.3; 0.6: circle 0.1\")").unwrap();
        assert!(vase.volume() > 0.05);
        assert!(solid("terrain(\"size=4 height=0.5 cells=16\")").unwrap().volume() > 2.0);
        assert!(solid("boulder(\"size=1\")").unwrap().volume() > 0.1);
        assert!(solid("rosette(\"leaves=8\")").unwrap().volume() > 0.0);
        let wood = solid("tree(\"height=4 points=300\", \"wood\")").unwrap();
        assert!(wood.bounding_box().max.y > 2.5);
        // the branches are unioned at their forks: one body, not a tube each
        assert_eq!(wood.decompose().len(), 1);
        assert!(solid("sdf(\"blob(1)\")").err().unwrap().contains("no field blob"));
        assert!(solid("tree(\"crown=blob\")").err().unwrap().contains("isn't one of"));
    }
}

/// One generated piece as a solid.
fn piece(m: &raea_core_grow::Mesh) -> Result<Manifold, String> {
    from_mesh(&m.v, &m.t)
}
