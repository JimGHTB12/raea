//! Bevy's EnvironmentMapLight from its own prefiltered cubemaps (<EnvironmentMapLight>) and Bevy's Skybox (<Skybox>), as
//! Bevy's examples put both on the camera: .ktx2 files decoded by Bevy's KTX2 reader, the light's intensity as the
//! first frame has it, and both rotations (and the sky's brightness) keyed, so rotate_environment_map turns them every
//! frame.
//!
//! Camera columns: envmap_diffuse_map, envmap_specular_map (the files), envmap_k (cd/m²), envmap_qx..qw (its
//! rotation; keyed); skybox_image (the file), skybox_k (its brightness, cd/m²; keyed), skybox_qx..qw (keyed).

use std::collections::BTreeMap;

use bevy::core_pipeline::Skybox;
use bevy::light::EnvironmentMapLight;
use bevy::prelude::*;
use raea_core_program::ir::V;
use raea_sys_frame::*;

use super::{EnvBake, EnvMaps};

/// a camera whose EnvironmentMapLight or Skybox is its own prefiltered one, not an <Environment>'s
#[derive(Component)]
pub struct Prefiltered;

/// the row's Quat `<name>_qx..qw` (Bevy's identity when it has none)
fn quat(vals: &BTreeMap<String, V>, name: &str) -> Quat {
    let q = Quat::from_xyzw(num(vals, &format!("{name}_qx"), 0.0), num(vals, &format!("{name}_qy"), 0.0), num(vals, &format!("{name}_qz"), 0.0), num(vals, &format!("{name}_qw"), 1.0));
    if q.length_squared() > 0.0 { q } else { Quat::IDENTITY }
}

/// The light and the sky as the first frame has them: each file loaded once (the sky's image is the light's specular
/// handle when it's the same file, as Bevy's asset server gives the one handle)
pub fn prefiltered(world: &mut World, e: Entity, vals: &BTreeMap<String, V>) -> Result<(), String> {
    let mut loaded: Vec<(String, Handle<Image>)> = Vec::new();
    let mut load = |world: &mut World, what: &str, p: &str| -> Result<Handle<Image>, String> {
        if let Some((_, h)) = loaded.iter().find(|(q, _)| q == p) {
            return Ok(h.clone());
        }
        let h = image_file(world, what, p)?;
        loaded.push((p.to_string(), h.clone()));
        Ok(h)
    };
    if let (Some(V::S(d)), Some(V::S(s))) = (vals.get("envmap_diffuse_map"), vals.get("envmap_specular_map")) {
        let (diffuse_map, specular_map) = (load(world, "EnvironmentMapLight", d)?, load(world, "EnvironmentMapLight", s)?);
        world.entity_mut(e).insert((EnvironmentMapLight { diffuse_map, specular_map, intensity: num(vals, "envmap_k", 1.0), rotation: quat(vals, "envmap"), ..default() }, Prefiltered));
    }
    if let Some(V::S(img)) = vals.get("skybox_image") {
        let image = if img.is_empty() { None } else { Some(load(world, "Skybox", img)?) };
        world.entity_mut(e).insert((Skybox { image, brightness: num(vals, "skybox_k", 0.0), rotation: quat(vals, "skybox") }, Prefiltered));
    }
    Ok(())
}

/// Each frame's rotations and the sky's brightness
pub fn sync_prefiltered(frame: Res<FrameRows>, mut q: Query<(Entity, Option<&mut EnvironmentMapLight>, Option<&mut Skybox>), (With<Prefiltered>, Without<EnvMaps>, Without<EnvBake>)>) {
    for (e, light, sky) in &mut q {
        let Some(v) = frame.vals(e) else { continue };
        if let Some(mut l) = light.filter(|_| v.contains_key("envmap_qw")) {
            let r = quat(&v, "envmap");
            if l.rotation != r {
                l.rotation = r;
            }
        }
        if let Some(mut s) = sky.filter(|_| v.contains_key("skybox_qw")) {
            let (r, k) = (quat(&v, "skybox"), num(&v, "skybox_k", 0.0));
            if s.rotation != r || s.brightness != k {
                s.rotation = r;
                s.brightness = k;
            }
        }
    }
}
