//! The views off while a scene's glTF models load, and on again before its first frame (raea #48): runtime/scene3d's
//! await_models.

use bevy::prelude::*;

/// The active cameras turned off while models load, so the render world bins no draws until the scene is whole
/// (#48): it still prepares meshes, textures and pipelines. Binned as each mesh turned ready, the scene's draws went
/// into a multidraw in the order loads landed (Bevy draws a batch set's bins in the order they were added and
/// swap-removes them), and that order decides a depth tie, as where a wall meets the floor: two renders of room3d
/// differed in 147-369 of their 7200 frames. Turned on again (views::on) before the first frame, every view bins the
/// whole scene into new phases in queue order, the same every run. (Turning views off after a draw was binned
/// instead loses what Bevy extracts only on a change.)
pub fn off(world: &mut World) -> Vec<Entity> {
    let mut cams = world.query::<(Entity, &mut Camera)>();
    let mut off = vec![];
    for (e, mut c) in cams.iter_mut(world) {
        if c.is_active {
            c.is_active = false;
            off.push(e);
        }
    }
    off
}

/// The cameras `off` turned off on again, and one update, then each morphed mesh's Mesh3d marked changed so the
/// first frame collects it again. Bevy 0.20 gives a morph mesh its descriptor (prepare_morph_descriptors) only once its
/// instance was collected (collect_meshes_for_gpu_building, which runs after it), and collects an instance only when it
/// changes: one first seen by a view on the update after loading would keep "no morph" for good (bevy-ts morph drew its
/// card unmorphed), where the loading updates used to collect it again by chance.
pub fn on(app: &mut App, cams: Vec<Entity>) {
    let world = app.world_mut();
    for e in cams {
        if let Some(mut c) = world.get_mut::<Camera>(e) {
            c.is_active = true;
        }
    }
    app.update();
    let world = app.world_mut();
    let mut morphed = world.query_filtered::<&mut Mesh3d, With<bevy::mesh::morph::MeshMorphWeights>>();
    for mut m in morphed.iter_mut(world) {
        m.set_changed();
    }
}
