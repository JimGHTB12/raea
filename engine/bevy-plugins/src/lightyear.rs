//! lightyear (server-authoritative networking with client prediction and rollback, on bevy_replicon): the declaration's
//! `"adapter": "lightyear"`. This is where a plugin truly needs Rust:
//! - **The protocol.** lightyear registers concrete Rust types (`app.component::<Position>().replicate().predict()`,
//!   `InputPlugin::<Move>`). A replicated component is serialized by its own serde impl on both peers, and prediction
//!   keeps a typed history of it to roll back, so a reflected type path alone can't stand in.
//! - **The simulation that's predicted.** `step` runs in FixedUpdate on the server and again on the client's predicted
//!   entity, and rollback re-runs it. It has to be a system in the App, not TypeScript between frames.
//!
//! The rest is generic. The components are reflected, so the generated TypeScript reads and writes them, and BRP's
//! world.query sees both peers. Connecting, spawning a player and sending input are methods, so TypeScript drives the
//! session:
//!
//!   lightyear/start      (server) {}                      -> the server entity
//!   lightyear/link       (server) {}                      -> {link}: an in-process crossbeam pair, its server half
//!                                                            linked; the client half waits for connect
//!   lightyear/connect    (client) {link}                  -> the client entity (Connect triggered)
//!   lightyear/disconnect (client) {}
//!   lightyear/spawn      (server) {position, owner?}      -> a replicated, predicted entity, controlled by the
//!                                                            client of link `owner`
//!   lightyear/input      (client) {x, z}                  -> this tick's Move on the client's predicted entities
//!   lightyear/status     (either) {}                      -> {connected, tick, entities: [{entity, position,
//!                                                            predicted, controlled}]}
//!
//! And the session's reflected types, which need no method at all: the state `Session` (Lobby, Playing: input moves
//! the player only while Playing), the event `Teleport` (world.trigger_event; world.observe+watch sees it), the message
//! `Announce` (world.write_message) and the resource `Announced` it fills.
//!
//! Two plugin names, each on demand (an App gets the half it names): `lightyear_server` and `lightyear_client`.
//! lightyear reads time from its own Instant. With lightyear_core's `test_utils`, that is mock_instant's global clock,
//! which the bridge advances once per lockstep frame (bridge::on_step), so a session is a function of its frames, as
//! the stepper in lightyear's own tests makes it.

use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Mutex;
use std::time::Duration;

use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use bevy_remote::BrpResult;
use lightyear::crossbeam::CrossbeamIo;
use lightyear::prelude::input::native::{ActionState, InputMarker, InputPlugin as MoveInputPlugin};
use lightyear::prelude::{client::*, server::*, *};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::bridge::{add_method, fail, install_remote, params};

/// The tick: one 60 fps frame, exactly the frame clock's step (TypeScript opens the peers at 60 fps).
fn tick() -> Duration {
    Duration::from_secs_f64(1.0 / 60.0)
}
/// Units a second at full input.
pub const SPEED: f32 = 3.0;

/// Where a networked thing is: replicated from the server, predicted on its controlling client.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(Component, Default)]
pub struct Position(pub Vec3);

/// A client's input: a direction on the ground, each axis -1, 0 or 1.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub struct Move {
    pub x: i8,
    pub z: i8,
}

impl MapEntities for Move {
    fn map_entities<M: EntityMapper>(&mut self, _: &mut M) {}
}

/// Where the session is: input moves the player only while Playing (a reflected, freely mutable state, so TypeScript
/// reads and sets it through raea/state.get and raea/state.set).
#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
#[reflect(State, FreelyMutableState)]
pub enum Session {
    #[default]
    Lobby,
    Playing,
}

/// Moves every networked thing on the peer it's triggered on to `to` (a reflected event: world.trigger_event, and
/// world.observe+watch sees it).
#[derive(Event, Clone, Copy, Debug, Reflect)]
#[reflect(Event)]
pub struct Teleport {
    pub to: Vec3,
}

/// A line for the peer's log, read by a system the next frame (a reflected message: world.write_message).
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(Message)]
pub struct Announce {
    pub text: String,
}

/// The lines Announce has written, in order (a reflected resource: world.get_resources).
#[derive(Resource, Clone, Debug, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct Announced(pub Vec<String>);

