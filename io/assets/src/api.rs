//! make()s through raea's API (fish narration, the `service` feature), with the paid-request cap and its ledger.

use super::*;


/// Where a run's paid requests are counted: the file the host names in $RAEA_GEN_LEDGER (void: one per eval scenario,
/// one per agent turn), whole; else the edit's own assets/paid.jsonl over the last 24 hours. Every render is its
/// own process, so the count has to outlive one.
pub(crate) fn ledger(dir: &Path) -> (PathBuf, Option<u64>) {
    match std::env::var_os("RAEA_GEN_LEDGER").filter(|v| !v.is_empty()) {
        Some(p) => (PathBuf::from(p), None),
        None => (dir.join("paid.jsonl"), Some(24 * 3600)),
    }
}

pub(crate) fn now_s() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Whether a paid request may go now. On by default and capped (the founder's decision, 2026-10-15): $RAEA_GEN_PAID=0
/// turns paid models off, and $RAEA_GEN_BUDGET=N sets the cap per run (default PAID_PER_RUN). A cached make() sends
/// nothing and counts nothing.
pub(crate) fn paid_allowed(model: &str, dir: &Path) -> Result<(), String> {
    if std::env::var("RAEA_GEN_PAID").as_deref() == Ok("0") {
        return Err(format!("{model} is paid generation, and the host turned it off (RAEA_GEN_PAID=0)"));
    }
    let n = std::env::var("RAEA_GEN_BUDGET").ok().and_then(|n| n.trim().parse::<usize>().ok()).unwrap_or(PAID_PER_RUN);
    let (file, window) = ledger(dir);
    let since = window.map(|w| now_s().saturating_sub(w)).unwrap_or(0);
    let spent = std::fs::read_to_string(&file).unwrap_or_default().lines()
        .filter_map(|l| serde_json::from_str::<J>(l).ok())
        .filter(|l| l["t"].as_u64().unwrap_or(0) >= since)
        .count();
    if spent >= n {
        return Err(format!("{model}: this run has used its {n} paid generations (RAEA_GEN_BUDGET); use tokensynth or sao (local), a cached take, or no sound"));
    }
    Ok(())
}

/// One paid request, counted when it's sent: a request that fails may still be billed.
pub(crate) fn count_paid(model: &str, dir: &Path) {
    let (file, _) = ledger(dir);
    if let Some(d) = file.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&file) {
        use std::io::Write;
        let _ = writeln!(f, "{}", json!({"t": now_s(), "model": model}));
    }
}

/// A setting: the process environment first, then raea's settings file ($RAEA_ENV_FILE, else ~/.raea/env).
pub(crate) fn setting(name: &str) -> Option<String> {
    if let Some(v) = std::env::var(name).ok().filter(|v| !v.trim().is_empty()) {
        return Some(v.trim().to_string());
    }
    let file = std::env::var_os("RAEA_ENV_FILE").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".raea/env"));
    std::fs::read_to_string(file).ok()?.lines().find_map(|l| {
        let (k, v) = l.trim().split_once('=')?;
        (k.trim() == name && !l.trim_start().starts_with('#')).then(|| v.trim().trim_matches('"').to_string())
    }).filter(|v| !v.is_empty())
}


/// A build without raea's API (the public engine) makes only with the local models.
#[cfg(not(feature = "service"))]
pub(crate) fn api(model: &str, _prompt: &str, _params: &J, _part: &Path) -> Result<(Vec<u8>, String), String> {
    Err(format!("{model} isn't in this build of raea (the local models are tokensynth for music and sao for foley)"))
}

