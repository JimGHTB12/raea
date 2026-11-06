//! A Character part's cloth (docs/cloth.md K2): a `model` row's `cloth` (a `.rseq` of the part's own vertices in its
//! bind pose, in the glTF's vertex order, baked by raea/body tools/cloth.ts `--bind`) and its `cloth_frame`, keyed per
//! frame. That frame's positions and normals go on the part's mesh (a copy, so another body wearing the same part is
//! untouched) and the skin poses them as it poses the part's own: its materials, its alpha cut-out, its morphs and its
//! bones stay as they are. The frame drawn is a function of the row at the frame, so `render --at` draws what a
//! straight render draws there.

use std::path::Path;
use std::sync::Arc;

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use raea_core_program::eval::Tup;
use raea_core_program::ir::V;
use raea_sys_frame::*;


#[derive(Component)]
pub struct ClothPlay {
    seq: Arc<raea_io_assets::meshseq::Seq>,
    /// the part's skinned meshes (entity, its copy), in the order the bake lays their vertices; empty until spawned
    parts: Vec<(Entity, Handle<Mesh>)>,
    last: Option<usize>,
}

/// A model row's cloth, opened beside its model (spawn.rs), when it has one.
pub fn spawn(world: &mut World, e: Entity, t: &Tup, inserted: &[Tup], dir: &Path) -> Result<(), String> {
    let Some(V::S(src)) = t.vals.get("cloth") else { return Ok(()) };
    let path = resolve_src(inserted, src, dir)?;
    let seq = Arc::new(raea_io_assets::meshseq::Seq::open(&path).map_err(|e| format!("model {}: cloth: {e}", t.id))?);
    world.entity_mut(e).insert(ClothPlay { seq, parts: Vec::new(), last: None });
    Ok(())
}

fn count(m: &Mesh) -> usize {
    match m.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(v)) => v.len(),
        _ => 0,
    }
}

/// Each cloth's frame for this frame's row, on its part's meshes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_cloth(
    mut commands: Commands,
    frame: Res<FrameRows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut q: Query<(Entity, &mut ClothPlay)>,
    kids: Query<&Children>,
    mut mesh3d: Query<&mut Mesh3d, With<bevy::mesh::skinning::SkinnedMesh>>,
    mut err: ResMut<Error>,
) {
    for (e, mut cp) in &mut q {
        let Some(v) = frame.vals(e) else { continue };
        let k = num(&v, "cloth_frame", 0.0).round().max(0.0) as usize;
        if cp.parts.is_empty() {
            // the model's scene spawned: its skinned meshes copied, each drawn from its copy from now on
            for x in descendants(e, &kids) {
                let Ok(mut m3) = mesh3d.get_mut(x) else { continue };
                let Some(src) = meshes.get(&m3.0).cloned() else { continue };
                let copy = meshes.add(src);
                m3.0 = copy.clone();
                commands.entity(x).insert(bevy::camera::visibility::NoFrustumCulling);
                cp.parts.push((x, copy));
            }
            if cp.parts.is_empty() {
                continue;
            }
        }
        if cp.last == Some(k) {
            continue;
        }
        let fr = match cp.seq.read(k.min(cp.seq.frames().saturating_sub(1))) {
            Ok(fr) => fr,
            Err(x) => {
                err.0 = Some(format!("cloth: {x}"));
                continue;
            }
        };
        let total: usize = cp.parts.iter().filter_map(|(_, h)| meshes.get(h)).map(count).sum();
        if total != fr.pos.len() {
            err.0 = Some(format!("cloth: the bake has {} vertices a frame, the part's meshes {total} (bake it from this part, with --bind)", fr.pos.len()));
            cp.last = Some(k);
            continue;
        }
        let mut at = 0;
        for (_, h) in &cp.parts {
            let Some(mut m) = meshes.get_mut(h) else { continue };
            let n = count(&m);
            m.insert_attribute(Mesh::ATTRIBUTE_POSITION, fr.pos[at..at + n].to_vec());
            if fr.nrm.len() == fr.pos.len() {
                m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, fr.nrm[at..at + n].to_vec());
            }
            at += n;
        }
        cp.last = Some(k);
    }
}
