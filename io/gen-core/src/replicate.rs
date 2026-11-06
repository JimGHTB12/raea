//! Replicate (api.replicate.com/v1): any model by `replicate:owner/name` (its latest version) or
//! `replicate:owner/name:version`. A prediction is created and its `urls.get` polled until it succeeds; its output
//! is a URL or a list of them. Media go as data URIs. ComfyUI on Replicate is the same call on
//! fofr/any-comfyui-workflow with the API-format graph as `workflow_json`. The key is REPLICATE_API_TOKEN;
//! RAEA_REPLICATE_API moves the base (a test's stand-in, a proxy).

use std::time::{Duration, Instant};

use serde_json::{Value as J, json};

use crate::http::{Body, call};
use crate::setting;

pub const COMFY: &str = "fofr/any-comfyui-workflow";

pub struct Replicate {
    token: String,
    base: String,
}

impl Replicate {
    pub fn from_settings() -> Result<Replicate, String> {
        let token = setting("REPLICATE_API_TOKEN").ok_or("Replicate needs REPLICATE_API_TOKEN in ~/.raea/env (replicate.com/account/api-tokens)")?;
        let base = setting("RAEA_REPLICATE_API").unwrap_or_else(|| "https://api.replicate.com/v1".into()).trim_end_matches('/').to_string();
        Ok(Replicate { token, base })
    }

    fn json(&self, method: &str, url: &str, body: Option<&J>) -> Result<J, String> {
        let h = vec![format!("Authorization: Bearer {}", self.token)];
        let r = call(method, url, &h, body.map(Body::Json).unwrap_or(Body::None), None, 120)?;
        let v = r.json().map_err(|e| format!("Replicate: {e}"))?;
        if !r.ok() {
            return Err(format!("Replicate {} answered {}: {}", crate::http::redact(url), r.code, v["detail"].as_str().map(str::to_string).unwrap_or(v.to_string())));
        }
        Ok(v)
    }

    /// A prediction run to the end: its output.
    pub fn run(&self, model: &str, input: &J, stall: Duration) -> Result<J, String> {
        let (url, body) = match model.split_once(':') {
            Some((_, version)) => (format!("{}/predictions", self.base), json!({"version": version, "input": input})),
            None => (format!("{}/models/{model}/predictions", self.base), json!({"input": input})),
        };
        let mut p = self.json("POST", &url, Some(&body))?;
        let get = p["urls"]["get"].as_str().ok_or_else(|| format!("Replicate: no urls.get in {p}"))?.to_string();
        let (mut seen, mut since, mut nap) = (String::new(), Instant::now(), 1);
        loop {
            match p["status"].as_str() {
                Some("succeeded") => return Ok(p["output"].clone()),
                Some(s @ ("failed" | "canceled")) => return Err(format!("Replicate {model} {s}: {}", p["error"].as_str().unwrap_or(""))),
                _ => {}
            }
            let now = format!("{}:{}", p["status"], p["logs"].as_str().map(|l| l.len()).unwrap_or(0));
            if now != seen {
                (seen, since) = (now, Instant::now());
            } else if since.elapsed() > stall {
                let _ = self.json("POST", &format!("{get}/cancel"), None);
                return Err(format!("Replicate {model} sat at {} for {}s, canceled", p["status"], stall.as_secs()));
            }
            std::thread::sleep(Duration::from_secs(nap));
            nap = (nap + 1).min(6);
            p = self.json("GET", &get, None)?;
        }
    }
}

/// A prediction's output as file URLs: a string, a list, or an object of them.
pub fn urls(out: &J) -> Vec<String> {
    match out {
        J::String(s) if s.starts_with("http") || s.starts_with("data:") => vec![s.clone()],
        J::Array(a) => a.iter().flat_map(urls).collect(),
        J::Object(o) => o.values().flat_map(urls).collect(),
        _ => vec![],
    }
}
