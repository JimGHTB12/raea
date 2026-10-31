//! A WebSocket's life on a thread of its own, for the page's net.ws calls: connect, then read (in short waits, so what
//! the page sends goes out between) until it closes; each event handed to `tell` as
//! {ev: "open" | "message" | "close", ...}.

use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use serde_json::{Value, json};
use tungstenite::client::IntoClientRequest;
use tungstenite::{Message, protocol::CloseFrame, protocol::frame::coding::CloseCode, stream::MaybeTlsStream};

/// What the page asks of an open socket.
pub enum Cmd {
    /// text, or with `binary` base64 bytes
    Send { data: String, binary: bool },
    Close(u16, String),
}

/// The socket `args` {url, protocols} opens, run until it closes or `rx` hangs up.
pub fn run(args: &Value, rx: Receiver<Cmd>, tell: &dyn Fn(Value)) {
    let fail = || tell(json!({"ev": "close", "code": 1006, "reason": "", "clean": false, "error": true}));
    let Some(mut req) = args["url"].as_str().and_then(|u| u.into_client_request().ok()) else { return fail() };
    let protos: Vec<String> = args["protocols"].as_array().into_iter().flatten().filter_map(|p| p.as_str().map(String::from)).collect();
    if !protos.is_empty()
        && let Ok(v) = protos.join(", ").parse()
    {
        req.headers_mut().insert("Sec-WebSocket-Protocol", v);
    }
    let Ok((mut ws, resp)) = tungstenite::connect(req) else { return fail() };
    let wait = Some(Duration::from_millis(15));
    match ws.get_mut() {
        MaybeTlsStream::Plain(s) => drop(s.set_read_timeout(wait)),
        MaybeTlsStream::Rustls(s) => drop(s.get_mut().set_read_timeout(wait)),
        _ => {}
    }
    let protocol = resp.headers().get("sec-websocket-protocol").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    tell(json!({"ev": "open", "protocol": protocol}));
    loop {
        loop {
            match rx.try_recv() {
                Ok(Cmd::Send { data, binary }) => {
                    let m = if binary { Message::Binary(B64.decode(data).unwrap_or_default().into()) } else { Message::Text(data.into()) };
                    if ws.send(m).is_err() {
                        return fail();
                    }
                }
                Ok(Cmd::Close(code, reason)) => {
                    let _ = ws.close(Some(CloseFrame { code: CloseCode::from(code), reason: reason.into() }));
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let _ = ws.close(None);
                    return;
                }
            }
        }
        match ws.read() {
            Ok(Message::Text(t)) => tell(json!({"ev": "message", "data": t.as_str()})),
            Ok(Message::Binary(b)) => tell(json!({"ev": "message", "data": B64.encode(&b), "binary": true})),
            Ok(Message::Close(f)) => {
                // answered (tungstenite replies to the close as it reads it); the clean end comes as ConnectionClosed
                let (code, reason) = f.map(|f| (u16::from(f.code), f.reason.to_string())).unwrap_or((1005, String::new()));
                let _ = ws.flush();
                tell(json!({"ev": "close", "code": code, "reason": reason, "clean": true}));
                return;
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(tungstenite::Error::ConnectionClosed) => return tell(json!({"ev": "close", "code": 1005, "reason": "", "clean": true})),
            Err(_) => return fail(),
        }
    }
}
