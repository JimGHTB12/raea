//! bevy-tnua, a floating character controller, on avian (the declaration's `"adapter": "tnua"`). Its controller is
//! generic over the game's control scheme (an enum deriving TnuaScheme), so a common scheme is named here: walk, with
//! jump, crouch and dash. Tnua wants its input fed every physics step, so TypeScript writes a component and this
//! adapter feeds it:
//!
//!   `TnuaInput { motion: [x, y, z], jump, crouch, dash }`: a reflected component, so the generated handle sets it
//!   (`app.insert(e, with(tnua.TnuaInput, {...}))`); motion is the wanted walk direction in world space, a length of 1
//!   walking at the character's `speed` (m/s; tnua's own default when not given)
//!
//!   tnua/character  {entity, float_height?, jump_height?, speed?}  -> null: makes a dynamic avian body a Tnua
//!                    character (controller, config, a sensor shape, rotation locked); give it a collider first

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_remote::BrpResult;
use bevy_tnua::builtins::{TnuaBuiltinCrouch, TnuaBuiltinCrouchConfig, TnuaBuiltinDash, TnuaBuiltinDashConfig, TnuaBuiltinJump, TnuaBuiltinJumpConfig, TnuaBuiltinWalk, TnuaBuiltinWalkConfig};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::prelude::*;
use serde_json::Value;

use crate::bridge::{add_method, fail, params};

#[derive(TnuaScheme)]
#[scheme(basis = TnuaBuiltinWalk)]
pub enum Moves {
    Jump(TnuaBuiltinJump),
    Crouch(TnuaBuiltinCrouch),
    Dash(TnuaBuiltinDash),
}

/// What the character is asked to do this step, set from TypeScript.
#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component, Default)]
pub struct TnuaInput {
    pub motion: Vec3,
    pub jump: bool,
    pub crouch: bool,
    pub dash: bool,
}

pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "tnua", install, group: None, on_demand: false });
}

fn install(app: &mut App) {
    app.add_plugins((TnuaControllerPlugin::<Moves>::new(FixedUpdate), TnuaAvian3dPlugin::new(FixedUpdate)))
        .register_type::<TnuaInput>()
        .add_systems(FixedUpdate, feed.in_set(TnuaUserControlsSystems));
    add_method(app, "tnua/character", character);
}

fn feed(mut q: Query<(&mut TnuaController<Moves>, &TnuaInput)>) {
    for (mut c, i) in &mut q {
        c.initiate_action_feeding();
        c.basis = TnuaBuiltinWalk { desired_motion: i.motion, ..default() };
        if i.jump {
            c.action(Moves::Jump(default()));
        }
        if i.crouch {
            c.action(Moves::Crouch(default()));
        }
        if i.dash {
            c.action(Moves::Dash(TnuaBuiltinDash { displacement: i.motion.normalize_or_zero() * 4.0, ..default() }));
        }
    }
}

#[derive(serde::Deserialize)]
struct Character {
    entity: u64,
    float_height: Option<f32>,
    jump_height: Option<f32>,
    speed: Option<f32>,
}

fn character(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let c: Character = params(p)?;
    let e = Entity::try_from_bits(c.entity).ok_or_else(|| fail(format!("{} isn't an entity", c.entity)))?;
    let mut walk = TnuaBuiltinWalkConfig { float_height: c.float_height.unwrap_or(1.5), ..default() };
    if let Some(s) = c.speed {
        walk.speed = s;
    }
    let config = MovesConfig {
        basis: walk,
        jump: TnuaBuiltinJumpConfig { height: c.jump_height.unwrap_or(2.0), ..default() },
        crouch: TnuaBuiltinCrouchConfig { float_offset: -0.6, ..default() },
        dash: TnuaBuiltinDashConfig::default(),
    };
    let handle = world.get_resource_mut::<Assets<MovesConfig>>().ok_or_else(|| fail("tnua's config assets aren't there (the App needs assets)"))?.add(config);
    world
        .get_entity_mut(e)
        .map_err(|e| fail(e.to_string()))?
        .insert((TnuaController::<Moves>::default(), TnuaConfig::<Moves>(handle), TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)), LockedAxes::ROTATION_LOCKED, TnuaInput::default()));
    Ok(Value::Null)
}
