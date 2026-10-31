//! bevy_enoki (2D particles). What had to be written by hand: a spawner is a ParticleSpawner over a material handle and
//! its effect a Particle2dEffect asset, neither reflected, so `Sparks` is a reflected component of ours that makes both
//! from plain numbers when it's added. It draws in the scene's 2D overlay (raea-sys-sprite's Draws2d brings its camera
//! up), in pixels from the frame's top left, +y up. enoki draws its spreads from rand::random, unseeded, so a spread
//! above 0 differs from run to run; at 0 (the default) every particle is the same and a render is repeatable.

use bevy::prelude::*;
use bevy_enoki::prelude::*;

/// A stream of particles from the entity it's on, built into an enoki spawner and effect when added.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default)]
#[require(raea_sys_sprite::Draws2d)]
pub struct Sparks {
    /// seconds between bursts
    pub every: f32,
    /// particles a burst
    pub amount: u32,
    /// seconds each lives
    pub lifetime: f32,
    /// pixels a second
    pub speed: f32,
    /// the way they leave (+y up the frame)
    pub direction: Vec2,
    /// how far from `direction` they spread (0 none, 1 all the way round; above 0 differs from run to run)
    pub spread: f32,
    /// size in pixels
    pub size: f32,
    /// linear RGBA
    pub color: Vec4,
    /// downward pull, pixels a second each second
    pub gravity: f32,
}

impl Default for Sparks {
    fn default() -> Self {
        Sparks { every: 0.05, amount: 4, lifetime: 1.0, speed: 120.0, direction: Vec2::Y, spread: 0.0, size: 6.0, color: Vec4::new(1.0, 0.6, 0.2, 1.0), gravity: 0.0 }
    }
}

pub(crate) fn install(app: &mut App) {
    app.add_plugins(EnokiPlugin);
    app.register_type::<Sparks>();
    app.add_systems(Update, build_sparks);
}

fn build_sparks(added: Query<(Entity, &Sparks), Added<Sparks>>, mut effects: ResMut<Assets<Particle2dEffect>>, mut commands: Commands) {
    for (e, s) in &added {
        let effect = effects.add(Particle2dEffect {
            spawn_rate: s.every.max(0.001),
            spawn_amount: s.amount,
            lifetime: Rval::new(s.lifetime, 0.0),
            linear_speed: Some(Rval::new(s.speed, 0.0)),
            direction: Some(Rval::new(s.direction.normalize_or(Vec2::Y), s.spread)),
            scale: Some(Rval::new(s.size, 0.0)),
            color: Some(LinearRgba::new(s.color.x, s.color.y, s.color.z, s.color.w)),
            gravity_direction: Some(Rval::new(Vec2::NEG_Y, 0.0)),
            gravity_speed: Some(Rval::new(s.gravity, 0.0)),
            ..default()
        });
        commands.entity(e).insert((ParticleSpawner::<ColorParticle2dMaterial>::default(), ParticleEffectHandle(effect)));
    }
}

/// The adapter's registration (the declaration's `"adapter": "enoki"`).
pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "enoki", install, group: None, on_demand: false });
}
