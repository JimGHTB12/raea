use super::*;

/// A closed mesh's volume (the divergence theorem over its triangles) and its open edges.
fn volume_and_open(v: &[f32], t: &[u32]) -> (f64, usize) {
    let p = |i: u32| [v[i as usize * 3] as f64, v[i as usize * 3 + 1] as f64, v[i as usize * 3 + 2] as f64];
    let mut vol = 0.0;
    let mut edges = std::collections::HashMap::new();
    for c in t.chunks(3) {
        let (a, b, d) = (p(c[0]), p(c[1]), p(c[2]));
        vol += (a[0] * (b[1] * d[2] - b[2] * d[1]) - a[1] * (b[0] * d[2] - b[2] * d[0]) + a[2] * (b[0] * d[1] - b[1] * d[0])) / 6.0;
        // directed: each halfedge once, and its reverse once (consistent winding, as Manifold checks)
        for k in 0..3 {
            let (x, y) = (c[k], c[(k + 1) % 3]);
            *edges.entry((x, y)).or_insert(0i32) += 1;
        }
    }
    let bad = edges.iter().filter(|((x, y), n)| **n != 1 || edges.get(&(*y, *x)) != Some(&1)).count();
    (vol, bad)
}

#[test]
fn a_sphere_is_closed_wound_outward_and_the_right_size() {
    let (v, t) = mesh_text("sphere(0.5)", Some(0.02)).unwrap();
    let (vol, open) = volume_and_open(&v, &t);
    let want = 4.0 / 3.0 * std::f64::consts::PI * 0.125;
    assert_eq!(open, 0);
    assert!((vol - want).abs() / want < 0.02, "{vol} vs {want}");
}

#[test]
fn smooth_union_fills_the_join_and_subtract_cuts() {
    let hard = mesh_text("union(sphere(0.3), move(sphere(0.3), 0.45, 0, 0))", Some(0.015)).unwrap();
    let soft = mesh_text("smooth_union(0.15, sphere(0.3), move(sphere(0.3), 0.45, 0, 0))", Some(0.015)).unwrap();
    let (vh, _) = volume_and_open(&hard.0, &hard.1);
    let (vs, open) = volume_and_open(&soft.0, &soft.1);
    assert_eq!(open, 0);
    // the blend adds material in the neck
    assert!(vs > vh * 1.02, "{vs} vs {vh}");
    let cut = mesh_text("subtract(box(1, 1, 1), cylinder(0.25, 2))", Some(0.02)).unwrap();
    let (vc, open) = volume_and_open(&cut.0, &cut.1);
    let want = 1.0 - std::f64::consts::PI * 0.0625;
    assert_eq!(open, 0);
    assert!((vc - want).abs() / want < 0.03, "{vc} vs {want}");
}

#[test]
fn round_cones_tubes_and_modifiers() {
    // a capsule 1 long, radius 0.1: a cylinder plus a sphere
    let (v, t) = mesh_text("capsule(0, 0, 0, 0, 1, 0, 0.1)", Some(0.01)).unwrap();
    let want = std::f64::consts::PI * 0.01 * 1.0 + 4.0 / 3.0 * std::f64::consts::PI * 0.001;
    let (vol, _) = volume_and_open(&v, &t);
    assert!((vol - want).abs() / want < 0.04, "{vol} vs {want}");
    // a bent tube through three points stays one closed piece
    let (v, t) = mesh_text("tube(0, 0, 0, 0.08, 0, 0.5, 0.1, 0.06, 0.3, 0.8, 0.1, 0.03)", Some(0.01)).unwrap();
    assert_eq!(volume_and_open(&v, &t).1, 0);
    // a shell is hollow: |d| - t is a skin 2t thick, 4 pi r^2 2t, far less than the full ball
    let (v, t) = mesh_text("shell(sphere(0.4), 0.02)", Some(0.01)).unwrap();
    let (vol, _) = volume_and_open(&v, &t);
    let skin = 4.0 * std::f64::consts::PI * 0.16 * 0.04;
    assert!((vol - skin).abs() / skin < 0.05, "{vol} vs {skin}");
    // mirror_x makes both sides; rotate keeps the volume; displace roughens but stays closed
    let (v, t) = mesh_text("mirror_x(move(sphere(0.2), 0.5, 0, 0))", Some(0.02)).unwrap();
    let (vm, _) = volume_and_open(&v, &t);
    assert!((vm - 2.0 * 4.0 / 3.0 * std::f64::consts::PI * 0.008).abs() < 0.004, "{vm}");
    let (v, t) = mesh_text("displace(sphere(0.5), 0.03, 6, 3, 7)", Some(0.02)).unwrap();
    assert_eq!(volume_and_open(&v, &t).1, 0);
}

#[test]
fn many_parts_are_quick_and_match_the_plain_union() {
    // 200 small spheres in a ring: the hashed union gives the same surface as the plain one
    let parts: Vec<String> = (0..200).map(|i| {
        let a = i as f64 / 200.0 * std::f64::consts::TAU;
        format!("move(sphere(0.03), {}, {}, 0)", a.cos(), a.sin())
    }).collect();
    let text = format!("smooth_union(0.02, {})", parts.join(", "));
    let t0 = std::time::Instant::now();
    let (v, t) = mesh_text(&text, Some(0.01)).unwrap();
    let (vol, open) = volume_and_open(&v, &t);
    assert_eq!(open, 0);
    assert!(vol > 0.0);
    assert!(t0.elapsed().as_secs_f64() < 30.0, "{:?}", t0.elapsed());
}

#[test]
fn bad_fields_say_why() {
    assert!(parse("blob(1)").unwrap_err().contains("no field blob"));
    assert!(parse("sphere(-1)").unwrap_err().contains("above 0"));
    assert!(parse("tube(0, 0, 0, 1)").unwrap_err().contains("2 or more points"));
    assert!(mesh_text("sphere(10)", Some(0.001)).unwrap_err().contains("cells"));
    // exponents read
    assert!(parse("sphere(1e-1)").is_ok());
}
