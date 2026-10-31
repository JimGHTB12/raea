//! A TSX entry bundled with bun (js/build.ts): one IIFE of the entry, @raea/ui and React, ready for any engine.
//! `BEVY_TSX_UI` names @raea/ui's source folder and `BEVY_TSX_MODULES` the node_modules to fall back to; both default
//! to raea's (sdk/ui/src, sdk/react/js/node_modules beside this crate).

use std::path::{Path, PathBuf};
use std::time::Instant;

fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The bundle of `entry`, as JS.
pub fn build(entry: &Path) -> Result<String, String> {
    let ui = std::env::var("BEVY_TSX_UI").map(PathBuf::from).unwrap_or_else(|_| here().join("../../sdk/ui/src"));
    let modules = std::env::var("BEVY_TSX_MODULES").map(PathBuf::from).unwrap_or_else(|_| here().join("../../sdk/react/js/node_modules"));
    let entry = entry.canonicalize().map_err(|e| format!("{}: {e}", entry.display()))?;
    let out = std::env::temp_dir().join(format!("raea-runtime-js-{}-{}.js", std::process::id(), entry.file_stem().and_then(|s| s.to_str()).unwrap_or("entry")));
    let t = Instant::now();
    let r = std::process::Command::new("bun")
        .arg(here().join("js/build.ts"))
        .arg(&entry)
        .arg(&out)
        .arg(&ui)
        .arg(&modules)
        .output()
        .map_err(|e| format!("bun (is it installed?): {e}"))?;
    if !r.status.success() {
        return Err(format!("bun build {}: {}", entry.display(), String::from_utf8_lossy(&r.stderr)));
    }
    let code = std::fs::read_to_string(&out).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&out);
    if crate::trace().is_some() {
        eprintln!("raea-runtime-js: bundled {} in {} ms ({} KB)", entry.display(), t.elapsed().as_millis(), code.len() / 1024);
    }
    Ok(code)
}
