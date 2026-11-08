//! A compute as written's storage buffers (docs/control.feature FC2d): each `var<storage(, ACCESS)> NAME: TYPE` its
//! source declares binds the compute's own buffer, its first contents given in TYPE's storage layout (raea-sys-shader's
//! storage_words) and kept from frame to frame, or a mesh's vertex or index slab in Bevy's MeshAllocator, as
//! shader_advanced/compute_mesh writes a mesh on the GPU: <Compute buffers={{ vertex_data: { mesh: "gen", slab:
//! "vertices" } }}>, the mesh the part named "gen" holds. A uniform's field given { start: NAME } or { end: NAME } is
//! that mesh's range in the slab, counted in the 4-byte elements NAME's array holds (the example's
//! `vertex_buffer_slice.range.start * 8`), packed each frame once the allocator has placed the mesh.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy::render::mesh::allocator::MeshAllocator;
use bevy::render::mesh::{RenderMesh, RenderMeshBufferInfo};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{Buffer, IndexFormat};
use bevy::shader::ShaderDefVal;
use serde_json::{Map, Value};

use crate::written::Decl;

/// What a storage buffer binds: its own first contents, or a mesh's slab (`id`: the mesh its part holds this frame).
#[derive(Clone, Debug)]
pub enum Bound {
    Own(Vec<u8>),
    Mesh { name: String, indices: bool, id: Option<AssetId<Mesh>> },
}

/// Uniforms that hold a slab's range: what they're packed from each frame.
#[derive(Clone, Debug)]
pub struct Ranged {
    src: String,
    defs: Vec<ShaderDefVal>,
    declared: Vec<(String, String)>,
    given: Map<String, Value>,
}

impl Ranged {
    pub fn new(src: &str, defs: &[ShaderDefVal], declared: Vec<(String, String)>, given: Map<String, Value>) -> Self {
        Ranged { src: src.to_string(), defs: defs.to_vec(), declared, given }
    }

    /// Each uniform's bytes by name, each range the one `at` (buffer, end) gives, and whether any field is a range.
    pub fn pack(&self, at: &dyn Fn(&str, bool) -> Option<u32>) -> Result<(BTreeMap<String, Vec<u8>>, bool), String> {
        let mut given = self.given.clone();
        let mut found = false;
        for v in given.values_mut() {
            subst(v, at, &mut found)?;
        }
        let slots = raea_sys_shader::uniform_slots(&self.src, &self.declared, &given, &self.defs)?;
        let bytes = self.declared.iter().zip(slots).map(|((n, _), s)| (n.clone(), s.iter().flat_map(|v| v.to_array()).flat_map(f32::to_ne_bytes).collect()));
        Ok((bytes.collect(), found))
    }
}

/// Each { start: NAME } or { end: NAME } in `v` replaced by the range `at` gives.
fn subst(v: &mut Value, at: &dyn Fn(&str, bool) -> Option<u32>, found: &mut bool) -> Result<(), String> {
    match v {
        Value::Object(o) if o.len() == 1 && (o.contains_key("start") || o.contains_key("end")) => {
            let (k, b) = o.iter().next().expect("one field");
            let name = b.as_str().ok_or_else(|| format!("{{ {k}: NAME }} names a storage buffer, got {b}"))?;
            let n = at(name, k == "end").ok_or_else(|| format!("{{ {k}: {name:?} }}: {name} is a storage buffer bound to a mesh's slab"))?;
            *found = true;
            *v = n.into();
        }
        Value::Object(o) => o.values_mut().try_for_each(|x| subst(x, at, found))?,
        Value::Array(a) => a.iter_mut().try_for_each(|x| subst(x, at, found))?,
        _ => {}
    }
    Ok(())
}

