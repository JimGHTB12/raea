//! `raea/input`: the devices a program or a test has none of, pressed as the device would press them, so input plugins
//! (leafwing, enhanced_input, Bevy's gamepads and picking, a game's own systems) read it the same as the real thing
//! (docs/plugins.feature PL2). One call does any of these:
//!
//!   key, pressed?               a keyboard key (KeyCode: "Space", "KeyW") held or let go ("input" in headless)
//!   mouse, pressed?             a mouse button ("Left") held or let go; with pointer, picking's pointer presses it too
//!   gamepad, pressed?, value?   a gamepad button ("South", "RightTrigger2"); value (0 to 1) for an analog one
//!   axis, value                 a gamepad stick or axis ("LeftStickX") at value (-1 to 1)
//!   pointer: [x, y]             the mouse pointer moved to (x, y) in the primary window's logical pixels, as picking
//!                               reads it ("picking" and "primary-window" in headless)
//!
//! The first gamepad call connects one virtual gamepad (an entity with Bevy's Gamepad, through GamepadConnectionEvent);
//! every gamepad call answers with its entity, and the others with null.
//!
//!   raea/input.gamepad  {}  -> {buttons: ["South", ...], axes: {LeftStickX: 0.5, ...}}: what Bevy's Gamepad on that
//!                              entity holds now, as a game reads it (BRP can't write a Gamepad as JSON: its axis map
//!                              is keyed by an enum); null before any gamepad call

use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent, RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent, RawGamepadEvent};
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_remote::BrpResult;
use serde_json::{Value, json};

use crate::bridge::{fail, params, reflected};

#[derive(serde::Deserialize)]
struct InputParams {
    key: Option<Value>,
    mouse: Option<Value>,
    gamepad: Option<Value>,
    axis: Option<Value>,
    value: Option<f32>,
    pointer: Option<[f32; 2]>,
    #[serde(default = "held")]
    pressed: bool,
}

fn held() -> bool {
    true
}

/// The virtual gamepad the first gamepad call connected
#[derive(Resource)]
struct Pad(Entity);

/// Where the virtual mouse pointer is, for the next move's delta
#[derive(Resource, Default)]
struct At(Vec2);

pub(crate) fn input(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: InputParams = params(p)?;
    if let Some(k) = p.key {
        let k: KeyCode = reflected(world, k).map_err(fail)?;
        let mut b = world.get_resource_mut::<ButtonInput<KeyCode>>().ok_or_else(|| fail("no keyboard input: the App needs InputPlugin"))?;
        if p.pressed { b.press(k) } else { b.release(k) }
    }
    if let Some(at) = p.pointer {
        pointer(world, PointerAction::Move { delta: Vec2::ZERO }, Vec2::from(at))?;
    }
    if let Some(m) = p.mouse {
        let m: MouseButton = reflected(world, m).map_err(fail)?;
        let mut b = world.get_resource_mut::<ButtonInput<MouseButton>>().ok_or_else(|| fail("no mouse input: the App needs InputPlugin"))?;
        if p.pressed { b.press(m) } else { b.release(m) }
        if let Some(at) = p.pointer {
            let button = match m {
                MouseButton::Right => PointerButton::Secondary,
                MouseButton::Middle => PointerButton::Middle,
                _ => PointerButton::Primary,
            };
            pointer(world, if p.pressed { PointerAction::Press(button) } else { PointerAction::Release(button) }, Vec2::from(at))?;
        }
    }
    if p.gamepad.is_none() && p.axis.is_none() {
        return Ok(Value::Null);
    }
    let pad = pad(world);
    if let Some(b) = p.gamepad {
        let b: GamepadButton = reflected(world, b).map_err(fail)?;
        let value = p.value.unwrap_or(if p.pressed { 1.0 } else { 0.0 });
        world.write_message(RawGamepadEvent::Button(RawGamepadButtonChangedEvent::new(pad, b, value)));
    }
    if let Some(a) = p.axis {
        let a: GamepadAxis = reflected(world, a).map_err(fail)?;
        let value = p.value.ok_or_else(|| fail("axis needs value (-1 to 1)"))?;
        world.write_message(RawGamepadEvent::Axis(RawGamepadAxisChangedEvent::new(pad, a, value)));
    }
    Ok(json!(pad.to_bits()))
}

pub(crate) fn gamepad(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Some(e) = world.get_resource::<Pad>().map(|p| p.0) else { return Ok(Value::Null) };
    let g = world.get::<Gamepad>(e).ok_or_else(|| fail("the virtual gamepad isn't connected yet: step a frame after the first gamepad call"))?;
    let buttons: Vec<String> = g.get_pressed().map(|b| format!("{b:?}")).collect();
    let axes: serde_json::Map<String, Value> = [GamepadAxis::LeftStickX, GamepadAxis::LeftStickY, GamepadAxis::LeftZ, GamepadAxis::RightStickX, GamepadAxis::RightStickY, GamepadAxis::RightZ]
        .into_iter()
        .filter_map(|a| g.get(a).map(|v| (format!("{a:?}"), json!(v))))
        .collect();
    Ok(json!({ "buttons": buttons, "axes": axes }))
}

fn pad(world: &mut World) -> Entity {
    if let Some(p) = world.get_resource::<Pad>() {
        return p.0;
    }
    let e = world.spawn_empty().id();
    let connected = GamepadConnection::Connected { name: "raea virtual gamepad".into(), vendor_id: None, product_id: None };
    world.write_message(GamepadConnectionEvent::new(e, connected));
    world.insert_resource(Pad(e));
    e
}

/// Picking's mouse pointer does `action` at `at` in the primary window
fn pointer(world: &mut World, action: PointerAction, at: Vec2) -> Result<(), bevy_remote::BrpError> {
    let window = world.query_filtered::<Entity, With<PrimaryWindow>>().iter(world).next().ok_or_else(|| fail("no primary window: the App needs one (\"primary-window\" in headless)"))?;
    let target = bevy::camera::NormalizedRenderTarget::Window(WindowRef::Entity(window).normalize(None).expect("an entity's window ref normalizes"));
    let action = match action {
        PointerAction::Move { .. } => {
            let mut last = world.get_resource_or_init::<At>();
            let delta = at - last.0;
            last.0 = at;
            PointerAction::Move { delta }
        }
        a => a,
    };
    world.write_message(PointerInput::new(PointerId::Mouse, Location { target, position: at }, action));
    Ok(())
}
