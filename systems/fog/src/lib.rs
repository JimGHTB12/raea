//! Fog as one medium in a grid along the view (froxel.rs), glass seen through that fog (froxel_glass.rs) and flames
//! drawn as volumes lit into it (fire.rs). Each frame's fog is a function of that frame.

use std::collections::BTreeMap;

use bevy::prelude::*;
use raea_core_program::ir::V;
use raea_sys_frame::*;
use raea_sys_frame::bevy_api;
use raea_sys_volume as volume;

pub mod froxel;
pub mod froxel_glass;
pub mod fire;
