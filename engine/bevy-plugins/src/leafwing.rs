//! leafwing-input-manager (the declaration's `"adapter": "leafwing"`): its plugin is generic over the game's action
//! enum, so the action set is named here, in Rust. `GameAction` is a common one (move and look on two axes, the usual
//! buttons), and TypeScript binds keys and gamepad buttons to it and reads it:
//!
//!   leafwing/player  {entity?, keys?: {Jump: "Space", ...}, buttons?: {Jump: "South", ...}, move?: "wasd" | "arrows"
//!                     | "left_stick", look?: "right_stick" | "mouse"}  -> the entity holding the InputMap
//!   leafwing/state   {entity}  -> {pressed: [action], just_pressed: [action], move: [x, y], look: [x, y]}
//!
//! A program with no keyboard presses keys with the bridge's `raea/input {key, pressed}`.
//!
//! A game with its own actions names its enum the same way: `InputManagerPlugin::<ItsAction>` in an adapter.

use bevy::prelude::*;
use bevy_remote::BrpResult;
use leafwing_input_manager::prelude::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

use crate::bridge::{add_method, fail, params};

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, serde::Serialize, serde::Deserialize)]
pub enum GameAction {
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(DualAxis)]
    Look,
    Jump,
    Crouch,
    Sprint,
    Fire,
    AltFire,
    Interact,
    Reload,
    Pause,
}

pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "leafwing", install, group: None, on_demand: false });
}

fn install(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<GameAction>::default());
    add_method(app, "leafwing/player", player);
    add_method(app, "leafwing/state", state);
}

fn from_name<T: serde::de::DeserializeOwned>(what: &str, name: &str) -> Result<T, bevy_remote::BrpError> {
    serde_json::from_value(json!(name)).map_err(|e| fail(format!("{what} {name}: {e}")))
}

#[derive(serde::Deserialize)]
struct Player {
    entity: Option<u64>,
    #[serde(default)]
    keys: BTreeMap<String, String>,
    #[serde(default)]
    buttons: BTreeMap<String, String>,
    #[serde(rename = "move")]
    moves: Option<String>,
    look: Option<String>,
}

fn player(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: Player = params(p)?;
    let mut map = InputMap::<GameAction>::default();
    for (a, k) in &p.keys {
        map.insert(from_name::<GameAction>("action", a)?, crate::bridge::reflected::<KeyCode>(world, json!(k)).map_err(fail)?);
    }
    for (a, b) in &p.buttons {
        map.insert(from_name::<GameAction>("action", a)?, crate::bridge::reflected::<GamepadButton>(world, json!(b)).map_err(fail)?);
    }
    match p.moves.as_deref() {
        None => {}
        Some("wasd") => {
            map.insert_dual_axis(GameAction::Move, VirtualDPad::wasd());
        }
        Some("arrows") => {
            map.insert_dual_axis(GameAction::Move, VirtualDPad::arrow_keys());
        }
        Some("left_stick") => {
            map.insert_dual_axis(GameAction::Move, GamepadStick::LEFT);
        }
        Some(o) => return Err(fail(format!("move: {o} isn't wasd, arrows or left_stick"))),
    }
    match p.look.as_deref() {
        None => {}
        Some("right_stick") => {
            map.insert_dual_axis(GameAction::Look, GamepadStick::RIGHT);
        }
        Some("mouse") => {
            map.insert_dual_axis(GameAction::Look, MouseMove::default());
        }
        Some(o) => return Err(fail(format!("look: {o} isn't right_stick or mouse"))),
    }
    let e = match p.entity {
        Some(b) => {
            let e = Entity::try_from_bits(b).ok_or_else(|| fail(format!("{b} isn't an entity")))?;
            world.get_entity_mut(e).map_err(|e| fail(e.to_string()))?.insert(map);
            e
        }
        None => world.spawn(map).id(),
    };
    Ok(json!(e.to_bits()))
}

#[derive(serde::Deserialize)]
struct Of {
    entity: u64,
}

fn state(In(p): In<Option<Value>>, q: Query<&ActionState<GameAction>>) -> BrpResult {
    let Of { entity } = params(p)?;
    let e = Entity::try_from_bits(entity).ok_or_else(|| fail(format!("{entity} isn't an entity")))?;
    let s = q.get(e).map_err(|_| fail(format!("{entity} has no ActionState (leafwing/player first)")))?;
    let names = |v: Vec<GameAction>| v.into_iter().map(|a| format!("{a:?}")).collect::<Vec<_>>();
    let m = s.axis_pair(&GameAction::Move);
    let l = s.axis_pair(&GameAction::Look);
    Ok(json!({"pressed": names(s.get_pressed()), "just_pressed": names(s.get_just_pressed()), "move": [m.x, m.y], "look": [l.x, l.y]}))
}

