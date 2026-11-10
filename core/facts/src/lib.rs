//! A project as one world of facts (docs/direction.feature, Part 1): entities with components by name, in the form
//! raea/ecs already has (`world.add({ Name: {...} }, { parent })`, `world.find({ with })`), so what raea reads from a
//! composition and what a check asks of it are the same thing. Nothing here is 3D's: a caption, a cut and a cue are
//! entities as a mesh is. The world is data (JSON a TypeScript world is built from, and Rust reads directly).

pub mod components;
pub use components::*;
