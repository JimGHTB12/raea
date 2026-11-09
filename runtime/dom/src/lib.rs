//! raea-runtime-dom: TSX roots on a Blitz page (host.rs: each root's JS, its ops applied, its events, timers, iframes
//! and host calls), getComputedStyle (computed.rs) and a page's video and audio elements (raea-runtime-dom-media).

pub mod computed;
pub mod host;
pub use raea_runtime_dom_media as media;
