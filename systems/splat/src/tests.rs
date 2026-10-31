use super::*;

fn cloud_of(pts: &[[f32; 3]]) -> Cloud {
    let raw = Raw {
        pos: pts.to_vec(),
        scale: vec![[0.1; 3]; pts.len()],
        rot: vec![[1.0, 0.0, 0.0, 0.0]; pts.len()],
        opacity: vec![1.0; pts.len()],
        color: vec![[0.5; 3]; pts.len()],
        deg: 0,
        rest: Vec::new(),
    };
    place(raw, Mat4::IDENTITY)
}

#[test]
fn half_floats_round_as_wgsl_reads_them() {
    assert_eq!(f16_bits(1.0), 0x3c00);
    assert_eq!(f16_bits(-2.0), 0xc000);
    assert_eq!(f16_bits(0.0), 0);
    assert_eq!(f16_bits(65504.0), 0x7bff);
    assert_eq!(f16_bits(1e6), 0x7c00);
    // subnormal: 2^-24
    assert_eq!(f16_bits(5.960_464_5e-8), 1);
}

#[test]
fn the_sort_is_far_to_near_and_stable() {
    let c = cloud_of(&[[0.0, 0.0, -1.0], [0.0, 0.0, -5.0], [1.0, 0.0, -5.0], [0.0, 0.0, -3.0]]);
    let p = params(&c, &BTreeMap::new(), 0.0);
    let (mut out, mut keys) = (Vec::new(), Vec::new());
    sort(&c, None, &p, Mat4::IDENTITY, Vec3::ZERO, Vec3::NEG_Z, &mut out, &mut keys);
    // 1 and 2 tie at depth 5: index order
    assert_eq!(out, vec![1, 2, 3, 0]);
    let again = out.clone();
    sort(&c, None, &p, Mat4::IDENTITY, Vec3::ZERO, Vec3::NEG_Z, &mut out, &mut keys);
    assert_eq!(out, again);
}

#[test]
fn effects_are_a_function_of_the_row() {
    let c = cloud_of(&[[0.3, 0.2, 0.1], [-0.4, 0.5, 0.2], [0.1, -0.3, 0.6]]);
    let mut v = BTreeMap::new();
    v.insert("disperse".to_string(), V::N(0.6));
    v.insert("explode".to_string(), V::N(0.3));
    v.insert("twist".to_string(), V::N(0.5));
    v.insert("seed".to_string(), V::N(7.0));
    let p = params(&c, &v, 0.0);
    let a: Vec<Vec3> = (0..3).map(|i| anim_center(&p, i, c.pos(i as usize), None)).collect();
    let b: Vec<Vec3> = (0..3).map(|i| anim_center(&p, i, c.pos(i as usize), None)).collect();
    assert_eq!(a, b);
    assert!(a.iter().zip(0..3).all(|(p, i)| (*p - c.pos(i)).length() > 1e-3));
}

#[test]
fn morph_matches_every_source_splat() {
    let a = cloud_of(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0]]);
    let b = cloud_of(&[[5.0, 5.0, 5.0], [6.0, 6.0, 6.0]]);
    let m = matched(&a, &b, 0.0);
    assert_eq!(m.len(), 4);
    // fitted to the source: about its centre, at its size
    assert!(m.iter().all(|g| g[0] > -1.5 && g[0] < 2.5));
}

#[test]
fn hilbert_neighbours_are_close() {
    // consecutive cells along the curve are face neighbours
    let mut cells: Vec<(u32, [i32; 3])> = Vec::new();
    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                let p = Vec3::new(x as f32, y as f32, z as f32) * (128.0 / 1023.0) + Vec3::splat(0.0001);
                cells.push((hilbert(p), [x, y, z]));
            }
        }
    }
    cells.sort();
    let far = cells.windows(2).filter(|w| (0..3).map(|k| (w[0].1[k] - w[1].1[k]).abs()).sum::<i32>() > 1).count();
    assert_eq!(far, 0, "the curve jumps {far} times");
}
