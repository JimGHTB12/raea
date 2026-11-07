//! Bevy's Lightmap on a model's meshes by their glTF mesh name (<Lightmap mesh image> in a <Model>): a light bake the
//! mesh is lit by in place of the scene's lights, as Bevy's lightmaps example puts one on each mesh whose GltfMeshName
//! it knows. The spec is one of the model row's `surfaces` (surface.rs): `lightmap` (the file, a .ktx2 Bevy reads
//! itself), `mesh` (the glTF mesh's name, every primitive of it), `bicubic` (1 for bicubic sampling) and uv_x0, uv_y0,
//! uv_x1, uv_y1 (the rect in the image its second UVs are relative to; the whole image by default).

use super::*;
use bevy::pbr::Lightmap;
use raea_core_program::eval::Tup;
use std::path::Path;

/// A bake read from its spec, and the glTF mesh it lies on.
#[derive(Clone)]
pub struct Baked {
    pub(crate) mesh: String,
    pub(crate) made: Lightmap,
}

/// The spec's bake (None for a spec that isn't one), its file decoded now.
pub fn of(world: &mut World, t: &Tup, vals: &BTreeMap<String, V>, inserted: &[Tup], dir: &Path) -> Result<Option<Baked>, String> {
    let Some(V::S(src)) = vals.get("lightmap") else { return Ok(None) };
    let Some(V::S(mesh)) = vals.get("mesh") else { return Err(format!("model {}: a Lightmap names the glTF mesh it lies on (mesh)", t.id)) };
    let path = resolve_src(inserted, src, dir).map_err(|e| format!("model {}: Lightmap {mesh}: {e}", t.id))?;
    let image = image_file(world, &format!("model {}: Lightmap {mesh}", t.id), &path.to_string_lossy())?;
    let uv_rect = Rect::new(num(vals, "uv_x0", 0.0), num(vals, "uv_y0", 0.0), num(vals, "uv_x1", 1.0), num(vals, "uv_y1", 1.0));
    Ok(Some(Baked { mesh: mesh.clone(), made: Lightmap { image, uv_rect, bicubic_sampling: num(vals, "bicubic", 0.0) > 0.5 } }))
}

/// Each mesh entity (with its glTF mesh's name) given the bake that names its mesh.
pub fn apply(world: &mut World, meshes: &[(Entity, Option<String>)], baked: &[&Baked]) {
    for (x, name) in meshes {
        if let Some(b) = baked.iter().find(|b| name.as_deref() == Some(b.mesh.as_str())) {
            world.entity_mut(*x).insert(b.made.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bake_lands_on_every_mesh_of_its_name_and_no_other() {
        let mut world = World::new();
        let made = Lightmap { image: Handle::default(), uv_rect: Rect::new(0.0, 0.0, 0.5, 1.0), bicubic_sampling: true };
        let (a, b, c) = (world.spawn_empty().id(), world.spawn_empty().id(), world.spawn_empty().id());
        let meshes = [(a, Some("cornell_box".to_string())), (b, Some("cornell_box".to_string())), (c, Some("Light".to_string()))];
        apply(&mut world, &meshes, &[&Baked { mesh: "cornell_box".into(), made }]);
        for x in [a, b] {
            let l = world.get::<Lightmap>(x).expect("each primitive of the named mesh");
            assert!(l.bicubic_sampling && l.uv_rect.max.x == 0.5);
        }
        assert!(world.get::<Lightmap>(c).is_none());
    }
}
