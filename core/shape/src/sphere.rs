//! A shape expression that is a sphere alone, which a renderer draws as its own sphere mesh (raea-render: Bevy's
//! Sphere::mesh(), as a Bevy Mesh3d's is) rather than as Manifold's; a cuboid alone likewise (Bevy's Cuboid mesh).

/// A sphere alone, as its expression asks for it
#[derive(Debug, PartialEq)]
pub enum SphereOf {
    /// `sphere(r)` (5 subdivisions, Bevy's default) or `sphere(r, n)`
    Ico { radius: f32, subdivisions: u32 },
    /// `uv_sphere(r, sectors, stacks)`
    Uv { radius: f32, sectors: u32, stacks: u32 },
}

/// The sphere `expr` is, when it is one alone; None for anything else (moved, cut, any other shape)
pub fn sphere_of(expr: &str) -> Option<SphereOf> {
    let e = expr.trim();
    let (uv, inner) = match e.strip_prefix("uv_sphere(") {
        Some(rest) => (true, rest),
        None => (false, e.strip_prefix("sphere(")?),
    };
    let inner = inner.strip_suffix(')').filter(|i| !i.contains('('))?;
    let n: Vec<f32> = inner.split(',').map(|s| s.trim().parse().ok()).collect::<Option<_>>()?;
    match (uv, n.as_slice()) {
        (false, &[radius]) => Some(SphereOf::Ico { radius, subdivisions: 5 }),
        (false, &[radius, s]) => Some(SphereOf::Ico { radius, subdivisions: s as u32 }),
        (true, &[radius, a, b]) => Some(SphereOf::Uv { radius, sectors: a as u32, stacks: b as u32 }),
        _ => None,
    }
}

/// The plane `expr` is, `plane(nx, ny, nz, w, h)` alone: its normal and full size, drawn as Bevy's Plane3d mesh (one
/// face, facing the normal, as PlaneMeshBuilder lays it out)
pub fn plane_of(expr: &str) -> Option<([f32; 3], [f32; 2])> {
    let inner = expr.trim().strip_prefix("plane(")?.strip_suffix(')')?;
    let n: Vec<f32> = inner.split(',').map(|s| s.trim().parse().ok()).collect::<Option<_>>()?;
    match n.as_slice() {
        &[x, y, z, w, h] => Some(([x, y, z], [w, h])),
        _ => None,
    }
}

/// The cuboid `expr` is, `box(w, h, d)` alone with each side above 0 (a 0 side is a plane: Manifold's), as its full
/// size: drawn as Bevy's Cuboid mesh, with its normals and texture coordinates, as a Bevy Mesh3d's is
pub fn cuboid_of(expr: &str) -> Option<[f32; 3]> {
    let inner = expr.trim().strip_prefix("box(")?.strip_suffix(')')?;
    let n: Vec<f32> = inner.split(',').map(|s| s.trim().parse().ok()).collect::<Option<_>>()?;
    match n.as_slice() {
        &[w, h, d] if w > 0.0 && h > 0.0 && d > 0.0 => Some([w, h, d]),
        _ => None,
    }
}

/// The cylinder `expr` is, `<Cylinder>`'s `rotate(cone(r, r, h), -90, 0, 0)` alone (both radii equal, above 0): its
/// radius and full height, drawn as Bevy's Cylinder mesh (32 sides, one segment), with its normals and texture
/// coordinates, as a Bevy Mesh3d's is
pub fn cylinder_of(expr: &str) -> Option<(f32, f32)> {
    let inner = expr.trim().strip_prefix("rotate(cone(")?.strip_suffix("), -90, 0, 0)")?;
    let n: Vec<f32> = inner.split(',').map(|s| s.trim().parse().ok()).collect::<Option<_>>()?;
    match n.as_slice() {
        &[a, b, h] if a == b && a > 0.0 && h > 0.0 => Some((a, h)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cylinder_alone_is_read_and_a_cone_or_anything_more_is_not() {
        assert_eq!(cylinder_of("rotate(cone(0.5, 0.5, 1), -90, 0, 0)"), Some((0.5, 1.0)));
        for e in ["rotate(cone(0.5, 0.2, 1), -90, 0, 0)", "cone(0.5, 0.5, 1)", "move(rotate(cone(0.5, 0.5, 1), -90, 0, 0), 0, 1, 0)"] {
            assert_eq!(cylinder_of(e), None, "{e}");
        }
    }

    #[test]
    fn a_box_alone_is_a_cuboid_and_a_plane_or_anything_more_is_not() {
        assert_eq!(cuboid_of(" box(1, 2, 0.5) "), Some([1.0, 2.0, 0.5]));
        for e in ["box(1, 1, 0)", "box(1, 1)", "box(1, 1, 1, 0.1)", "move(box(1, 1, 1), 0, 1, 0)", "sphere(1)"] {
            assert_eq!(cuboid_of(e), None, "{e}");
        }
    }

    #[test]
    fn a_plane_alone_is_read_with_its_normal_and_its_solid_is_the_flat_box() {
        assert_eq!(plane_of(" plane(0, -1, 0, 4, 6) "), Some(([0.0, -1.0, 0.0], [4.0, 6.0])));
        for e in ["plane(0, 1, 0, 4)", "box(4, 0, 6)", "move(plane(0, 1, 0, 1, 1), 0, 1, 0)"] {
            assert_eq!(plane_of(e), None, "{e}");
        }
        // laid out as raea/mesh's <Plane3d>: facing y it spans x and z, facing x it spans y and z, facing z x and y
        for (e, size) in [("plane(0, 1, 0, 4, 6)", (4.0, 0.0, 6.0)), ("plane(-1, 0, 0, 4, 6)", (0.0, 4.0, 6.0)), ("plane(0, 0, 1, 4, 6)", (4.0, 6.0, 0.0))] {
            let b = crate::solid(e).unwrap().bounding_box();
            let got = (b.max.x - b.min.x, b.max.y - b.min.y, b.max.z - b.min.z);
            assert!((got.0 - size.0).abs() < 1e-6 && (got.1 - size.1).abs() < 1e-6 && (got.2 - size.2).abs() < 1e-6, "{e}: {got:?}");
        }
    }

    #[test]
    fn a_sphere_alone_is_read_and_anything_more_is_not() {
        assert_eq!(sphere_of("sphere(2)"), Some(SphereOf::Ico { radius: 2.0, subdivisions: 5 }));
        assert_eq!(sphere_of(" sphere(50, 9) "), Some(SphereOf::Ico { radius: 50.0, subdivisions: 9 }));
        assert_eq!(sphere_of("uv_sphere(1, 32, 18)"), Some(SphereOf::Uv { radius: 1.0, sectors: 32, stacks: 18 }));
        for e in ["move(sphere(1), 0, 1, 0)", "minus(sphere(1), box(1, 1, 1))", "box(1, 1, 1)", "uv_sphere(1, 32)", "sphere()"] {
            assert_eq!(sphere_of(e), None, "{e}");
        }
    }
}
