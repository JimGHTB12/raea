//! bevy_lookup_curve (the declaration's `"calls": "lookup_curve"`): a curve asset made and sampled from TypeScript.
//! `LookupCurve` is an asset, not a component, so the registry gives it no root to write; a curve is made from knots
//! (or loaded from a .curve.ron the editor saved) and named by the number this returns.
//!
//!   lookup_curve/curve   {knots: [{position: [x, y], interpolation?: "Cubic" | "Linear" | "Constant",
//!                        left_tangent?, right_tangent?: {slope, mode: "Aligned" | "Free", weight}}]} | {path}
//!                        -> curve (a number); knots are sorted by x
//!   lookup_curve/sample  {curve, x: number | [number]}  -> y or [y]; null while a loaded curve isn't there yet
//!   lookup_curve/knots   {curve}                         -> [knot] as given (null while loading)

use bevy::prelude::*;
use bevy_lookup_curve::{Knot, KnotInterpolation, LookupCurve, Tangent};
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::{fail, params};

pub fn calls(app: &mut App) {
    app.init_resource::<Curves>();
    crate::methods!(app, "lookup_curve/curve" => curve, "lookup_curve/sample" => sample, "lookup_curve/knots" => knots);
}

/// The curves TypeScript made or loaded, by the number it was given
#[derive(Resource, Default)]
struct Curves(Vec<Handle<LookupCurve>>);

#[derive(serde::Deserialize)]
struct KnotIn {
    position: [f32; 2],
    interpolation: Option<KnotInterpolation>,
    left_tangent: Option<Tangent>,
    right_tangent: Option<Tangent>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Make {
    Knots { knots: Vec<KnotIn> },
    Path { path: String },
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Xs {
    One(f32),
    Many(Vec<f32>),
}

#[derive(serde::Deserialize)]
struct Sample {
    curve: usize,
    x: Xs,
}

#[derive(serde::Deserialize)]
struct Of {
    curve: usize,
}

fn curve(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let handle = match params::<Make>(p)? {
        Make::Knots { knots } => {
            if knots.is_empty() {
                return Err(fail("lookup_curve/curve: a curve needs a knot"));
            }
            let knots = knots
                .into_iter()
                .map(|k| Knot {
                    position: Vec2::from(k.position),
                    interpolation: k.interpolation.unwrap_or(KnotInterpolation::Cubic),
                    left_tangent: k.left_tangent.unwrap_or_default(),
                    right_tangent: k.right_tangent.unwrap_or_default(),
                    ..Knot::default()
                })
                .collect();
            world.resource_mut::<Assets<LookupCurve>>().add(LookupCurve::new(knots))
        }
        Make::Path { path } => world.resource::<AssetServer>().load(path),
    };
    let mut curves = world.resource_mut::<Curves>();
    curves.0.push(handle);
    Ok(json!(curves.0.len() - 1))
}

fn of(world: &World, n: usize) -> Result<Option<&LookupCurve>, bevy_remote::BrpError> {
    let h = world.resource::<Curves>().0.get(n).ok_or_else(|| fail(format!("no curve {n} (lookup_curve/curve makes one)")))?;
    Ok(world.resource::<Assets<LookupCurve>>().get(h))
}

fn sample(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let s: Sample = params(p)?;
    let Some(c) = of(world, s.curve)? else { return Ok(Value::Null) };
    Ok(match s.x {
        Xs::One(x) => json!(c.lookup(x)),
        Xs::Many(xs) => json!(xs.iter().map(|&x| c.lookup(x)).collect::<Vec<_>>()),
    })
}

fn knots(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Of { curve } = params(p)?;
    let Some(c) = of(world, curve)? else { return Ok(Value::Null) };
    serde_json::to_value(c.knots()).map_err(|e| fail(e.to_string()))
}
