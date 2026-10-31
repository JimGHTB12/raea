//! Any asset through Bevy's AssetServer, for every asset plugin at once (docs/plugins.feature PL5): a plugin that adds a
//! loader (obj, vox, aseprite, a game's own) has nothing of its own to call, so a program loads a file and reads how it
//! went with these, as `raea/input` drives every input plugin.
//!
//!   raea/asset.load   {path, type?}  -> id (a number); type, an asset's type path ("bevy_mesh::mesh::Mesh"), picks
//!                                       the loader when an extension has several; without it the extension decides
//!   raea/asset.state  {id}           -> {state: "Loading" | "Loaded" | "Failed", type?, error?, vertices?}: Loaded once
//!                                       the asset and everything it depends on are in; type is its type path, and a
//!                                       Mesh gives its vertex count

use std::any::TypeId;

use bevy::asset::{LoadedUntypedAsset, RecursiveDependencyLoadState, UntypedHandle};
use bevy::prelude::*;
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::bridge::{add_method, fail, params};

pub(crate) fn install(app: &mut App) {
    add_method(app, "raea/asset.load", load);
    add_method(app, "raea/asset.state", state);
}

/// What raea/asset.load started, by the number it gave: a handle of a known type, or a load whose type the file decides
enum Loading {
    Typed(UntypedHandle),
    Untyped(Handle<LoadedUntypedAsset>),
}

#[derive(Resource, Default)]
struct Loads(Vec<Loading>);

#[derive(serde::Deserialize)]
struct Load {
    path: String,
    #[serde(rename = "type")]
    ty: Option<String>,
}

#[derive(serde::Deserialize)]
struct Of {
    id: usize,
}

fn load(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: Load = params(p)?;
    let server = world.get_resource::<AssetServer>().ok_or_else(|| fail("no AssetServer: the App needs AssetPlugin (\"assets\" in headless)"))?;
    let l = match p.ty {
        Some(t) => {
            let reg = world.resource::<AppTypeRegistry>().read();
            let id = reg.get_with_type_path(&t).ok_or_else(|| fail(format!("no asset type {t:?} is registered")))?.type_id();
            Loading::Typed(server.load_builder().load_erased(id, p.path))
        }
        None => Loading::Untyped(server.load_builder().load_untyped(p.path)),
    };
    let mut loads = world.get_resource_or_init::<Loads>();
    loads.0.push(l);
    Ok(json!(loads.0.len() - 1))
}

fn state(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: Of = params(p)?;
    let server = world.resource::<AssetServer>();
    let loading = world.get_resource::<Loads>().and_then(|l| l.0.get(p.id)).ok_or_else(|| fail(format!("no load {}", p.id)))?;
    // an untyped load is first a LoadedUntypedAsset, which holds the asset's own handle once the file has said its type
    let handle = match loading {
        Loading::Typed(h) => Some(h.clone()),
        Loading::Untyped(h) => world.resource::<Assets<LoadedUntypedAsset>>().get(h).map(|l| l.handle.clone()),
    };
    let watched = match (loading, &handle) {
        (_, Some(h)) => h.id(),
        (Loading::Untyped(h), None) => h.id().untyped(),
        (Loading::Typed(_), None) => unreachable!(),
    };
    let (st, error) = match server.recursive_dependency_load_state(watched) {
        RecursiveDependencyLoadState::Loaded => ("Loaded", None),
        RecursiveDependencyLoadState::Failed(e) => ("Failed", Some(e.to_string())),
        _ => ("Loading", None),
    };
    let mut out = json!({ "state": st });
    if let Some(e) = error {
        out["error"] = json!(e);
    }
    if let Some(h) = handle.filter(|_| st == "Loaded") {
        // its type path when reflected, else the name its loader gives (bevy_vox_scene's VoxelScene isn't reflected)
        let reg = world.resource::<AppTypeRegistry>().read();
        out["type"] = json!(match reg.get(h.type_id()) {
            Some(r) => r.type_info().type_path().to_string(),
            None => bevy::tasks::block_on(server.get_asset_loader_with_asset_type_id(h.type_id())).map_or_else(|_| format!("{:?}", h.type_id()), |l| l.asset_type_name().to_string()),
        });
        if h.type_id() == TypeId::of::<Mesh>()
            && let Some(m) = world.get_resource::<Assets<Mesh>>().and_then(|a| a.get(h.id().typed::<Mesh>()))
        {
            out["vertices"] = json!(m.count_vertices());
        }
    }
    Ok(out)
}
