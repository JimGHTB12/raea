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

/// A storage buffer's contents for its WGSL type as `src` declares it, in WGSL's storage layout (storage_layout.rs):
/// `array<T, N>` or `array<T>` of numbers or [x, y(, z(, w))], a matrix, or a struct of the source's from {field: value}
/// (a vec3 takes 16 bytes, as a vec4). An empty one is 16 zero bytes (a binding can't be empty).
pub fn storage(src: &str, defs: &[ShaderDefVal], name: &str, ty: &str, v: &Value) -> Result<ShaderBuffer, String> {
    Ok(ShaderBuffer::from(crate::storage_layout::words(src, defs, name, ty, v)?))
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
        let sb = storage(&p.src, &p.defs, name, ty, x)?;
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
        let w = words(super::storage("", &[], "colors", "array<vec4<f32>, 5>", &v).unwrap());
        assert_eq!(w[..8].iter().map(|x| f32::from_bits(*x)).collect::<Vec<_>>(), vec![1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
        // a vec3 takes 16 bytes; a u32 rounds
        let w = words(super::storage("", &[], "p", "array<vec3<f32>>", &serde_json::json!([[1, 2, 3], [4, 5, 6]])).unwrap());
        assert_eq!(f32::from_bits(w[4]), 4.0);
        assert_eq!(words(super::storage("", &[], "n", "array<u32>", &serde_json::json!([2.6, 7])).unwrap())[..2], [3, 7]);
        assert!(super::storage("", &[], "p", "array<vec3<f32>>", &serde_json::json!([[1, 2]])).unwrap_err().contains("element 0"));
        assert!(super::storage("", &[], "m", "array<bool>", &serde_json::json!([])).is_err());
    }
}
