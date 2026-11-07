//! A field of grass (`<Grass>` in raea/3d): a stylized meadow's clumps of blades over a box of ground, made by the
//! engine from a few grids the edit samples once (the ground's height, clumps per square metre, which colour patch,
//! and a size), instead of a placement per clump sent from the page. The box is cut into square tiles; each tile is
//! one mesh, as detailed as its distance from the camera asks (`lod`: near tiles every clump and every blade in three
//! segments; farther ones a share of the clumps, fewer and wider single-segment blades), made again only when the
//! camera moves it across a step. So a frame's cost is the blades it can show, and a render of frame N, which places
//! the camera before it draws, has the tiles play has.
//!
//! The blades are the meadow's (Dylearn's Waterfowl grass): tapered, leaning, root to tip in their patch's two
//! greens with a warm highlight on the tip, their normals the ground's (a slope turned from the sun darkens as one).
//! They're cut-outs (a masked material) so the paint look's wind bends them; each vertex's second UV is how far up its
//! blade it is, which the paint's vertex stage bends by on any slope (paint_vertex.wesl).
//!
//! The row: kind `grass`, its `grass` column the spec as JSON (`Spec`), `shadow` (cast, 0 or 1: none by default).

use std::collections::BTreeMap;
use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::Aabb;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use raea_core_program::ir::V;
use serde_json::Value as J;

mod spec;
pub use spec::*;
#[cfg(test)]
mod tests;

pub fn install(app: &mut App) {
    app.add_systems(PostUpdate, retile.before(bevy::transform::TransformSystems::Propagate).before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate));
}

/// A field on its row's entity: its spec, its material, and each tile's mesh at the detail it was made for.
#[derive(Component)]
pub struct Field {
    spec: Arc<Spec>,
    material: Handle<StandardMaterial>,
    shadow: bool,
    tiles: Vec<Tile>,
}

struct Tile {
    ix: u32,
    iz: u32,
    /// the step it was made at (None: not yet)
    lod: Option<usize>,
    entity: Option<Entity>,
}

/// A grass row: its field, tiles made as the camera first sees them (retile).
pub fn spawn(world: &mut World, e: Entity, vals: &BTreeMap<String, V>) -> Result<(), String> {
    let Some(V::S(s)) = vals.get("grass") else { return Err("a grass row without its spec (`grass`)".into()) };
    let j: J = serde_json::from_str(s).map_err(|err| format!("grass: its spec isn't JSON: {err}"))?;
    let spec = Spec::from_json(&j)?;
    let material = world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        double_sided: true,
        cull_mode: None,
        // a cut-out, so the paint look's wind bends it (its blades are opaque: every vertex's alpha is 1)
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });
    let (nx, nz) = spec.tiles();
    let tiles = (0..nz).flat_map(|iz| (0..nx).map(move |ix| Tile { ix, iz, lod: None, entity: None })).collect();
    let shadow = matches!(vals.get("shadow"), Some(V::N(x)) if *x > 0.0);
    world.entity_mut(e).insert((Field { spec: Arc::new(spec), material, shadow, tiles }, Transform::IDENTITY, Visibility::Visible));
    Ok(())
}

/// Each frame: each field's tiles at the detail the camera's distance asks, a tile made again only when it changes
/// step. The camera's own transform (not the propagated one: this runs before propagation, so the tiles it makes are
/// in this frame's visibility).
pub fn retile(
    mut commands: Commands,
    mut fields: Query<(Entity, &mut Field)>,
    cams: Query<(&Camera, &Transform), With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(eye) = cams.iter().filter(|(c, _)| c.is_active).min_by_key(|(c, _)| c.order).map(|(_, t)| t.translation) else { return };
    for (fe, mut f) in &mut fields {
        let f = &mut *f;
        for t in &mut f.tiles {
            let want = f.spec.lod_at(f.spec.distance(t.ix, t.iz, eye));
            if t.lod == Some(want) && t.entity.is_some() {
                continue;
            }
            t.lod = Some(want);
            if let Some(old) = t.entity.take() {
                commands.entity(old).despawn();
            }
            let Some((mesh, aabb)) = f.spec.tile_mesh(t.ix, t.iz, want) else { continue };
            let mut ec = commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(f.material.clone()), Transform::IDENTITY, Visibility::Inherited, aabb, ChildOf(fe)));
            if !f.shadow {
                ec.insert(NotShadowCaster);
            }
            t.entity = Some(ec.id());
        }
    }
}

/// A tile's blades as one mesh: positions in the world, the ground's normal, colours, the blade's (side, height) as
/// its UV and (height, its clump's phase) as its second UV, for the wind.
pub(crate) struct Blades {
    pub pos: Vec<[f32; 3]>,
    pub nrm: Vec<[f32; 3]>,
    pub col: Vec<[f32; 4]>,
    pub uv: Vec<[f32; 2]>,
    pub uv1: Vec<[f32; 2]>,
    pub idx: Vec<u32>,
}

impl Blades {
    pub(crate) fn new() -> Blades {
        Blades { pos: Vec::new(), nrm: Vec::new(), col: Vec::new(), uv: Vec::new(), uv1: Vec::new(), idx: Vec::new() }
    }

    /// The mesh and its bounds (widened by how far the wind can push a tip), or None when the tile has no blades.
    pub(crate) fn mesh(self, sway: f32) -> Option<(Mesh, Aabb)> {
        if self.pos.is_empty() {
            return None;
        }
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for p in &self.pos {
            lo = lo.min(Vec3::from_array(*p));
            hi = hi.max(Vec3::from_array(*p));
        }
        let aabb = Aabb::from_min_max(lo - Vec3::splat(sway), hi + Vec3::splat(sway));
        let mut m = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.pos);
        m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.nrm);
        m.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uv);
        m.insert_attribute(Mesh::ATTRIBUTE_UV_1, self.uv1);
        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.col);
        m.insert_indices(Indices::U32(self.idx));
        Some((m, aabb))
    }
}
