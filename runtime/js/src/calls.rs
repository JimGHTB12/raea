//! Calls across a root's edge, both ways (src/host.rs runs them):
//!
//! - A root calling the app: `call(name, args)` (@raea/ui's wire, sdk/ui/src/wire.ts) is a promise of what the app's
//!   function for `name` answers. The app registers them on `HostCalls`; an answer can come at once or later, from any
//!   thread (`Answer` is Send), and the promise settles in the frame it arrives. A name nobody registered falls to the
//!   world API (src/world.rs: any BRP method), then to the app's `otherwise`.
//! - The app (an agent, a test) calling a root: the root names its actions with `actions([...])` (a name, what it does,
//!   its args), and `SpaceActions` keeps the list and runs one: handed to the root as the host event `action` at its
//!   next step, its result (or error) back through the root's emit `actionDone`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

use bevy::prelude::*;
use serde_json::{Value, json};

type Done = (Entity, u64, Result<Value, String>);

/// What a host call answers: `ok` or `err`, once, now or later (it's Send: an app can answer from a task).
pub struct Answer {
    root: Entity,
    id: u64,
    tx: Sender<Done>,
}

impl Answer {
    pub fn ok(self, v: Value) {
        let _ = self.tx.send((self.root, self.id, Ok(v)));
    }
    pub fn err(self, e: impl Into<String>) {
        let _ = self.tx.send((self.root, self.id, Err(e.into())));
    }
    /// The root that called.
    pub fn root(&self) -> Entity {
        self.root
    }
}

type HostFn = Box<dyn Fn(&mut World, Value, Answer) + Send + Sync>;
type Fallback = Box<dyn Fn(&mut World, &str, Value) -> Option<Result<Value, String>> + Send + Sync>;

/// The app's functions a root can call, by name.
#[derive(Resource)]
pub struct HostCalls {
    fns: HashMap<String, HostFn>,
    /// what answers a name no function was registered for (a table of plugin methods): Some(result), or None when it
    /// doesn't know the name either
    otherwise: Option<Fallback>,
    tx: Sender<Done>,
    rx: Mutex<Receiver<Done>>,
    /// host events a call's work sends its root later (a socket's messages: src/net.rs)
    pushed: Mutex<Receiver<(Entity, Value)>>,
}

impl Default for HostCalls {
    /// With the platform's built-in calls (src/net.rs: fetch, WebSocket, localStorage's persistence; src/webaudio.rs:
    /// decodeAudioData), which an app's own `on` under the same name replaces.
    fn default() -> Self {
        let (tx, rx) = channel();
        let (push, pushed) = channel();
        let mut c = HostCalls { fns: HashMap::new(), otherwise: None, tx, rx: Mutex::new(rx), pushed: Mutex::new(pushed) };
        crate::net::install(&mut c, push);
        #[cfg(feature = "media")]
        crate::webaudio::install(&mut c);
        c
    }
}

impl HostCalls {
    /// `name`'s function: given the World (in the frame the root called), the args, and the answer to give.
    pub fn on(&mut self, name: &str, f: impl Fn(&mut World, Value, Answer) + Send + Sync + 'static) -> &mut Self {
        self.fns.insert(name.to_string(), Box::new(f));
        self
    }

    /// What answers the names no function was registered for, at once: Some(result), or None for a name it doesn't
    /// know either (the root's promise is refused).
    pub fn otherwise(&mut self, f: impl Fn(&mut World, &str, Value) -> Option<Result<Value, String>> + Send + Sync + 'static) -> &mut Self {
        self.otherwise = Some(Box::new(f));
        self
    }

    /// Whether the app answers `name`.
    pub fn has(&self, name: &str) -> bool {
        self.fns.contains_key(name)
    }

