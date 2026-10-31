//! The only way the suite talks to a substrate: MCP tools/call over HTTP JSON-RPC at http://127.0.0.1:PORT/mcp, the
//! shape apps/legacy/shell/src/mcp.rs answers (a result's content[0].text carries the answer; isError marks a tool's error).
//! A plain HTTP/1.1 client on std's TcpStream, one connection per call.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::{json, Value};

pub struct Mcp {
    pub port: u16,
    id: u64,
}

impl Mcp {
    pub fn new(port: u16) -> Mcp {
        Mcp { port, id: 0 }
    }

    /// One JSON-RPC request; the `result` or the error's message.
    pub fn rpc(&mut self, method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
        self.id += 1;
        let body = json!({"jsonrpc": "2.0", "id": self.id, "method": method, "params": params}).to_string();
        let answer = post(self.port, &body, timeout)?;
        let v: Value = serde_json::from_str(&answer).map_err(|e| format!("{method}: not JSON ({e}): {}", clip(&answer, 200)))?;
        if let Some(e) = v.get("error") {
            return Err(format!("{method}: {}", e["message"].as_str().unwrap_or(&e.to_string())));
        }
        Ok(v["result"].clone())
    }

    /// The names of the tools the substrate offers.
    pub fn tools(&mut self) -> Result<Vec<String>, String> {
        let r = self.rpc("tools/list", json!({}), Duration::from_secs(5))?;
        Ok(r["tools"].as_array().map(|t| t.iter().filter_map(|t| t["name"].as_str().map(str::to_string)).collect()).unwrap_or_default())
    }

    /// tools/call NAME ARGS: the first text content, parsed as JSON when it is JSON (else as a string).
    pub fn call(&mut self, name: &str, args: Value) -> Result<Value, String> {
        let r = self.rpc("tools/call", json!({"name": name, "arguments": args}), Duration::from_secs(20))?;
        let text = r["content"].as_array().and_then(|c| c.iter().find_map(|c| c["text"].as_str())).unwrap_or("").to_string();
        if r["isError"].as_bool() == Some(true) {
            return Err(format!("{name} {}: {text}", clip(&args.to_string(), 160)));
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
    }

    /// MCP `input`: one event; the frame it landed before.
    pub fn input(&mut self, event: Value) -> Result<u64, String> {
        let r = self.call("input", event)?;
        frame_of(&r).ok_or_else(|| format!("input answered without a frame number: {}", clip(&r.to_string(), 200)))
    }

    /// MCP `ui`: the tree as presented in the latest frame.
    pub fn ui(&mut self) -> Result<Value, String> {
        let r = self.call("ui", json!({}))?;
        if frame_of(&r).is_none() || !r["nodes"].is_array() {
            return Err(format!("ui answered something that isn't a tree ({{frame, window, nodes}}): {}", clip(&r.to_string(), 200)));
        }
        Ok(r)
    }
}

/// A frame number from an answer: a bare number, or {frame: N}.
pub fn frame_of(v: &Value) -> Option<u64> {
    v.as_u64().or_else(|| v["frame"].as_u64()).or_else(|| v["frame"].as_f64().map(|f| f as u64)).or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
}

pub fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n { s.to_string() } else { format!("{}…", s.chars().take(n).collect::<String>()) }
}

fn post(port: u16, body: &str, timeout: Duration) -> Result<String, String> {
    let mut s = TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_secs(2)).map_err(|e| format!("connect 127.0.0.1:{port}: {e}"))?;
    s.set_read_timeout(Some(timeout)).ok();
    s.set_write_timeout(Some(timeout)).ok();
    s.set_nodelay(true).ok();
    let req = format!(
        "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nAccept: application/json, text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).map_err(|e| format!("write: {e}"))?;
    let mut r = BufReader::new(s);
    let mut status = String::new();
    r.read_line(&mut status).map_err(|e| format!("read: {e}"))?;
    let code: u16 = status.split_whitespace().nth(1).and_then(|c| c.parse().ok()).ok_or(format!("bad status line {status:?}"))?;
    let (mut len, mut chunked) = (None, false);
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).map_err(|e| format!("read: {e}"))? == 0 {
            break;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            match k.trim().to_ascii_lowercase().as_str() {
                "content-length" => len = v.trim().parse::<usize>().ok(),
                "transfer-encoding" => chunked = v.to_ascii_lowercase().contains("chunked"),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    if chunked {
        loop {
            let mut size = String::new();
            r.read_line(&mut size).map_err(|e| format!("read: {e}"))?;
            let n = usize::from_str_radix(size.trim().split(';').next().unwrap_or("0"), 16).map_err(|_| format!("bad chunk size {size:?}"))?;
            if n == 0 {
                break;
            }
            let mut buf = vec![0; n + 2];
            r.read_exact(&mut buf).map_err(|e| format!("read: {e}"))?;
            out.extend_from_slice(&buf[..n]);
        }
    } else if let Some(n) = len {
        out.resize(n, 0);
        r.read_exact(&mut out).map_err(|e| format!("read: {e}"))?;
    } else {
        r.read_to_end(&mut out).map_err(|e| format!("read: {e}"))?;
    }
    let text = String::from_utf8_lossy(&out).to_string();
    if !(200..300).contains(&code) {
        return Err(format!("HTTP {code}: {}", clip(&text, 200)));
    }
    // a server that answers as an event stream: the last data line
    if text.starts_with("event:") || text.starts_with("data:") {
        return text.lines().filter_map(|l| l.strip_prefix("data:")).last().map(|d| d.trim().to_string()).ok_or("an empty event stream".into());
    }
    Ok(text)
}
