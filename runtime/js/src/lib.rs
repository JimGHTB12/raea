//! raea-runtime-js: TypeScript running natively in a Bevy app, under raea's Blitz page (engine/page). What's here is the
//! part the page and raea-runtime-react build on: the JS engines (src/js: JavaScriptCore on Apple, QuickJS elsewhere; the
//! `portable` feature forces QuickJS, and `BEVY_TSX_ENGINE=jsc|quickjs` picks at run time), native calls
//! (src/calls.rs), the World API from engine/bevy-world (src/world.rs), bundling (src/bundle.rs), fetches (src/net.rs,
//! src/html_page.rs), and video and audio decoding (src/media_html.rs). Canvas 2D, the accessibility records, images,
//! offscreen targets and the 3D view element are systems (systems/canvas, systems/a11y, systems/page).
//!
//! Its bevy_ui host (elements as bevy_ui nodes, with their controls, svg, layout snapshot and painter) went in the
//! overhaul (docs/overhaul.md, 10-27): no app mounted it after docs/blitz-bevy.feature B5d, since every page is Blitz's.

pub mod calls;
pub mod html_page;
pub mod js;
pub mod media_html;
#[cfg(feature = "media")]
mod media_play;
#[cfg(feature = "media")]
mod media_sound;
#[cfg(feature = "media")]
pub mod webaudio;
pub mod net;
pub mod os;
pub mod world;

use bevy::prelude::*;

pub use calls::{Answer, HostCalls, SpaceActions};
pub use js::{Engine, JsEngine};

/// The frame number a page's script reads (`__ui_frame`), set by the app before anything is delivered in a frame.
#[derive(Resource, Default)]
pub struct UiFrame(pub u64);

/// BEVY_TSX_TRACE (or the feed's RAEA_FEED_BEVY_TRACE): what the host logs beyond warnings.
pub fn trace() -> Option<String> {
    std::env::var("BEVY_TSX_TRACE").ok().or_else(|| feed_var("BEVY_TRACE"))
}

/// RAEA_FEED_<name>.
pub(crate) fn feed_var(name: &str) -> Option<String> {
    std::env::var(format!("RAEA_FEED_{name}")).ok()
}

/// Every schedule of a page's App, its render world's too, on the single-threaded executor, as live play's engine
/// (raea-render engine.rs, lightspeed F2): a page's systems are few, and the multi-threaded executor spawned a task per
/// system a frame (ZeroHeap: pong's frame at about 4000 Rust allocations against its budget of 172)
pub fn one_thread(app: &mut App) {
    use bevy::ecs::schedule::{Schedules, SingleThreadedExecutor};
    for (_, s) in app.world_mut().resource_mut::<Schedules>().iter_mut() {
        s.set_executor(SingleThreadedExecutor::new());
    }
    if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
        for (_, s) in render.world_mut().resource_mut::<Schedules>().iter_mut() {
            s.set_executor(SingleThreadedExecutor::new());
        }
    }
}
