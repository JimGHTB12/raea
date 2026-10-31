//! bevy_rapier2d (docs/plugins.feature PL3). What had to be written by hand: rapier's Collider holds a shape object
//! and isn't reflected upstream, so a body spawned from TypeScript could have no collider.
//! `ColliderShape` is a reflected component of ours that builds the Collider from plain numbers when it's added or
//! changed; RigidBody, Velocity, Friction and the rest are rapier's own, reflected.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// A 2D collider's shape, built into rapier's Collider on the entity it's on (metres, +y up).
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default)]
pub enum ColliderShape {
    Ball { radius: f32 },
    /// half of its size on each axis
    Cuboid { half_size: Vec2 },
    /// along y: half the straight part's height, and the radius of its ends
    Capsule { half_height: f32, radius: f32 },
}

impl Default for ColliderShape {
    fn default() -> Self {
        Self::Ball { radius: 0.5 }
    }
}

rapier_adapter!("rapier2d");

fn build_colliders(changed: Query<(Entity, &ColliderShape), Changed<ColliderShape>>, mut commands: Commands) {
    for (e, s) in &changed {
        let c = match *s {
            ColliderShape::Ball { radius } => Collider::ball(radius),
            ColliderShape::Cuboid { half_size: h } => Collider::cuboid(h.x, h.y),
            ColliderShape::Capsule { half_height, radius } => Collider::capsule_y(half_height, radius),
        };
        commands.entity(e).insert(c);
    }
}
