//! The methods TypeScript calls on a declared Bevy plugin with no reflected API of its own (docs/plugins.feature PL0b).
//! A declaration's `"calls": "<module>"` names a module here (or one of raea-bevy-plugins' own, for calls that need
//! its bridge); tools/bevy-plugins.sh sync writes the module lines and this crate's dependencies by feature, and
//! raea-bevy-plugins' registration adds the module's `calls(app)` to the plugin's install. Each method is a system on
//! bevy_remote's table, so a program (raea/bevy's BevyApp.call) and any BRP client reach it alike.

use bevy::prelude::*;
use bevy_remote::{BrpError, BrpResult, RemoteMethodSystemId, RemoteMethods};
use serde_json::Value;

// the calls a declaration names, by feature
// >>> declared modules: written by tools/bevy-plugins.sh sync; don't edit
#[cfg(feature = "console")]
pub mod console;
#[cfg(feature = "debugdump")]
pub mod debugdump;
#[cfg(feature = "egui")]
pub mod egui;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "kira")]
pub mod kira;
#[cfg(feature = "lookup_curve")]
pub mod lookup_curve;
#[cfg(feature = "navigator")]
pub mod navigator;
#[cfg(feature = "noise")]
pub mod noise;
#[cfg(feature = "noisy")]
pub mod noisy;
#[cfg(feature = "northstar")]
pub mod northstar;
#[cfg(feature = "quinnet")]
pub mod quinnet;
#[cfg(feature = "save")]
pub mod save;
#[cfg(feature = "simple_tilemap")]
pub mod simple_tilemap;
#[cfg(feature = "sprite3d")]
pub mod sprite3d;
#[cfg(feature = "trauma_shake")]
pub mod trauma_shake;
#[cfg(feature = "tweening")]
pub mod tweening;
// <<< declared modules

/// Adds a method to an App's table (after RemotePlugin, which sets the table up): a calls module's way in.
pub fn add_method<M>(app: &mut App, name: &str, handler: impl IntoSystem<In<Option<Value>>, BrpResult, M> + 'static) {
    let id = app.world_mut().register_system(handler);
    app.world_mut().get_resource_or_init::<RemoteMethods>().insert(name, RemoteMethodSystemId::Instant(id));
}

/// Several methods at once: `methods!(app, "noisy/sample" => sample, "noisy/grid" => grid)`.
#[macro_export]
macro_rules! methods {
    ($app:expr, $($name:literal => $handler:expr),+ $(,)?) => {
        $($crate::add_method($app, $name, $handler);)+
    };
}

/// A method's params, deserialized (BRP's INVALID_PARAMS when they don't fit).
pub fn params<T: serde::de::DeserializeOwned>(p: Option<Value>) -> Result<T, BrpError> {
    serde_json::from_value(p.unwrap_or(Value::Null)).map_err(|e| BrpError { code: bevy_remote::error_codes::INVALID_PARAMS, message: e.to_string(), data: None })
}

/// A method's failure, in words.
pub fn fail(message: impl Into<String>) -> BrpError {
    BrpError { code: bevy_remote::error_codes::INTERNAL_ERROR, message: message.into(), data: None }
}

/// The entity a method was given by its bits (what TypeScript holds for one).
pub fn entity(bits: u64) -> Result<Entity, BrpError> {
    Entity::try_from_bits(bits).ok_or_else(|| fail(format!("{bits} isn't an entity")))
}
