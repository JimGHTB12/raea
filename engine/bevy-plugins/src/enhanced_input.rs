//! bevy_enhanced_input (the declaration's `"adapter": "enhanced_input"`): its contexts and actions are Rust types (a
//! context is a component registered with add_input_context, an action a type with an output), so a common set is
//! named here. `OnFoot` is the context; Move (Vec2), Look (Vec2), Jump, Crouch, Sprint, Fire and Interact (bool) are its
//! actions, bound to WASD / mouse / the usual keys and the gamepad's sticks and buttons.
//!
//!   enhanced_input/pawn   {entity?}  -> the entity carrying OnFoot and its actions
//!   enhanced_input/state  {entity}   -> {Move: [x, y], Look: [x, y], Jump: bool, ...}: each action's value now

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_remote::BrpResult;
use serde_json::{Map, Value, json};

use crate::bridge::{add_method, fail, params};

/// The on-foot input context: a game's player carries it.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct OnFoot;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Move;
#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Look;
#[derive(InputAction)]
#[action_output(bool)]
pub struct Jump;
#[derive(InputAction)]
#[action_output(bool)]
pub struct Crouch;
#[derive(InputAction)]
#[action_output(bool)]
pub struct Sprint;
#[derive(InputAction)]
#[action_output(bool)]
pub struct Fire;
#[derive(InputAction)]
#[action_output(bool)]
pub struct Interact;

pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "enhanced_input", install, group: None, on_demand: false });
}

fn install(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin).add_input_context::<OnFoot>().register_type::<OnFoot>();
    add_method(app, "enhanced_input/pawn", pawn);
    add_method(app, "enhanced_input/state", state);
}

#[derive(serde::Deserialize, Default)]
struct Pawn {
    entity: Option<u64>,
}

fn bundle() -> impl Bundle {
    (
        OnFoot,
        actions!(OnFoot[
            (Action::<Move>::new(), DeadZone::default(), Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick()))),
            (Action::<Look>::new(), Bindings::spawn((Spawn(Binding::mouse_motion()), Axial::right_stick()))),
            (Action::<Jump>::new(), bindings![KeyCode::Space, GamepadButton::South]),
            (Action::<Crouch>::new(), bindings![KeyCode::ControlLeft, GamepadButton::East]),
            (Action::<Sprint>::new(), bindings![KeyCode::ShiftLeft, GamepadButton::LeftThumb]),
            (Action::<Fire>::new(), bindings![MouseButton::Left, GamepadButton::RightTrigger2]),
            (Action::<Interact>::new(), bindings![KeyCode::KeyE, GamepadButton::West]),
        ]),
    )
}

fn pawn(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let p: Pawn = if p.is_some() { params(p)? } else { Pawn::default() };
    let e = match p.entity {
        Some(b) => {
            let e = Entity::try_from_bits(b).ok_or_else(|| fail(format!("{b} isn't an entity")))?;
            world.get_entity_mut(e).map_err(|e| fail(e.to_string()))?.insert(bundle());
            e
        }
        None => world.spawn(bundle()).id(),
    };
    Ok(json!(e.to_bits()))
}

#[derive(serde::Deserialize)]
struct Of {
    entity: u64,
}

fn put<A: InputAction>(out: &mut Map<String, Value>, world: &mut World, actions: &[Entity], name: &str, f: impl Fn(&A::Output) -> Value) {
    for &a in actions {
        if let Some(v) = world.get::<Action<A>>(a) {
            out.insert(name.into(), f(&**v));
        }
    }
}

fn state(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Of { entity } = params(p)?;
    let e = Entity::try_from_bits(entity).ok_or_else(|| fail(format!("{entity} isn't an entity")))?;
    let actions: Vec<Entity> = world.get::<Actions<OnFoot>>(e).ok_or_else(|| fail(format!("{entity} has no OnFoot actions (enhanced_input/pawn first)")))?.iter().collect();
    let mut out = Map::new();
    let v2 = |v: &Vec2| json!([v.x, v.y]);
    let b = |v: &bool| json!(*v);
    put::<Move>(&mut out, world, &actions, "Move", v2);
    put::<Look>(&mut out, world, &actions, "Look", v2);
    put::<Jump>(&mut out, world, &actions, "Jump", b);
    put::<Crouch>(&mut out, world, &actions, "Crouch", b);
    put::<Sprint>(&mut out, world, &actions, "Sprint", b);
    put::<Fire>(&mut out, world, &actions, "Fire", b);
    put::<Interact>(&mut out, world, &actions, "Interact", b);
    Ok(Value::Object(out))
}
