//! bevy_kira_audio (the declaration's `"calls": "kira"`): sounds played and steered from TypeScript. Its Audio channel
//! is a resource of queued commands and its instances are assets, neither reflected, so a program plays through these
//! and names an instance by the number play returns. All on the main channel (Audio).
//!
//!   kira/play    {path, volume?: dB, rate?, looped?, paused?}  -> instance (a number); the file through the AssetServer
//!   kira/state   {instance}         -> {state: "Queued" | "Playing" | "Paused" | "Stopping" | "Stopped" | ..., position}
//!   kira/stop    {instance?}        -> null; no instance: everything on the channel
//!   kira/volume  {db, instance?}    -> null; no instance: the channel's volume

use bevy::prelude::*;
use bevy_kira_audio::prelude::{Audio, AudioControl, AudioInstance, AudioSource, AudioTween, Decibels};
use bevy_kira_audio::PlaybackState;
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::{fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "kira/play" => play, "kira/state" => state, "kira/stop" => stop, "kira/volume" => volume);
}

/// The instances TypeScript played, by the number it was given (made by the first kira/play)
#[derive(Resource, Default)]
struct Instances(Vec<Handle<AudioInstance>>);

#[derive(serde::Deserialize)]
struct Play {
    path: String,
    volume: Option<f32>,
    rate: Option<f64>,
    #[serde(default)]
    looped: bool,
    #[serde(default)]
    paused: bool,
}

#[derive(serde::Deserialize, Default)]
struct Of {
    instance: Option<usize>,
}

#[derive(serde::Deserialize)]
struct Volume {
    db: f32,
    instance: Option<usize>,
}

fn play(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: Play = params(p)?;
    let source: Handle<AudioSource> = world.resource::<AssetServer>().load(p.path);
    let handle = {
        let audio = world.resource::<Audio>();
        let mut cmd = audio.play(source);
        if let Some(db) = p.volume {
            cmd.with_volume(Decibels(db));
        }
        if let Some(rate) = p.rate {
            cmd.with_playback_rate(rate);
        }
        if p.looped {
            cmd.looped();
        }
        if p.paused {
            cmd.paused();
        }
        cmd.handle()
    };
    let mut instances = world.get_resource_or_init::<Instances>();
    instances.0.push(handle);
    Ok(json!(instances.0.len() - 1))
}

fn handle(world: &World, n: usize) -> Result<Handle<AudioInstance>, bevy_remote::BrpError> {
    world.get_resource::<Instances>().and_then(|i| i.0.get(n)).cloned().ok_or_else(|| fail(format!("no instance {n} (kira/play makes one)")))
}

fn state(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Of { instance } = params(p)?;
    let h = handle(world, instance.ok_or_else(|| fail("kira/state: give an instance"))?)?;
    let s = world.resource::<Audio>().state(&h);
    let name = match s {
        PlaybackState::Queued => "Queued",
        PlaybackState::Playing { .. } => "Playing",
        PlaybackState::Paused { .. } => "Paused",
        PlaybackState::Pausing { .. } => "Pausing",
        PlaybackState::Stopped => "Stopped",
        PlaybackState::Stopping { .. } => "Stopping",
        PlaybackState::WaitingToResume { .. } => "WaitingToResume",
        PlaybackState::Resuming { .. } => "Resuming",
    };
    Ok(json!({ "state": name, "position": s.position() }))
}

fn stop(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Of { instance } = if p.is_some() { params(p)? } else { Of::default() };
    match instance {
        Some(n) => {
            let h = handle(world, n)?;
            if let Some(mut i) = world.resource_mut::<Assets<AudioInstance>>().get_mut(&h) {
                i.stop(AudioTween::default());
            }
        }
        None => drop(world.resource::<Audio>().stop()),
    }
    Ok(Value::Null)
}

fn volume(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let v: Volume = params(p)?;
    match v.instance {
        Some(n) => {
            let h = handle(world, n)?;
            if let Some(mut i) = world.resource_mut::<Assets<AudioInstance>>().get_mut(&h) {
                i.set_decibels(Decibels(v.db), AudioTween::default());
            }
        }
        None => drop(world.resource::<Audio>().set_volume(Decibels(v.db))),
    }
    Ok(Value::Null)
}
