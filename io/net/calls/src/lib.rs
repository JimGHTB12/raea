//! The net.* host calls under @raea/ui's platform layer (fetch, WebSocket and localStorage, sdk/ui/src/platform.ts),
//! answered the same on both app targets (docs/targets.feature T2a, whose call surface lists them): the web target's
//! runtime/js net.rs and the native host's js.rs each hand a call here with the root that made it and how to answer.
//! Nothing here knows Bevy or Rune: a root is whatever key its host gives it (an Entity on web, the page on native).
//!
//!   net.fetch {url, method, headers: [[k, v]], body: base64 | null, redirect: "follow" | "error" | "manual"}
//!             -> {status, statusText, headers: [[k, v]], body: base64, url, redirected}; a network failure is refused
//!   net.ws.open {id, url, protocols}      a socket on a thread of its own; what happens to it comes back as the host
//!                                         event `net.ws` {id, ev: "open" | "message" | "close", ...}
//!   net.ws.send {id, data, binary?}       text, or base64 bytes
//!   net.ws.close {id, code, reason}
//!   net.storage.get {area} -> {k: v}      localStorage kept across runs, in $BEVY_TSX_STORAGE/<area>.json; without
//!   net.storage.set {area, change}        that variable these are refused and storage stays in the page's memory
//!
//! fetch and the sockets (raea-io-net's http and ws) block on threads of their own, never the caller's frame.

use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};

use raea_io_net::ws::Cmd;
use serde_json::{Value, json};

/// Every call this crate answers.
pub const CALLS: [&str; 6] = ["net.fetch", "net.ws.open", "net.ws.send", "net.ws.close", "net.storage.get", "net.storage.set"];

/// A call's answer, once, from any thread.
pub type Done = Box<dyn FnOnce(Result<Value, String>) + Send>;
/// A host event for a root (the wire's `named` message, `net.ws`).
pub type Push<K> = Arc<dyn Fn(K, Value) + Send + Sync>;

/// The calls' state, shared by every root of a host: the open sockets, by root and the page's id for each.
pub struct Net<K> {
    sockets: Arc<Mutex<HashMap<(K, u64), Sender<Cmd>>>>,
    push: Push<K>,
}

impl<K> Clone for Net<K> {
    fn clone(&self) -> Self {
        Net { sockets: self.sockets.clone(), push: self.push.clone() }
    }
}

impl<K: Copy + Eq + Hash + Send + 'static> Net<K> {
    pub fn new(push: Push<K>) -> Net<K> {
        Net { sockets: Arc::default(), push }
    }

    /// `name` (one of CALLS) from `root` with `args`, answered through `done`, now or from a thread of its own.
    pub fn call(&self, root: K, name: &str, args: Value, done: Done) {
        match name {
            "net.fetch" => {
                std::thread::spawn(move || done(raea_io_net::http::fetch(&args)));
            }
            "net.ws.open" => {
                let id = args["id"].as_u64().unwrap_or(0);
                let (tx, rx) = channel();
                self.sockets.lock().unwrap().insert((root, id), tx);
                let (push, sockets) = (self.push.clone(), self.sockets.clone());
                std::thread::spawn(move || {
                    raea_io_net::ws::run(&args, rx, &|ev: Value| {
                        let mut m = ev;
                        m["id"] = json!(id);
                        push(root, raea_core_wire::named("net.ws", m));
                    });
                    sockets.lock().unwrap().remove(&(root, id));
                });
                done(Ok(Value::Null));
            }
            "net.ws.send" | "net.ws.close" => {
                let cmd = if name == "net.ws.send" {
                    Cmd::Send { data: args["data"].as_str().unwrap_or("").to_string(), binary: args["binary"] == json!(true) }
                } else {
                    Cmd::Close(args["code"].as_u64().unwrap_or(1000) as u16, args["reason"].as_str().unwrap_or("").to_string())
                };
                let key = (root, args["id"].as_u64().unwrap_or(0));
                done(match self.sockets.lock().unwrap().get(&key) {
                    Some(tx) if tx.send(cmd).is_ok() => Ok(Value::Null),
                    _ => Err("no such socket".into()),
                });
            }
            "net.storage.get" => done(get(&args)),
            "net.storage.set" => done(set(&args)),
            _ => done(Err(format!("{name} isn't a net call"))),
        }
    }
}

const NO_STORE: &str = "no storage (BEVY_TSX_STORAGE isn't set)";

fn get(args: &Value) -> Result<Value, String> {
    let p = store(args).ok_or(NO_STORE)?;
    Ok(std::fs::read_to_string(p).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({})))
}

fn set(args: &Value) -> Result<Value, String> {
    let p = store(args).ok_or(NO_STORE)?;
    let mut m: serde_json::Map<String, Value> = std::fs::read_to_string(&p).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    let c = &args["change"];
    if c["clear"] == json!(true) {
        m.clear();
    } else if let Some(k) = c["k"].as_str() {
        match c["v"].as_str() {
            Some(v) => m.insert(k.to_string(), json!(v)),
            None => m.remove(k),
        };
    }
    let _ = std::fs::create_dir_all(p.parent().unwrap_or(std::path::Path::new(".")));
    std::fs::write(&p, Value::Object(m).to_string()).map(|()| Value::Null).map_err(|e| e.to_string())
}

fn store(args: &Value) -> Option<PathBuf> {
    let dir = std::env::var_os("BEVY_TSX_STORAGE")?;
    let area = args["area"].as_str().filter(|a| a.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))?;
    Some(PathBuf::from(dir).join(format!("{area}.json")))
}

#[cfg(feature = "serve")]
pub mod serve;
