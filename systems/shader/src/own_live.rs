//! A Material's values over the frames (docs/control.feature FC2a): its uniforms and read-only storage buffers as the
//! frame gives them. The rows send a part's <Material> uniforms and buffers as keyed `own:uniforms` component rows
//! from each frame they change (raea-runtime-look own_uniforms), and `patch` packs them into the part's OwnMaterial
//! and its ShaderBuffers, as Bevy's examples change a material's fields and buffers in their update systems
//! (shader/storage_buffer's colours, sprite_material's dissolve).

use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderDefVal;
use bevy::sprite_render::{MeshMaterial2d, SpriteMaterial};
use serde_json::Value;

use crate::layout;
use crate::own::OwnMaterial;

/// What a material's uniforms and buffers are packed from: its source as compiled (its struct types), its uniforms'
/// and storage buffers' (name, WGSL type) in slot order, and its defs (a struct field under `@if`).
#[derive(Debug)]
pub struct Packing {
    pub src: String,
    pub uniforms: Vec<(String, String)>,
    pub storages: Vec<(String, String)>,
    pub defs: Vec<ShaderDefVal>,
}

/// A storage buffer's contents for its WGSL type, `array<T, N>` or `array<T>` with T an f32, u32 or i32 or a vec2-4
/// of one: each element a number or [x, y(, z(, w))], in WGSL's storage layout (a vec3 takes 16 bytes, as a vec4).
/// An empty one is 16 zero bytes (a binding can't be empty).
pub fn storage(name: &str, ty: &str, v: &Value) -> Result<ShaderBuffer, String> {
    let inner = ty.strip_prefix("array<").and_then(|t| t.strip_suffix('>')).ok_or_else(|| format!("storage buffer {name}: its type is array<T> or array<T, N>, got {ty} (the rest are FC2a's to come)"))?;
    let el = match inner.rfind(',') {
        Some(i) if inner[i + 1..].trim().parse::<u64>().is_ok() => inner[..i].trim(),
        _ => inner.trim(),
    };
    let (k, n) = layout::kind(el).ok_or_else(|| format!("storage buffer {name}: its element {el} isn't f32, u32, i32 or a vec2/3/4 of one"))?;
    let stride = if n == 1 { 1 } else if n == 2 { 2 } else { 4 };
    let items = v.as_array().ok_or_else(|| format!("storage buffer {name} is a list of its elements, got {v}"))?;
    let mut words = vec![0u32; (items.len() * stride).max(4)];
    for (i, it) in items.iter().enumerate() {
        let comps: Vec<f64> = match it {
            Value::Array(a) => a.iter().filter_map(Value::as_f64).collect(),
            x => x.as_f64().into_iter().collect(),
        };
        if comps.len() != n {
            return Err(format!("storage buffer {name}: element {i} is {n} numbers ({el}), got {it}"));
        }
        for (c, x) in comps.iter().enumerate() {
            words[i * stride + c] = layout::word(k, *x as f32);
        }
    }
    Ok(ShaderBuffer::from(words))
}

/// The frame's values for a part whose Material isn't on yet, applied when it is (own.rs put).
#[derive(Component)]
pub struct Pending(pub Value);

/// The frame's values a row gave before the part's Material was on, now that it is.
pub fn settle(world: &mut World, e: Entity) -> Result<(), String> {
    match world.entity_mut(e).take::<Pending>() {
        Some(p) => patch(world, e, &p.0),
        None => Ok(()),
    }
}

/// The part's own Material, if it holds it as `C` (a 3D, 2D or sprite's).
fn handle<C: Component + std::ops::Deref<Target = Handle<OwnMaterial>>>(world: &World, e: Entity) -> Option<Handle<OwnMaterial>> {
    world.get::<C>(e).map(|m| (**m).clone())
}

/// An `own:uniforms` component row: the frame's {uniforms, buffers} packed into the part's own Material (3D, 2D or a
/// sprite's) and its storage buffers.
pub fn patch(world: &mut World, e: Entity, v: &Value) -> Result<(), String> {
    let h = [handle::<MeshMaterial3d<OwnMaterial>>(world, e), handle::<MeshMaterial2d<OwnMaterial>>(world, e), handle::<SpriteMaterial<OwnMaterial>>(world, e)];
    let h = h.into_iter().flatten().next();
    // a row applied before the part's Material is put on (shade_rows runs after the rows): kept for `put`
    let Some(h) = h else {
        world.entity_mut(e).insert(Pending(v.clone()));
        return Ok(());
    };
    let Some((p, bufs)) = world.resource::<Assets<OwnMaterial>>().get(&h).map(|m| (m.packing.clone(), m.buffers())) else { return Err("own:uniforms: its Material isn't made".into()) };
    if let Some(given) = v.get("uniforms").and_then(Value::as_object) {
        let slots = layout::slots(&p.src, &p.uniforms, given, &p.defs)?;
        if let Some(mut m) = world.resource_mut::<Assets<OwnMaterial>>().get_mut(&h) {
            m.set_slots(slots);
        }
    }
    let given = v.get("buffers").and_then(Value::as_object).cloned().unwrap_or_default();
    for ((name, ty), b) in p.storages.iter().zip(bufs) {
        let Some(x) = given.get(name) else { continue };
        let sb = storage(name, ty, x)?;
        if let Some(mut cur) = world.resource_mut::<Assets<ShaderBuffer>>().get_mut(&b) {
            *cur = sb;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_storage_buffer_is_laid_out_as_wgsl_stores_its_type() {
        let words = |b: bevy::render::storage::ShaderBuffer| match b.data {
            bevy::render::storage::ShaderBufferData::Initialized { data, .. } => data.chunks(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect::<Vec<_>>(),
            _ => vec![],
        };
        let v = serde_json::json!([[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]]);
        let w = words(super::storage("colors", "array<vec4<f32>, 5>", &v).unwrap());
        assert_eq!(w[..8].iter().map(|x| f32::from_bits(*x)).collect::<Vec<_>>(), vec![1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
        // a vec3 takes 16 bytes; a u32 rounds
        let w = words(super::storage("p", "array<vec3<f32>>", &serde_json::json!([[1, 2, 3], [4, 5, 6]])).unwrap());
        assert_eq!(f32::from_bits(w[4]), 4.0);
        assert_eq!(words(super::storage("n", "array<u32>", &serde_json::json!([2.6, 7])).unwrap())[..2], [3, 7]);
        assert!(super::storage("p", "array<vec3<f32>>", &serde_json::json!([[1, 2]])).unwrap_err().contains("element 0"));
        assert!(super::storage("m", "array<mat4x4<f32>>", &serde_json::json!([])).is_err());
    }
}
