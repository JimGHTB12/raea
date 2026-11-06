//! Tripo 3D through Tripo's own CLI (`npm install -g tripo-cli`, the `tripo` binary), on the founder's own key: a step
//! whose model is `tripo:<model>` (`tripo:` alone is Tripo's default, P2; `tripo:H3.1` the denser, cheaper H series).
//! The CLI does the upload, the polling and the download, and prints one JSON line (`model_file`, `preview`,
//! `credits_consumed`), so raea reads that and nothing else.
//!
//! The key: TRIPO_API_KEY (the environment, then ~/.raea/env), else whatever `tripo login` saved in ~/.tripo. Each user
//! brings their own; raea never ships one.
//!
//! **Credits live in the API wallet, not the app's.** A key draws on https://developers.tripo3d.ai/en/billing (the
//! API platform). Credits bought on the main tripo3d.ai dashboard (the Studio web app) are a separate wallet the API
//! never sees: on 2026-11-05 a freshly topped-up Studio account still answered `balance 0` and exit 4 until credits
//! were added on the developers billing page. Every insufficient-credits error below says so.
//!
//! Price: Tripo bills credits; one credit is 1¢ (RAEA_TRIPO_CENTS_PER_CREDIT overrides), the same as fal's resale of
//! the same models (`fal:tripo3d/...`, priced in credits at $0.01). A plan quotes with `tripo make --dry-run`, which
//! sends nothing.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Map, Value as J};

use crate::setting;

/// Where a key's credits come from (not the tripo3d.ai dashboard's wallet).
pub const BILLING: &str = "https://developers.tripo3d.ai/en/billing";

/// Why a Tripo step failed, so routing can tell "no credits" (nothing was submitted) from everything else.
#[derive(Debug)]
pub enum Fail {
    /// exit 4: the key's API wallet is empty; nothing was submitted or charged
    Credits(String),
    /// exit 3: no key, a bad key, or a key for the other region
    Auth(String),
    Other(String),
}

impl std::fmt::Display for Fail {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Fail::Credits(m) | Fail::Auth(m) | Fail::Other(m) => f.write_str(m),
        }
    }
}

/// The CLI's binary: RAEA_TRIPO_BIN, else `tripo` on PATH.
pub fn bin() -> String {
    setting("RAEA_TRIPO_BIN").unwrap_or_else(|| "tripo".into())
}

pub fn cents_per_credit() -> f64 {
    setting("RAEA_TRIPO_CENTS_PER_CREDIT").and_then(|c| c.parse().ok()).unwrap_or(1.0)
}

