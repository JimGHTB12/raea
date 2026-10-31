//! The MCP server: a bridge to the app's Bevy Remote Protocol (src/brp.rs). The two contract tools
//! (sdk/ui/conformance/conformance.feature, "The input tool" and "The ui tool"), `frame_png`, `state` and the page's actions are
//! the host's methods (`page.`, or the prefix the app serves them under: `serve`); `brp` calls any method by name
//! (Bevy's own world.query, world.get_components, registry.schema, schedule.graph, a plugin's, ...), and `brp_methods`
//! lists them (rpc.discover). Streamable HTTP, JSON answers only, on 127.0.0.1:{port}/mcp.

use std::io::{BufRead, BufReader, Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde_json::{Value, json};

/// What the app's work answers with (a picture, an action's value) on the frame it's done: src/brp.rs turns it into
/// the method's result.
pub enum Answer {
    Json(Value),
    Png(Vec<u8>, Value),
    Err(String),
}

/// Who the server says it is, and the prefix its tools call the host's methods under (`serve`).
pub struct Server {
    pub name: String,
    pub instructions: String,
    pub prefix: String,
}

static SERVER: std::sync::OnceLock<Server> = std::sync::OnceLock::new();

fn prefix() -> &'static str {
    SERVER.get().map_or("page", |s| s.prefix.as_str())
}

