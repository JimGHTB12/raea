//! The network and storage under the platform layer's fetch, WebSocket and localStorage (sdk/ui/src/platform.ts,
//! docs/ts-native.feature P17), as host calls every root has unless the app registers its own under the same names:
//!
//!   net.fetch {url, method, headers: [[k, v]], body: base64 | null, redirect: "follow" | "error" | "manual"}
//!             -> {status, statusText, headers: [[k, v]], body: base64, url, redirected}; a network failure is refused
//!   net.ws.open {id, url, protocols}      a socket on a thread of its own; what happens to it comes back as the host
//!                                         event `net.ws` {id, ev: "open" | "message" | "close", ...}
//!   net.ws.send {id, data, binary?}       text, or base64 bytes
//!   net.ws.close {id, code, reason}
//!   net.storage.get {area} -> {k: v}      localStorage kept across runs, in $BEVY_TSX_STORAGE/<area>.json; without
//!   net.storage.set {area, change}        that variable these are refused and storage stays in memory
//!
//! fetch and the sockets (raea-io-net's http and ws) block on threads of their own, never the frame. There's no clipboard call: the platform's
//! navigator.clipboard is the process's own until an app registers `net.clipboard.read`/`write` (a headless test
//! never touches the system's).

use std::collections::HashMap;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use raea_io_net::ws::Cmd;
use serde_json::{Value, json};

use crate::calls::HostCalls;

type Sockets = Arc<Mutex<HashMap<(Entity, u64), Sender<Cmd>>>>;

/// The calls, onto `calls` (HostCalls's default has them); `push` carries a socket's events to its root.
pub fn install(calls: &mut HostCalls, push: Sender<(Entity, Value)>) {
    calls.on("net.fetch", |_w, args, answer| {
        std::thread::spawn(move || match raea_io_net::http::fetch(&args) {
            Ok(v) => answer.ok(v),
            Err(e) => answer.err(e),
        });
    });
    let sockets: Sockets = Arc::default();
    let s = sockets.clone();
    calls.on("net.ws.open", move |_w, args, answer| {
        let root = answer.root();
        let id = args["id"].as_u64().unwrap_or(0);
        let (tx, rx) = channel();
        s.lock().unwrap().insert((root, id), tx);
        let (push, s) = (push.clone(), s.clone());
        std::thread::spawn(move || {
            raea_io_net::ws::run(&args, rx, &|ev: Value| {
                let mut m = ev;
                m["id"] = json!(id);
                let _ = push.send((root, raea_core_wire::named("net.ws", m)));
            });
            s.lock().unwrap().remove(&(root, id));
        });
        answer.ok(Value::Null);
    });
    let s = sockets.clone();
    calls.on("net.ws.send", move |_w, args, answer| {
        let key = (answer.root(), args["id"].as_u64().unwrap_or(0));
        let msg = Cmd::Send { data: args["data"].as_str().unwrap_or("").to_string(), binary: args["binary"] == json!(true) };
        match s.lock().unwrap().get(&key) {
            Some(tx) if tx.send(msg).is_ok() => answer.ok(Value::Null),
            _ => answer.err("no such socket"),
        }
    });
    let s = sockets;
    calls.on("net.ws.close", move |_w, args, answer| {
        let key = (answer.root(), args["id"].as_u64().unwrap_or(0));
        let code = args["code"].as_u64().unwrap_or(1000) as u16;
        let reason = args["reason"].as_str().unwrap_or("").to_string();
        match s.lock().unwrap().get(&key) {
            Some(tx) if tx.send(Cmd::Close(code, reason)).is_ok() => answer.ok(Value::Null),
            _ => answer.err("no such socket"),
        }
    });
    calls.on("net.storage.get", |_w, args, answer| match store(&args) {
        Some(p) => answer.ok(std::fs::read_to_string(p).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(json!({}))),
        None => answer.err("no storage (BEVY_TSX_STORAGE isn't set)"),
    });
    calls.on("net.storage.set", |_w, args, answer| {
        let Some(p) = store(&args) else { return answer.err("no storage (BEVY_TSX_STORAGE isn't set)") };
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
        match std::fs::write(&p, Value::Object(m).to_string()) {
            Ok(()) => answer.ok(Value::Null),
            Err(e) => answer.err(e.to_string()),
        }
    });
}

fn store(args: &Value) -> Option<std::path::PathBuf> {
    let dir = std::env::var_os("BEVY_TSX_STORAGE")?;
    let area = args["area"].as_str().filter(|a| a.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))?;
    Some(std::path::PathBuf::from(dir).join(format!("{area}.json")))
}
