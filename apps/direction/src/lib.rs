//! How raea takes direction (docs/direction.feature). `raea world` (world.rs): a composition's frame as one world of
//! facts, its 3D, page and timeline as entities with components, printed or queried across media.

mod world;
pub use world::{cli, find, Find, USAGE};
