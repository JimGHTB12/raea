//! StandardMaterial's fields that raea/3d's MeshStandardMaterial had no prop for (docs/shaders.feature SH7): the row's
//! `fields` column, a JSON object of the props as written (colours as hex, raea-core-scene's fields_of), read once when
//! the material is made. A Model's surface takes them too (surface.rs copies them from the material made for it).
//!   specularTint "#rrggbb"               the highlight's colour
//!   anisotropy n | {strength, rotation}  stretched highlights (brushed metal, hair); rotation in radians
//!   attenuation {distance, color}        light absorbed through a transmissive volume (with transmission, thickness)
//!   parallax {scale, method, layers, steps}  depthMap's parallax: "occlusion" or "relief" (steps: its search)
//!   depthBias n, fog false               drawn nearer in depth; left out of the fog
//!   uvTransform {offset, rotation, scale}  every map's UVs moved, turned and scaled (over `repeat`)
//!   renderMethod "forward" | "deferred", diffuseTransmission 0..1
//! The specular, specular tint and anisotropy maps stay off: each is a Bevy feature that adds bindings, and on Metal
//! without bindless the opaque pipeline then needs more than 16 samplers (bevy-plugins' package.json `bevy` note).

use super::*;
use bevy::math::Affine2;
use bevy::material::OpaqueRendererMethod;
use bevy::pbr::ParallaxMappingMethod;
use serde_json::{Map, Value};

/// The `fields` column onto `m` (`who` names the row in errors).
pub fn apply(m: &mut StandardMaterial, vals: &BTreeMap<String, V>, who: &str) -> Result<(), String> {
    let Some(V::S(s)) = vals.get("fields") else { return Ok(()) };
    let o: Map<String, Value> = serde_json::from_str(s).map_err(|e| format!("{who}: fields: {e}"))?;
    let n = |v: &Value, k: &str| v.as_f64().map(|x| x as f32).ok_or_else(|| format!("{who}: {k} is a number, got {v}"));
    let c = |v: &Value, k: &str| v.as_str().and_then(|h| Srgba::hex(h.trim_start_matches('#')).ok()).map(Color::from).ok_or_else(|| format!("{who}: {k} is a colour (#rrggbb), got {v}"));
    let pair = |v: Option<&Value>, k: &str, d: Vec2| -> Result<Vec2, String> {
        match v {
            None => Ok(d),
            Some(Value::Array(a)) if a.len() == 2 => Ok(Vec2::new(n(&a[0], k)?, n(&a[1], k)?)),
            Some(x) => Err(format!("{who}: {k} is [u, v], got {x}")),
        }
    };
    for (k, v) in &o {
        match (k.as_str(), v) {
            ("specularTint", _) => m.specular_tint = c(v, k)?,
            ("anisotropy", Value::Object(a)) => {
                m.anisotropy_strength = a.get("strength").map(|x| n(x, "anisotropy.strength")).transpose()?.unwrap_or(1.0);
                m.anisotropy_rotation = a.get("rotation").map(|x| n(x, "anisotropy.rotation")).transpose()?.unwrap_or(0.0);
            }
            ("anisotropy", _) => m.anisotropy_strength = n(v, k)?,
            ("attenuation", Value::Object(a)) => {
                if let Some(d) = a.get("distance") {
                    m.attenuation_distance = n(d, "attenuation.distance")?;
                }
                if let Some(x) = a.get("color") {
                    m.attenuation_color = c(x, "attenuation.color")?;
                }
            }
            ("parallax", Value::Object(p)) => {
                if let Some(x) = p.get("scale") {
                    m.parallax_depth_scale = n(x, "parallax.scale")?;
                }
                if let Some(x) = p.get("layers") {
                    m.max_parallax_layer_count = n(x, "parallax.layers")?;
                }
                let steps = p.get("steps").map(|x| n(x, "parallax.steps")).transpose()?.unwrap_or(5.0) as u32;
                m.parallax_mapping_method = match p.get("method").and_then(Value::as_str) {
                    None | Some("occlusion") => ParallaxMappingMethod::Occlusion,
                    Some("relief") => ParallaxMappingMethod::Relief { max_steps: steps },
                    Some(x) => return Err(format!("{who}: parallax.method is \"occlusion\" or \"relief\", got {x:?}")),
                };
            }
            ("depthBias", _) => m.depth_bias = n(v, k)?,
            ("fog", Value::Bool(b)) => m.fog_enabled = *b,
            ("uvTransform", Value::Object(u)) => {
                let scale = pair(u.get("scale"), "uvTransform.scale", Vec2::ONE)?;
                let offset = pair(u.get("offset"), "uvTransform.offset", Vec2::ZERO)?;
                let turn = u.get("rotation").map(|x| n(x, "uvTransform.rotation")).transpose()?.unwrap_or(0.0);
                m.uv_transform = Affine2::from_scale_angle_translation(scale, turn, offset);
            }
            ("renderMethod", Value::String(r)) => {
                m.opaque_render_method = match r.as_str() {
                    "forward" => OpaqueRendererMethod::Forward,
                    "deferred" => OpaqueRendererMethod::Deferred,
                    _ => return Err(format!("{who}: renderMethod is \"forward\" or \"deferred\", got {r:?}")),
                }
            }
            ("diffuseTransmission", _) => m.diffuse_transmission = n(v, k)?,
            _ => return Err(format!("{who}: fields has no {k} ({v})")),
        }
    }
    Ok(())
}

/// The fields `apply` sets, copied from the material made for a Model's surface onto its glTF material.
pub fn copy(to: &mut StandardMaterial, from: &StandardMaterial) {
    to.specular_tint = from.specular_tint;
    to.anisotropy_strength = from.anisotropy_strength;
    to.anisotropy_rotation = from.anisotropy_rotation;
    to.attenuation_distance = from.attenuation_distance;
    to.attenuation_color = from.attenuation_color;
    to.parallax_depth_scale = from.parallax_depth_scale;
    to.max_parallax_layer_count = from.max_parallax_layer_count;
    to.parallax_mapping_method = from.parallax_mapping_method;
    to.depth_bias = from.depth_bias;
    to.fog_enabled = from.fog_enabled;
    to.uv_transform = from.uv_transform;
    to.opaque_render_method = from.opaque_render_method;
    to.diffuse_transmission = from.diffuse_transmission;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_land_on_the_material_and_unknown_ones_are_refused() {
        let mut vals = BTreeMap::new();
        vals.insert("fields".to_string(), V::S(r##"{"specularTint":"#ff0000","anisotropy":{"strength":0.5,"rotation":1},"parallax":{"method":"relief","steps":8,"scale":0.2},"fog":false,"uvTransform":{"scale":[2,3]},"renderMethod":"deferred"}"##.into()));
        let mut m = StandardMaterial::default();
        apply(&mut m, &vals, "m").unwrap();
        assert_eq!(m.specular_tint, Color::from(Srgba::hex("ff0000").unwrap()));
        assert_eq!((m.anisotropy_strength, m.anisotropy_rotation), (0.5, 1.0));
        assert!(matches!(m.parallax_mapping_method, ParallaxMappingMethod::Relief { max_steps: 8 }));
        assert!(!m.fog_enabled && m.parallax_depth_scale == 0.2);
        assert_eq!(m.uv_transform.matrix2.x_axis.x, 2.0);
        assert!(matches!(m.opaque_render_method, OpaqueRendererMethod::Deferred));
        vals.insert("fields".to_string(), V::S(r#"{"shine":1}"#.into()));
        assert!(apply(&mut m, &vals, "m").is_err());
    }
}
