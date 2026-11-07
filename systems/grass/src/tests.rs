use super::*;
use serde_json::json;

fn field(density: f32) -> Spec {
    let n = 5 * 5;
    Spec::from_json(&json!({
        "area": [-4, -4, 4, 4], "grid": [5, 5],
        "height": (0..n).map(|i| (i % 5) as f32 * 0.25).collect::<Vec<_>>(),
        "density": vec![density; n],
        "patch": (0..n).map(|i| i % 2).collect::<Vec<_>>(),
        "tints": [["#2c4a1a", "#5f8a2c"], ["#4d7424", "#a3c04a"]],
        "seed": 7, "tile": 4,
    }))
    .unwrap()
}

fn verts(s: &Spec, lod: usize) -> usize {
    s.tile_mesh(0, 0, lod).map_or(0, |(m, _)| m.count_vertices())
}

#[test]
fn a_spec_reads_and_tiles_its_area() {
    let s = field(10.0);
    assert_eq!(s.tiles(), (2, 2));
    assert_eq!(s.lod.len(), 3);
    // the ground rises a quarter metre a column: x from -4 to 4 is 0 to 1
    assert!((s.ground(0.0, 0.0) - 0.5).abs() < 1e-5);
    assert!(s.normal(0.0, 0.0).x < 0.0, "the ground rises toward +x, so it faces -x");
}

#[test]
fn a_tile_is_the_same_each_time_and_scales_with_density() {
    let s = field(10.0);
    let a = s.tile_mesh(1, 0, 0).unwrap().0;
    let b = s.tile_mesh(1, 0, 0).unwrap().0;
    assert_eq!(a.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().get_bytes(), b.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().get_bytes());
    // 16 m2 at 10 a m2: about 160 clumps of 14 blades of 8 vertices
    let n = verts(&s, 0);
    assert!((14_000..22_000).contains(&n), "{n} vertices");
    assert!(field(0.0).tile_mesh(0, 0, 0).is_none(), "no clumps, no mesh");
}

#[test]
fn farther_steps_draw_fewer_blades() {
    let s = field(10.0);
    let (near, mid, far) = (verts(&s, 0), verts(&s, 1), verts(&s, 2));
    assert!(near > 4 * mid && mid > 2 * far && far > 0, "{near} {mid} {far}");
    assert_eq!(s.lod_at(1.0), 0);
    assert_eq!(s.lod_at(10.0), 1);
    assert_eq!(s.lod_at(500.0), 2);
}

#[test]
fn the_wind_reads_each_vertex_height_above_its_root() {
    let s = field(10.0);
    let m = s.tile_mesh(0, 0, 0).unwrap().0;
    let Some(bevy::mesh::VertexAttributeValues::Float32x2(uv1)) = m.attribute(Mesh::ATTRIBUTE_UV_1) else { panic!("no second uv") };
    assert!(uv1.iter().all(|u| u[0] >= 0.0 && u[0] < 0.6));
    assert!(uv1.iter().any(|u| u[0] == 0.0) && uv1.iter().any(|u| u[0] > 0.1));
}

#[test]
fn a_bad_spec_says_what_is_wrong() {
    let e = Spec::from_json(&json!({"area": [0, 0, 1, 1], "grid": [2, 2], "height": [0, 0, 0], "density": [1, 1, 1, 1]})).unwrap_err();
    assert!(e.contains("height has 3 values"), "{e}");
}
