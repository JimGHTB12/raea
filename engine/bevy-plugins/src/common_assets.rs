//! bevy_common_assets (the declaration's `"adapter": "common_assets"`): its loaders are generic over the asset type a
//! game deserializes into, so the type is named here: `Data`, any document as JSON (serde_json::Value), and `Row`, one
//! CSV row as column -> text. A game loads JSON, RON, TOML, YAML, MessagePack, CBOR and CSV files through Bevy's
//! AssetServer and reads them from TypeScript:
//!
//!   common_assets/load  {path}  -> null: starts loading (relative to the asset folder, or absolute)
//!   common_assets/get   {path}  -> the document as JSON (a CSV is a list of rows), or null while it loads; a failed
//!                                  load is an error with the loader's message

use std::collections::{BTreeMap, HashMap};

use bevy::asset::{LoadState, UntypedAssetId};
use bevy::prelude::*;
use bevy_common_assets::{cbor::CborAssetPlugin, csv::CsvAssetPlugin, csv::LoadedCsv, json::JsonAssetPlugin, msgpack::MsgPackAssetPlugin, ron::RonAssetPlugin, toml::TomlAssetPlugin, yaml::YamlAssetPlugin};
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::bridge::{add_method, fail, params};

/// A data document, whatever its shape.
#[derive(Asset, TypePath, serde::Deserialize, Debug)]
#[serde(transparent)]
pub struct Data(pub Value);

/// One CSV row: its columns by header.
#[derive(Asset, TypePath, serde::Deserialize, Debug)]
#[serde(transparent)]
pub struct Row(pub BTreeMap<String, String>);

#[derive(Resource, Default)]
struct Loads {
    data: HashMap<String, Handle<Data>>,
    csv: HashMap<String, Handle<LoadedCsv<Row>>>,
}

pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "common_assets", install, group: None, on_demand: false });
}

fn install(app: &mut App) {
    app.add_plugins((
        JsonAssetPlugin::<Data>::new(&["json"]),
        RonAssetPlugin::<Data>::new(&["data.ron"]),
        TomlAssetPlugin::<Data>::new(&["toml"]),
        YamlAssetPlugin::<Data>::new(&["yaml", "yml"]),
        MsgPackAssetPlugin::<Data>::new(&["msgpack"]),
        CborAssetPlugin::<Data>::new(&["cbor"]),
        CsvAssetPlugin::<Row>::new(&["csv"]),
    ))
    .init_resource::<Loads>();
    add_method(app, "common_assets/load", load);
    add_method(app, "common_assets/get", get);
}

#[derive(serde::Deserialize)]
struct Path {
    path: String,
}

fn load(In(p): In<Option<Value>>, server: Res<AssetServer>, mut loads: ResMut<Loads>) -> BrpResult {
    let Path { path } = params(p)?;
    if path.ends_with(".csv") {
        let h = server.load::<LoadedCsv<Row>>(path.clone());
        loads.csv.insert(path, h);
    } else {
        let h = server.load::<Data>(path.clone());
        loads.data.insert(path, h);
    }
    Ok(Value::Null)
}

fn state(server: &AssetServer, id: impl Into<UntypedAssetId>) -> Result<bool, bevy_remote::BrpError> {
    match server.load_state(id) {
        LoadState::Failed(e) => Err(fail(e.to_string())),
        LoadState::Loaded => Ok(true),
        _ => Ok(false),
    }
}

fn get(In(p): In<Option<Value>>, server: Res<AssetServer>, loads: Res<Loads>, data: Res<Assets<Data>>, csv: Res<Assets<LoadedCsv<Row>>>) -> BrpResult {
    let Path { path } = params(p)?;
    if let Some(h) = loads.data.get(&path) {
        if !state(&server, h.id())? {
            return Ok(Value::Null);
        }
        return Ok(data.get(h).map(|d| d.0.clone()).unwrap_or(Value::Null));
    }
    if let Some(h) = loads.csv.get(&path) {
        if !state(&server, h.id())? {
            return Ok(Value::Null);
        }
        return Ok(csv.get(h).map(|c| json!(c.rows.iter().map(|r| &r.0).collect::<Vec<_>>())).unwrap_or(Value::Null));
    }
    Err(fail(format!("{path}: common_assets/load it first")))
}
