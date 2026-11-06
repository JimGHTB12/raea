//! OpenRouter (openrouter.ai/api/v1): its image-output models (`openrouter:google/gemini-3.1-flash-image`,
//! `openrouter:black-forest-labs/flux-3-image`, …) for image.generate and image.edit, and any of its text models for
//! text.generate (the prompt-writing LLM nodes ComfyUI workflows use). One chat completion: the prompt and any images
//! as the founder's message, `modalities` asking for images back, which arrive as data URIs. The key is
//! OPENROUTER_API_KEY.

use std::path::{Path, PathBuf};

use serde_json::{Value as J, json};

use crate::http::{Body, call, key_header};

pub struct OpenRouter {
    headers: Vec<String>,
}

impl OpenRouter {
    pub fn from_settings() -> Result<OpenRouter, String> {
        let key = key_header("OPENROUTER_API_KEY", "Bearer", "OpenRouter needs OPENROUTER_API_KEY in ~/.raea/env (openrouter.ai/keys)")?;
        Ok(OpenRouter { headers: vec![key, "X-Title: raea".to_string()] })
    }

    fn post(&self, body: &J) -> Result<J, String> {
        let r = call("POST", "https://openrouter.ai/api/v1/chat/completions", &self.headers, Body::Json(body), None, 600)?;
        let v = r.json().map_err(|e| format!("OpenRouter: {e}"))?;
        if !r.ok() || v.get("error").is_some() {
            return Err(format!("OpenRouter answered {}: {}", r.code, v["error"]["message"].as_str().map(str::to_string).unwrap_or(v.to_string())));
        }
        Ok(v)
    }

    /// One completion: the text it wrote and the images it made (data URIs), with what OpenRouter billed in cents.
    pub fn complete(&self, model: &str, prompt: &str, system: Option<&str>, images: &[String], want_images: bool, extra: &serde_json::Map<String, J>) -> Result<(String, Vec<String>, f64), String> {
        let mut content = vec![json!({"type": "text", "text": prompt})];
        content.extend(images.iter().map(|u| json!({"type": "image_url", "image_url": {"url": u}})));
        let mut messages = Vec::new();
        if let Some(s) = system {
            messages.push(json!({"role": "system", "content": s}));
        }
        messages.push(json!({"role": "user", "content": content}));
        let mut body = json!({"model": model, "messages": messages, "usage": {"include": true}});
        if want_images {
            body["modalities"] = json!(["image", "text"]);
        }
        for (k, v) in extra {
            body[k.as_str()] = v.clone();
        }
        let v = self.post(&body)?;
        let msg = &v["choices"][0]["message"];
        let text = msg["content"].as_str().unwrap_or("").to_string();
        let imgs: Vec<String> = msg["images"].as_array().into_iter().flatten().filter_map(|i| i["image_url"]["url"].as_str().map(str::to_string)).collect();
        Ok((text, imgs, v["usage"]["cost"].as_f64().unwrap_or(0.0) * 100.0))
    }
}

/// A data URI (or an https URL) written to `stem` with the extension its type says.
pub fn save(uri: &str, stem: &Path) -> Result<PathBuf, String> {
    if uri.starts_with("http") {
        return crate::made::download(uri, stem);
    }
    let (head, data) = uri.strip_prefix("data:").and_then(|r| r.split_once(',')).ok_or("not a data URI")?;
    let ext = match head.split(';').next().unwrap_or("") { "image/jpeg" => "jpg", "image/webp" => "webp", "image/svg+xml" => "svg", _ => "png" };
    let out = stem.with_extension(ext);
    std::fs::write(&out, unbase64(data)?).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(out)
}

fn unbase64(s: &str) -> Result<Vec<u8>, String> {
    let val = |c: u8| -> Option<u32> {
        Some(match c { b'A'..=b'Z' => c - b'A', b'a'..=b'z' => c - b'a' + 26, b'0'..=b'9' => c - b'0' + 52, b'+' | b'-' => 62, b'/' | b'_' => 63, _ => return None } as u32)
    };
    let bytes: Vec<u32> = s.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=').map(|c| val(c).ok_or("bad base64")).collect::<Result<_, _>>()?;
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for ch in bytes.chunks(4) {
        let n = ch.iter().enumerate().fold(0u32, |n, (i, v)| n | v << (18 - 6 * i));
        out.push((n >> 16) as u8);
        if ch.len() > 2 { out.push((n >> 8) as u8); }
        if ch.len() > 3 { out.push(n as u8); }
    }
    Ok(out)
}
