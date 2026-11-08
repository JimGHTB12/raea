//! Scripted input for an interactive example (docs/control.feature FC0a: "scripted input for an interactive one"):
//! RAEA_PARITY_INPUT=FILE, the script `raea play --input-script` takes, {"FRAME": [keys], ...}, each entry held from
//! the update numbered FRAME (the probe's count) until the next. "Pointer:X:Y" puts the mouse pointer at X, Y in the
//! window's logical px, ":down" while its primary button is held; the pointer is bevy_picking's, written as the
//! PointerInput messages a mouse sends (a Move when it moves, a Press and a Release on the button's edges), on the
//! picture the cameras draw into (the offscreen image while pictures are taken, else the primary window). Any other
//! entry is a KeyCode's name ("Space", "KeyA", "ArrowLeft"): held in ButtonInput<KeyCode> while its entry lasts, as
//! Bevy's keyboard_input_system holds a pressed key (`keys`).

use std::collections::BTreeMap;

use bevy::camera::RenderTarget;
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Resource)]
pub(crate) struct Script {
    at: BTreeMap<u64, Vec<String>>,
    pointer: Option<(Vec2, bool)>,
    held: Vec<KeyCode>,
}

/// The script at RAEA_PARITY_INPUT, if set; a script that can't be read ends the run, as the port's would.
pub(crate) fn load() -> Option<Script> {
    let p = std::env::var("RAEA_PARITY_INPUT").ok()?;
    let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("RAEA_PARITY_INPUT {p}: {e}"));
    let at: BTreeMap<u64, Vec<String>> = serde_json::from_str(&text).unwrap_or_else(|e| panic!("RAEA_PARITY_INPUT {p}: {{\"FRAME\": [keys]}}: {e}"));
    Some(Script { at, pointer: None, held: Vec::new() })
}

/// "Pointer:X:Y[:down]" among an entry's keys
fn pointer_of(keys: &[String]) -> Option<(Vec2, bool)> {
    keys.iter().find_map(|k| {
        let p: Vec<&str> = k.strip_prefix("Pointer:")?.split(':').collect();
        let (x, y) = (p.first()?.parse().ok()?, p.get(1)?.parse().ok()?);
        Some((Vec2::new(x, y), p.get(2) == Some(&"down")))
    })
}

/// This update's pointer, as PointerInput messages, before bevy_picking reads them in PreUpdate.
pub(crate) fn scripted(
    mut s: ResMut<Script>,
    probe: Res<super::Probe>,
    offscreen: Option<Res<super::Offscreen>>,
    window: Query<Entity, With<PrimaryWindow>>,
    mut out: MessageWriter<PointerInput>,
) {
    let Some(now) = s.at.range(..=probe.frame).next_back().and_then(|(_, k)| pointer_of(k)) else { return };
    let target = match &offscreen {
        Some(o) => RenderTarget::Image(o.0.clone().into()).normalize(None),
        None => RenderTarget::Window(bevy::window::WindowRef::Primary).normalize(window.single().ok()),
    };
    let Some(target) = target else { return };
    let location = Location { target, position: now.0 };
    let was = s.pointer;
    if was.map(|w| w.0) != Some(now.0) {
        let delta = was.map_or(Vec2::ZERO, |w| now.0 - w.0);
        out.write(PointerInput::new(PointerId::Mouse, location.clone(), PointerAction::Move { delta }));
    }
    let held = was.is_some_and(|w| w.1);
    if now.1 && !held {
        out.write(PointerInput::new(PointerId::Mouse, location.clone(), PointerAction::Press(PointerButton::Primary)));
    } else if !now.1 && held {
        out.write(PointerInput::new(PointerId::Mouse, location, PointerAction::Release(PointerButton::Primary)));
    }
    s.pointer = Some(now);
}

/// This update's keys into ButtonInput<KeyCode>, after Bevy's input systems (and the probe's no_input, which resets it
/// every update): a key just pressed on the update its entry starts, held while it lasts (pressed, not just pressed),
/// just released on the update after, so `input_just_pressed` and `pressed` read it as a keyboard's.
pub(crate) fn keys(mut s: ResMut<Script>, probe: Res<super::Probe>, input: Option<ResMut<ButtonInput<KeyCode>>>) {
    let Some(mut input) = input else { return };
    let names = s.at.range(..=probe.frame).next_back().map(|(_, k)| k.clone()).unwrap_or_default();
    let now: Vec<KeyCode> = names.iter().filter(|k| !k.starts_with("Pointer:")).map(|k| key_code(k)).collect();
    for k in &now {
        input.press(*k);
        if s.held.contains(k) {
            input.clear_just_pressed(*k);
        }
    }
    for k in s.held.iter().filter(|k| !now.contains(k)) {
        // pressed first, so a release no_input's reset left nothing to release still reads as one
        input.press(*k);
        input.release(*k);
        input.clear_just_pressed(*k);
    }
    s.held = now;
}

/// A KeyCode by its variant's name, as Bevy reflects it ("Space", "KeyA").
fn key_code(name: &str) -> KeyCode {
    use bevy::reflect::{enums::{DynamicEnum, DynamicVariant}, FromReflect};
    KeyCode::from_reflect(&DynamicEnum::new(name, DynamicVariant::Unit)).unwrap_or_else(|| panic!("RAEA_PARITY_INPUT: {name} isn't a KeyCode's name"))
}
