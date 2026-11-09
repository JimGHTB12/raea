//! A WGSL source read as text (docs/control.feature FC2): an own uniform struct's fields and their uniform layout
//! (layout), a storage buffer's words in its type's storage layout (storage_layout), and a Material's or UI
//! material's bindings renumbered into raea's slots (ui_bind). raea-sys-shader builds its materials on them.

pub mod layout;
pub mod storage_layout;
pub mod ui_bind;
