//! A camera row's place among the cameras drawing into one target (Bevy's Camera order, clear_color and viewport:
//! <Camera3d order clear_color viewport>, 3d/two_passes and 3d/split_screen).

use super::*;
use bevy::camera::Viewport;

/// The camera row's `order` among the cameras on its target, `clear` "none" to draw over the ones before without
/// clearing, and its viewport (`vp_x`, `vp_y`, `vp_w`, `vp_h`: physical px from the target's top left), on its Camera.
pub fn camera_pass(world: &mut World, e: Entity, vals: &BTreeMap<String, V>) {
    let Some(mut c) = world.get_mut::<Camera>(e) else { return };
    c.order = vals.get("order").map_or(0, |v| v.num() as isize);
    if matches!(vals.get("clear"), Some(V::S(s)) if s == "none") {
        c.clear_color = ClearColorConfig::None;
    }
    let n = |k: &str| vals.get(k).map(|v| v.num().max(0.0) as u32);
    if let (Some(x), Some(y), Some(w), Some(h)) = (n("vp_x"), n("vp_y"), n("vp_w"), n("vp_h")) {
        c.viewport = Some(Viewport { physical_position: UVec2::new(x, y), physical_size: UVec2::new(w.max(1), h.max(1)), ..default() });
    }
}
