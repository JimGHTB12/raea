//! How raea takes direction (docs/direction.feature). `raea world` (world.rs): a composition's frame as one world of
//! facts, its 3D, page and timeline as entities with components, printed or queried across media.

mod world;
pub use world::{cli, find, Find, USAGE};

// how `raea render` ends: its findings counted last, and 1 on an error not let through (DR3)
mod rendered;
pub use rendered::rendered;

// `raea findings`: the project's record read back, fixed, new and carried (DR3)
pub mod findings;
