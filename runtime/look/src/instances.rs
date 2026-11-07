//! <Instances> (sdk/render 3d/instances.tsx): a Mesh or Model whose props carry `at`, its placements 9 numbers a copy
//! (position, XYZ euler radians, scale), written as the row's `inst` column: 10 numbers a copy (position, rotation as
//! a quaternion xyzw, scale), space-separated. The engine draws one copy per placement, sharing the mesh and material
//! (runtime/scene3d's instance.rs; rows from runtime/scene). The placements are the first frame's; a later frame's differing list is warned of.

use super::*;
use bevy::math::{EulerRot, Quat};

fn packed(at: &[Value]) -> Option<String> {
    if at.len() % 9 != 0 {
        eprintln!("WARN  Instances: placements are 9 numbers a copy, got {}", at.len());
        return None;
    }
    let f = |v: &Value| v.as_f64().unwrap_or(0.0);
    let nums: Vec<String> = at
        .chunks(9)
        .flat_map(|c| {
            let q = Quat::from_euler(EulerRot::XYZ, f(&c[3]) as f32, f(&c[4]) as f32, f(&c[5]) as f32);
            [f(&c[0]), f(&c[1]), f(&c[2]), q.x as f64, q.y as f64, q.z as f64, q.w as f64, f(&c[6]), f(&c[7]), f(&c[8])]
        })
        .map(|v| format!("{}", r6(v)))
        .collect();
    Some(nums.join(" "))
}

/// The `inst` column of a mesh or model row whose part is an <Instances>.
pub fn instances_column(row: &mut BTreeMap<String, V>, its: &[Item]) {
    let Some(Value::Array(at)) = its[0].props.get("at") else { return };
    if its.iter().skip(1).any(|i| !matches!(i.props.get("at"), Some(Value::Array(a)) if std::ptr::eq(a, at) || a == at)) {
        eprintln!("WARN  Instances {}: its placements change over the shot; the first frame's are drawn", its[0].props.get("name").and_then(Value::as_str).unwrap_or("?"));
    }
    if let Some(s) = packed(at) {
        row.insert("inst".into(), V::S(s));
    }
}

/// A model row's shadows (a mesh row writes its own): `shadow={false}` casts none, `receiveShadow={false}` takes none
/// (the engine marks each of its meshes, and an <Instances>' copies).
pub fn model_shadows(row: &mut BTreeMap<String, V>, its: &[Item]) {
    if its[0].props.get("shadow") == Some(&Value::Bool(false)) {
        row.insert("shadow".into(), V::N(0.0));
    }
    if its[0].props.get("receiveShadow") == Some(&Value::Bool(false)) {
        row.insert("receive".into(), V::N(0.0));
    }
}
