//! The Bevy Remote Protocol: Bevy's own JSON-RPC over HTTP (bevy_remote), with every built-in method (world.query,
//! world.get_components, world.insert_components, world.write_message, registry.schema, schedule.graph, rpc.discover,
//! ...) and the host's own beside them, under a prefix (`page.`; an app may serve them under another name too:
//! `methods`):
//!
//!   page.input                one input event (src/inject.rs: the conformance contract's `input`); answers {frame},
//!                             the frame it lands before
//!   page.ui+watch             the tree (src/tree.rs) once a frame at least {after} (default: the latest input's) has
//!                             ended
//!   page.frame_png+watch      {req}: the composited window, {png (base64), frame, w, h}, once read back
//!   page.state                frame, window, scale, and what the app adds (tree::Extra)
//!   page.a11y                 the page's accessibility tree, one record per data-testid
//!   page.actions              the page's actions (`emit("actions", ...)`: raea-runtime-js's SpaceActions)
//!   page.action+watch         {req, name, args?}: one run, its value once the page has run it
//!
//! A `+watch` method answers as an event stream (bevy_remote's HTTP transport): its handler runs at the end of every
//! frame until it has an answer. Those that start work (a picture) take a `req`, any string unique to the caller, so a
//! run's work is started once and found again on the frames after. The MCP server (src/mcp.rs) is a bridge to these
//! methods, so an agent reaches the same world through either.
//!
//! Handlers run in bevy_remote's RemoteLast, after Last: an input taken there goes in at the next frame's start
//! (inject::pump), and a tree read there is the frame just laid out and drawn.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};

use base64::Engine as _;
use bevy::prelude::*;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::{BrpError, BrpResult, RemotePlugin, error_codes};
use raea_runtime_js::calls::SpaceActions;
use serde_json::{Value, json};

use crate::inject::{Frame, Pending};
use crate::mcp::Answer;

/// The port the bin's BRP listens on: RAEA_PAGE_BRP_PORT, else one the system has free (each run has its own).
pub fn port() -> u16 {
    std::env::var("RAEA_PAGE_BRP_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or_else(free_port)
}

/// A port the system has free.
pub fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map(|a| a.port()).unwrap_or(15702)
}

/// The port this app's BRP took (src/mcp.rs talks to it).
pub static PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);

/// Work a `+watch` method started, by its caller's `req`: answered on a later frame. An answered run stays here as
/// None: bevy_remote runs a watcher every frame until it finds its caller gone, which it does only on the next answer
/// it sends, so the frame after the answer runs the handler again and the run mustn't start a second time (an action
/// ran twice: a toggle undone, two tabs opened).
#[derive(Resource, Default)]
pub struct Started(pub HashMap<String, Option<Mutex<Receiver<Answer>>>>);

/// The host's methods under `prefix` (page, or an app's own name for them).
pub fn methods(r: RemotePlugin, prefix: &str) -> RemotePlugin {
    r.with_method_main(format!("{prefix}.input"), input)
        .with_watching_method_main(format!("{prefix}.ui+watch"), ui)
        .with_watching_method_main(format!("{prefix}.frame_png+watch"), frame_png)
        .with_method_main(format!("{prefix}.state"), state)
        .with_method_main(format!("{prefix}.actions"), actions)
        .with_watching_method_main(format!("{prefix}.action+watch"), action)
}

/// The plugins on `port`, the host's methods under `page.` (an app adds its own to `r` first: `with`).
pub fn plugins(port: u16, with: impl FnOnce(RemotePlugin) -> RemotePlugin) -> (RemotePlugin, RemoteHttpPlugin) {
    PORT.store(port, std::sync::atomic::Ordering::SeqCst);
    eprintln!("raea-page-host: brp http://127.0.0.1:{port}");
    let r = methods(with(RemotePlugin::default()), "page").with_method_main("page.a11y", |In(_): In<Option<Value>>, world: &mut World| -> BrpResult {
        Ok(crate::page::a11y(world))
    });
    (r, RemoteHttpPlugin::default().with_port(port))
}

pub fn invalid(m: impl Into<String>) -> BrpError {
    BrpError { code: error_codes::INVALID_PARAMS, message: m.into(), data: None }
}

pub fn internal(m: impl Into<String>) -> BrpError {
    BrpError { code: error_codes::INTERNAL_ERROR, message: m.into(), data: None }
}

