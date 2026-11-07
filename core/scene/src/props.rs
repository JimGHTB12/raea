//! The props a 3D part carries: numbers, vectors, colours and its transform.

use bevy::math::{EulerRot, Mat4, Quat, Vec3};
use serde_json::{Map, Value};

pub fn num(p: &Map<String, Value>, k: &str) -> Option<f64> {
    p.get(k).and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
}

pub fn vec3(p: &Map<String, Value>, k: &str, d: [f64; 3]) -> [f64; 3] {
    p.get(k).and_then(|v| v.as_array()).filter(|a| a.len() == 3).map(|a| [a[0].as_f64().unwrap_or(d[0]), a[1].as_f64().unwrap_or(d[1]), a[2].as_f64().unwrap_or(d[2])]).unwrap_or(d)
}

pub fn scale3(p: &Map<String, Value>) -> [f64; 3] {
    match p.get("scale") {
        Some(Value::Number(n)) => {
            let s = n.as_f64().unwrap_or(1.0);
            [s, s, s]
        }
        _ => vec3(p, "scale", [1.0; 3]),
    }
}

/// A part's own transform, as three.js composes it: scale, then rotation (Euler XYZ, radians), then position.
pub fn local(p: &Map<String, Value>) -> Mat4 {
    let pos = vec3(p, "position", [0.0; 3]);
    let r = vec3(p, "rotation", [0.0; 3]);
    let s = scale3(p);
    // a rotation as a quaternion [x, y, z, w] (a rigid body's, physics.tsx) wins over the Euler angles
    let q = p.get("quaternion").and_then(Value::as_array).filter(|q| q.len() == 4).map(|q| Quat::from_xyzw(q[0].as_f64().unwrap_or(0.0) as f32, q[1].as_f64().unwrap_or(0.0) as f32, q[2].as_f64().unwrap_or(0.0) as f32, q[3].as_f64().unwrap_or(1.0) as f32).normalize());
    Mat4::from_scale_rotation_translation(
        Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32),
        q.unwrap_or_else(|| Quat::from_euler(EulerRot::XYZ, r[0] as f32, r[1] as f32, r[2] as f32)),
        Vec3::new(pos[0] as f32, pos[1] as f32, pos[2] as f32),
    )
}

pub fn hex_of(c: &str) -> Option<String> {
    hex_cow(c).map(std::borrow::Cow::into_owned)
}

/// `hex_of`, borrowing where the colour is already hex or named (ZeroHeap: a live 3D frame read each mesh's colour)
pub fn hex_cow(c: &str) -> Option<std::borrow::Cow<'_, str>> {
    use std::borrow::Cow;
    let c = c.trim();
    // #rrggbbaa too: a layered Canvas3D's transparent clear (scene_clear)
    if c.starts_with('#') && (c.len() == 7 || c.len() == 4 || c.len() == 9) {
        return Some(Cow::Borrowed(c));
    }
    let named = [("black", "#000000"), ("white", "#ffffff"), ("red", "#ff0000"), ("transparent", "#000000")];
    named.iter().find(|(n, _)| *n == c).map(|(_, h)| Cow::Borrowed(*h)).or_else(|| {
        let inner = c.strip_prefix("rgb(").or_else(|| c.strip_prefix("rgba("))?.trim_end_matches(')');
        let v: Vec<u8> = inner.split(',').take(3).filter_map(|x| x.trim().parse::<f64>().ok()).map(|x| x.round().clamp(0.0, 255.0) as u8).collect();
        (v.len() == 3).then(|| Cow::Owned(format!("#{:02x}{:02x}{:02x}", v[0], v[1], v[2])))
    })
}

pub fn hex(c: Option<&Value>) -> Option<String> {
    c.and_then(Value::as_str).and_then(hex_of)
}

pub fn rgb(h: &str) -> [f64; 3] {
    let h = h.trim_start_matches('#');
    let h = if h.len() == 3 { h.chars().flat_map(|c| [c, c]).collect::<String>() } else { h.to_string() };
    let b = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("cc"), 16).unwrap_or(204) as f64 / 255.0;
    [b(0), b(2), b(4)]
}

pub fn n(p: &Map<String, Value>, k: &str, d: f64) -> f64 {
    num(p, k).unwrap_or(d)
}

/// Light intensity: a DirectionalLight of intensity 1 is 8000 lux (as before); a point or spot light of intensity 1
/// lights a surface 3 units away about as brightly (8000 lux x 9 m² x 4π ≈ 900,000 lumens).
pub const LUX: f64 = 8000.0;
pub const LUMENS: f64 = 9.0e5;
/// An AmbientLight of intensity 1, in cd/m² (as before).
pub const AMBIENT: f64 = 400.0;
/// A light's power in Bevy's units (docs/control.feature FC1f): a Bevy light's own `lux` (directional), `lumens`
/// (point, spot, rect) or `nits` (ambient, cd/m²) as written; a 3d.tsx light's relative `intensity` times LUX, LUMENS
/// or AMBIENT.
pub fn light_power(kind: &str, p: &Map<String, Value>) -> f64 {
    let (field, k) = match kind {
        "directional" => ("lux", LUX),
        "ambient" => ("nits", AMBIENT),
        _ => ("lumens", LUMENS),
    };
    num(p, field).unwrap_or_else(|| num(p, "intensity").unwrap_or(1.0) * k)
}

/// An <Environment> of intensity 1: the panorama's values (a Poly Haven HDR's sky is about 1) times this, in cd/m².
pub const ENV: f64 = 1000.0;
/// Bevy's default exposure (EV100): exposure 0; +1 is a stop brighter.
pub const EV100: f64 = 9.7;

/// StandardMaterial's other fields as a material's props give them (docs/shaders.feature SH7), for its row's `fields`
/// column: the props present, colours as hex (raea-sys-material's fields.rs reads them).
pub fn fields_of(p: &Map<String, Value>) -> Option<String> {
    let mut o = Map::new();
    for k in ["specularTint", "anisotropy", "attenuation", "parallax", "depthBias", "fog", "uvTransform", "renderMethod", "diffuseTransmission"] {
        let Some(v) = p.get(k) else { continue };
        let v = match (k, v) {
            ("specularTint", _) => Value::String(hex(Some(v))?),
            ("attenuation", Value::Object(a)) => {
                let mut a = a.clone();
                if let Some(c) = a.get("color").and_then(|c| hex(Some(c))) {
                    a.insert("color".into(), Value::String(c));
                }
                Value::Object(a)
            }
            _ => v.clone(),
        };
        o.insert(k.into(), v);
    }
    (!o.is_empty()).then(|| Value::Object(o).to_string())
}
