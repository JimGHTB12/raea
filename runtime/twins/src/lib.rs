//! raea-runtime-twins: a page's elements as Bevy entities (bridge.rs: twins with their boxes, the page's commands,
//! owners and the DOM's events through Bevy's bubbling, docs/blitz-bevy.feature B3) and a page laid over a camera's
//! picture as raea lays it (overlay.rs, H10).

pub mod bridge;
pub mod overlay;

pub use bridge::{ElementBox, OpIds, PageCommands, PageElement, PageQuery};
