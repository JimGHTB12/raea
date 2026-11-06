//! made.app (www.made.app/api): one key for 118 image, video and audio models (fal's, ElevenLabs', Kling's, Veo's and
//! made's own), priced at the provider's cost: every job and quote says `markupPercent: 0`. A request is quoted first
//! (POST /v1/cost, free), sent (POST /v1/generations, 202 with a job, debited on acceptance) and polled until it
//! succeeds; a failed or canceled job is refunded. Reference media go up through /v1/media (reserve, PUT, confirm) or
//! are an https URL or an earlier `job_…` id, which made reads as that job's output.
//!
//! The key is MADE_API_KEY (the environment, then ~/.raea/env).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{Value as J, json};

use crate::http::{Body, call, key_header, redact};
use crate::setting;

pub const BASE: &str = "https://www.made.app/api";

/// Fields every generation takes; a model's catalog row adds its own (`allowedParams`, `inputMap`).
const COMMON: [&str; 13] = ["model", "prompt", "aspect_ratio", "duration", "resolution", "audio", "medias", "count", "request_id", "max_cost_cents", "quality", "image_size", "characters"];

pub struct Made {
    headers: Vec<String>,
    base: String,
}

impl Made {
    pub fn from_settings() -> Result<Made, String> {
        let key = key_header("MADE_API_KEY", "Bearer", "made.app needs a key: put MADE_API_KEY=made_sk_… in ~/.raea/env (www.made.app/account/api-keys)")?;
        Ok(Made { headers: vec![key], base: setting("MADE_API").unwrap_or_else(|| BASE.into()) })
    }

    fn get(&self, path: &str) -> Result<J, String> {
        let r = call("GET", &format!("{}{path}", self.base), &self.headers, Body::None, None, 60)?;
        answer(path, &r)
    }

    fn post(&self, path: &str, body: &J) -> Result<J, String> {
        let r = call("POST", &format!("{}{path}", self.base), &self.headers, Body::Json(body), None, 120)?;
        answer(path, &r)
    }

    /// The catalog, each model with its prices, roles and parameters; kept a day in ~/.cache/raea-gen/made-models.json.
    pub fn models(&self) -> Result<Vec<J>, String> {
        let cache = cache_dir().join("made-models.json");
        let fresh = std::fs::metadata(&cache).and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|age| age < Duration::from_secs(24 * 3600));
        if fresh {
            if let Some(m) = std::fs::read(&cache).ok().and_then(|b| serde_json::from_slice::<J>(&b).ok()) {
                return Ok(m["models"].as_array().cloned().unwrap_or_default());
            }
        }
        let v = self.get("/v1/models")?;
        let _ = std::fs::create_dir_all(cache_dir());
        let _ = std::fs::write(&cache, v.to_string());
        Ok(v["models"].as_array().cloned().unwrap_or_default())
    }

    pub fn model(&self, id: &str) -> Result<J, String> {
        self.models()?.into_iter().find(|m| m["id"] == id).ok_or_else(|| format!("made.app has no model `{id}` (raea gen models lists them)"))
    }

    /// What a request would cost, in cents, without running it: (what made charges, what the provider charges).
    /// made rounds a job up to a whole credit (10¢), so the two differ on small jobs (direct.rs).
    pub fn cost(&self, body: &J) -> Result<(u64, u64), String> {
        let v = self.post("/v1/cost", body)?;
        let c = v["cost"]["customerCostCents"].as_u64().ok_or_else(|| format!("/v1/cost: no customerCostCents in {v}"))?;
        Ok((c, v["cost"]["providerCostCents"].as_u64().unwrap_or(c)))
    }

    pub fn balance_cents(&self) -> Result<u64, String> {
        let v = self.get("/v1/balance")?;
        Ok((v["credits"].as_f64().unwrap_or(0.0) * 10.0).round() as u64)
    }

    /// A job, created: 202 and the job (its id, status, costCents).
    pub fn create(&self, body: &J) -> Result<J, String> {
        self.post("/v1/generations", body)
    }

    pub fn job(&self, id: &str) -> Result<J, String> {
        self.get(&format!("/v1/generations/{id}"))
    }

    pub fn cancel(&self, id: &str) -> Result<J, String> {
        self.post(&format!("/v1/generations/{id}/cancel"), &json!({}))
    }

    /// The job once it's done: succeeded, or an error saying how it ended. A job whose status and progress stay the same
    /// for `stall` is given up on (and canceled, which refunds it).
    pub fn wait(&self, id: &str, stall: Duration) -> Result<J, String> {
        let (mut seen, mut since, mut nap) = (String::new(), Instant::now(), 1u64);
        loop {
            let j = self.job(id)?;
            match j["status"].as_str().unwrap_or("") {
                "succeeded" => return Ok(j),
                s @ ("failed" | "canceled" | "cancelled") => return Err(format!("made.app job {id} {s}: {}", j["error"].as_str().unwrap_or("no reason given"))),
                _ => {}
            }
            let now = format!("{}:{}", j["status"], j["progress"]);
            if now != seen {
                (seen, since) = (now, Instant::now());
            } else if since.elapsed() > stall {
                let _ = self.cancel(id);
                return Err(format!("made.app job {id} sat at {seen} for {}s, canceled (refunded)", stall.as_secs()));
            }
            std::thread::sleep(Duration::from_secs(nap));
            nap = (nap + 1).min(8);
        }
    }

    /// A local file as made media (`media_…`): reserved, PUT to the signed URL with the headers it names, confirmed.
    pub fn upload(&self, file: &Path) -> Result<String, String> {
        let mime = mime_of(file);
        let v = self.post("/v1/media", &json!({"content_type": mime}))?;
        let id = v["media"]["id"].as_str().ok_or_else(|| format!("/v1/media: no id in {v}"))?.to_string();
        let url = v["upload"]["url"].as_str().ok_or("/v1/media: no upload url")?;
        let headers: Vec<String> = v["upload"]["headers"].as_object().map(|h| h.iter().map(|(k, v)| format!("{k}: {}", v.as_str().unwrap_or(""))).collect()).unwrap_or_default();
        let r = call("PUT", url, &headers, Body::File(file), None, 600)?;
        if !r.ok() {
            return Err(format!("upload of {} answered {}: {}", file.display(), r.code, r.text(200)));
        }
        self.post(&format!("/v1/media/{id}/confirm"), &json!({}))?;
        Ok(id)
    }
}

