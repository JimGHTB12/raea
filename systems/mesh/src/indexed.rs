//! A <BufferMesh> given its indices and normals, as Bevy's Mesh holds one (docs/control.feature FC2d): its vertices as
//! given, its index buffer kept (U32), not unrolled into flat triangles. A compute that writes a mesh's slabs (Bevy's
//! shader_advanced/compute_mesh, <Compute buffers={{ NAME: { mesh, slab } }}>) writes exactly this layout: position,
//! normal and uv_0 interleaved in Bevy's attribute order, and the indices. Its index count need not be a whole number
//! of triangles, as Bevy's need not.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use serde_json::Value;

type Colors = fn(&Value) -> Result<Option<Vec<[f32; 4]>>, String>;

/// The mesh, or None for one drawn as flat triangles (no indices or no normals, or no uvs when `uvs` needs them).
pub fn buffer_indexed(json: &str, uvs: bool, colors: Colors) -> Result<Option<Mesh>, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("a buffer mesh isn't JSON: {e}"))?;
    let nums = |k: &str| -> Result<Option<Vec<f32>>, String> {
        match v.get(k) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Array(a)) => a.iter().map(|x| x.as_f64().map(|f| f as f32).ok_or_else(|| format!("a buffer mesh's {k}: numbers, got {x}"))).collect::<Result<Vec<_>, _>>().map(Some),
            Some(x) => Err(format!("a buffer mesh's {k}: an array of numbers, got {x}")),
        }
    };
    let (Some(pos), Some(idx), Some(nor)) = (nums("positions")?, nums("indices")?, nums("normals")?) else { return Ok(None) };
    let own_uvs = nums("uvs")?;
    if uvs && own_uvs.is_none() {
        return Ok(None);
    }
    let n = pos.len() / 3;
    if pos.len() % 3 != 0 || n == 0 || nor.len() != pos.len() || own_uvs.as_ref().is_some_and(|u| u.len() != n * 2) {
        return Err(format!("a buffer mesh's positions and normals are x, y, z and its uvs u, v for each of its {n} vertices"));
    }
    if let Some(i) = idx.iter().find(|&&i| i < 0.0 || i.fract() != 0.0 || i as usize >= n) {
        return Err(format!("a buffer mesh's indices are its vertices, 0 to {}, got {i}", n - 1));
    }
    let v3 = |a: &[f32]| a.chunks(3).map(|c| [c[0], c[1], c[2]]).collect::<Vec<_>>();
    let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, v3(&pos));
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, v3(&nor));
    if let Some(u) = own_uvs {
        m.insert_attribute(Mesh::ATTRIBUTE_UV_0, u.chunks(2).map(|c| [c[0], c[1]]).collect::<Vec<_>>());
    }
    if let Some(c) = colors(&v)? {
        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
    }
    m.insert_indices(Indices::U32(idx.iter().map(|&i| i as u32).collect()));
    Ok(Some(m))
}

#[cfg(test)]
mod tests {
    #[test]
    fn indices_and_normals_keep_the_index_buffer() {
        let none = |_: &serde_json::Value| Ok(None);
        let j = r#"{"positions": [0,0,0, 1,0,0, 0,1,0, 0,0,0], "normals": [0,0,1, 0,0,1, 0,0,1, 0,0,1], "uvs": [0,0, 1,0, 0,1, 0,0], "indices": [0,1,2, 2,1]}"#;
        let m = super::buffer_indexed(j, true, none).unwrap().unwrap();
        assert_eq!((m.count_vertices(), m.indices().map(|i| i.len())), (4, Some(5)));
        assert_eq!(m.attributes().count(), 3);
        // flat triangles: without normals, or a picture's uvs it doesn't give
        assert!(super::buffer_indexed(r#"{"positions": [0,0,0, 1,0,0, 0,1,0], "indices": [0,1,2]}"#, false, none).unwrap().is_none());
        assert!(super::buffer_indexed(r#"{"positions": [0,0,0, 1,0,0, 0,1,0], "normals": [0,0,1, 0,0,1, 0,0,1], "indices": [0,1,2]}"#, true, none).unwrap().is_none());
        assert!(super::buffer_indexed(r#"{"positions": [0,0,0, 1,0,0, 0,1,0], "normals": [0,0,1, 0,0,1, 0,0,1], "indices": [0,1,3]}"#, false, none).unwrap_err().contains("0 to 2"));
    }
}