/// Each declared storage buffer's binding from the spec's `buffers` ({NAME: first contents | {mesh, slab}}).
pub fn bound(src: &str, defs: &[ShaderDefVal], decls: &[Decl], given: Option<&Value>) -> Result<BTreeMap<String, Bound>, String> {
    let empty = Map::new();
    let given = match given {
        None | Some(Value::Null) => &empty,
        Some(Value::Object(m)) => m,
        Some(x) => return Err(format!("a compute's buffers are {{NAME: value}}, got {x}")),
    };
    let declared: Vec<(&String, &String)> = decls.iter().filter_map(|d| if let Decl::Buffer { name, ty, .. } = d { Some((name, ty)) } else { None }).collect();
    if let Some(k) = given.keys().find(|k| !declared.iter().any(|(n, _)| n == k)) {
        return Err(format!("buffer {k}: the source declares no var<storage> {k}"));
    }
    let mut out = BTreeMap::new();
    for (name, ty) in declared {
        let v = given.get(name).ok_or_else(|| format!("storage buffer {name} has no value: its first contents, or {{ mesh, slab }}"))?;
        let b = match v.get("mesh") {
            Some(m) => {
                let mesh = m.as_str().ok_or_else(|| format!("buffer {name}: mesh is a part's name, got {m}"))?;
                let indices = match v.get("slab").and_then(Value::as_str) {
                    Some("vertices") => false,
                    Some("indices") => true,
                    s => return Err(format!("buffer {name}: slab is \"vertices\" or \"indices\", got {s:?}")),
                };
                if !matches!(ty.replace(' ', "").as_str(), "array<f32>" | "array<u32>" | "array<i32>") {
                    return Err(format!("buffer {name}: a mesh's slab binds as array<f32>, array<u32> or array<i32>, got {ty}"));
                }
                Bound::Mesh { name: mesh.to_string(), indices, id: None }
            }
            None => Bound::Own(raea_sys_shader::storage_words(src, defs, name, ty, v)?.iter().flat_map(|w| w.to_le_bytes()).collect()),
        };
        out.insert(name.clone(), b);
    }
    Ok(out)
}

/// A mesh's vertex or index slab, and the mesh's range in it in 4-byte elements (None until it's allocated, or for
/// 16-bit indices).
pub fn slab_of(alloc: &MeshAllocator, meshes: &RenderAssets<RenderMesh>, id: AssetId<Mesh>, indices: bool) -> Option<(Buffer, [u32; 2])> {
    let m = meshes.get(id)?;
    let (s, unit) = if indices {
        let RenderMeshBufferInfo::Indexed { index_format: IndexFormat::Uint32, .. } = m.buffer_info else { return None };
        (alloc.mesh_index_slice(&id)?, 4)
    } else {
        (alloc.mesh_vertex_slice(&id)?, m.layout.0.layout().array_stride as u32)
    };
    Some((s.buffer.clone(), [s.range.start * unit / 4, s.range.end * unit / 4]))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn buffers_bind_their_own_contents_or_a_meshs_slab_and_uniforms_take_its_range() {
        let src = "struct R { a: u32, b: u32 }\n@group(0) @binding(0) var<uniform> r: R;\n@group(0) @binding(1) var<storage, read_write> v: array<f32>;\n@group(0) @binding(2) var<storage> own: array<f32>;\n";
        let d = crate::written::decls(src).unwrap();
        let b = bound(src, &[], &d, Some(&json!({"v": {"mesh": "gen", "slab": "vertices"}, "own": [1, 2]}))).unwrap();
        assert!(matches!(&b["v"], Bound::Mesh { name, indices: false, .. } if name == "gen"));
        assert!(matches!(&b["own"], Bound::Own(x) if x.len() == 16 && x[4..8] == 2f32.to_le_bytes()));
        assert!(bound(src, &[], &d, Some(&json!({"v": {"mesh": "gen", "slab": "x"}, "own": [1]}))).unwrap_err().contains("slab"));
        assert!(bound(src, &[], &d, Some(&json!({"own": [1]}))).unwrap_err().contains("v has no value"));
        let r = Ranged::new(src, &[], vec![("r".into(), "R".into())], json!({"r": {"a": {"start": "v"}, "b": {"end": "v"}}}).as_object().unwrap().clone());
        let (u, found) = r.pack(&|n, end| (n == "v").then_some(if end { 80 } else { 24 })).unwrap();
        assert!(found);
        assert_eq!((u32::from_ne_bytes(u["r"][0..4].try_into().unwrap()), u32::from_ne_bytes(u["r"][4..8].try_into().unwrap())), (24, 80));
        assert!(r.pack(&|_, _| None).unwrap_err().contains("mesh's slab"));
    }
}
