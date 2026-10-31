//! Bakes for compositions (sdk/raea/src/physics.tsx): rigid bodies baked with Rapier, parts of a glTF cut out as
//! bodies, and goo (tools/goo: MLS-MPM on Metal) baked to a mesh sequence. Every bake is a pure function of what it's
//! asked (the request, and the files it names by content), kept in the composition's asset store under that key, so a
//! render, a `render --at` and a play read the same bytes, and nothing is simulated at a frame. Each call takes the
//! composition's folder (`dir`); raea-runtime-react registers them as native calls:
//!
//!   native: physics.part  {src, nodes, material?}            -> {file, centre, lo, hi, tris}
//!   native: physics.bake  {fps, frames, gravity?, substeps?, bodies: [...]}  -> {file, bodies: {name: {p, q}}}
//!   native: physics.goo   {spec}                              -> {file, stats}
//!   raea physics bake REQUEST.json [-o OUT.json]       (the same bake from the command line)
//!
//! Rapier (Apache-2.0) because it's a pure Rust library raea calls in process: no Bevy app, no Python. It steps the
//! same inputs to the same bits on every run on one machine, and the same world type can step a live game frame by
//! frame (physics.tsx, "Games"). It's built without `enhanced-determinism` since Bevy 0.20 (runtime/react/Cargo.toml says
//! why), so a bake isn't promised the same bits across platforms, and bakes made before 0.20 differ from new ones
//! (docs/bevy-upgrade.md, "Keeping the next bump small").

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

mod glb;
mod goo;
mod sim;
pub use glb::part;
pub use goo::goo;
pub use sim::simulate;

type R<T> = Result<T, String>;

fn hex_sha(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}

/// A file's identity for a cache key: its path, size and modification time (a bake names big models; hashing 86 MB
/// on every call would cost more than the bake).
fn file_id(p: &Path) -> R<Value> {
    let m = std::fs::metadata(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let t = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_nanos() as u64).unwrap_or(0);
    Ok(json!([p.display().to_string(), m.len(), t]))
}

/// Where a bake is: the composition's asset store, its media srcs as files, and goo's setup script (tools/goo).
pub struct Place<'a> {
    pub store: PathBuf,
    pub media: &'a dyn Fn(&str) -> PathBuf,
    pub goo_setup: PathBuf,
}

static MADE_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static RUNNING_SINCE: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

/// Seconds spent making things on first use so far (a bake, a solve), with the one running now.
pub fn making_secs() -> f64 {
    let done = MADE_NS.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1e9;
    done + RUNNING_SINCE.lock().ok().and_then(|r| r.map(|t| t.elapsed().as_secs_f64())).unwrap_or(0.0)
}

/// `f` timed as made on first use (a bake into the asset store: goo's SPH, a solve), so settle doesn't count it.
pub fn making<T>(f: impl FnOnce() -> T) -> T {
    let t0 = std::time::Instant::now();
    if let Ok(mut r) = RUNNING_SINCE.lock() {
        *r = Some(t0);
    }
    let out = f();
    if let Ok(mut r) = RUNNING_SINCE.lock() {
        *r = None;
    }
    MADE_NS.fetch_add(t0.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
    out
}

/// `physics.bake`: the simulation for a request, made once (keyed by the request and its files) and kept.
pub fn bake(at: &Place, req: &Value) -> R<Value> {
    let mut files = Vec::new();
    for b in req.get("bodies").and_then(Value::as_array).into_iter().flatten() {
        if let Some(s) = b.get("src").and_then(Value::as_str) {
            files.push(file_id(&(at.media)(s))?);
        }
    }
    let key = hex_sha(json!(["physics", 1, req, files]).to_string().as_bytes());
    let store = &at.store;
    let out = store.join(format!("{key}.physics.json"));
    if let Ok(b) = std::fs::read(&out) {
        if let Ok(mut v) = serde_json::from_slice::<Value>(&b) {
            v["file"] = json!(out.display().to_string());
            return Ok(v);
        }
    }
    let t0 = std::time::Instant::now();
    let mut v = simulate(at, req)?;
    v["seconds"] = json!(t0.elapsed().as_secs_f64());
    std::fs::create_dir_all(store).map_err(|e| format!("{}: {e}", store.display()))?;
    std::fs::write(&out, v.to_string()).map_err(|e| format!("{}: {e}", out.display()))?;
    eprintln!("raea: physics baked: {} bodies, {} frames in {:.2} s -> {}", req["bodies"].as_array().map(|a| a.len()).unwrap_or(0), req["frames"], t0.elapsed().as_secs_f64(), out.display());
    v["file"] = json!(out.display().to_string());
    Ok(v)
}

