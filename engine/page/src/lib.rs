//! raea-page: HTML, CSS and React pages in Bevy (docs/blitz-bevy.feature), as one plugin, and a composition's page.
//! The layers under it: raea-sys-page (a Blitz document restyled, laid out and painted by Vello on Bevy's device),
//! raea-runtime-twins (a page's elements as Bevy entities, and a page over a camera), raea-runtime-dom (TSX roots on a
//! page: their scripts, events and media) and raea-runtime-input (pointer and keys to the page they hit). Here: the
//! composition's page (motion.rs, html.rs) and the page handed to a render's encoder (feed.rs).

pub mod feed;
pub mod html;
pub mod motion;
/// B3 H11's scenarios: the hooks together (bench/bridge/run.sh)
#[cfg(test)]
mod scenarios;

use bevy::prelude::*;

/// Pages in an App: the page system, the twins (React roots' op ids from raea-runtime-dom), a page over a camera, and
/// getComputedStyle answered by the page's own cascade (stylo's).
pub struct PagePlugin;

impl Plugin for PagePlugin {
    fn build(&self, app: &mut App) {
        raea_runtime_dom::computed::register();
        app.add_plugins((
            raea_sys_page::PaintPlugin,
            raea_runtime_twins::bridge::BridgePlugin::<raea_runtime_dom::host::Roots>::default(),
            raea_runtime_twins::overlay::OverlayPlugin,
        ));
    }
}
