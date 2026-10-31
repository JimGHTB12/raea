//! What the camera sees: its matrix, and whether a seam is in frame and not hidden behind any part.

use super::*;

/// The frame's camera as a world-to-view matrix, its eye, its vertical focal scale, and the fog's far edge.
pub(crate) fn camera(its: &[Item]) -> Option<(Mat4, Vec3, f32, f32)> {
    let c = its.iter().find(|i| i.kind == "camera")?;
    let p = vec3(&c.props, "position", [0.0, 0.0, 5.0]);
    let l = vec3(&c.props, "lookAt", [0.0, 0.0, 0.0]);
    let fov = c.props.get("fov").and_then(Value::as_f64).unwrap_or(50.0) as f32;
    let eye = Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32);
    let far = its.iter().find(|i| i.kind == "fog").and_then(|f| f.props.get("far").and_then(Value::as_f64)).unwrap_or(f64::MAX) as f32;
    Some((look_at_rh(eye, Vec3::new(l[0] as f32, l[1] as f32, l[2] as f32), Vec3::Y), eye, 1.0 / (fov.to_radians() * 0.5).tan(), far))
}

/// The view matrix of an eye looking at `center`, right-handed: glam's `Mat4::look_at_rh`, deprecated in glam 0.33
/// for `glam::camera::rh::view::look_at_mat4`, which bevy::math doesn't re-export. The same arithmetic, here.
pub(crate) fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    let f = (center - eye).normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);
    Mat4::from_cols(
        bevy::math::Vec4::new(s.x, u.x, -f.x, 0.0),
        bevy::math::Vec4::new(s.y, u.y, -f.y, 0.0),
        bevy::math::Vec4::new(s.z, u.z, -f.z, 0.0),
        bevy::math::Vec4::new(-eye.dot(s), -eye.dot(u), eye.dot(f), 1.0),
    )
}

/// Whether a ray from `p` to the eye passes through a solid (the first `skip` metres left out: the seam's own
/// surfaces).
pub(crate) fn blocked(p: Vec3, eye: Vec3, skip: f32, solids: &[(&Solid, Mat4)]) -> bool {
    let len = (eye - p).length();
    if len <= skip {
        return false;
    }
    let dir = (eye - p) / len;
    let from = p + dir * skip;
    for (s, w) in solids {
        let inv = w.inverse();
        let (a, b) = (inv.transform_point3(from), inv.transform_point3(eye));
        let d = b - a;
        // the segment against the solid's box, in its own space
        let (mut t0, mut t1) = (0f32, 1f32);
        for k in 0..3 {
            if d[k].abs() < 1e-9 {
                if a[k] < s.lo[k] || a[k] > s.hi[k] {
                    t1 = -1.0;
                }
                continue;
            }
            let (u, v) = ((s.lo[k] - a[k]) / d[k], (s.hi[k] - a[k]) / d[k]);
            t0 = t0.max(u.min(v));
            t1 = t1.min(u.max(v));
        }
        if t0 > t1 {
            continue;
        }
        let steps = ((t1 - t0) * d.length() / (0.7 * s.cell)).ceil().max(1.0) as usize;
        for k in 0..=steps {
            let q = a + d * (t0 + (t1 - t0) * k as f32 / steps as f32);
            if s.at(q) >= 3 {
                return true;
            }
        }
    }
    false
}

/// Whether enough of a seam shows on this frame: inside the frame, in front, nearer than the fog's far edge, not hidden,
/// and at least a few pixels across.
pub(crate) fn shows(its: &[Item], seam: &[Vec3], solids: &[(&Solid, Mat4)], skip: f32, w: f32, h: f32) -> bool {
    let Some((view, eye, f, far)) = camera(its) else { return true };
    let (mut lo, mut hi, mut seen) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN), 0);
    for p in seam {
        let q = view.transform_point3(*p);
        if q.z >= -0.05 || -q.z >= far {
            continue;
        }
        let s = Vec3::new(q.x / -q.z * f * h / w, q.y / -q.z * f, 0.0);
        if s.x.abs() > 1.0 || s.y.abs() > 1.0 || blocked(*p, eye, skip, solids) {
            continue;
        }
        seen += 1;
        lo = lo.min(s);
        hi = hi.max(s);
    }
    seen >= 3.min(seam.len()).max(1) && ((hi.x - lo.x) * w * 0.5).max((hi.y - lo.y) * h * 0.5) >= 6.0
}
