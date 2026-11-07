//! The page host's Bevy Remote Protocol, as a client: one JSON-RPC call over HTTP, its result or its error. A +watch
//! method answers as an event stream; its first event is the answer, and the connection is dropped after it (bevy_remote
//! then stops watching). The same client as sdk/ui/parity's bevy.rs.

use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

use serde_json::{Value, json};

pub fn free_port() -> Result<u16, String> {
    std::net::TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map(|a| a.port()).map_err(|e| e.to_string())
}

pub fn call(port: u16, method: &str, params: Value, limit: Duration) -> Result<Value, String> {
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).to_string();
    let mut sock = std::net::TcpStream::connect(("127.0.0.1", port)).map_err(|e| format!("brp on {port}: {e}"))?;
    sock.set_read_timeout(Some(limit)).ok();
    write!(sock, "POST / HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).map_err(|e| e.to_string())?;
    let mut r = BufReader::new(sock);
    let (mut stream, mut chunked, mut len) = (false, false, None::<usize>);
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).map_err(|e| format!("{method}: {e}"))? == 0 {
            return Err(format!("{method}: the page host closed the connection"));
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
        all.lines().filter(|l| l.trim_start().starts_with('{')).collect::<Vec<_>>().join("")
    };
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{method}: {e}: {}", &text[..text.len().min(200)]))?;
    match (v.get("result"), v.get("error")) {
        (_, Some(e)) => Err(format!("{method}: {}", e["message"].as_str().map(str::to_string).unwrap_or_else(|| e.to_string()))),
        (Some(r), _) => Ok(r.clone()),
        _ => Err(format!("{method}: {text}")),
    }
}
