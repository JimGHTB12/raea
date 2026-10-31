//! An edit as raea's renderers read it: rows spawned once and keyed per frame (`ir`), each frame's values read off
//! them (`eval`), and every frame's at once, column by column (`timeline`).

pub mod eval;
pub mod ir;
pub mod timeline;
