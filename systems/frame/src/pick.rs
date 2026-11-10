//! The part under a point of the frame (docs/preview.feature PV1a): the nearest mesh along the pointer's ray from the
//! frame's camera, as the layer pass draws it (layer.rs: the same camera, skinned meshes as posed), answered as the
//! nearest part up from it with a LayerKey. What the live stage's pick answers over 3D; a render's layers answer the
//! same question from its mask.

use bevy::prelude::*;

use crate::layer::{LayerKey, posed};

/// A part hit: its key when it or a parent is stamped, its name, where the ray met it and how far along, and the box
/// on the frame (x, y, w, h in 0..1) of every mesh of that part.
#[derive(Clone, Debug, PartialEq)]
pub struct PartHit {
    pub key: Option<String>,
    pub name: String,
    pub entity: Entity,
    pub point: Vec3,
    pub distance: f32,
    pub rect: [f32; 4],
}

/// The part under (`nx`, `ny`), 0..1 across and down the frame; None over no mesh, or with no 3D camera.
pub fn part_at(world: &mut World, nx: f32, ny: f32) -> Option<PartHit> {
    let mut cams = world.query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>();
    let (cam, at) = cams.iter(world).filter(|(c, _)| c.is_active).min_by_key(|(c, _)| c.order)?;
    let ndc = |z: f32| cam.ndc_to_world(at, Vec3::new(nx * 2.0 - 1.0, 1.0 - ny * 2.0, z));
    let (near, far) = (ndc(1.0)?, ndc(0.01)?);
    let dir = (far - near).normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }
    let clip_from_world = cam.clip_from_view() * Mat4::from(at.affine().inverse());
    let mut parts = world.query::<(Entity, &Mesh3d, &GlobalTransform, Option<&InheritedVisibility>, Option<&bevy::mesh::skinning::SkinnedMesh>)>();
    let (meshes, binds) = (world.resource::<Assets<Mesh>>(), world.resource::<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>());
    // each mesh drawn: the part it belongs to and its box on the frame, and the nearest hit among them
    let mut drawn: Vec<(Entity, [f32; 4])> = Vec::new();
    let mut best: Option<(f32, Vec3, Entity)> = None;
    for (e, m, place, shown, skin) in parts.iter(world) {
        if shown.is_some_and(|s| !s.get()) {
            continue;
        }
        let Some(mesh) = meshes.get(&m.0).filter(|m| m.primitive_topology() == bevy::mesh::PrimitiveTopology::TriangleList) else { continue };
        let Ok(Some(bevy::mesh::VertexAttributeValues::Float32x3(pos))) = mesh.try_attribute_option(Mesh::ATTRIBUTE_POSITION) else { continue };
        let joints = skin.and_then(|s| Some((s, binds.get(&s.inverse_bindposes)?))).map(|(s, b)| s.joints.iter().zip(b.iter()).map(|(j, ib)| world.get::<GlobalTransform>(*j).map_or(Mat4::IDENTITY, |g| Mat4::from(g.affine()) * *ib)).collect::<Vec<_>>());
        let posed = posed(mesh, pos, joints.as_deref(), Mat4::from(place.affine()));
        let root = root_of(world, e);
        let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        for p in &posed {
            let c = clip_from_world * p.extend(1.0);
            if c.w > 1e-5 {
                let s = Vec2::new((c.x / c.w + 1.0) * 0.5, (1.0 - c.y / c.w) * 0.5);
                (lo, hi) = (lo.min(s), hi.max(s));
            }
        }
        if lo.x <= hi.x {
            drawn.push((root, [lo.x, lo.y, hi.x, hi.y]));
        }
        let corners: Vec<usize> = match mesh.try_indices_option() {
            Ok(Some(ix)) => ix.iter().collect(),
            _ => (0..posed.len()).collect(),
        };
        for t in corners.chunks_exact(3) {
            let (Some(p0), Some(p1), Some(p2)) = (posed.get(t[0]), posed.get(t[1]), posed.get(t[2])) else { continue };
            if let Some(d) = ray_tri(near, dir, *p0, *p1, *p2) {
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, near + dir * d, root));
                }
            }
        }
    }
    let (distance, point, root) = best?;
    let rect = drawn.iter().filter(|(r, _)| *r == root).fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |a, (_, b)| [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])]);
    let (x0, y0, x1, y1) = (rect[0].clamp(0.0, 1.0), rect[1].clamp(0.0, 1.0), rect[2].clamp(0.0, 1.0), rect[3].clamp(0.0, 1.0));
    Some(PartHit {
        key: world.get::<LayerKey>(root).map(|k| k.0.clone()),
        name: world.get::<Name>(root).map_or("mesh".into(), |n| n.as_str().to_string()),
        entity: root,
        point,
        distance,
        rect: [x0, y0, x1 - x0, y1 - y0],
    })
}

/// The part a mesh belongs to: the nearest entity up from it (itself first) with a LayerKey, else the mesh itself.
fn root_of(world: &World, e: Entity) -> Entity {
    let mut at = e;
    loop {
        if world.get::<LayerKey>(at).is_some() {
            return at;
        }
        match world.get::<ChildOf>(at) {
            Some(p) => at = p.parent(),
            None => return e,
        }
    }
}

/// Where along the ray (origin `o`, unit direction `d`) it meets the triangle, either face (Möller-Trumbore).
pub fn ray_tri(o: Vec3, d: Vec3, p0: Vec3, p1: Vec3, p2: Vec3) -> Option<f32> {
    let (e1, e2) = (p1 - p0, p2 - p0);
    let pv = d.cross(e2);
    let det = e1.dot(pv);
    if det.abs() < 1e-12 {
        return None;
    }
    let tv = o - p0;
    let u = tv.dot(pv) / det;
    let qv = tv.cross(e1);
    let v = d.dot(qv) / det;
    let t = e2.dot(qv) / det;
    (u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > 0.0).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ray_meets_the_triangle_in_front_of_it_and_not_one_beside_or_behind() {
        let (a, b, c) = (Vec3::new(-1.0, -1.0, -5.0), Vec3::new(1.0, -1.0, -5.0), Vec3::new(0.0, 1.0, -5.0));
        assert_eq!(ray_tri(Vec3::ZERO, Vec3::NEG_Z, a, b, c), Some(5.0));
        assert_eq!(ray_tri(Vec3::ZERO, Vec3::NEG_Z, c, b, a), Some(5.0), "either face");
        assert_eq!(ray_tri(Vec3::new(3.0, 0.0, 0.0), Vec3::NEG_Z, a, b, c), None, "beside it");
        assert_eq!(ray_tri(Vec3::ZERO, Vec3::Z, a, b, c), None, "behind the ray");
    }
}
