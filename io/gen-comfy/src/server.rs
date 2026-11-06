//! A ComfyUI server as a place steps run: this machine's (`raea gen comfy setup` installs one) or one hosted
//! anywhere that speaks ComfyUI's own HTTP API (a Modal app, a rented GPU). RAEA_COMFY_URL says where (default
//! http://127.0.0.1:8188); RAEA_COMFY_TOKEN, when set, goes as a bearer token (a hosted server behind auth).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{Value as J, json};

use raea_io_gen_core::http::{Body, call};
use raea_io_gen_core::setting;

pub struct Comfy {
    pub url: String,
    token: Option<String>,
}

impl Comfy {
    pub fn from_settings() -> Comfy {
        Comfy { url: setting("RAEA_COMFY_URL").unwrap_or_else(|| "http://127.0.0.1:8188".into()).trim_end_matches('/').to_string(), token: setting("RAEA_COMFY_TOKEN") }
    }

    fn headers(&self) -> Vec<String> {
        self.token.iter().map(|t| format!("Authorization: Bearer {t}")).collect()
    }

    fn get(&self, path: &str, secs: u32) -> Result<J, String> {
        let r = call("GET", &format!("{}{path}", self.url), &self.headers(), Body::None, None, secs)?;
        if !r.ok() {
            return Err(format!("ComfyUI {path} answered {}: {}", r.code, r.text(300)));
        }
        r.json()
    }

    /// Whether a server answers at all.
    pub fn up(&self) -> bool {
        self.get("/system_stats", 5).is_ok()
    }

    /// Whether it answers, giving a hosted one (a Modal app scaled to zero) up to `limit` to start; this machine's
    /// is asked once.
    pub fn wake(&self, limit: Duration) -> bool {
        if self.url.contains("127.0.0.1") || self.url.contains("localhost") {
            return self.up();
        }
        let t = Instant::now();
        while t.elapsed() < limit {
            if self.get("/system_stats", 60).is_ok() {
                return true;
            }
            std::thread::sleep(Duration::from_secs(3));
        }
        false
    }

    /// The server's node schema, kept for the importer (schema::CACHE_NAME, in made::cached).
    pub fn save_schema(&self) -> Result<usize, String> {
        let v = self.get("/object_info", 120)?;
        let n = v.as_object().map(|o| o.len()).unwrap_or(0);
        let f = raea_io_gen_core::made::cached(crate::schema::CACHE_NAME);
        let _ = std::fs::create_dir_all(f.parent().unwrap_or(Path::new(".")));
        std::fs::write(&f, v.to_string()).map_err(|e| format!("{}: {e}", f.display()))?;
        Ok(n)
    }

    /// A file into the server's input folder; the name a Load node uses for it.
    pub fn upload(&self, file: &Path) -> Result<String, String> {
        let fields = vec![("image".to_string(), format!("@{}", file.display())), ("overwrite".to_string(), "true".to_string())];
        let r = call("POST", &format!("{}/upload/image", self.url), &self.headers(), Body::Form(&fields), None, 600)?;
        if !r.ok() {
            return Err(format!("ComfyUI upload of {} answered {}: {}", file.display(), r.code, r.text(300)));
        }
        let v = r.json()?;
        let name = v["name"].as_str().ok_or("ComfyUI upload: no name")?;
        Ok(match v["subfolder"].as_str().filter(|s| !s.is_empty()) { Some(sub) => format!("{sub}/{name}"), None => name.to_string() })
    }

    /// An API-format graph queued; its prompt id. A graph the server refuses says which nodes and why.
    pub fn queue(&self, graph: &J) -> Result<String, String> {
        let body = json!({"prompt": graph, "client_id": "raea-gen"});
        let r = call("POST", &format!("{}/prompt", self.url), &self.headers(), Body::Json(&body), None, 120)?;
        let v = r.json().unwrap_or(J::Null);
        if !r.ok() {
            let mut why = v["error"]["message"].as_str().unwrap_or("refused").to_string();
            for (node, e) in v["node_errors"].as_object().into_iter().flatten() {
                let errs: Vec<String> = e["errors"].as_array().into_iter().flatten().map(|x| format!("{} ({})", x["message"].as_str().unwrap_or(""), x["details"].as_str().unwrap_or(""))).collect();
                why += &format!("; node {node} {}: {}", e["class_type"].as_str().unwrap_or(""), errs.join(", "));
            }
            return Err(format!("ComfyUI refused the graph: {why}"));
        }
        v["prompt_id"].as_str().map(str::to_string).ok_or_else(|| format!("ComfyUI /prompt: no prompt_id in {v}"))
    }

    /// The prompt's history once it ends; an error when it failed. Given up after `limit` with no result.
    pub fn wait(&self, id: &str, limit: Duration) -> Result<J, String> {
        let t = Instant::now();
        let mut nap = 1;
        loop {
            let h = self.get(&format!("/history/{id}"), 30)?;
            if let Some(entry) = h.get(id) {
                if entry["status"]["status_str"] == "error" {
                    let msg = entry["status"]["messages"].as_array().into_iter().flatten().filter(|m| m[0] == "execution_error").map(|m| format!("{} {}: {}", m[1]["node_type"].as_str().unwrap_or(""), m[1]["node_id"].as_str().unwrap_or(""), m[1]["exception_message"].as_str().unwrap_or("").trim())).collect::<Vec<_>>().join("; ");
                    return Err(format!("ComfyUI run failed: {msg}"));
                }
                if entry["status"]["completed"] == true || entry["outputs"].as_object().is_some_and(|o| !o.is_empty()) {
                    return Ok(entry.clone());
                }
            }
            if t.elapsed() > limit {
                return Err(format!("ComfyUI prompt {id} gave nothing back in {}s", limit.as_secs()));
            }
            std::thread::sleep(Duration::from_secs(nap));
            nap = (nap + 1).min(5);
        }
    }

    /// One node's files from a finished run, downloaded into `dir`.
    pub fn fetch(&self, entry: &J, node: &str, dir: &Path) -> Result<Vec<PathBuf>, String> {
        let out = &entry["outputs"][node];
        let mut files = Vec::new();
        for key in ["images", "gifs", "videos", "video", "audio", "3d"] {
            for f in out[key].as_array().into_iter().flatten() {
                let (name, sub, ty) = (f["filename"].as_str().unwrap_or(""), f["subfolder"].as_str().unwrap_or(""), f["type"].as_str().unwrap_or("output"));
                if name.is_empty() {
                    continue;
                }
                let dest = dir.join(name.rsplit('/').next().unwrap_or(name));
                let q = |s: &str| s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect::<String>();
                let url = format!("{}/view?filename={}&subfolder={}&type={}", self.url, q(name), q(sub), q(ty));
                let r = call("GET", &url, &self.headers(), Body::None, Some(&dest), 600)?;
                if !r.ok() {
                    return Err(format!("ComfyUI /view {name} answered {}", r.code));
                }
                files.push(dest);
            }
        }
        if files.is_empty() {
            return Err(format!("ComfyUI node {node} saved no files (outputs: {})", out));
        }
        Ok(files)
    }
}
