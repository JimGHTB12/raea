//! raea's shapes for a browser (docs/r3f.feature R3F5): a part's `data-shape` (a CSG expression of core/shape's
//! language: box, sphere, minus, hull, extrude, text3d ...) meshed by core/shape itself, so three.js draws the triangles
//! raea's engine draws. The answer is one Float32Array: the vertex count, then x, y, z of each vertex (three a
//! triangle), then each vertex's normal.
use wasm_bindgen::prelude::*;

/// The mesh of a shape expression: [n, positions (3n), normals (3n)]
#[wasm_bindgen]
pub fn mesh(expr: &str) -> Result<Vec<f32>, JsValue> {
    let t = raea_core_shape::cached(expr).map_err(|e| JsValue::from_str(&e))?;
    Ok(pack(&t.positions, &t.normals))
}

fn pack(positions: &[[f32; 3]], normals: &[[f32; 3]]) -> Vec<f32> {
    let mut out = Vec::with_capacity(1 + 6 * positions.len());
    out.push(positions.len() as f32);
    out.extend(positions.iter().flatten());
    out.extend(normals.iter().flatten());
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_box_is_twelve_triangles_with_its_normals_after_its_positions() {
        let t = raea_core_shape::cached("box(1, 2, 3)").unwrap();
        let out = super::pack(&t.positions, &t.normals);
        assert_eq!(out[0] as usize, 36);
        assert_eq!(out.len(), 1 + 36 * 6);
        let ys: Vec<f32> = out[1..1 + 108].chunks(3).map(|p| p[1]).collect();
        assert!(ys.iter().all(|y| (y.abs() - 1.0).abs() < 1e-5), "{ys:?}");
    }
}
