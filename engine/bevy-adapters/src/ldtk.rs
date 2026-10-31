//! bevy_ecs_ldtk (docs/plugins.feature PL4). A world is its LdtkProjectHandle (reflected; it requires LevelSet,
//! Transform and Visibility itself), but which levels spawn is chosen by LevelSelection, a resource that isn't
//! reflected, so a project spawned from TypeScript spawned no level. What had to be written by hand: `CurrentLevel`,
//! a reflected component of ours that sets LevelSelection to its level's index when it's added or changed. Levels draw through bevy_ecs_tilemap
//! in the scene's 2D overlay (Draws2d brings its camera up).

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

/// The level to show, by its index in the project (0 the first); put it on the LdtkProjectHandle's entity.
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component, Default)]
#[require(raea_sys_sprite::Draws2d)]
pub struct CurrentLevel {
    pub index: usize,
}

pub(crate) fn install(app: &mut App) {
    if !app.is_plugin_added::<LdtkPlugin>() {
        app.add_plugins(LdtkPlugin);
    }
    app.register_type::<CurrentLevel>();
    app.add_systems(Update, select_level);
}

fn select_level(changed: Query<&CurrentLevel, Changed<CurrentLevel>>, mut commands: Commands) {
    if let Some(c) = changed.iter().last() {
        commands.insert_resource(LevelSelection::index(c.index));
    }
}

pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "ldtk", install, group: None, on_demand: false });
}
