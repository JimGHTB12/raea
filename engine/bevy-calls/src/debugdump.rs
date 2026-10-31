//! bevy_mod_debugdump (the declaration's `"calls": "debugdump"`): a schedule's systems, sets and ordering as a Graphviz
//! dot graph, asked for from TypeScript or any BRP client.
//!
//!   debugdump/schedules  {}               -> [name]: the App's schedules
//!   debugdump/schedule   {schedule}       -> dot text of that schedule ("Update", "FixedUpdate", "PostUpdate", ...)

use bevy::ecs::schedule::Schedules;
use bevy::prelude::*;
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::{fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "debugdump/schedules" => schedules, "debugdump/schedule" => schedule);
}

fn schedules(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    let s = world.resource::<Schedules>();
    let mut names: Vec<String> = s.iter().map(|(label, _)| format!("{label:?}")).collect();
    names.sort();
    Ok(json!(names))
}

#[derive(serde::Deserialize)]
struct Of {
    schedule: String,
}

fn schedule(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Of { schedule } = params(p)?;
    let s = world.resource::<Schedules>();
    let (_, sched) = s.iter().find(|(label, _)| format!("{label:?}") == schedule).ok_or_else(|| fail(format!("no schedule {schedule} (debugdump/schedules lists them)")))?;
    let settings = bevy_mod_debugdump::schedule_graph::Settings::default();
    Ok(json!(bevy_mod_debugdump::schedule_graph::schedule_graph_dot(sched, world, &settings)))
}
