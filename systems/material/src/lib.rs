//! Materials: a row's mesh (primitives and buffers, mesh.rs), its standard material and maps (maps.rs), a model's materials re-covered (surface.rs), decals
//! (decal.rs), and
//! surfaces beyond the standard material: toon (cel) surfaces and their ink outlines (toon.rs), meshes that cast
//! shadows without being drawn (shadow_only.rs), materials written in TypeScript (shader_material.rs, docs/bevy-ts.feature
//! G20).

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use raea_core_program::ir::V;
use raea_sys_frame::*;

pub mod decal;
pub mod fields;
pub mod maps;
pub mod mesh;
pub mod surface;
pub mod toon;
pub mod shadow_only;
pub mod shader_material;
