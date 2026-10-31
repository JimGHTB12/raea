//! The page half of input replay (docs/blitz-bevy.feature B3 H10): the input log `raea play --input-log` writes, each
//! frame's keys and pointer by frame index, recorded from a live App's pointer and played back into a render's. The
//! render gets Bevy `PointerInput` messages from the log at each frame, before picking runs, so picking, the pages'
//! route, :hover and :active, the twins' DOM events and pages on meshes all replay through the live path.
//!
//! The frame is the app's: it sets [`InputFrame`] in `First` before `PickingSystems::PostInput` (WorldPagePlugin from
//! ScenePlugin's SceneFrame). The log holds the pointer as it stood at each frame's end: a press and release within
//! one frame aren't in it, as they aren't in the games' log. Keys are the ones held, by their DOM names ("ArrowUp",
//! "a"), replayed as a window's key messages (KeyboardInput and WindowEvent::KeyboardInput) when the set changes.

use bevy::camera::NormalizedRenderTarget;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::reflect::FromReflect;
use bevy::reflect::enums::{DynamicEnum, DynamicVariant};
use bevy::window::WindowEvent;
use bevy::picking::PickingSystems;
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput, PointerLocation, PointerPressState};
use bevy::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The frame the log is keyed by this update.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputFrame(pub i64);

/// One frame's input, as `raea play --input-log` writes it and `raea render --input` reads it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameInput {
    pub keys: Vec<String>,
    pub pointer: Option<LogPointer>,
    /// played live (a game's isLive), as the play marks every frame it logs
    pub live: bool,
}

/// The pointer at a frame, in its target's px: where it is and whether its primary button is down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogPointer {
    pub x: f64,
    pub y: f64,
    pub down: bool,
}

/// The log: each frame's input by its index (a JSON object keyed by the frame's number).
pub type InputLog = BTreeMap<i64, FrameInput>;

/// A log read from its JSON: `{"FRAME": {"keys": [..], "pointer": {"x", "y", "down"}, "live"}}`.
pub fn parse(json: &str) -> Result<InputLog, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("the input log: {e}"))?;
    let o = v.as_object().ok_or("the input log isn't an object of frames")?;
    let mut log = InputLog::new();
    for (f, i) in o {
        let f: i64 = f.parse().map_err(|_| format!("the input log: frame {f:?} isn't a number"))?;
        let keys = i["keys"].as_array().map(|k| k.iter().filter_map(|k| k.as_str().map(String::from)).collect()).unwrap_or_default();
        let p = &i["pointer"];
        let pointer = p.is_object().then(|| LogPointer { x: p["x"].as_f64().unwrap_or(0.0), y: p["y"].as_f64().unwrap_or(0.0), down: p["down"].as_bool().unwrap_or(false) });
        log.insert(f, FrameInput { keys, pointer, live: i["live"].as_bool().unwrap_or(false) });
    }
    Ok(log)
}

/// A log as its JSON.
pub fn to_json(log: &InputLog) -> String {
    let mut o = serde_json::Map::new();
    for (f, i) in log {
        let mut e = json!({"keys": i.keys, "live": i.live});
        if let Some(p) = i.pointer {
            e["pointer"] = json!({"x": p.x, "y": p.y, "down": p.down});
        }
        o.insert(f.to_string(), e);
    }
    Value::Object(o).to_string()
}

/// Record the mouse pointer on `target` each frame into `log` (written out with [`to_json`]).
#[derive(Resource)]
pub struct InputRecord {
    pub target: NormalizedRenderTarget,
    pub log: InputLog,
}

/// Play `log` back as the mouse pointer on `target`.
#[derive(Resource)]
pub struct InputReplay {
    pub target: NormalizedRenderTarget,
    pub log: InputLog,
    /// the pointer and keys as the last frame left them
    last: Option<LogPointer>,
    held: Vec<String>,
}

impl InputReplay {
    pub fn new(target: NormalizedRenderTarget, log: InputLog) -> Self {
        InputReplay { target, log, last: None, held: Vec::new() }
    }
}

/// Records with an [`InputRecord`] and replays with an [`InputReplay`], whichever the App has.
pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFrame>()
            .add_message::<KeyboardInput>()
            .add_message::<WindowEvent>()
            .add_systems(First, (replay, replay_keys).run_if(resource_exists::<InputReplay>).in_set(PickingSystems::PostInput))
            .add_systems(PreUpdate, record.run_if(resource_exists::<InputRecord>).after(PickingSystems::ProcessInput).before(PickingSystems::Backend));
    }
}

