//! glam's (Bevy 0.20's) answers for the TS Transform/Quat helpers' inputs (docs/control.feature FC1d), printed as JSON
//! for sdk/raea/conformance/glam.json: `cargo test -p raea-core-scene --test glam_fixture -- --ignored --nocapture`.

use bevy::math::{EulerRot, Quat, Vec3};
use bevy::transform::components::Transform;

fn q(q: Quat) -> String {
    format!("[{},{},{},{}]", q.x, q.y, q.z, q.w)
}

#[test]
#[ignore]
fn print_fixture() {
    let mut out = Vec::new();
    for a in [0.0f32, 0.5, -1.25, std::f32::consts::FRAC_PI_2, 3.0] {
        out.push(format!("{{\"op\":\"x\",\"a\":{a},\"q\":{}}}", q(Quat::from_rotation_x(a))));
        out.push(format!("{{\"op\":\"y\",\"a\":{a},\"q\":{}}}", q(Quat::from_rotation_y(a))));
        out.push(format!("{{\"op\":\"z\",\"a\":{a},\"q\":{}}}", q(Quat::from_rotation_z(a))));
    }
    let n = Vec3::new(1.0, 2.0, 3.0).normalize();
    out.push(format!("{{\"op\":\"axis\",\"axis\":[{},{},{}],\"a\":1,\"q\":{}}}", n.x, n.y, n.z, q(Quat::from_axis_angle(n, 1.0))));
    let orders = [("XYZ", EulerRot::XYZ), ("XZY", EulerRot::XZY), ("YXZ", EulerRot::YXZ), ("YZX", EulerRot::YZX), ("ZXY", EulerRot::ZXY), ("ZYX", EulerRot::ZYX)];
    for (name, o) in orders {
        for (a, b, c) in [(0.3f32, -0.7, 1.1), (0.5, std::f32::consts::FRAC_PI_2, 0.3)] {
            out.push(format!("{{\"op\":\"euler\",\"order\":\"{name}\",\"abc\":[{a},{b},{c}],\"q\":{}}}", q(Quat::from_euler(o, a, b, c))));
        }
    }
    let (p, r) = (Quat::from_rotation_y(0.4), Quat::from_rotation_x(-0.9));
    out.push(format!("{{\"op\":\"mul\",\"a\":{},\"b\":{},\"q\":{}}}", q(p), q(r), q(p * r)));
    let v = p * Vec3::new(1.0, 2.0, 3.0);
    out.push(format!("{{\"op\":\"mulVec3\",\"q\":{},\"v\":[1,2,3],\"out\":[{},{},{}]}}", q(p), v.x, v.y, v.z));
    for (eye, target, up) in [([0.0f32, 2.0, 7.0], [0.0f32, 0.5, 0.0], [0.0f32, 1.0, 0.0]), ([3.0, -1.0, 2.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]), ([0.0, 5.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])] {
        let t = Transform::from_translation(Vec3::from(eye)).looking_at(Vec3::from(target), Vec3::from(up));
        out.push(format!("{{\"op\":\"lookingAt\",\"eye\":{eye:?},\"target\":{target:?},\"up\":{up:?},\"q\":{}}}", q(t.rotation)));
    }
    println!("GLAM_FIXTURE [{}]", out.join(","));
}
