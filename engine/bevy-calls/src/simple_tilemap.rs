//! bevy_simple_tilemap (the declaration's `"calls": "simple_tilemap"`): a map of tiles set from TypeScript. Its
//! `TileMap` isn't reflected (a tile change is queued and applied into 64 by 64 chunks in Update), so the map is made
//! and edited through calls; the plugin draws it in 2D with a renderer.
//!
//!   simple_tilemap/map    {image, tile: [w, h], columns, rows, at?: [x, y, z]}  -> entity: a map over that sprite
//!                         sheet (cells numbered row by row from the top left), its Transform at `at`
//!   simple_tilemap/set    {entity, tiles: [{at: [x, y] | [x, y, layer], sprite, color?: [r, g, b, a], flip_x?,
//!                         flip_y?} | {at, clear: true}]}                       -> count queued (applied next frame)
//!   simple_tilemap/get    {entity, at}                                         -> {sprite, color, flip_x, flip_y} or null
//!   simple_tilemap/clear  {entity, layer?}                                     -> null: every tile, or one layer's

use bevy::prelude::*;
use bevy_remote::BrpResult;
use bevy_simple_tilemap::{Tile, TileFlags, TileMap};
use serde_json::{Value, json};

use crate::{entity, fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "simple_tilemap/map" => map, "simple_tilemap/set" => set, "simple_tilemap/get" => get, "simple_tilemap/clear" => clear);
}

#[derive(serde::Deserialize)]
struct Map {
    image: String,
    tile: [u32; 2],
    columns: u32,
    rows: u32,
    #[serde(default)]
    at: [f32; 3],
}

#[derive(serde::Deserialize)]
struct Set {
    entity: u64,
    tiles: Vec<TileAt>,
}

#[derive(serde::Deserialize)]
struct TileAt {
    at: Vec<i32>,
    #[serde(default)]
    sprite: u32,
    color: Option<[f32; 4]>,
    #[serde(default)]
    flip_x: bool,
    #[serde(default)]
    flip_y: bool,
    #[serde(default)]
    clear: bool,
}

#[derive(serde::Deserialize)]
struct Get {
    entity: u64,
    at: Vec<i32>,
}

#[derive(serde::Deserialize)]
struct Clear {
    entity: u64,
    layer: Option<i32>,
}

fn pos(at: &[i32]) -> Result<IVec3, bevy_remote::BrpError> {
    match at {
        [x, y] => Ok(IVec3::new(*x, *y, 0)),
        [x, y, z] => Ok(IVec3::new(*x, *y, *z)),
        _ => Err(fail(format!("a tile's at is [x, y] or [x, y, layer], not {at:?}"))),
    }
}

fn tilemap(world: &mut World, bits: u64) -> Result<Mut<'_, TileMap>, bevy_remote::BrpError> {
    let e = entity(bits)?;
    world.get_mut::<TileMap>(e).ok_or_else(|| fail(format!("entity {bits} has no TileMap (simple_tilemap/map makes one)")))
}

fn map(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let m: Map = params(p)?;
    let image = world.resource::<AssetServer>().load(m.image);
    let layout = world.resource_mut::<Assets<TextureAtlasLayout>>().add(TextureAtlasLayout::from_grid(UVec2::from(m.tile), m.columns, m.rows, None, None));
    let e = world.spawn((TileMap::new(image, layout), Transform::from_translation(Vec3::from(m.at)))).id();
    Ok(json!(e.to_bits()))
}

fn set(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let s: Set = params(p)?;
    let tiles = s
        .tiles
        .iter()
        .map(|t| {
            let mut flags = TileFlags::empty();
            flags.set(TileFlags::FLIP_X, t.flip_x);
            flags.set(TileFlags::FLIP_Y, t.flip_y);
            let tile = (!t.clear).then(|| Tile { sprite_index: t.sprite, color: t.color.map_or(Color::WHITE, |[r, g, b, a]| Color::srgba(r, g, b, a)), flags });
            Ok((pos(&t.at)?, tile))
        })
        .collect::<Result<Vec<_>, bevy_remote::BrpError>>()?;
    let n = tiles.len();
    tilemap(world, s.entity)?.set_tiles(tiles);
    Ok(json!(n))
}

fn get(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let g: Get = params(p)?;
    let at = pos(&g.at)?;
    let map = tilemap(world, g.entity)?;
    // a chunk is a square of tiles from its origin, row by row (its side is the crate's, read from the chunk)
    let tile = map.chunks.values().find_map(|c| {
        let side = (c.tiles.len() as f64).sqrt() as i32;
        let (dx, dy) = (at.x - c.origin.x, at.y - c.origin.y);
        (c.origin.z == at.z && (0..side).contains(&dx) && (0..side).contains(&dy)).then(|| c.tiles[(dx + dy * side) as usize].clone()).flatten()
    });
    Ok(match tile {
        Some(t) => {
            let c = t.color.to_srgba();
            json!({ "sprite": t.sprite_index, "color": [c.red, c.green, c.blue, c.alpha], "flip_x": t.flags.contains(TileFlags::FLIP_X), "flip_y": t.flags.contains(TileFlags::FLIP_Y) })
        }
        None => Value::Null,
    })
}

fn clear(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let c: Clear = params(p)?;
    let mut map = tilemap(world, c.entity)?;
    match c.layer {
        Some(l) => map.clear_layer(l),
        None => map.clear(),
    }
    Ok(Value::Null)
}