fn teleport(e: On<Teleport>, mut q: Query<&mut Position>) {
    for mut p in &mut q {
        p.0 = e.to;
    }
}

fn announced(mut r: MessageReader<Announce>, mut log: ResMut<Announced>) {
    log.0.extend(r.read().map(|a| a.text.clone()));
}

/// The protocol both peers register (in the same order).
fn protocol(app: &mut App) {
    app.init_state::<Session>();
    app.add_message::<Announce>();
    app.init_resource::<Announced>();
    app.add_observer(teleport);
    app.add_systems(Update, announced);
    app.add_plugins(MoveInputPlugin::<Move>::default());
    app.component::<Position>().replicate().predict();
    app.register_type::<Position>().register_type::<Move>();
}

/// The predicted simulation: the server's authoritative entities and the client's predicted ones move alike.
fn step(time: Res<Time<Fixed>>, session: Res<State<Session>>, mut q: Query<(&mut Position, &ActionState<Move>), Or<(With<Predicted>, With<Replicate>)>>) {
    if *session.get() != Session::Playing {
        return;
    }
    let dt = time.delta_secs();
    for (mut p, a) in &mut q {
        let m = a.0;
        if m.x != 0 || m.z != 0 {
            p.0 += Vec3::new(m.x as f32, 0.0, m.z as f32) * SPEED * dt;
        }
    }
}

fn base(app: &mut App) {
    if !app.is_plugin_added::<bevy::state::app::StatesPlugin>() {
        app.add_plugins(bevy::state::app::StatesPlugin);
    }
    if !app.is_plugin_added::<bevy::input::InputPlugin>() {
        app.add_plugins(bevy::input::InputPlugin);
    }
    install_remote(app);
}

/// The client halves of links made on the server, waiting for a client App to connect them (by token).
static PENDING: Mutex<BTreeMap<u64, CrossbeamIo>> = Mutex::new(BTreeMap::new());
static NEXT_LINK: Mutex<u64> = Mutex::new(0);

/// The server's links by token: the server-side entity of each.
#[derive(Resource, Default)]
struct Links(BTreeMap<u64, Entity>);

#[derive(Resource)]
struct ServerEntity(Entity);

#[derive(Resource)]
struct ClientEntity(Entity);

fn entity_json(e: Entity) -> Value {
    json!(e.to_bits())
}

fn start(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    if let Some(s) = world.get_resource::<ServerEntity>() {
        return Ok(entity_json(s.0));
    }
    let e = world.spawn(RawServer).id();
    world.insert_resource(ServerEntity(e));
    world.trigger(Start { entity: e });
    // A raw server is started once it is linked, and crossbeam has no listening socket to link it (each pair links
    // its own ends), so it is linked here: replicon's ServerState goes to Running and replication sends.
    world.entity_mut(e).insert(Linked);
    world.flush();
    Ok(entity_json(e))
}

fn link(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    let server = world.get_resource::<ServerEntity>().ok_or_else(|| fail("lightyear/start the server first"))?.0;
    let token = {
        let mut n = NEXT_LINK.lock().unwrap();
        *n += 1;
        *n
    };
    let (client_io, server_io) = CrossbeamIo::new_pair();
    let e = world
        .spawn((
            LinkOf { endpoint: server },
            Link::default(),
            PeerAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), token as u16)),
            Linked,
            server_io,
            PingManager::new(PingConfig { ping_interval: Duration::default() }),
            ReplicationSender,
            ReplicationReceiver,
        ))
        .id();
    world.get_resource_or_init::<Links>().0.insert(token, e);
    PENDING.lock().unwrap().insert(token, client_io);
    Ok(json!({"link": token, "entity": e.to_bits()}))
}

#[derive(Deserialize)]
struct Connect_ {
    link: u64,
}

fn connect(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Connect_ { link } = params(p)?;
    let io = PENDING.lock().unwrap().remove(&link).ok_or_else(|| fail(format!("no link {link} waiting (lightyear/link on the server makes one)")))?;
    let e = world.spawn((Client, PingManager::new(PingConfig { ping_interval: Duration::default() }), ReplicationSender, ReplicationReceiver, RawClient, io)).id();
    world.insert_resource(ClientEntity(e));
    world.trigger(Connect { entity: e });
    world.flush();
    Ok(entity_json(e))
}