pub fn input(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let v = p.ok_or_else(|| invalid("input needs an event"))?;
    let v = if v.get("event").is_some_and(Value::is_object) { v["event"].clone() } else { v };
    let kind = v.get("type").or_else(|| v.get("kind")).or_else(|| v.get("event")).and_then(Value::as_str).unwrap_or("");
    const KINDS: &[&str] = &["move", "mousemove", "down", "up", "mousedown", "mouseup", "leave", "mouseleave", "wheel", "scroll", "key", "keydown", "keyup", "text", "insert", "ime", "resize"];
    if !KINDS.contains(&kind) {
        return Err(invalid(format!("no event {kind:?}: {}", KINDS.join(", "))));
    }
    let next = world.resource::<Frame>().0 + 1;
    let frame = world.resource_mut::<Pending>().queue(v, next);
    crate::inject::LAST_INPUT.fetch_max(frame, std::sync::atomic::Ordering::SeqCst);
    Ok(json!({"frame": frame}))
}

pub fn ui(In(p): In<Option<Value>>) -> BrpResult<Option<Value>> {
    let after = p.as_ref().and_then(|p| p["after"].as_u64()).unwrap_or_else(|| crate::inject::LAST_INPUT.load(std::sync::atomic::Ordering::SeqCst));
    Ok(crate::tree::LATEST.lock().unwrap().as_ref().filter(|t| t["frame"].as_u64().unwrap_or(0) >= after).cloned())
}

/// The caller's `req`, required by methods that start work.
pub fn req(p: &Option<Value>) -> Result<String, BrpError> {
    p.as_ref().and_then(|p| p.get("req")).map(|r| r.to_string()).ok_or_else(|| invalid("needs a req: any string unique to this call"))
}

/// A started run's answer, once there is one.
pub fn done(started: &mut Started, req: &str) -> BrpResult<Option<Value>> {
    let Some(Some(rx)) = started.0.get(req) else { return Ok(None) };
    let a = match rx.lock().unwrap().try_recv() {
        Ok(a) => a,
        Err(std::sync::mpsc::TryRecvError::Empty) => return Ok(None),
        Err(_) => Answer::Err("the work stopped without an answer".into()),
    };
    started.0.insert(req.to_string(), None);
    match a {
        Answer::Json(v) => Ok(Some(v)),
        Answer::Png(png, mut v) => {
            v["png"] = json!(base64::engine::general_purpose::STANDARD.encode(&png));
            Ok(Some(v))
        }
        Answer::Err(e) => Err(internal(e)),
    }
}

pub fn frame_png(In(p): In<Option<Value>>, mut started: ResMut<Started>, mut pending: ResMut<Pending>) -> BrpResult<Option<Value>> {
    let req = req(&p)?;
    if !started.0.contains_key(&req) {
        let (tx, rx) = channel();
        pending.png.push(tx);
        started.0.insert(req.clone(), Some(Mutex::new(rx)));
    }
    done(&mut started, &req)
}

pub fn state(In(_): In<Option<Value>>) -> BrpResult {
    crate::tree::STATE.lock().unwrap().clone().ok_or_else(|| internal("no frame has ended yet"))
}

pub fn actions(In(_): In<Option<Value>>, acts: Option<Res<SpaceActions>>) -> BrpResult {
    let acts = acts.ok_or_else(|| internal("no page with actions is mounted"))?;
    serde_json::to_value(acts.list()).map_err(|e| internal(e.to_string()))
}

pub fn action(In(p): In<Option<Value>>, mut started: ResMut<Started>, acts: Option<ResMut<SpaceActions>>) -> BrpResult<Option<Value>> {
    let req = req(&p)?;
    if !started.0.contains_key(&req) {
        let mut acts = acts.ok_or_else(|| internal("no page with actions is mounted"))?;
        let p = p.unwrap_or_default();
        let name = p["name"].as_str().ok_or_else(|| invalid("action needs a name"))?;
        let run = acts.run(name, p.get("args").cloned().unwrap_or(Value::Null));
        let (tx, rx) = channel();
        // the page answers on its own thread's time; the answer waits here for the next frame's check
        std::thread::spawn(move || {
            let _ = tx.send(match run.recv_timeout(std::time::Duration::from_secs(15)) {
                Ok(Ok(v)) => Answer::Json(json!({"value": v})),
                Ok(Err(e)) => Answer::Err(e),
                Err(_) => Answer::Err("the page didn't finish the action in 15 s".into()),
            });
        });
        started.0.insert(req.clone(), Some(Mutex::new(rx)));
    }
    done(&mut started, &req)
}
