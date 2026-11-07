//! Rune over Bevy (docs/targets.feature T6c): the app's tree drawn into a Bevy Image on Bevy's device, with no copy
//! (raea-rune-bevy). `window` opens it in a Bevy window, the tree a layer over Bevy's frame; `shot` draws the check's
//! frames the same way, so a check shows that Bevy's device draws what Rune's own renderer does (`--draw bevy`).

pub mod shot;
pub mod window;

use bevy::prelude::*;
use bevy::window::ExitCondition;

/// A headless Bevy app (no window, no winit) with Rune's layers: what the check and a video draw with.
pub fn headless() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<bevy::winit::WinitPlugin>()
            .set(WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit, close_when_requested: false, ..default() })
            .set(bevy::render::RenderPlugin { synchronous_pipeline_compilation: true, ..default() }),
    );
    app.add_plugins(raea_rune_bevy::RuneLayerPlugin);
    app.finish();
    app.cleanup();
    app
}
