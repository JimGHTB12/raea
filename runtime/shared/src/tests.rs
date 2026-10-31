use super::*;
use bevy::state::reflect::ReflectFreelyMutableState;

#[derive(Resource, Reflect, Debug, PartialEq, Clone, Default)]
#[reflect(Resource)]
struct Score {
    points: u32,
}

#[derive(Component, Reflect, Debug, PartialEq, Clone)]
#[reflect(Component)]
struct Health {
    hp: u32,
}

/// Never shared: React reaches it by reflection alone.
#[derive(Resource, Reflect, Debug, PartialEq, Clone)]
#[reflect(Resource)]
struct Level {
    n: u32,
}

fn app() -> (App, impl Fn() -> Vec<Value>) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).insert_resource(ReactOutbox(Mutex::new(tx)));
    let sent = move || -> Vec<Value> { rx.try_iter().map(|m| serde_json::from_str::<Value>(&m).unwrap()).filter(|m| m["name"] == "raea-world").flat_map(|m| m["value"].as_array().unwrap().clone()).collect() };
    (app, sent)
}

// one test, since React's asks are one queue per process
#[test]
fn react_and_the_world_share_one_value() {
    let (mut app, sent) = app();
    app.share_resource::<Score>().share_component::<Health>();
    let (path, hp) = (Score::type_path(), Health::type_path());

    // a system's value reaches React once, when it changes
    app.insert_resource(Score { points: 1 });
    app.update();
    assert_eq!(sent(), vec![json!({"path": path, "value": {"points": 1}})]);
    app.update();
    assert_eq!(sent(), Vec::<Value>::new());

    // React's write is the World's in the next PreUpdate, and comes back as the World's
    ASKS.lock().unwrap().push(Ask::Write { path: path.into(), entity: None, value: json!({"points": 5}) });
    app.update();
    assert_eq!(app.world().resource::<Score>(), &Score { points: 5 });
    assert_eq!(sent(), vec![json!({"path": path, "value": {"points": 5}})]);

    // a component by entity: changed, written by React, removed
    let e = app.world_mut().spawn(Health { hp: 2 }).id();
    app.update();
    assert_eq!(sent(), vec![json!({"path": hp, "entity": e.to_bits(), "value": {"hp": 2}})]);
    ASKS.lock().unwrap().push(Ask::Write { path: hp.into(), entity: Some(e.to_bits()), value: json!({"hp": 3}) });
    app.update();
    assert_eq!(app.world().get::<Health>(e), Some(&Health { hp: 3 }));
    assert_eq!(sent(), vec![json!({"path": hp, "entity": e.to_bits(), "value": {"hp": 3}})]);
    app.world_mut().entity_mut(e).remove::<Health>();
    app.world_mut().remove_resource::<Score>();
    app.update();
    let mut got = sent();
    got.sort_by_key(|v| v.get("entity").is_some());
    assert_eq!(got, vec![json!({"path": path, "value": null}), json!({"path": hp, "entity": e.to_bits(), "value": null})]);

    // a type registered for reflection and never shared: watched, it's sent when it changes; written, it's set
    let level = Level::type_path();
    app.register_type::<Level>().insert_resource(Level { n: 1 });
    app.update();
    assert_eq!(sent(), Vec::<Value>::new());
    ASKS.lock().unwrap().push(Ask::Watch { path: level.into(), entity: None });
    app.update();
    assert_eq!(sent(), vec![json!({"path": level, "value": {"n": 1}})]);
    app.update();
    assert_eq!(sent(), Vec::<Value>::new());
    ASKS.lock().unwrap().push(Ask::Write { path: level.into(), entity: None, value: json!({"n": 7}) });
    app.update();
    assert_eq!(app.world().resource::<Level>(), &Level { n: 7 });
    assert_eq!(sent(), vec![json!({"path": level, "value": {"n": 7}})]);
    // unwatched, nothing more comes
    ASKS.lock().unwrap().push(Ask::Unwatch { path: level.into(), entity: None });
    app.update();
    app.insert_resource(Level { n: 8 });
    app.update();
    assert_eq!(sent(), Vec::<Value>::new());

    // a component never shared, watched on one entity: absent first (null), then its value, then gone
    let mana = Mana::type_path();
    app.register_type::<Mana>();
    let e = app.world_mut().spawn_empty().id();
    ASKS.lock().unwrap().push(Ask::Watch { path: mana.into(), entity: Some(e.to_bits()) });
    app.update();
    assert_eq!(sent(), vec![json!({"path": mana, "entity": e.to_bits(), "value": null})]);
    ASKS.lock().unwrap().push(Ask::Write { path: mana.into(), entity: Some(e.to_bits()), value: json!({"mp": 4}) });
    app.update();
    assert_eq!(app.world().get::<Mana>(e), Some(&Mana { mp: 4 }));
    assert_eq!(sent(), vec![json!({"path": mana, "entity": e.to_bits(), "value": {"mp": 4}})]);
    app.world_mut().entity_mut(e).remove::<Mana>();
    app.update();
    assert_eq!(sent(), vec![json!({"path": mana, "entity": e.to_bits(), "value": null})]);

    // a query watched by React's key: its rows, sent again when they change
    ASKS.lock().unwrap().push(Ask::Unwatch { path: mana.into(), entity: Some(e.to_bits()) });
    let q = json!({"data": {"components": [mana]}});
    ASKS.lock().unwrap().push(Ask::WatchRead { key: "q".into(), method: "world.query".into(), params: q });
    app.world_mut().entity_mut(e).insert(Mana { mp: 1 });
    app.update();
    assert_eq!(sent(), vec![json!({"key": "q", "value": [{"entity": e.to_bits(), "components": {mana: {"mp": 1}}}]})]);
    app.update();
    assert_eq!(sent(), Vec::<Value>::new());
    app.world_mut().despawn(e);
    app.update();
    assert_eq!(sent(), vec![json!({"key": "q", "value": []})]);
    ASKS.lock().unwrap().push(Ask::UnwatchRead { key: "q".into() });

    // an observed event: what it saw that frame, once
    app.register_type::<Ping>();
    ASKS.lock().unwrap().push(Ask::Observe { params: json!({"key": "o", "event": Ping::type_path()}) });
    app.update();
    sent();
    app.world_mut().trigger(Ping { n: 3 });
    app.update();
    let got = sent();
    assert_eq!(got.len(), 1);
    assert_eq!((got[0]["key"].clone(), got[0]["events"][0]["event"].clone()), (json!("o"), json!({"n": 3})));
    app.update();
    assert_eq!(sent(), Vec::<Value>::new());
    ASKS.lock().unwrap().push(Ask::Unobserve { key: "o".into() });
    app.update();
    app.world_mut().trigger(Ping { n: 4 });
    app.update();
    assert_eq!(sent(), Vec::<Value>::new());

    // an element's twin by its op id: null until there is one, then its entity
    ASKS.lock().unwrap().push(Ask::WatchRead { key: "t".into(), method: TWIN.into(), params: json!({"op": 42}) });
    app.update();
    assert_eq!(sent(), vec![json!({"key": "t", "value": null})]);
    let page = app.world_mut().spawn_empty().id();
    let tw = app.world_mut().spawn(raea_runtime_twins::PageElement { page, node: Default::default(), op: Some(42) }).id();
    app.update();
    assert_eq!(sent(), vec![json!({"key": "t", "value": tw.to_bits()})]);
    ASKS.lock().unwrap().push(Ask::UnwatchRead { key: "t".into() });

    // a state, watched (useAppState) and set through /world's queue (setAppState): it changes at the next
    // StateTransition, and comes back
    app.add_plugins(bevy::state::app::StatesPlugin).init_state::<Mode>().register_type::<Mode>();
    let mode = Mode::type_path();
    ASKS.lock().unwrap().push(Ask::WatchRead { key: "s".into(), method: "raea/state.get".into(), params: json!({"state": mode}) });
    app.update();
    assert_eq!(sent(), vec![json!({"key": "s", "value": "Menu"})]);
    let t = raea_bevy_world::send("raea/state.set", json!({"state": mode, "value": "Play"}));
    app.update();
    assert_eq!(raea_bevy_world::answer(t), Some(json!({"value": null})));
    app.update();
    assert_eq!(app.world().resource::<State<Mode>>().get(), &Mode::Play);
    assert_eq!(sent(), vec![json!({"key": "s", "value": "Play"})]);
    ASKS.lock().unwrap().push(Ask::UnwatchRead { key: "s".into() });

    // a React system: every frame its tick and its query's rows, whether or not they changed; none once it's gone
    let hp = Health::type_path();
    ASKS.lock().unwrap().push(Ask::System { key: "sys".into(), query: Some(json!({"data": {"components": [hp]}})) });
    app.update();
    let a = sent();
    app.update();
    let b = sent();
    assert_eq!((a.len(), b.len()), (1, 1));
    assert_eq!(a[0]["key"], json!("sys"));
    assert_eq!(b[0]["tick"].as_u64(), a[0]["tick"].as_u64().map(|t| t + 1));
    assert_eq!(a[0]["value"], b[0]["value"]);
    assert!(a[0]["value"].is_array());
    ASKS.lock().unwrap().push(Ask::Unsystem { key: "sys".into() });
    app.update();
    assert_eq!(sent(), Vec::<Value>::new());

    // the Engine's reads in render and live play: React's reads are the Engine's World's (where a composition's
    // meshes are) and come back through React's App; a twin read stays on the page; a new Engine App (a seek back
    // replays from frame 0 in one) starts with the reads still live; an unwatch stops it in both. (split is what
    // asks does with the setting on and an Engine claiming the queue: the setting is the process's, so not set here.)
    let engine = || {
        let mut e = App::new();
        e.add_plugins(MinimalPlugins).register_type::<Mana>();
        engine_side(&mut e);
        e
    };
    let hand_on = |app: &mut App, asks: Vec<Ask>| {
        let (here, there) = split(asks, true);
        ENGINE_ASKS.lock().unwrap().extend(there);
        apply(app.world_mut(), here);
    };
    let mp = Mana::type_path();
    let q = json!({"data": {"components": [mp]}});
    let mut eng = engine();
    let mana = eng.world_mut().spawn(Mana { mp: 3 }).id();
    hand_on(&mut app, vec![
        Ask::WatchRead { key: "m".into(), method: "world.query".into(), params: q.clone() },
        Ask::WatchRead { key: "tw".into(), method: TWIN.into(), params: json!({"op": 42}) },
    ]);
    eng.update();
    app.update();
    let got = sent();
    assert!(got.contains(&json!({"key": "tw", "value": tw.to_bits()})), "{got:?}");
    let m: Vec<&Value> = got.iter().filter(|p| p["key"] == "m").collect();
    assert_eq!(m.len(), 1, "{got:?}");
    assert_eq!(m[0]["value"][0]["entity"], json!(mana.to_bits()));
    assert_eq!(m[0]["value"][0]["components"][mp], json!({"mp": 3}));
    // a new Engine App starts with the read
    let mut eng = engine();
    eng.world_mut().spawn(Mana { mp: 9 });
    eng.update();
    app.update();
    let got = sent();
    assert_eq!(got.iter().find(|p| p["key"] == "m").map(|p| p["value"][0]["components"][mp].clone()), Some(json!({"mp": 9})), "{got:?}");
    // gone from both once unwatched
    hand_on(&mut app, vec![Ask::UnwatchRead { key: "m".into() }, Ask::UnwatchRead { key: "tw".into() }]);
    eng.update();
    eng.world_mut().spawn(Mana { mp: 1 });
    eng.update();
    app.update();
    assert_eq!(sent(), Vec::<Value>::new());
    assert!(ENGINE_LIVE.lock().unwrap().is_empty());
}

#[derive(States, Reflect, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(State, FreelyMutableState)]
enum Mode {
    #[default]
    Menu,
    Play,
}

#[derive(Event, Reflect, Clone)]
#[reflect(Event)]
struct Ping {
    n: u32,
}

#[derive(Component, Reflect, Debug, PartialEq, Clone)]
#[reflect(Component)]
struct Mana {
    mp: u32,
}