    /// The answers that have come in, each for its root, as the wire's answers.
    pub fn answers(&self) -> Vec<(Entity, Value)> {
        let mut out: Vec<(Entity, Value)> = self.rx.lock().unwrap().try_iter().map(|(root, id, r)| (root, raea_core_wire::answer(id, r))).collect();
        out.extend(self.pushed.lock().unwrap().try_iter());
        out
    }
}

/// Root `root`'s call `name` (the wire's call `id`), run on the World.
pub fn call(world: &mut World, root: Entity, id: u64, name: &str, args: Value) {
    world.init_resource::<HostCalls>();
    world.resource_scope(|world, calls: Mut<HostCalls>| {
        let answer = Answer { root, id, tx: calls.tx.clone() };
        match calls.fns.get(name) {
            Some(f) => f(world, args, answer),
            // the world API: any BRP method, run now (src/world.rs)
            None if crate::world::READS.contains(&name) || crate::world::WRITES.contains(&name) => match raea_bevy_world::call(world, name, args) {
                Ok(v) => answer.ok(v),
                Err(e) => answer.err(e),
            },
            None => match calls.otherwise.as_ref().and_then(|f| f(world, name, args)) {
                Some(Ok(v)) => answer.ok(v),
                Some(Err(e)) => answer.err(e),
                None => answer.err(format!("the host has no call {name:?} (an app adds one with raea_runtime_js::HostCalls::on)")),
            },
        }
    });
}

/// One of a root's actions, as it named it.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActionSpec {
    pub name: String,
    #[serde(default)]
    pub about: String,
    #[serde(default)]
    pub args: Value,
}

/// The actions of the root that last named some, and the runs of them waiting on it.
#[derive(Resource, Default)]
pub struct SpaceActions {
    list: Vec<ActionSpec>,
    owner: Option<Entity>,
    next: u64,
    waiting: HashMap<u64, Sender<Result<Value, String>>>,
    /// runs asked for, for the root's next step
    asked: Vec<Value>,
}

impl SpaceActions {
    /// What the root can do, as it named its actions.
    pub fn list(&self) -> &[ActionSpec] {
        &self.list
    }

    /// Runs action `name` with `args` as a click in the root would: handed to it at its next step; the result arrives
    /// on the receiver (an error if it has no such action, or it threw).
    pub fn run(&mut self, name: &str, args: Value) -> Receiver<Result<Value, String>> {
        let (tx, rx) = channel();
        if !self.list.iter().any(|a| a.name == name) {
            let names: Vec<&str> = self.list.iter().map(|a| a.name.as_str()).collect();
            let _ = tx.send(Err(format!("the space has no action {name:?} (it has: {})", names.join(", "))));
            return rx;
        }
        self.next += 1;
        self.waiting.insert(self.next, tx);
        self.asked.push(raea_core_wire::named("action", json!({"id": self.next, "name": name, "args": args})));
        rx
    }

    /// A root's list, as it emitted it (`actions`).
    pub fn set(&mut self, root: Entity, v: Value) {
        self.list = serde_json::from_value(v).unwrap_or_default();
        self.owner = Some(root);
    }

    /// A run's end, as the root emitted it (`actionDone` {id, value} or {id, error}).
    pub fn done(&mut self, v: &Value) {
        let Some(tx) = v["id"].as_u64().and_then(|id| self.waiting.remove(&id)) else { return };
        let _ = tx.send(match v.get("error").filter(|e| !e.is_null()) {
            Some(e) => Err(e.as_str().map(str::to_string).unwrap_or_else(|| e.to_string())),
            None => Ok(v.get("value").cloned().unwrap_or(Value::Null)),
        });
    }

    /// The runs asked for since the last step, as host events for the root that owns the actions.
    pub fn take_asked(&mut self) -> Vec<(Entity, Value)> {
        let asked = std::mem::take(&mut self.asked);
        match self.owner {
            Some(r) => asked.into_iter().map(|m| (r, m)).collect(),
            None => vec![],
        }
    }
}

