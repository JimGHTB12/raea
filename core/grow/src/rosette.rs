//! A rosette: leaves set around a centre at the golden angle (137.5°, phyllotaxis, as an agave, a succulent, a
//! lettuce, a pine cone's scales or a flower's petals), the inner ones smaller and more upright, smooth-unioned in the
//! distance field so the leaves grow out of one heart instead of sitting on it.

use raea_core_sdf::{Node, Rot};

use crate::mesh::Mesh;

/// `leaves=21 length=0.3 width=0.09 thick=0.02 tilt=25 rise=50 blend=0.3 step=0`: `leaves` leaves of up to `length` m,
/// tilted `tilt` degrees above level at the outside rising to `rise` at the centre; `blend` is the smooth union's reach
/// as a share of the width.
pub fn rosette(text: &str) -> Result<Mesh, String> {
    let (mut n, mut l, mut w, mut th, mut tilt, mut rise, mut blend, mut step) = (21.0, 0.3, 0.09, 0.02, 25.0, 50.0, 0.3, 0.0);
    for kv in text.split_whitespace() {
        let (k, v) = kv.split_once('=').ok_or_else(|| format!("rosette: {kv:?} isn't name=value"))?;
        let v: f64 = v.parse().map_err(|_| format!("rosette: {k} takes a number, got {v:?}"))?;
        match k {
            "leaves" => n = v,
            "length" => l = v,
            "width" => w = v,
            "thick" => th = v,
            "tilt" => tilt = v,
            "rise" => rise = v,
            "blend" => blend = v,
            "step" => step = v,
            o => return Err(format!("rosette: no setting {o}: leaves, length, width, thick, tilt, rise, blend, step")),
        }
    }
    let n = (n.round() as usize).clamp(1, 400);
    if l <= 0.0 || w <= 0.0 || th <= 0.0 {
        return Err("rosette: length, width and thick above 0".into());
    }
    let golden = 180.0 * (3.0 - 5f64.sqrt());
    let leaves: Vec<Node> = (0..n).map(|i| {
        // the oldest (outermost) leaf is i = n - 1
        let age = (i as f64 + 1.0) / n as f64;
        let s = 0.35 + 0.65 * age.sqrt();
        let up = rise + (tilt - rise) * age;
        let (ll, ww) = (l * s, w * s.sqrt());
        let a = up.to_radians();
        let leaf = Node::Rotate(Box::new(Node::Ellipsoid([ll / 2.0, th / 2.0, ww / 2.0])), Rot::degrees(0.0, 0.0, up));
        let out = Node::Move(Box::new(leaf), [ll / 2.0 * a.cos(), ll / 2.0 * a.sin(), 0.0]);
        Node::Rotate(Box::new(out), Rot::degrees(0.0, i as f64 * golden, 0.0))
    }).collect();
    let node = Node::SmoothUnion(w * blend, leaves);
    let step = if step > 0.0 { step } else { (th / 3.0).max(l / 200.0) };
    let (v, t) = raea_core_sdf::mesh(&node, Some(step))?;
    Ok(Mesh { v, t })
}
