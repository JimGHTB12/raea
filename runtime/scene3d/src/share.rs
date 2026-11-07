//! Equal parts share their assets (docs/control.feature FC1a, closing C1): a mesh row's Mesh is made once per shape and
//! a plain StandardMaterial once per value, so N equal parts hold one Handle<Mesh> and one Handle<StandardMaterial>
//! and Bevy's automatic batching draws them as one, as it does for an example that shares its handles.
//! A shared material a frame changes (a part's colour animating) is first copied to that part alone (sync_meshes),
//! so one part's change never shows on another. RAEA_SHARE_ASSETS=0 makes every part its own again.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use bevy::prelude::*;

/// A part whose StandardMaterial other parts hold too: sync_meshes copies it before changing it.
#[derive(Component)]
pub struct SharedMaterial;

#[derive(Resource, Default)]
struct Shared {
    meshes: HashMap<u64, Handle<Mesh>>,
    materials: HashMap<String, Handle<StandardMaterial>>,
}

/// Whether parts share (on unless RAEA_SHARE_ASSETS=0).
pub fn sharing() -> bool {
    std::env::var("RAEA_SHARE_ASSETS").map_or(true, |v| v != "0")
}

/// The mesh for `key` (everything that decides its vertices), made by `make` the first time.
pub fn shared_mesh(world: &mut World, key: &str, make: impl FnOnce() -> Result<Mesh, String>) -> Result<Handle<Mesh>, String> {
    if !sharing() {
        let m = make()?;
        return Ok(world.resource_mut::<Assets<Mesh>>().add(m));
    }
    let mut h = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut h);
    let k = h.finish();
    if let Some(m) = world.get_resource::<Shared>().and_then(|s| s.meshes.get(&k)).cloned() {
        return Ok(m);
    }
    let m = make()?;
    let m = world.resource_mut::<Assets<Mesh>>().add(m);
    world.get_resource_or_insert_with(Shared::default).meshes.insert(k, m.clone());
    Ok(m)
}

/// The material equal to `m`, field for field (its Debug form is the key: every field, a texture by its handle), made
/// the first time; and whether it's shared (to mark the part SharedMaterial).
pub fn shared_material(world: &mut World, m: StandardMaterial) -> (Handle<StandardMaterial>, bool) {
    if !sharing() {
        return (world.resource_mut::<Assets<StandardMaterial>>().add(m), false);
    }
    let key = format!("{m:?}");
    if let Some(h) = world.get_resource::<Shared>().and_then(|s| s.materials.get(&key)).cloned() {
        return (h, true);
    }
    let h = world.resource_mut::<Assets<StandardMaterial>>().add(m);
    world.get_resource_or_insert_with(Shared::default).materials.insert(key, h.clone());
    (h, true)
}
