use super::*;

/// The mesh's volume (divergence theorem), its bad halfedges (unpaired or doubled), and its height range.
fn measure(m: &Mesh) -> (f64, usize, f32, f32) {
    let p = |i: u32| [m.v[i as usize * 3] as f64, m.v[i as usize * 3 + 1] as f64, m.v[i as usize * 3 + 2] as f64];
    let mut vol = 0.0;
    let mut edges = std::collections::HashMap::new();
    for c in m.t.chunks(3) {
        let (a, b, d) = (p(c[0]), p(c[1]), p(c[2]));
        vol += (a[0] * (b[1] * d[2] - b[2] * d[1]) - a[1] * (b[0] * d[2] - b[2] * d[0]) + a[2] * (b[0] * d[1] - b[1] * d[0])) / 6.0;
        for k in 0..3 {
            let (x, y) = (c[k], c[(k + 1) % 3]);
            *edges.entry((x, y)).or_insert(0i32) += 1;
        }
    }
    // directed: each halfedge once and its reverse once, so the winding is consistent (as Manifold checks)
    let bad = edges.iter().filter(|((x, y), n)| **n != 1 || edges.get(&(*y, *x)) != Some(&1)).count();
    let ys = m.v.chunks(3).map(|q| q[1]);
    let (lo, hi) = ys.fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
    (vol, bad, lo, hi)
}

#[test]
fn a_loft_is_closed_outward_and_its_volume_is_right() {
    let m = loft("0: circle 0.5; 1: circle 0.5", 64).unwrap();
    let (vol, open, ..) = measure(&m);
    assert_eq!(open, 0);
    let want = std::f64::consts::PI * 0.25;
    assert!((vol - want).abs() / want < 0.01, "{vol} vs {want}");
    // a square base to a round top, through a waist; any winding of a point outline
    let m = loft("0: rect 1 1; 0.5: circle 0.25; 1: 0.4 0, 0 0.4, -0.4 0, 0 -0.4", 0).unwrap();
    let (vol, open, lo, hi) = measure(&m);
    assert_eq!(open, 0);
    assert!(vol > 0.1 && vol < 1.0, "{vol}");
    assert_eq!((lo, hi), (0.0, 1.0));
    assert!(loft("1: circle 1; 0: circle 1", 0).unwrap_err().contains("go up"));
}

#[test]
fn terrain_is_a_closed_slab_and_flat_terrain_is_a_box() {
    let m = terrain("size=4 depth=2 height=0 base=0.5 cells=8").unwrap();
    let (vol, open, ..) = measure(&m);
    assert_eq!(open, 0);
    assert!((vol - 4.0).abs() < 1e-4, "{vol}");
    let m = terrain("size=10 height=2 cells=64 seed=3 flat=0.3").unwrap();
    let (vol, open, _, hi) = measure(&m);
    assert_eq!(open, 0);
    assert!(vol > 50.0 && hi <= 2.0, "{vol} {hi}");
    let (v0, _, _, h0) = measure(&terrain("size=10 height=2 cells=64 seed=4").unwrap());
    assert!((v0 - vol).abs() > 1e-3 || h0 != hi, "different seeds, different ground");
}

#[test]
fn a_boulder_rests_on_the_ground_and_is_one_closed_piece() {
    let m = boulder("size=1 seed=2").unwrap();
    let (vol, open, lo, hi) = measure(&m);
    assert_eq!(open, 0);
    assert_eq!(lo, 0.0);
    assert!(hi > 0.4 && hi < 1.2, "{hi}");
    assert!(vol > 0.1 && vol < 0.6, "{vol}");
}

#[test]
fn a_rosette_is_closed_and_wider_than_tall() {
    let m = rosette("leaves=13 length=0.3").unwrap();
    let (vol, open, lo, hi) = measure(&m);
    assert_eq!(open, 0);
    assert!(vol > 0.0);
    let xs: Vec<f32> = m.v.chunks(3).map(|q| q[0]).collect();
    let span = xs.iter().cloned().fold(f32::MIN, f32::max) - xs.iter().cloned().fold(f32::MAX, f32::min);
    assert!(span > hi - lo, "{span} vs {}", hi - lo);
}

#[test]
fn a_tree_grows_to_its_height_thins_outward_and_is_the_same_twice() {
    let text = "height=5 points=600 seed=3";
    let t = tree::Tree::parse(text).unwrap();
    let s = tree::grow(&t);
    let tips = s.children.iter().filter(|c| c.is_empty()).count();
    assert!(tips > 40, "{tips} tips");
    // the pipe model: the trunk is the thickest, many times a twig
    let r0 = s.radius[0];
    assert!(s.radius.iter().all(|r| *r <= r0 + 1e-12));
    assert!(r0 > t.tip * 8.0, "{r0}");
    let wood = tree(text, "wood").unwrap();
    let (vol, open, lo, hi) = measure(&wood);
    assert_eq!(open, 0);
    assert!(vol > 0.0);
    assert!(lo > -0.2 && hi > 3.5 && hi < 5.6, "{lo} {hi}");
    let leaves = tree(text, "leaves").unwrap();
    assert_eq!(measure(&leaves).1, 0);
    assert_eq!(tree(text, "all").unwrap().v, tree(text, "all").unwrap().v);
    assert!(tree::Tree::parse("crown=blob").unwrap_err().contains("isn't one of"));
    assert!(tree::Tree::parse("hight=3").unwrap_err().contains("no setting"));
}
