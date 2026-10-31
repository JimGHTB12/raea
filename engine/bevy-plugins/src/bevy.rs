//! Bevy's own optional features (the declaration's builtin "bevy" entry): gamepads (gilrs), RectLight's LUTs, PCSS,
//! the pbr texture features, picking backends, gizmos, dev tools, hot reload, every image format, meshlets and Solari.
//! They need no plugin of raea's: turning a feature on puts its plugin in Bevy's DefaultPlugins, which raea-render's Engine
//! builds. One exception is handled here: bevy_audio's AudioPlugin plays to the device, not into raea's mixer, so a
//! render wouldn't hear it and a play would hear it twice. The Engine leaves it out; a TypeScript App that wants
//! Bevy's audio asks for it by name (`plugins: ["bevy_audio"]`).

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "bevy", install: |_| {}, group: Some(group), on_demand: false });
    raea_runtime_plugins::register(raea_runtime_plugins::Registered {
        name: "bevy_audio",
        install: |app| {
            app.add_plugins(bevy::audio::AudioPlugin::default());
        },
        group: None,
        on_demand: true,
    });
}

fn group(b: PluginGroupBuilder) -> PluginGroupBuilder {
    b.disable::<bevy::audio::AudioPlugin>()
}
