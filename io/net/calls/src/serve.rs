//! T2b's recorded set of calls (docs/targets.feature), for each target's test: a local HTTP server and WebSocket echo,
//! the script that makes the calls (recorded/net.js: a fetch, storage set and get, a socket's open, send and close)
//! and the answers both targets must give (recorded/net.json). A test runs `script` in its host, reads what it
//! recorded (logged as `recorded JSON` and emitted as `recorded`) and checks it with `check`.

use std::io::{Read, Write};
use std::net::TcpListener;

use serde_json::Value;

/// The script, its servers' URLs in it.
const SCRIPT: &str = include_str!("../recorded/net.js");
/// What it records, with the servers' URLs as HTTP and WS.
pub const RECORDED: &str = include_str!("../recorded/net.json");

/// The servers: an HTTP one answering every request "hello", and a WebSocket one echoing each message (and a close).
pub struct Servers {
    pub http: String,
    pub ws: String,
}

pub fn serve() -> Servers {
    let h = TcpListener::bind("127.0.0.1:0").unwrap();
    let w = TcpListener::bind("127.0.0.1:0").unwrap();
    let s = Servers { http: format!("http://{}", h.local_addr().unwrap()), ws: format!("ws://{}", w.local_addr().unwrap()) };
    std::thread::spawn(move || {
        for mut c in h.incoming().flatten() {
            let mut req = Vec::new();
            let mut b = [0u8; 1024];
            while !req.windows(4).any(|w| w == b"\r\n\r\n") {
                match c.read(&mut b) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => req.extend_from_slice(&b[..n]),
                }
            }
            let _ = c.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello");
        }
    });
    std::thread::spawn(move || {
        for c in w.incoming().flatten() {
            std::thread::spawn(move || {
                let Ok(mut ws) = tungstenite::accept(c) else { return };
                while let Ok(m) = ws.read() {
                    if m.is_text() || m.is_binary() {
                        let _ = ws.send(m);
                    }
                }
            });
        }
    });
    s
}

impl Servers {
    /// The script, pointed at these servers.
    pub fn script(&self) -> String {
        SCRIPT.replace("%HTTP%", &self.http).replace("%WS%", &self.ws)
    }

    /// What a host recorded (the JSON the script logged), against RECORDED: Ok, or how they differ.
    pub fn check(&self, recorded: &str) -> Result<(), String> {
        let got: Value = serde_json::from_str(&recorded.replace(&self.http, "HTTP").replace(&self.ws, "WS")).map_err(|e| format!("recorded: {e}"))?;
        let want: Value = serde_json::from_str(RECORDED).unwrap();
        let (g, w) = (got.as_array().cloned().unwrap_or_default(), want.as_array().cloned().unwrap_or_default());
        for (i, (g, w)) in g.iter().zip(&w).enumerate() {
            if g != w {
                return Err(format!("step {}: got {g}, recorded {w}", i + 1));
            }
        }
        if g.len() != w.len() {
            return Err(format!("{} steps, recorded {}: {}", g.len(), w.len(), got));
        }
        Ok(())
    }
}
