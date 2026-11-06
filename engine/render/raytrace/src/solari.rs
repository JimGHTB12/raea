//! RT3 (docs/raytracing.feature): Bevy's Solari, ray-traced lighting, for a scene whose camera asks for it
//! (<SolariLighting> or <Pathtracer> in raea/3d: SolariLighting or Pathtracer under <Camera3d>). Its plugins come in
//! only then: SolariLightingPlugin makes every opaque material deferred, which would change every other scene.
//! The shared GPU device asks for every feature its adapter has (raea-gpu-device), so ray queries are on where the GPU
//! has them; where it doesn't, Solari's plugins say so and the scene draws with raster lighting.
//!
//! Solari traces a mesh only in one layout (positions, normals, UV 0, tangents, nothing else, u32 indices), so each
//! mesh gets a copy in that layout for its RaytracingMesh3d, made once per mesh. Skinned and morphed meshes (a
//! <Character>) are left raster-lit, as Solari 0.20 can't trace them (RT5).

use super::*;
use bevy::camera::CameraMainTextureUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::solari::pathtracer::{Pathtracer, PathtracingPlugin};
use bevy::solari::prelude::{RaytracingMesh3d, SolariLighting, SolariPlugins};

pub fn add(app: &mut App) {
    app.add_plugins((SolariPlugins, PathtracingPlugin)).add_systems(PostUpdate, (cameras, meshes).before(bevy::camera::CameraUpdateSystems));
}

/// A camera or mesh already set up (or left out).
#[derive(Component)]
struct Done;

// Solari's camera needs: no MSAA and a main texture it can write to (SolariLighting's docs); Hdr, the prepasses and
// deferred come with its required components
fn cameras(mut cmd: Commands, q: Query<(Entity, Option<&CameraMainTextureUsages>), (Or<(With<SolariLighting>, With<Pathtracer>)>, Without<Done>)>) {
    for (e, u) in &q {
        let usages = u.map_or_else(|| CameraMainTextureUsages::default().0, |u| u.0) | bevy::render::render_resource::TextureUsages::STORAGE_BINDING;
        cmd.entity(e).insert((Msaa::Off, CameraMainTextureUsages(usages), Done));
    }
}

type Pending = (With<MeshMaterial3d<StandardMaterial>>, Without<RaytracingMesh3d>, Without<Done>);

fn meshes(
    mut cmd: Commands,
    on: Query<(), Or<(With<SolariLighting>, With<Pathtracer>)>>,
    q: Query<(Entity, &Mesh3d, Has<bevy::mesh::skinning::SkinnedMesh>, Has<bevy::mesh::morph::MeshMorphWeights>), Pending>,
    mut assets: ResMut<Assets<Mesh>>,
    mut copies: Local<HashMap<AssetId<Mesh>, Option<Handle<Mesh>>>>,
    mut said: Local<bool>,
) {
    if on.is_empty() {
        return;
    }
    for (e, m, skinned, morphed) in &q {
        if skinned || morphed {
            if !*said {
                *said = true;
                eprintln!("raea: <SolariLighting>: a skinned or morphed mesh (a <Character>) draws raster-lit; Bevy 0.20's Solari can't trace it (docs/raytracing.feature RT5)");
            }
            cmd.entity(e).insert(Done);
            continue;
        }
        // each mesh's traceable form, made once: itself when it's in Solari's layout, else a copy; None when it can't
        // be traced (not triangles, no normals, or its data only on the GPU)
        let h = match copies.get(&m.0.id()) {
            Some(h) => h.clone(),
            None => {
                let Some(t) = assets.get(&m.0).map(traceable) else { continue };
                let h = match t {
                    Some(Traceable::Same) => Some(m.0.clone()),
                    Some(Traceable::Copy(mesh)) => Some(assets.add(mesh)),
                    None => None,
                };
                copies.insert(m.0.id(), h.clone());
                h
            }
        };
        match h {
            Some(h) => cmd.entity(e).insert((RaytracingMesh3d(h), Done)),
            None => cmd.entity(e).insert(Done),
        };
    }
}

enum Traceable {
    Same,
    Copy(Mesh),
}

fn traceable(src: &Mesh) -> Option<Traceable> {
    if src.primitive_topology() != PrimitiveTopology::TriangleList {
        return None;
    }
    let ids: Vec<_> = src.attributes().map(|(a, _)| a.id).collect();
    let layout = [Mesh::ATTRIBUTE_POSITION.id, Mesh::ATTRIBUTE_NORMAL.id, Mesh::ATTRIBUTE_UV_0.id, Mesh::ATTRIBUTE_TANGENT.id];
    if ids == layout && matches!(src.indices(), Some(Indices::U32(_))) {
        return Some(Traceable::Same);
    }
    let pos = src.attribute(Mesh::ATTRIBUTE_POSITION)?.clone();
    let n = pos.len();
    let normal = src.attribute(Mesh::ATTRIBUTE_NORMAL)?.clone();
    let uv = src.attribute(Mesh::ATTRIBUTE_UV_0).cloned().unwrap_or(VertexAttributeValues::Float32x2(vec![[0.0, 0.0]; n]));
    let mut out = Mesh::new(PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default());
    out.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    out.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normal);
    out.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    let idx: Vec<u32> = match src.indices() {
        Some(Indices::U32(v)) => v.clone(),
        Some(Indices::U16(v)) => v.iter().map(|&i| i as u32).collect(),
        None => (0..n as u32).collect(),
    };
    out.insert_indices(Indices::U32(idx));
    match src.attribute(Mesh::ATTRIBUTE_TANGENT) {
        Some(t) => out.insert_attribute(Mesh::ATTRIBUTE_TANGENT, t.clone()),
        None => out.generate_tangents().ok()?,
    }
    Some(Traceable::Copy(out))
}