fn record(frame: Res<InputFrame>, mut rec: ResMut<InputRecord>, pointers: Query<(&PointerId, &PointerLocation, &PointerPressState)>, keys: Option<Res<ButtonInput<Key>>>) {
    let pointer = pointers.iter().find(|(id, _, _)| **id == PointerId::Mouse).and_then(|(_, at, press)| {
        let at = at.location.as_ref().filter(|l| l.target == rec.target)?;
        Some(LogPointer { x: at.position.x as f64, y: at.position.y as f64, down: press.is_primary_pressed() })
    });
    let mut held: Vec<String> = keys.map(|k| k.get_pressed().map(name).collect()).unwrap_or_default();
    held.sort();
    rec.log.insert(frame.0, FrameInput { keys: held, pointer, live: true });
}

/// A key's DOM name, as WindowInput names it for the page (Bevy's names are the DOM's).
fn name(k: &Key) -> String {
    match k {
        Key::Character(c) => c.to_string(),
        Key::Space => " ".into(),
        other => format!("{other:?}"),
    }
}

/// The key a DOM name is: a character, or the unit variant of that name.
fn key(name: &str) -> Option<Key> {
    if name == " " {
        return Some(Key::Space);
    }
    if name.chars().count() == 1 {
        return Some(Key::Character(name.into()));
    }
    Key::from_reflect(&DynamicEnum::new(name, DynamicVariant::Unit))
}

/// The physical key a logged name stands for, where one does (the modifiers, which the page's key events track by it).
fn code(name: &str) -> KeyCode {
    match name {
        "Shift" => KeyCode::ShiftLeft,
        "Control" => KeyCode::ControlLeft,
        "Alt" => KeyCode::AltLeft,
        "Meta" | "Super" => KeyCode::SuperLeft,
        _ => KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
    }
}

/// The frame's keys against the last frame's: each let go, then each newly held, as a window sends them (no window in
/// a render: the messages carry a placeholder entity).
fn replay_keys(frame: Res<InputFrame>, mut rep: ResMut<InputReplay>, mut keyboard: MessageWriter<KeyboardInput>, mut window: MessageWriter<WindowEvent>) {
    let Some(now) = rep.log.get(&frame.0).map(|i| i.keys.clone()) else { return };
    let held = std::mem::replace(&mut rep.held, now.clone());
    let up = held.iter().filter(|k| !now.contains(k)).map(|k| (k, ButtonState::Released));
    let down = now.iter().filter(|k| !held.contains(k)).map(|k| (k, ButtonState::Pressed));
    for (n, state) in up.chain(down).collect::<Vec<_>>() {
        let Some(logical_key) = key(n) else {
            warn!("raea-page: the input log's key {n:?} isn't one Bevy names");
            continue;
        };
        let text = (state == ButtonState::Pressed).then(|| match &logical_key {
            Key::Character(c) => Some(c.clone()),
            Key::Space => Some(" ".into()),
            _ => None,
        });
        let k = KeyboardInput { key_code: code(n), logical_key, state, text: text.flatten(), repeat: false, window: Entity::PLACEHOLDER };
        keyboard.write(k.clone());
        window.write(WindowEvent::KeyboardInput(k));
    }
}

fn replay(frame: Res<InputFrame>, mut rep: ResMut<InputReplay>, mut out: MessageWriter<PointerInput>) {
    let Some(p) = rep.log.get(&frame.0).and_then(|i| i.pointer) else { return };
    let location = Location { target: rep.target.clone(), position: Vec2::new(p.x as f32, p.y as f32) };
    let last = rep.last.replace(p);
    if last.is_none_or(|l| (l.x, l.y) != (p.x, p.y)) {
        out.write(PointerInput::new(PointerId::Mouse, location.clone(), PointerAction::Move { delta: last.map_or(Vec2::ZERO, |l| Vec2::new((p.x - l.x) as f32, (p.y - l.y) as f32)) }));
    }
    match (last.is_some_and(|l| l.down), p.down) {
        (false, true) => {
            out.write(PointerInput::new(PointerId::Mouse, location, PointerAction::Press(PointerButton::Primary)));
        }
        (true, false) => {
            out.write(PointerInput::new(PointerId::Mouse, location, PointerAction::Release(PointerButton::Primary)));
        }
        _ => {}
    }
}
