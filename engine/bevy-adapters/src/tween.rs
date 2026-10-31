//! bevy_tween (docs/plugins.feature PL13). Its tweens are reflected (TimeRunner, TimeSpan, EaseKind and the
//! Tween<TargetComponent, …> interpolators), but every system that ticks them filters on TimeContext<()>, a marker
//! that isn't reflected, so a runner spawned from TypeScript would never run. What had to be written by hand: the
//! marker becomes a required component of the runner and of each span, so a tween spawned from TypeScript (a
//! TimeRunner, and a child with a TimeSpan, an EaseKind and a Tween) gets it and runs on Bevy's default clock.

use bevy::prelude::*;
use bevy_tween::bevy_time_runner::{TimeContext, TimeRunner, TimeSpan};

pub(crate) fn install(app: &mut App) {
    if !app.is_plugin_added::<bevy_tween::TweenCorePlugin<()>>() {
        app.add_plugins(bevy_tween::DefaultTweenPlugins::default());
    }
    app.register_required_components::<TimeRunner, TimeContext<()>>();
    app.register_required_components::<TimeSpan, TimeContext<()>>();
}

pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "tween", install, group: None, on_demand: false });
}
