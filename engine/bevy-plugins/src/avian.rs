//! avian3d's calls: the parts of its API that aren't components (the declaration's `"calls": "avian"`). Its bodies,
//! colliders, velocities and RayCaster/RayHits are reflected components, so the generated TypeScript reaches them with
//! no code here. A one-shot query is a method on SpatialQuery (a SystemParam), and its collision events aren't
//! reflected, so these few methods are written by hand:
//!
//!   avian3d/cast_ray      {origin, direction, max?, solid?, exclude?}  -> {entity, distance, normal, point} | null
//!   avian3d/cast_rays     {rays: [...as above]}                        -> [hit | null]
//!   avian3d/collisions    {}  -> [{a, b}] since the last call: collisions that started (CollisionStart, for the
//!                               bodies with CollisionEventsEnabled)
//!
//! Entities are their bits (Entity::to_bits), as BRP writes them.

use std::sync::{Arc, Mutex};

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::bridge::{add_method, fail, params};

#[derive(serde::Deserialize)]
struct Ray {
    origin: [f32; 3],
    direction: [f32; 3],
    #[serde(default)]
    max: Option<f32>,
    #[serde(default = "yes")]
    solid: bool,
    #[serde(default)]
    exclude: Vec<u64>,
}

fn yes() -> bool {
    true
}

fn cast(q: &SpatialQuery, r: &Ray) -> Result<Value, bevy_remote::BrpError> {
    let dir = Dir3::new(Vec3::from(r.direction)).map_err(|e| fail(format!("direction: {e}")))?;
    let mut filter = SpatialQueryFilter::default();
    for b in &r.exclude {
        filter.excluded_entities.insert(Entity::try_from_bits(*b).ok_or_else(|| fail(format!("exclude: {b} isn't an entity")))?);
    }
    let origin = Vec3::from(r.origin);
    Ok(match q.cast_ray(origin, dir, r.max.unwrap_or(f32::MAX), r.solid, &filter) {
        Some(h) => {
            let p = origin + *dir * h.distance;
            json!({"entity": h.entity.to_bits(), "distance": h.distance, "normal": [h.normal.x, h.normal.y, h.normal.z], "point": [p.x, p.y, p.z]})
        }
        None => Value::Null,
    })
}

fn cast_ray(In(p): In<Option<Value>>, q: SpatialQuery) -> BrpResult {
    cast(&q, &params::<Ray>(p)?)
}

#[derive(serde::Deserialize)]
struct Rays {
    rays: Vec<Ray>,
}

fn cast_rays(In(p): In<Option<Value>>, q: SpatialQuery) -> BrpResult {
    let rs = params::<Rays>(p)?;
    rs.rays.iter().map(|r| cast(&q, r)).collect::<Result<Vec<_>, _>>().map(Value::from)
}

/// Collisions started since the last `avian3d/collisions`.
#[derive(Resource, Clone, Default)]
struct Started(Arc<Mutex<Vec<(Entity, Entity)>>>);

fn collisions(In(_): In<Option<Value>>, s: Res<Started>) -> BrpResult {
    let v: Vec<Value> = s.0.lock().unwrap().drain(..).map(|(a, b)| json!({"a": a.to_bits(), "b": b.to_bits()})).collect();
    Ok(Value::from(v))
}

/// Adds the methods (after avian's PhysicsPlugins, in the plugin's install).
pub fn calls(app: &mut App) {
    crate::bridge::install_remote(app);
    add_method(app, "avian3d/cast_ray", cast_ray);
    add_method(app, "avian3d/cast_rays", cast_rays);
    let started = Started::default();
    app.insert_resource(started.clone());
    app.add_observer(move |e: On<CollisionStart>| {
        started.0.lock().unwrap().push((e.collider1, e.collider2));
    });
    add_method(app, "avian3d/collisions", collisions);
}
