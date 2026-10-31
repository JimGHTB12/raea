//! A composition's frames handed to raea-runtime-scene: each frame's 3D walked into its parts once, and the camera's
//! and parts' rows made from them.

use std::collections::BTreeMap;

use raea_core_program::ir::{Program, V};
use raea_core_scene::Item;
use super::Frame;

pub(crate) fn items(f: &Frame) -> Vec<Item> {
    raea_runtime_scene::items_of(&f.scene)
}

/// Each frame's items, walked once (a frame's tree was walked again for each kind of part: 11 times a live frame)
pub(crate) fn all_items(frames: &[(i64, Frame)]) -> Vec<Vec<Item>> {
    frames.iter().map(|(_, f)| items(f)).collect()
}

/// The camera row's 3D columns: where it is and looks (keyed), its lens, the ambient light, the post-processing.
pub(crate) fn camera(frames: &[(i64, Frame)], p: &mut Program, row: &mut BTreeMap<String, V>) {
    raea_runtime_scene::camera(&all_items(frames), p, row)
}

/// The lights, meshes, models and pictures in space, as rows.
pub(crate) fn rows(frames: &[(i64, Frame)], p: &mut Program) {
    raea_runtime_scene::rows(&all_items(frames), p)
}
