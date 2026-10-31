//! bevy_egui (the declaration's `"calls": "egui"`): no methods (its API is EguiGlobalSettings and EguiContextSettings,
//! both reflected), only a fix. bevy_egui 0.43 runs setup_primary_egui_context_system in PreStartup and again in
//! PreUpdate, each with its own "a context exists" Local; when two cameras are there before the first frame (a
//! render's scene camera and a camera drawing into an image) the PreUpdate copy can make the second one primary too,
//! and run_egui_context_pass_loop_system panics on the shared schedule. The first primary context made is kept, and
//! any later one is taken off its camera. The primary context's pass schedule is made here too: bevy_egui leaves it
//! to the first UI system, and with none (an App with egui and a camera only) the pass loop panics on it missing.

use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext, setup_primary_egui_context_system};

pub fn calls(app: &mut App) {
    app.init_schedule(EguiPrimaryContextPass);
    app.add_systems(PreUpdate, one_primary.after(setup_primary_egui_context_system));
}

fn one_primary(mut commands: Commands, primaries: Query<Entity, With<PrimaryEguiContext>>, mut first: Local<Option<Entity>>) {
    if first.is_none_or(|e| !primaries.contains(e)) {
        *first = primaries.iter().min();
    }
    for e in &primaries {
        if Some(e) != *first {
            commands.entity(e).remove::<(EguiContext, PrimaryEguiContext)>();
        }
    }
}
