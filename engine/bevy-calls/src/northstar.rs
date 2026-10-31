//! bevy_northstar (the declaration's `"calls": "northstar"`): grids and paths over them. Its Grid is a component that
//! isn't reflected (generic over the neighbourhood, its cells and graph private), so TypeScript builds a grid and finds
//! paths through methods. A grid is an entity with the plugin's own OrdinalGrid3d (the neighbourhood the declaration
//! adds the plugin for), so its agents and debug views read it as one they spawned.
//!
//!   northstar/grid  {size: [w, h] | [w, h, d], walls?: [[x, y] | [x, y, z], ...], chunk?, grid?}  -> grid (an
//!                   entity); w and h at least 3, chunk (at least 3) defaults to 16; given grid, that one is rebuilt
//!   northstar/path  {grid, from, to, mode?: "refined" | "coarse" | "waypoints" | "astar" | "thetastar"}  -> {cost,
//!                   points: [[x, y, z], ...]} (the last point is `to`), or null when nothing joins them

use bevy::prelude::*;
use bevy_northstar::prelude::{GridSettingsBuilder, Nav, OrdinalGrid3d, PathfindArgs};
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::{entity, fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "northstar/grid" => grid, "northstar/path" => path);
}

/// [x, y] or [x, y, z] as a cell
fn cell(v: &[u32]) -> Result<UVec3, bevy_remote::BrpError> {
    match *v {
        [x, y] => Ok(UVec3::new(x, y, 0)),
        [x, y, z] => Ok(UVec3::new(x, y, z)),
        _ => Err(fail(format!("a cell is [x, y] or [x, y, z], not {v:?}"))),
    }
}

#[derive(serde::Deserialize)]
struct GridParams {
    size: Vec<u32>,
    #[serde(default)]
    walls: Vec<Vec<u32>>,
    chunk: Option<u32>,
    grid: Option<u64>,
}

fn grid(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: GridParams = params(p)?;
    let size = cell(&p.size).map(|s| if p.size.len() == 2 { s.with_z(1) } else { s })?;
    if size.x < 3 || size.y < 3 || size.z < 1 {
        return Err(fail(format!("a grid is at least 3 by 3: {:?}", p.size)));
    }
    let chunk = p.chunk.unwrap_or(16);
    if chunk < 3 {
        return Err(fail("chunk is at least 3"));
    }
    let settings = GridSettingsBuilder::new_3d(size.x, size.y, size.z).chunk_size(chunk).build();
    let mut g = OrdinalGrid3d::new(&settings);
    for w in &p.walls {
        let at = cell(w)?;
        if at.cmpge(size).any() {
            return Err(fail(format!("wall {w:?} is outside the grid")));
        }
        g.set_nav(at, Nav::Impassable);
    }
    g.build();
    let e = match p.grid {
        Some(bits) => entity(bits)?,
        None => world.spawn_empty().id(),
    };
    world.get_entity_mut(e).map_err(|_| fail(format!("no grid {e}")))?.insert(g);
    Ok(json!(e.to_bits()))
}

#[derive(serde::Deserialize)]
struct PathParams {
    grid: u64,
    from: Vec<u32>,
    to: Vec<u32>,
    mode: Option<String>,
}

fn path(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: PathParams = params(p)?;
    let e = entity(p.grid)?;
    let g = world.get::<OrdinalGrid3d>(e).ok_or_else(|| fail(format!("{e} has no grid: make one with northstar/grid")))?;
    let args = PathfindArgs::new(cell(&p.from)?, cell(&p.to)?);
    let mut args = match p.mode.as_deref().unwrap_or("refined") {
        "refined" => args.refined(),
        "coarse" => args.coarse(),
        "waypoints" => args.waypoints(),
        "astar" => args.astar(),
        "thetastar" => args.thetastar(),
        m => return Err(fail(format!("mode {m}: refined, coarse, waypoints, astar or thetastar"))),
    };
    Ok(match g.pathfind(&mut args) {
        Some(path) => json!({ "cost": path.cost(), "points": path.path().iter().map(|c| [c.x, c.y, c.z]).collect::<Vec<_>>() }),
        None => Value::Null,
    })
}