/// The port the bin serves on: RAEA_PAGE_MCP_PORT (0: no server), default 7477.
pub fn port() -> u16 {
    std::env::var("RAEA_PAGE_MCP_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(7477)
}

fn tools() -> Value {
    let obj = |props: Value, required: &[&str]| json!({"type": "object", "properties": props, "required": required});
    json!([
        {"name": "input", "description": "One input event in window points, posted as the window gets it (the messages winit writes, picking and focus see them as real): move {x,y}; down/up {x,y,button,clicks}; leave; wheel {x,y,dy,dx?,unit: pixel|line,phase?}; key {key, mods, state?: down|up} (no state: down then up); text {text}; ime {marked?, commit?}; resize {w,h}. Returns {frame}: the frame it landed before (the first frame that saw it).",
         "inputSchema": obj(json!({"type": {"type": "string"}, "x": {"type": "number"}, "y": {"type": "number"}, "button": {"type": "string"}, "clicks": {"type": "integer"}, "dy": {"type": "number"}, "dx": {"type": "number"}, "unit": {"type": "string"}, "phase": {"type": "string"}, "key": {"type": "string"}, "mods": {"type": "array", "items": {"type": "string"}}, "state": {"type": "string"}, "text": {"type": "string"}, "marked": {"type": "string"}, "commit": {"type": "string"}, "w": {"type": "number"}, "h": {"type": "number"}, "event": {"type": "object"}}), &[])},
        {"name": "ui", "description": "The interface as presented in the latest frame: {frame, window: {w,h}, nodes: [{id, role, name, rect: {x,y,w,h} (window points), visible, hovered, focused, pressed, selected, value, scroll?: {top, max}, text?}]}, one per element with a data-testid.",
         "inputSchema": obj(json!({}), &[])},
        {"name": "frame_png", "description": "The composited window as a PNG (the render target read back from the GPU).",
         "inputSchema": obj(json!({}), &[])},
        {"name": "state", "description": "The app's state: frame, window, scale, and what the app adds.",
         "inputSchema": obj(json!({}), &[])},
        {"name": "actions", "description": "The page's actions, as it names them (its emit(\"actions\", ...)): [{name, about, args}].",
         "inputSchema": obj(json!({}), &[])},
        {"name": "action", "description": "Runs one of the page's actions as a click there would: {name, args?}. Returns what it returned.",
         "inputSchema": obj(json!({"name": {"type": "string"}, "args": {}}), &["name"])},
        {"name": "brp", "description": format!("Any Bevy Remote Protocol method on the running app, by name: Bevy's own (world.query, world.get_components, world.insert_components, world.mutate_components, world.get_resources, world.write_message, world.trigger_event, registry.schema, schedule.list, schedule.graph, ...), the host's ({}.*) and any plugin's. A +watch method answers with its first result. See brp_methods.", prefix()),
         "inputSchema": obj(json!({"method": {"type": "string"}, "params": {}}), &["method"])},
        {"name": "brp_methods", "description": "The Bevy Remote Protocol methods this app answers (rpc.discover): [name].",
         "inputSchema": obj(json!({}), &[])}
    ])
}

/// Whether the app's BRP listens (its Startup has run: `ready`), and the waits on it.
static READY: (std::sync::Mutex<bool>, std::sync::Condvar) = (std::sync::Mutex::new(false), std::sync::Condvar::new());

/// The app's Startup has run, so BRP's listener is bound (PostStartup, src/lib.rs).
pub fn ready() {
    *READY.0.lock().unwrap() = true;
    READY.1.notify_all();
}

/// One BRP call on loopback: its result, or its error's message. A `+watch` method streams events; the first is the
/// answer, and the connection is dropped after it (bevy_remote then stops watching).
pub fn brp(method: &str, params: Value, limit: Duration) -> Result<Value, String> {
    static ID: AtomicU64 = AtomicU64::new(1);
    // the MCP server answers from the process's start, BRP once the app has built its plugins and run Startup (a GPU
    // device made first: seconds on a busy machine): a first call waits for that, within its limit
    {
        let up = READY.0.lock().unwrap();
        let (up, _) = READY.1.wait_timeout_while(up, limit, |up| !*up).unwrap();
        if !*up {
            return Err(format!("{method}: the app's BRP didn't start in {limit:?}"));
        }
    }
    let port = crate::brp::PORT.load(Ordering::SeqCst);
    let body = json!({"jsonrpc": "2.0", "id": ID.fetch_add(1, Ordering::SeqCst), "method": method, "params": params}).to_string();
    let mut sock = std::net::TcpStream::connect(("127.0.0.1", port)).map_err(|e| format!("brp on {port}: {e}"))?;
    sock.set_read_timeout(Some(limit)).ok();
    write!(sock, "POST / HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
        .map_err(|e| e.to_string())?;
    let mut r = BufReader::new(sock);
    let (mut stream, mut chunked, mut len) = (false, false, None::<usize>);
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).map_err(|e| format!("{method}: {e}"))? == 0 {
            return Err(format!("{method}: the app closed the connection"));
        }
        let l = line.trim_end().to_ascii_lowercase();
        if l.is_empty() {
            break;
        }
        stream |= l.starts_with("content-type:") && l.contains("event-stream");
        chunked |= l.starts_with("transfer-encoding:") && l.contains("chunked");
        if let Some(n) = l.strip_prefix("content-length:") {
            len = n.trim().parse().ok();
        }
    }
    let text = if stream {
        // the first event's data line (chunk sizes and blank lines between are skipped)
        loop {
            let mut line = String::new();
            if r.read_line(&mut line).map_err(|_| format!("{method}: no answer in {} s", limit.as_secs()))? == 0 {
                return Err(format!("{method}: the stream ended without an answer"));
            }
            if let Some(d) = line.trim_end().strip_prefix("data:") {
                break d.trim().to_string();
            }
        }
    } else if let Some(n) = len.filter(|_| !chunked) {
        let mut b = vec![0; n];
        r.read_exact(&mut b).map_err(|e| e.to_string())?;
        String::from_utf8_lossy(&b).into_owned()
    } else {
        let mut all = String::new();
        r.read_to_string(&mut all).map_err(|e| e.to_string())?;
        // chunked: the JSON is the text between the first chunk's size line and the end
        all.lines().filter(|l| l.trim_start().starts_with('{') || l.trim_start().starts_with('[')).collect::<Vec<_>>().join("")
    };
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{method}: {e}: {text}"))?;
    match (v.get("result"), v.get("error")) {
        (_, Some(e)) => Err(e["message"].as_str().map(str::to_string).unwrap_or_else(|| e.to_string())),
        (Some(r), _) => Ok(r.clone()),
        _ => Err(format!("{method}: {text}")),
    }
}

/// A unique `req` for a +watch method that starts work.
fn req() -> String {
    static N: AtomicU64 = AtomicU64::new(1);
    format!("mcp-{}", N.fetch_add(1, Ordering::SeqCst))
}

fn text(v: &Value) -> Value {
    json!({"content": [{"type": "text", "text": v.to_string()}], "structuredContent": v})
}