// ---------------------------------------------------------------- synchronous calls

/// A call the root's JS waits on, answered on its own thread (`__ui_sync(name, json)`, sdk/ui's `wire.callSync`): for
/// the few web APIs that return at once with what only the host has, as getImageData does a canvas's pixels and
/// getComputedStyle an element's style. Process-wide, by name; the last registration wins. A plain one (`on_sync`)
/// sees no World; a context one (`on_sync_ctx`) gets the calling root's state, as its host lent it (`with_sync_ctx`).
pub type SyncFn = fn(&Value) -> Result<Value, String>;

/// A synchronous call answered from the calling root's state: what its host passed `with_sync_ctx` around the JS call
/// (raea-page's `PageSync`, raea-runtime-js's own `RootSync`), to downcast.
pub type SyncCtxFn = fn(&mut dyn std::any::Any, &Value) -> Result<Value, String>;

#[derive(Clone, Copy)]
enum SyncCall {
    Plain(SyncFn),
    Ctx(SyncCtxFn),
}

static SYNC: std::sync::RwLock<Option<HashMap<String, SyncCall>>> = std::sync::RwLock::new(None);

thread_local! {
    /// the state lent to this thread's JS call (with_sync_ctx); taken while a handler has it, so a call it makes back
    /// into JS can't reach it twice
    static CTX: std::cell::Cell<Option<*mut dyn std::any::Any>> = const { std::cell::Cell::new(None) };
}

/// Registers `f` as the synchronous call `name`.
pub fn on_sync(name: &str, f: SyncFn) {
    SYNC.write().unwrap().get_or_insert_with(HashMap::new).insert(name.to_string(), SyncCall::Plain(f));
}

/// Registers `f` as the synchronous call `name`, answered from the calling root's state (with_sync_ctx's `ctx`). A
/// call made with nothing lent (outside a host's step, input or mount) is an error.
pub fn on_sync_ctx(name: &str, f: SyncCtxFn) {
    SYNC.write().unwrap().get_or_insert_with(HashMap::new).insert(name.to_string(), SyncCall::Ctx(f));
}

/// `f` (a root's JS call) with `ctx` lent to the context calls it makes; the state lent before is back after, so
/// nesting is fine. `ctx` should own nothing borrowed (raw pointers, valid for this call, as PageSync's are).
pub fn with_sync_ctx<T>(ctx: &mut dyn std::any::Any, f: impl FnOnce() -> T) -> T {
    struct Back(Option<*mut dyn std::any::Any>);
    impl Drop for Back {
        fn drop(&mut self) {
            CTX.with(|c| c.set(self.0));
        }
    }
    let _back = Back(CTX.with(|c| c.replace(Some(ctx as *mut dyn std::any::Any))));
    f()
}

/// Runs the synchronous call `name` on `json`: `{"ok": value}` or `{"err": message}`, as JSON.
pub fn sync(name: &str, json: &str) -> String {
    let f = SYNC.read().unwrap().as_ref().and_then(|m| m.get(name).copied());
    let out = match (f, serde_json::from_str::<Value>(json)) {
        (None, _) => Err(format!("no synchronous host call {name}")),
        (_, Err(e)) => Err(e.to_string()),
        (Some(SyncCall::Plain(f)), Ok(v)) => f(&v),
        (Some(SyncCall::Ctx(f)), Ok(v)) => match CTX.with(|c| c.take()) {
            None => Err(format!("{name}: called outside a step")),
            Some(p) => {
                // SAFETY: p is with_sync_ctx's `ctx`, borrowed for its call, which this is inside of; taken out of CTX
                // while f has it, so nothing else reaches it
                let r = f(unsafe { &mut *p }, &v);
                CTX.with(|c| c.set(Some(p)));
                r
            }
        },
    };
    match out {
        Ok(v) => json!({ "ok": v }),
        Err(e) => json!({ "err": e }),
    }
    .to_string()
}
