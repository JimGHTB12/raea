//! bevy_light_2d (2D lights). What had to be written by hand: its lights only light a Camera2d carrying its Light2d,
//! which isn't reflected, and raea makes that camera itself (raea-sys-sprite's 2D overlay). So a scene's 2D lights
//! bring the overlay up (each gets Draws2d) and the overlay gets Light2d, its ambient from `Lighting2d` when an entity
//! carries one (else none: only the lights light the frame).

use bevy::prelude::*;
use bevy_light_2d::prelude::*;
use raea_sys_sprite::{Camera2dOver, Draws2d};

/// The 2D overlay's ambient light, from any entity it's on.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default)]
#[require(Draws2d)]
pub struct Lighting2d {
    /// linear RGB
    pub color: Vec3,
    pub brightness: f32,
}

impl Default for Lighting2d {
    fn default() -> Self {
        Lighting2d { color: Vec3::ONE, brightness: 0.0 }
    }
}

pub(crate) fn install(app: &mut App) {
    app.add_plugins(Light2dPlugin);
    app.register_type::<Lighting2d>();
    app.add_systems(Update, (mark_lights, light_overlay).chain());
}

fn mark_lights(lights: Query<Entity, (Or<(Added<PointLight2d>, Added<SpotLight2d>)>, Without<Draws2d>)>, mut commands: Commands) {
    for e in &lights {
        commands.entity(e).insert(Draws2d);
    }
}

fn light_overlay(overlay: Query<Entity, (With<Camera2dOver>, Without<Light2d>)>, lighting: Query<&Lighting2d>, lights: Query<(), Or<(With<PointLight2d>, With<SpotLight2d>, With<Lighting2d>)>>, mut commands: Commands) {
    if lights.is_empty() {
        return;
    }
    let l = lighting.iter().next().cloned().unwrap_or_default();
    for e in &overlay {
        let ambient_light = AmbientLight2d { color: LinearRgba::rgb(l.color.x, l.color.y, l.color.z).into(), brightness: l.brightness };
        commands.entity(e).insert(Light2d { ambient_light, ..default() });
    }
}

/// The adapter's registration (the declaration's `"adapter": "light_2d"`).
pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "light_2d", install, group: None, on_demand: false });
}
