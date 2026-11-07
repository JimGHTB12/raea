//! A shape expression that is a sphere alone, which a renderer draws as its own sphere mesh (raea-render: Bevy's
//! Sphere::mesh(), as a Bevy Mesh3d's is) rather than as Manifold's.

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

#[cfg(test)]
mod tests {
    use super::*;

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
