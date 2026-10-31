//! bevy_tweening (the declaration's `"calls": "tweening"`): a tween from TypeScript, by its lens's name and numbers.
//! A tween holds its lens as a boxed closure (a trait object), so nothing about it can be reflected; this names the
//! crate's own lenses and builds the tween from plain values, on the entity whose component it animates.
//!
//!   tweening/tween    {entity, lens, start, end, duration (s), ease?: "QuadraticInOut" (Bevy's EaseFunction by
//!                     name; default Linear), repeat?: n | "infinite", mirrored?, speed?}  -> null
//!   tweening/playing  {entity}  -> true while its tween runs (a finished one is removed)
//!
//! lenses, on the entity's Transform: position, scale (start and end [x, y, z]), rotation ([x, y, z, w]), rotate_x,
//! rotate_y and rotate_z (radians); colours, start and end sRGB [r, g, b, a] from 0 to 1: sprite_color (its Sprite),
//! text_color (its TextColor) and background_color (a UI node's BackgroundColor).

use std::time::Duration;

use bevy::prelude::*;
use bevy_remote::BrpResult;
use bevy_tweening::lens::{SpriteColorLens, TextColorLens, TransformPositionLens, TransformRotateXLens, TransformRotateYLens, TransformRotateZLens, TransformRotationLens, TransformScaleLens, UiBackgroundColorLens};
use bevy_tweening::{RepeatCount, RepeatStrategy, Tween, TweenAnim};
use serde_json::{Value, json};

use crate::{entity, fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "tweening/tween" => tween, "tweening/playing" => playing);
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Repeat {
    Times(u32),
    Forever(String),
}

#[derive(serde::Deserialize)]
struct Ask {
    entity: u64,
    lens: String,
    start: Value,
    end: Value,
    duration: f32,
    ease: Option<EaseFunction>,
    repeat: Option<Repeat>,
    #[serde(default)]
    mirrored: bool,
    speed: Option<f64>,
}

#[derive(serde::Deserialize)]
struct Of {
    entity: u64,
}

fn num<T: serde::de::DeserializeOwned>(v: &Value, lens: &str) -> Result<T, bevy_remote::BrpError> {
    serde_json::from_value(v.clone()).map_err(|e| fail(format!("{lens}'s start and end: {e}")))
}

fn rgba(v: &Value, lens: &str) -> Result<Color, bevy_remote::BrpError> {
    let [r, g, b, a] = num::<[f32; 4]>(v, lens)?;
    Ok(Color::srgba(r, g, b, a))
}

fn tween(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let a: Ask = params(p)?;
    let e = entity(a.entity)?;
    if world.get_entity(e).is_err() {
        return Err(fail(format!("no entity {}", a.entity)));
    }
    let ease = a.ease.unwrap_or(EaseFunction::Linear);
    let d = Duration::from_secs_f32(a.duration.max(0.0));
    let (s, t, l) = (&a.start, &a.end, a.lens.as_str());
    let tw = match l {
        "position" => Tween::new(ease, d, TransformPositionLens { start: Vec3::from(num::<[f32; 3]>(s, l)?), end: Vec3::from(num::<[f32; 3]>(t, l)?) }),
        "scale" => Tween::new(ease, d, TransformScaleLens { start: Vec3::from(num::<[f32; 3]>(s, l)?), end: Vec3::from(num::<[f32; 3]>(t, l)?) }),
        "rotation" => Tween::new(ease, d, TransformRotationLens { start: Quat::from_array(num(s, l)?), end: Quat::from_array(num(t, l)?) }),
        "rotate_x" => Tween::new(ease, d, TransformRotateXLens { start: num(s, l)?, end: num(t, l)? }),
        "rotate_y" => Tween::new(ease, d, TransformRotateYLens { start: num(s, l)?, end: num(t, l)? }),
        "rotate_z" => Tween::new(ease, d, TransformRotateZLens { start: num(s, l)?, end: num(t, l)? }),
        "sprite_color" => Tween::new(ease, d, SpriteColorLens { start: rgba(s, l)?, end: rgba(t, l)? }),
        "text_color" => Tween::new(ease, d, TextColorLens { start: rgba(s, l)?, end: rgba(t, l)? }),
        "background_color" => Tween::new(ease, d, UiBackgroundColorLens { start: rgba(s, l)?, end: rgba(t, l)? }),
        _ => return Err(fail(format!("no lens {l}: position, scale, rotation, rotate_x, rotate_y, rotate_z, sprite_color, text_color or background_color"))),
    };
    let strategy = if a.mirrored { RepeatStrategy::MirroredRepeat } else { RepeatStrategy::Repeat };
    let tw = match a.repeat {
        None => tw,
        Some(Repeat::Times(n)) => tw.with_repeat(RepeatCount::Finite(n.max(1)), strategy),
        Some(Repeat::Forever(f)) if f == "infinite" => tw.with_repeat(RepeatCount::Infinite, strategy),
        Some(Repeat::Forever(f)) => return Err(fail(format!("repeat is a count or \"infinite\", not {f}"))),
    };
    let mut anim = TweenAnim::new(tw);
    if let Some(sp) = a.speed {
        anim.speed = sp;
    }
    world.entity_mut(e).insert(anim);
    Ok(Value::Null)
}

fn playing(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Of { entity: bits } = params(p)?;
    Ok(json!(world.get::<TweenAnim>(entity(bits)?).is_some()))
}
