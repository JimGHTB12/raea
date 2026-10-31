//! vleue_navigator (the declaration's `"calls": "navigator"`): nav meshes and paths over them. Its NavMesh and the
//! components that keep one updated aren't reflected, so TypeScript builds a mesh from its outline and finds paths
//! through methods. A mesh is an asset in the plugin's Assets<NavMesh>, so its debug gizmos and anything else that
//! reads the asset see it.
//!
//!   navigator/mesh  {edges: [[x, y], ...], obstacles?: [[[x, y], ...], ...], mesh?}  -> mesh (a number); given
//!                   mesh, that one is rebuilt in place
//!   navigator/path  {mesh, from: [x, y], to: [x, y]}  -> {length, points: [[x, y], ...]} (the last point is `to`), or
//!                   null when an end is off the mesh or nothing joins them

use bevy::prelude::*;
use bevy_remote::BrpResult;
use serde_json::{Value, json};
use vleue_navigator::NavMesh;

use crate::{fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "navigator/mesh" => mesh, "navigator/path" => path);
}

/// The meshes built from TypeScript, by number
#[derive(Resource, Default)]
struct Meshes(Vec<Handle<NavMesh>>);

#[derive(serde::Deserialize)]
struct MeshParams {
    edges: Vec<Vec2>,
    #[serde(default)]
    obstacles: Vec<Vec<Vec2>>,
    mesh: Option<usize>,
}

fn mesh(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: MeshParams = params(p)?;
    if p.edges.len() < 3 {
        return Err(fail("a nav mesh's edges need at least 3 points"));
    }
    let built = NavMesh::from_edge_and_obstacles(p.edges, p.obstacles);
    let mut assets = world.get_resource_mut::<Assets<NavMesh>>().ok_or_else(|| fail("no nav meshes: the App needs the navigator plugin"))?;
    let handle = match p.mesh {
        None => assets.add(built),
        Some(n) => {
            let h = world.get_resource_or_init::<Meshes>().0.get(n).cloned().ok_or_else(|| fail(format!("no nav mesh {n}")))?;
            world.resource_mut::<Assets<NavMesh>>().insert(h.id(), built).map_err(|e| fail(e.to_string()))?;
            return Ok(json!(n));
        }
    };
    let mut meshes = world.get_resource_or_init::<Meshes>();
    meshes.0.push(handle);
    Ok(json!(meshes.0.len() - 1))
}

#[derive(serde::Deserialize)]
struct PathParams {
    mesh: usize,
    from: Vec2,
    to: Vec2,
}

fn path(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: PathParams = params(p)?;
    let h = world.get_resource_or_init::<Meshes>().0.get(p.mesh).cloned().ok_or_else(|| fail(format!("no nav mesh {}", p.mesh)))?;
    let assets = world.resource::<Assets<NavMesh>>();
    let nav = assets.get(&h).ok_or_else(|| fail(format!("nav mesh {} is gone", p.mesh)))?;
    Ok(match nav.path(p.from, p.to) {
        Some(path) => json!({ "length": path.length, "points": path.path.iter().map(|v| [v.x, v.y]).collect::<Vec<_>>() }),
        None => Value::Null,
    })
}
