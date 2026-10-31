//! bevy_ecs_tilemap (docs/plugins.feature PL4). Its map and tile components are reflected, but a map is only drawn
//! with the parts TilemapBundle and TileBundle add beside them, and some of those aren't reflected
//! (TilemapRenderSettings, the material handle, SyncToRenderWorld), so a map spawned from TypeScript was never
//! extracted. What had to be written by hand: those parts become required components, of a map's TilemapTexture and
//! of a tile's TilePos, each at the bundle's default. A map is then its TilemapTexture, TilemapSize, TilemapTileSize and
//! TilemapGridSize (and an anchor, a type or spacing when not the default); a tile its TilePos, TilemapId and
//! TileTextureIndex. The tiles needn't be in its TileStorage to draw.

use bevy::prelude::*;
use bevy::render::sync_world::SyncToRenderWorld;
use bevy_ecs_tilemap::FrustumCulling;
use bevy_ecs_tilemap::prelude::*;

pub(crate) fn install(app: &mut App) {
    // ldtk and tiled add it too, each guarded; all the registered plugins share one App
    if !app.is_plugin_added::<TilemapPlugin>() {
        app.add_plugins(TilemapPlugin);
    }
    app.register_required_components::<TilemapTexture, TilemapRenderSettings>()
        .register_required_components::<TilemapTexture, TilemapSpacing>()
        .register_required_components::<TilemapTexture, TilemapType>()
        .register_required_components::<TilemapTexture, TileStorage>()
        .register_required_components::<TilemapTexture, TilemapAnchor>()
        .register_required_components::<TilemapTexture, FrustumCulling>()
        .register_required_components::<TilemapTexture, MaterialTilemapHandle<StandardTilemapMaterial>>()
        .register_required_components::<TilemapTexture, Visibility>()
        .register_required_components::<TilemapTexture, SyncToRenderWorld>();
    app.register_required_components::<TilePos, TileTextureIndex>()
        .register_required_components::<TilePos, TileVisible>()
        .register_required_components::<TilePos, TileFlip>()
        .register_required_components::<TilePos, TileColor>()
        .register_required_components::<TilePos, TilePosOld>()
        .register_required_components::<TilePos, SyncToRenderWorld>();
}

pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "tilemap", install, group: None, on_demand: false });
}
