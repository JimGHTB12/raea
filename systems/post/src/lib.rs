//! The camera's passes after the scene is drawn, each a function of the frame: the lens (Fisheye, LensDistortion), motion
//! blur, Bevy's depth of field and effect stack patched to keep the frame's alpha, and ink outlines.

use std::collections::BTreeMap;

use bevy::prelude::*;
use raea_core_program::ir::V;
use raea_sys_frame::*;
use raea_sys_frame::bevy_api;

pub mod lens;
pub mod motion_blur;
pub mod dof;
pub mod effect_stack;
pub mod outline;
