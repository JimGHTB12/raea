//! An HTTP(S) request: `send` (one hop, whatever its status) and `fetch`, the page's net.fetch (sdk/ui's platform.ts):
//!
//!   {url, method, headers: [[k, v]], body: base64 | null, redirect: "follow" | "error" | "manual"}
//!   -> {status, statusText, headers: [[k, v]], body: base64, url, redirected}; a network failure is an Err

#[cfg(not(target_arch = "wasm32"))]
use std::io::Read;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use base64::Engine as _;
#[cfg(not(target_arch = "wasm32"))]
use base64::engine::general_purpose::STANDARD as B64;
#[cfg(not(target_arch = "wasm32"))]
use serde_json::json;
use serde_json::Value;

#[cfg(not(target_arch = "wasm32"))]
/// One request, sent: any status is an answer; Err only when none came (the connection, TLS, a timeout).
pub fn send(req: ureq::Request, body: Option<&[u8]>) -> Result<ureq::Response, ureq::Transport> {
    let r = match body {
        Some(b) => req.send_bytes(b),
        None => req.call(),
    };
    match r {
        Ok(r) | Err(ureq::Error::Status(_, r)) => Ok(r),
        Err(ureq::Error::Transport(t)) => Err(t),
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// An answer's headers, names lowercase, a repeated header once a value.
pub fn headers(resp: &ureq::Response) -> Vec<(String, String)> {
    resp.headers_names().iter().flat_map(|n| resp.all(n).into_iter().map(|v| (n.to_ascii_lowercase(), v.to_string()))).collect()
}

#[cfg(target_arch = "wasm32")]
/// A browser build has no HTTP of its own: the page fetches.
pub fn fetch(args: &Value) -> Result<Value, String> {
    Err(format!("{}: no HTTP in a browser build (the page fetches it)", args["url"].as_str().unwrap_or("?")))
}

#[cfg(not(target_arch = "wasm32"))]
/// The page's net.fetch, made on the calling thread.
pub fn fetch(args: &Value) -> Result<Value, String> {
    let url = args["url"].as_str().ok_or("no url")?;
    raea_io_home::offline(&format!("fetch {url}"))?;
    let method = args["method"].as_str().unwrap_or("GET");
    let follow = args["redirect"].as_str().unwrap_or("follow") == "follow";
    let agent = ureq::AgentBuilder::new().redirects(if follow { 20 } else { 0 }).timeout(Duration::from_secs(300)).build();
    let mut req = agent.request(method, url);
    for h in args["headers"].as_array().into_iter().flatten() {
        if let (Some(k), Some(v)) = (h[0].as_str(), h[1].as_str()) {
            req = req.set(k, v);
        }
    }
    let body = args["body"].as_str().map(|b| B64.decode(b).unwrap_or_default());
    let resp = send(req, body.as_deref()).map_err(|e| e.to_string())?;
    if !follow && (300..400).contains(&resp.status()) && args["redirect"] == json!("error") {
        return Err("redirect".into());
    }
    let headers: Vec<Value> = headers(&resp).into_iter().map(|(k, v)| json!([k, v])).collect();
    let (status, text, final_url) = (resp.status(), resp.status_text().to_string(), resp.get_url().to_string());
    let mut bytes = vec![];
    resp.into_reader().take(1 << 30).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    Ok(json!({"status": status, "statusText": text, "headers": headers, "body": B64.encode(&bytes), "redirected": final_url != url, "url": final_url}))
}