fn disconnect(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    let e = world.get_resource::<ClientEntity>().ok_or_else(|| fail("not connected"))?.0;
    world.trigger(Disconnect { entity: e });
    world.flush();
    Ok(Value::Null)
}

#[derive(Deserialize)]
struct Spawn {
    #[serde(default)]
    position: [f32; 3],
    #[serde(default)]
    owner: Option<u64>,
}

fn spawn(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let s: Spawn = params(p)?;
    let e = world.spawn((Position(Vec3::from(s.position)), Replicate::to_clients(NetworkTarget::All), PredictionTarget::to_clients(NetworkTarget::All)));
    let id = e.id();
    if let Some(token) = s.owner {
        let owner = *world.get_resource::<Links>().and_then(|l| l.0.get(&token)).ok_or_else(|| fail(format!("no link {token}")))?;
        world.entity_mut(id).insert(ControlledBy { owner, lifetime: Default::default() });
    }
    world.flush();
    Ok(entity_json(id))
}

#[derive(Deserialize)]
struct Input {
    #[serde(default)]
    x: i8,
    #[serde(default)]
    z: i8,
}

fn input(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let i: Input = params(p)?;
    let mut q = world.query_filtered::<Entity, (With<Predicted>, With<Position>)>();
    let es: Vec<Entity> = q.iter(world).collect();
    if es.is_empty() {
        return Err(fail("no predicted entity yet (it arrives a few frames after lightyear/spawn)"));
    }
    for e in &es {
        let mut em = world.entity_mut(*e);
        if !em.contains::<InputMarker<Move>>() {
            em.insert(InputMarker::<Move>::default());
        }
        match em.get_mut::<ActionState<Move>>() {
            Some(mut a) => a.0 = Move { x: i.x.clamp(-1, 1), z: i.z.clamp(-1, 1) },
            None => {
                em.insert(ActionState(Move { x: i.x.clamp(-1, 1), z: i.z.clamp(-1, 1) }));
            }
        }
    }
    Ok(json!(es.iter().map(|e| e.to_bits()).collect::<Vec<_>>()))
}

fn status(In(_): In<Option<Value>>, world: &mut World) -> BrpResult {
    let tick = world.get_resource::<LocalTimeline>().map(|t| t.tick().0 as i64).unwrap_or(-1);
    let connected = if let Some(c) = world.get_resource::<ClientEntity>() {
        json!(world.entity(c.0).contains::<Connected>())
    } else {
        let mut q = world.query_filtered::<(), (With<LinkOf>, With<Connected>)>();
        json!(q.iter(world).count())
    };
    let mut q = world.query::<(Entity, &Position, Has<Predicted>, Has<Controlled>)>();
    let entities: Vec<Value> = q.iter(world).map(|(e, p, pr, c)| json!({"entity": e.to_bits(), "position": [p.0.x, p.0.y, p.0.z], "predicted": pr, "controlled": c})).collect();
    Ok(json!({"connected": connected, "tick": tick, "entities": entities}))
}

fn install_server(app: &mut App) {
    base(app);
    app.add_plugins(ServerPlugins { tick_duration: tick() });
    protocol(app);
    app.add_systems(FixedUpdate, step);
    add_method(app, "lightyear/start", start);
    add_method(app, "lightyear/link", link);
    add_method(app, "lightyear/spawn", spawn);
    add_method(app, "lightyear/status", status);
}

fn install_client(app: &mut App) {
    base(app);
    app.add_plugins(ClientPlugins { tick_duration: tick() });
    protocol(app);
    app.insert_resource(PredictionManager::default());
    app.add_systems(FixedUpdate, step);
    add_method(app, "lightyear/connect", connect);
    add_method(app, "lightyear/disconnect", disconnect);
    add_method(app, "lightyear/input", input);
    add_method(app, "lightyear/status", status);
}

/// The adapter's registration: the two halves, each on demand, and the mock clock stepped once per lockstep frame.
pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "lightyear_server", install: install_server, group: None, on_demand: true });
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "lightyear_client", install: install_client, group: None, on_demand: true });
    crate::bridge::on_step(|dt| mock_instant::global::MockClock::advance(dt));
}