fn answer(path: &str, r: &crate::http::Reply) -> Result<J, String> {
    let v = r.json().map_err(|e| format!("made.app {path} answered {}: {e}", r.code))?;
    if !r.ok() {
        let why = match r.code {
            401 => "the key was refused (MADE_API_KEY)",
            402 => "not enough credits (www.made.app/pricing)",
            429 => "too many refunded failures in a row; wait before sending more",
            _ => "",
        };
        return Err(format!("made.app {path} answered {}{}: {}", r.code, if why.is_empty() { String::new() } else { format!(", {why}") }, v["error"].as_str().map(str::to_string).unwrap_or(v.to_string())));
    }
    Ok(v)
}

/// A request body with only the fields this model takes: the common ones, its `allowedParams` and its `inputMap`'s.
/// made refuses an undeclared field rather than ignoring it, so a step's extra settings are dropped here, by name.
pub fn fit(model: &J, body: J) -> (J, Vec<String>) {
    let allowed: Vec<&str> = model["allowedParams"].as_array().map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    let mapped: Vec<&str> = model["inputMap"].as_object().map(|m| m.keys().map(String::as_str).collect()).unwrap_or_default();
    let mut out = serde_json::Map::new();
    let mut dropped = Vec::new();
    for (k, v) in body.as_object().cloned().unwrap_or_default() {
        if COMMON.contains(&k.as_str()) || allowed.contains(&k.as_str()) || mapped.contains(&k.as_str()) {
            out.insert(k, v);
        } else {
            dropped.push(k);
        }
    }
    (J::Object(out), dropped)
}

/// A finished job's outputs: each asset's URL and type (image, video, audio).
pub fn outputs(job: &J) -> Vec<(String, String)> {
    job["result"]["assets"].as_array().map(|a| a.iter().filter_map(|x| Some((x["url"].as_str()?.to_string(), x["type"].as_str().unwrap_or("").to_string()))).collect()).unwrap_or_default()
}

/// A signed asset URL downloaded to `out`; the file's extension from the URL's path.
pub fn download(url: &str, out_stem: &Path) -> Result<PathBuf, String> {
    let ext = Path::new(redact(url)).extension().and_then(|e| e.to_str()).unwrap_or("bin").to_ascii_lowercase();
    let out = out_stem.with_extension(&ext);
    let r = call("GET", url, &[], Body::None, Some(&out), 900)?;
    if !r.ok() {
        let _ = std::fs::remove_file(&out);
        return Err(format!("download of {} answered {}", redact(url), r.code));
    }
    Ok(out)
}

pub fn mime_of(file: &Path) -> &'static str {
    match file.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("mp4" | "m4v") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",
        Some("wav") => "audio/wav",
        Some("mp3") => "audio/mpeg",
        Some("m4a") => "audio/mp4",
        _ => "application/octet-stream",
    }
}

/// A file of raea gen's cache (cache_dir) by its name.
pub fn cached(name: &str) -> PathBuf {
    cache_dir().join(name)
}

/// Where raea gen keeps fetched catalogs and generated files: $RAEA_GEN_CACHE, else ~/.cache/raea-gen.
pub fn cache_dir() -> PathBuf {
    raea_io_home::env_path("RAEA_GEN_CACHE", ".cache/raea-gen")
}