/// Whether this host can run Tripo itself: the CLI answers, and a key is set or `tripo login` saved one.
pub fn available() -> bool {
    let has_key = setting("TRIPO_API_KEY").is_some() || home_dir().join(".tripo/config.json").exists();
    has_key && Command::new(bin()).arg("--version").output().is_ok_and(|o| o.status.success())
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn command() -> Command {
    let mut c = Command::new(bin());
    // the CLI asks its host to name itself on every call (its agent docs); a user's own value wins
    if std::env::var_os("TRIPO_CLIENT_APP").is_none() {
        c.env("TRIPO_CLIENT_APP", "raea");
    }
    if let Some(k) = setting("TRIPO_API_KEY") {
        c.env("TRIPO_API_KEY", k);
    }
    if let Some(r) = setting("TRIPO_REGION") {
        c.env("TRIPO_REGION", r);
    }
    c
}

/// `tripo make`'s arguments for a step: its inputs (a prompt, or 1 image, or 2-4 views as `image` + `refs`, as local
/// paths or URLs), the model, seed, and every x_<name> as `-p name=value` (Tripo's own parameter names:
/// x_texture_quality=detailed, x_face_limit=8000, x_pbr=true). `texture: false` asks for bare geometry.
pub fn args(model: &str, inputs: &Map<String, J>, media: &[String], out: &Path, dry: bool) -> Vec<String> {
    let mut a: Vec<String> = vec!["make".into()];
    if media.is_empty() {
        a.push(inputs.get("prompt").and_then(|p| p.as_str()).unwrap_or("").to_string());
    } else {
        a.extend(media.iter().take(4).cloned());
    }
    if !model.is_empty() {
        a.extend(["--model".into(), model.to_string()]);
    }
    if let Some(s) = inputs.get("seed").and_then(|s| s.as_u64()) {
        a.extend(["--seed".into(), s.to_string()]);
    }
    if inputs.get("texture") == Some(&J::Bool(false)) {
        a.extend(["-p".into(), "texture=false".into()]);
    }
    for (k, v) in inputs {
        if let Some(name) = k.strip_prefix("x_") {
            let v = match v { J::String(s) => s.clone(), other => other.to_string() };
            a.extend(["-p".into(), format!("{name}={v}")]);
        }
    }
    a.extend(["--json".into(), "--yes".into(), "--no-open".into(), "--quiet".into()]);
    if dry {
        a.push("--dry-run".into());
    } else {
        a.extend(["-o".into(), out.display().to_string()]);
    }
    a
}

fn last_json(stdout: &[u8]) -> Option<J> {
    String::from_utf8_lossy(stdout).lines().rev().find_map(|l| serde_json::from_str::<J>(l.trim()).ok())
}

fn fail(code: Option<i32>, stdout: &[u8], stderr: &[u8]) -> Fail {
    let said = last_json(stdout).map(|v| v["error"]["message"].as_str().or(v["message"].as_str()).or(v["error"].as_str()).map(str::to_string).unwrap_or(v.to_string()))
        .unwrap_or_else(|| String::from_utf8_lossy(stderr).lines().filter(|l| !l.trim().is_empty()).take(3).collect::<Vec<_>>().join(" | "));
    match code {
        Some(4) => Fail::Credits(format!("tripo: insufficient credits ({said}). A key's credits are the API wallet's: add them at {BILLING} (not the tripo3d.ai dashboard, a separate wallet), or `tripo topup`")),
        Some(3) => Fail::Auth(format!("tripo: no usable key ({said}). Set TRIPO_API_KEY=tsk_… in ~/.raea/env (from {}), or run `tripo login`; `tripo doctor` explains a region mismatch", BILLING.replace("/billing", "/api-keys"))),
        Some(5) => Fail::Other(format!("tripo: refused by content policy ({said})")),
        Some(6) => Fail::Other(format!("tripo: the task failed, credits refunded ({said})")),
        Some(c) => Fail::Other(format!("tripo make exited {c}: {said}")),
        None => Fail::Other(format!("tripo make was killed: {said}")),
    }
}

/// The credits a step would cost, from `tripo make --dry-run` (no upload, no submit, nothing billed). None when Tripo
/// publishes no estimate (P1, legacy models, chained steps): never read as zero.
pub fn quote(model: &str, inputs: &Map<String, J>, media: &[String]) -> Result<Option<u64>, String> {
    let o = command().args(args(model, inputs, media, Path::new("."), true)).output().map_err(|e| format!("tripo didn't start ({e}): npm install -g tripo-cli"))?;
    if !o.status.success() {
        return Err(fail(o.status.code(), &o.stdout, &o.stderr).to_string());
    }
    let v = last_json(&o.stdout).ok_or("tripo --dry-run printed no JSON")?;
    if v["valid"] == false {
        return Err(format!("tripo: {}", v["errors"]));
    }
    Ok(v["generation_credits_estimate"].as_u64())
}

/// What a step made: the model file first, then its preview render, and what it cost in credits.
pub struct Made {
    pub files: Vec<PathBuf>,
    pub credits: u64,
    pub task: Option<String>,
}

/// One step through `tripo make`, its files kept in `out`. Blocks until the CLI is done (it polls Tripo itself).
pub fn run(model: &str, inputs: &Map<String, J>, media: &[String], out: &Path) -> Result<Made, Fail> {
    // absolute: the CLI runs inside `out`, so a relative one would nest a second copy of itself there
    let out = &std::path::absolute(out).unwrap_or(out.to_path_buf());
    std::fs::create_dir_all(out).map_err(|e| Fail::Other(format!("{}: {e}", out.display())))?;
    let o = command().current_dir(out).args(args(model, inputs, media, out, false)).output().map_err(|e| Fail::Other(format!("tripo didn't start ({e}): npm install -g tripo-cli")))?;
    if !o.status.success() {
        return Err(fail(o.status.code(), &o.stdout, &o.stderr));
    }
    let v = last_json(&o.stdout).ok_or(Fail::Other("tripo make printed no JSON".into()))?;
    let at = |k: &str| v[k].as_str().map(|p| under(out, p));
    let dir = at("output_dir").unwrap_or(out.to_path_buf());
    let mut files: Vec<PathBuf> = [at("model_file"), at("preview")].into_iter().flatten().collect();
    if files.is_empty() {
        files = v["files"].as_array().into_iter().flatten().filter_map(|f| f.as_str()).map(|f| dir.join(f)).filter(|f| !is_json(f)).collect();
    }
    files.retain(|f| f.exists());
    if files.is_empty() {
        return Err(Fail::Other(format!("tripo make answered with no files: {}", v.to_string().chars().take(300).collect::<String>())));
    }
    Ok(Made { files, credits: v["credits_consumed"].as_u64().unwrap_or(0), task: v["task_id"].as_str().map(str::to_string) })
}

/// The CLI's own task record, not a made file.
fn is_json(f: &Path) -> bool {
    f.extension().is_none_or(|e| e == "json")
}

/// A path the CLI printed: as it is when absolute, else under the directory it ran in.
fn under(out: &Path, p: &str) -> PathBuf {
    let p = Path::new(p);
    if p.is_absolute() { p.to_path_buf() } else { out.join(p) }
}

/// The fal endpoint serving the same Tripo model, for a host without the CLI or a key with no credits: H3.1 by
/// default (fal names Tripo's models h3.1, p1, p2), by what the step is given.
pub fn fal_twin(model: &str, views: usize) -> String {
    let m = match model.to_ascii_lowercase().trim_start_matches("tripo-") {
        "" | "p2" | "p2.0" => "p2",
        "p1" | "p1.0" => "p1",
        _ => "h3.1",
    }
    .to_string();
    let kind = match views { 0 => "text-to-3d", 1 => "image-to-3d", _ => "multiview-to-3d" };
    format!("tripo3d/{m}/{kind}")
}

/// The key's balance in credits (`tripo balance`), for `raea gen balance`.
pub fn balance() -> Result<(u64, u64), String> {
    let o = command().args(["balance", "--json"]).output().map_err(|e| format!("tripo didn't start ({e})"))?;
    if !o.status.success() {
        return Err(fail(o.status.code(), &o.stdout, &o.stderr).to_string());
    }
    let v = last_json(&o.stdout).ok_or("tripo balance printed no JSON")?;
    Ok((v["balance"].as_u64().unwrap_or(0), v["frozen"].as_u64().unwrap_or(0)))
}
