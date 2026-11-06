//! Modal: GPU code of your own, two ways. A ComfyUI server deployed on Modal (tools/comfy/modal.py: `modal deploy`
//! prints its URL) is a ComfyUI server like any other: RAEA_COMFY_URL points at it and comfy steps run there. And any
//! web endpoint you deploy that speaks raea's step contract is a backend by `modal:<url>` (or `modal:<name>`, with
//! RAEA_MODAL_<NAME> holding the URL): it's POSTed {op, inputs} (media inline as data URIs) and answers
//! {files: [{url} | {data, ext}], cents?}. Modal's proxy auth, when the endpoint asks for it, is MODAL_KEY and
//! MODAL_SECRET (sent as Modal-Key and Modal-Secret).

use std::path::{Path, PathBuf};

use serde_json::{Value as J, json};

use crate::http::{Body, call};
use crate::setting;

/// The endpoint a step names: a URL, or a name whose URL is RAEA_MODAL_<NAME>.
pub fn endpoint(name: &str) -> Result<String, String> {
    if name.starts_with("https://") {
        return Ok(name.to_string());
    }
    let var = format!("RAEA_MODAL_{}", name.to_ascii_uppercase().replace(['-', '.'], "_"));
    setting(&var).ok_or(format!("modal:{name} has no URL: deploy it and put {var}=https://… in ~/.raea/env"))
}

/// One step on a Modal endpoint: its files written beside `stem`, and what it says it cost.
pub fn run(name: &str, op: &str, inputs: &J, stem: &Path) -> Result<(Vec<PathBuf>, u64), String> {
    let url = endpoint(name)?;
    let mut h = Vec::new();
    if let (Some(k), Some(s)) = (setting("MODAL_KEY"), setting("MODAL_SECRET")) {
        h.push(format!("Modal-Key: {k}"));
        h.push(format!("Modal-Secret: {s}"));
    }
    // a cold container may take minutes to start: the request waits for it
    let r = call("POST", &url, &h, Body::Json(&json!({"op": op, "inputs": inputs})), None, 1800)?;
    let v = r.json().map_err(|e| format!("modal:{name} answered {}: {e}", r.code))?;
    if !r.ok() {
        return Err(format!("modal:{name} answered {}: {}", r.code, v["error"].as_str().map(str::to_string).unwrap_or(v.to_string())));
    }
    let mut files = Vec::new();
    for (k, f) in v["files"].as_array().into_iter().flatten().enumerate() {
        let s = if k == 0 { stem.to_path_buf() } else { PathBuf::from(format!("{}-{k}", stem.display())) };
        if let Some(u) = f["url"].as_str() {
            files.push(crate::made::download(u, &s)?);
        } else if let Some(d) = f["data"].as_str() {
            let ext = f["ext"].as_str().unwrap_or("bin");
            files.push(crate::openrouter::save(&format!("data:application/octet-stream;base64,{d}"), &s).and_then(|p| {
                let to = p.with_extension(ext);
                std::fs::rename(&p, &to).map(|_| to).map_err(|e| e.to_string())
            })?);
        }
    }
    if files.is_empty() {
        return Err(format!("modal:{name} answered with no files: {}", v.to_string().chars().take(300).collect::<String>()));
    }
    Ok((files, v["cents"].as_u64().unwrap_or(0)))
}
