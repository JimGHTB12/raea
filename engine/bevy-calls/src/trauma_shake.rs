//! bevy_trauma_shake (the declaration's `"calls": "trauma_shake"`): a camera (or any entity) with `Shake` shakes by
//! its trauma, which decays each second. `Shake` and `ShakeSettings` are reflected (the declaration's `reflect`), so
//! TypeScript puts them on an entity with the generated handles; trauma is private and only ever added, so it is a call.
//!
//!   trauma_shake/add  {entity?, amount}  -> trauma after: on that entity's Shake, or on every Shake (as TraumaEvent)
//!                                           when no entity is given ([trauma per Shake]); clamped to 0..1
//!   trauma_shake/get  {entity}           -> its trauma now (0..1)

use bevy::prelude::*;
use bevy::reflect::{PartialReflect, ReflectRef};
use bevy_remote::BrpResult;
use bevy_trauma_shake::Shake;
use serde_json::{Value, json};

use crate::{entity, fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "trauma_shake/add" => add, "trauma_shake/get" => get);
}

#[derive(serde::Deserialize)]
struct Add {
    entity: Option<u64>,
    amount: f32,
}

#[derive(serde::Deserialize)]
struct Of {
    entity: u64,
}

// the private trauma field, through Shake's reflection
fn trauma(s: &Shake) -> f32 {
    match s.reflect_ref() {
        ReflectRef::Struct(st) => st.field("trauma").and_then(|f| f.try_downcast_ref::<f32>()).copied().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn add(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let a: Add = params(p)?;
    match a.entity {
        Some(bits) => {
            let e = entity(bits)?;
            let mut s = world.get_mut::<Shake>(e).ok_or_else(|| fail(format!("entity {bits} has no Shake (put trauma_shake.Shake on it)")))?;
            s.add_trauma(a.amount);
            Ok(json!(trauma(&s)))
        }
        None => {
            let mut q = world.query::<&mut Shake>();
            let mut after = vec![];
            for mut s in q.iter_mut(world) {
                s.add_trauma(a.amount);
                after.push(trauma(&s));
            }
            Ok(json!(after))
        }
    }
}

fn get(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Of { entity: bits } = params(p)?;
    let s = world.get::<Shake>(entity(bits)?).ok_or_else(|| fail(format!("entity {bits} has no Shake")))?;
    Ok(json!(trauma(s)))
}
