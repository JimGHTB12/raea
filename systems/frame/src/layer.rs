//! A drawn thing's layer key (docs/review.feature R2a): where its JSX is written, its place in a list, and the uses of
//! the edit's components it was drawn in, as engine/bundle's source stamps give them. A 3D part carries it as this
//! component (runtime/scene bevy_parts makes it a component row of the part), never as a column its kind reads.

use bevy::prelude::*;

/// "file:line:col[#index]|use|use…": the part's stamp, then its use chain, nearest first
#[derive(Component, Reflect, Default, Clone, Debug, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct LayerKey(pub String);

/// The key a stamped element or part's props give it (raea-sys-layers owns it: the page's half reads it too).
pub use raea_sys_layers::{key_from_props, key_of};


/// With `raea render --layers`, frame `f`'s 3D recorded as layer indices (raea-sys-layers, docs/review.feature R2b):
/// every shown mesh drawn on the CPU from the frame's camera, nearest first, as the layer of the nearest part up from
/// it with a LayerKey (none: it still hides what's behind it). Skinned meshes are posed as their joints are; morph
/// targets and anything not a mesh (sprites, splats, particles) are not drawn.
pub fn record(world: &mut World, f: i64) {
    if !raea_sys_layers::on() {
        return;
    }
    let mut cams = world.query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>();
    let Some((cam, at)) = cams.iter(world).filter(|(c, _)| c.is_active).min_by_key(|(c, _)| c.order) else { return };
    let Some(size) = cam.physical_target_size() else { return };
    let clip_from_world = cam.clip_from_view() * Mat4::from(at.affine().inverse());
    let mut layers: Vec<raea_sys_layers::Layer> = Vec::new();
    let mut tris: Vec<raea_sys_layers::Tri> = Vec::new();
    let mut parts = world.query::<(Entity, &Mesh3d, &GlobalTransform, Option<&InheritedVisibility>, Option<&bevy::mesh::skinning::SkinnedMesh>)>();
    let (meshes, binds) = (world.resource::<Assets<Mesh>>(), world.resource::<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>());
    for (e, m, place, shown, skin) in parts.iter(world) {
        if shown.is_some_and(|s| !s.get()) {
            continue;
        }
        let Some(mesh) = meshes.get(&m.0).filter(|m| m.primitive_topology() == bevy::mesh::PrimitiveTopology::TriangleList) else { continue };
        let Ok(Some(bevy::mesh::VertexAttributeValues::Float32x3(pos))) = mesh.try_attribute_option(Mesh::ATTRIBUTE_POSITION) else { continue };
        let joints = skin.and_then(|s| Some((s, binds.get(&s.inverse_bindposes)?))).map(|(s, b)| s.joints.iter().zip(b.iter()).map(|(j, ib)| world.get::<GlobalTransform>(*j).map_or(Mat4::IDENTITY, |g| Mat4::from(g.affine()) * *ib)).collect::<Vec<_>>());
        let posed = posed(mesh, pos, joints.as_deref(), Mat4::from(place.affine()));
        let clip: Vec<[f32; 4]> = posed.iter().map(|p| (clip_from_world * p.extend(1.0)).to_array()).collect();
        let v = keyed(world, e).map_or(0, |(key, name)| match layers.iter().position(|l| l.key == key) {
            Some(i) => i as u16 + 1,
            None => {
                layers.push(raea_sys_layers::Layer::of(key, "mesh", name, None, None));
                layers.len() as u16
            }
        });
        let corners: Vec<usize> = match mesh.try_indices_option() {
            Ok(Some(ix)) => ix.iter().collect(),
            _ => (0..clip.len()).collect(),
        };
        tris.extend(corners.chunks_exact(3).filter_map(|t| Some(([*clip.get(t[0])?, *clip.get(t[1])?, *clip.get(t[2])?], v))));
    }
    let mask = raea_sys_layers::raster((size.x, size.y), tris);
    raea_sys_layers::with(|r| r.frame_3d(f, size.x, &mask, &layers));
}

/// A mesh's vertices in the world: posed by its joints' matrices (a skin's, in place of its transform) or `place`.
fn posed(mesh: &Mesh, pos: &[[f32; 3]], joints: Option<&[Mat4]>, place: Mat4) -> Vec<Vec3> {
    use bevy::mesh::VertexAttributeValues as A;
    let joints = joints.filter(|_| !raea_sys_layers::mutant("bind"));
    let skin = joints.zip(mesh.try_attribute_option(Mesh::ATTRIBUTE_JOINT_INDEX).ok().flatten()).zip(mesh.try_attribute_option(Mesh::ATTRIBUTE_JOINT_WEIGHT).ok().flatten());
    match skin {
        Some(((joints, A::Uint16x4(ix)), A::Float32x4(wt))) => pos.iter().zip(ix).zip(wt).map(|((p, i), w)| {
            let m = (0..4).fold(Mat4::ZERO, |m, k| m + joints.get(i[k] as usize).copied().unwrap_or(Mat4::IDENTITY) * w[k]);
            m.transform_point3(Vec3::from(*p))
        }).collect(),
        _ => pos.iter().map(|p| place.transform_point3(Vec3::from(*p))).collect(),
    }
}

/// The key and name of the nearest entity up from `e` (itself first) with a LayerKey.
fn keyed(world: &World, mut e: Entity) -> Option<(String, String)> {
    loop {
        if let Some(k) = world.get::<LayerKey>(e) {
            return Some((k.0.clone(), world.get::<Name>(e).map_or("mesh".into(), |n| n.as_str().to_string())));
        }
        e = world.get::<ChildOf>(e)?.parent();
    }
}
