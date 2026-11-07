//! The native target's media (docs/targets.feature T3d): where an `<img>` or `<video>` source is (src.rs), its image
//! (images.rs) or player (video.rs), the news @raea/ui hears of them (`media`: load, error, loadedmetadata, ...), and
//! the app's fonts, registered with Rune's text system before it starts (fonts.rs). view.rs builds the page's media
//! elements for raea-rune-page's view.

pub mod fonts;
pub mod images;
pub mod src;
pub mod video;
pub mod view;

pub use src::{base, set_base};
