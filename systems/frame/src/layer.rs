//! A drawn thing's layer key (docs/review.feature R2a): where its JSX is written, its place in a list, and the uses of
//! the edit's components it was drawn in, as engine/bundle's source stamps give them. A 3D part carries it as this
//! component (runtime/scene bevy_parts makes it a component row of the part), never as a column its kind reads.

use bevy::prelude::*;

/// "file:line:col[#index]|use|use…": the part's stamp, then its use chain, nearest first
#[derive(Component, Reflect, Default, Clone, Debug, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct LayerKey(pub String);

/// The key a stamped element or part's props give it (raea-sys-layers owns it: the page's half reads it too).
pub use raea_sys_layers::key_of;

