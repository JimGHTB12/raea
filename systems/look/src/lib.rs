//! Looks: a model's own (pixel art or the site's dithered grain, look.rs), the frame's (look/frame_look.rs) and the
//! paint look's surfaces, every standard material painted in a few flat tones (paint.rs).

use std::collections::{BTreeMap, HashMap};

use bevy::camera::RenderTarget;
use bevy::render::render_resource::TextureFormat;
use bevy::prelude::*;
use raea_core_program::ir::V;
use raea_sys_frame::*;
use raea_sys_frame::bevy_api;
use raea_sys_compose::compose;
use raea_sys_post::outline;
use raea_sys_fog::froxel_glass;

pub mod look;
pub mod paint;