fn tool(name: &str, args: Value) -> Value {
    let long = Duration::from_secs(20);
    let m = |method: &str| format!("{}.{method}", prefix());
    let r = match name {
        "input" => brp(&m("input"), args, long),
        // read here, not through the ui method: each frame in turn from the one after the last handed out (src/tree.rs TREES)
        "ui" => {
            static SEEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let after = (SEEN.load(Ordering::SeqCst) + 1).max(crate::inject::LAST_INPUT.load(Ordering::SeqCst));
            match crate::tree::tree_after(after, Duration::from_secs(5)) {
                Some(t) => {
                    SEEN.store(t["frame"].as_u64().unwrap_or(0), Ordering::SeqCst);
                    Ok(t)
                }
                None => Err(format!("no frame {after} in 5 s")),
            }
        }
        "frame_png" | "screenshot" => match brp(&m("frame_png+watch"), json!({"req": req()}), long) {
            Ok(mut v) => {
                let png = v.as_object_mut().and_then(|o| o.remove("png")).and_then(|p| p.as_str().map(str::to_string)).unwrap_or_default();
                return json!({"content": [{"type": "image", "data": png, "mimeType": "image/png"}, {"type": "text", "text": v.to_string()}], "structuredContent": v});
            }
            Err(e) => Err(e),
        },
        "state" => brp(&m("state"), json!({}), long),
        "actions" => brp(&m("actions"), json!({}), long),
        "action" => {
            let mut a = if args.is_object() { args } else { json!({}) };
            a["req"] = json!(req());
            brp(&m("action+watch"), a, long)
        }
        "brp" => brp(args["method"].as_str().unwrap_or(""), args.get("params").cloned().unwrap_or(Value::Null), long),
        "brp_methods" => brp("rpc.discover", Value::Null, long).map(|d| json!(d["methods"].as_array().map(|m| m.iter().filter_map(|x| x["name"].as_str()).collect::<Vec<_>>()).unwrap_or_default())),
        _ => Err(format!("no tool {name}")),
    };
    match r {
        Ok(v) => text(&v),
        Err(e) => json!({"content": [{"type": "text", "text": e}], "isError": true}),
    }
}

fn handle(msg: &Value) -> Option<Value> {
    let id = msg.get("id").cloned()?;
    let method = msg["method"].as_str().unwrap_or("");
    let result: Result<Value, Value> = match method {
        "initialize" => Ok(json!({
            "protocolVersion": msg["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": SERVER.get().map_or("raea-page-host", |s| s.name.as_str()), "version": env!("CARGO_PKG_VERSION")},
            "instructions": SERVER.get().map_or("A raea page. Drive it with input, read it with ui; brp reaches the whole ECS (Bevy Remote Protocol).", |s| s.instructions.as_str()),
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tools()})),
        "tools/call" => Ok(tool(msg["params"]["name"].as_str().unwrap_or(""), msg["params"]["arguments"].clone())),
        _ => Err(json!({"code": -32601, "message": format!("no method {method}")})),
    };
    Some(match result {
        Ok(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
        Err(e) => json!({"jsonrpc": "2.0", "id": id, "error": e}),
    })
}

/// Starts the server thread on `port` (0: none), as `server` says.
pub fn serve(port: u16, server: Server) {
    let _ = SERVER.set(server);
    if port == 0 {
        return;
    }
    std::thread::Builder::new()
        .name("page-host-mcp".into())
        .spawn(move || {
            let server = match tiny_http::Server::http(("127.0.0.1", port)) {
                Ok(s) => s,
                Err(e) => return eprintln!("{}: mcp can't listen on {port}: {e}", SERVER.get().map_or("raea-page-host", |s| s.name.as_str())),
            };
            eprintln!("{}: mcp http://127.0.0.1:{port}/mcp", SERVER.get().map_or("raea-page-host", |s| s.name.as_str()));
            for mut req in server.incoming_requests() {
                std::thread::spawn(move || {
                    if req.method() != &tiny_http::Method::Post {
                        let _ = req.respond(tiny_http::Response::empty(405));
                        return;
                    }
                    let mut body = String::new();
                    let _ = req.as_reader().read_to_string(&mut body);
                    let answer = match serde_json::from_str::<Value>(&body) {
                        // a batch, or one message
                        Ok(Value::Array(msgs)) => {
                            let out: Vec<Value> = msgs.iter().filter_map(handle).collect();
                            if out.is_empty() { None } else { Some(Value::Array(out)) }
                        }
                        Ok(msg) => handle(&msg),
                        Err(e) => Some(json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": e.to_string()}})),
                    };
                    let resp = match answer {
                        Some(a) => tiny_http::Response::from_string(a.to_string())
                            .with_header("Content-Type: application/json".parse::<tiny_http::Header>().unwrap())
                            .boxed(),
                        None => tiny_http::Response::empty(202).boxed(),
                    };
                    let _ = req.respond(resp);
                });
            }
        })
        .unwrap();
}
