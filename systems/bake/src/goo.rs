//! Goo (tools/goo: MLS-MPM, or SPH) baked to a mesh sequence.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::*;

fn goo_dir() -> PathBuf {
    std::env::var_os("RAEA_GOO").filter(|v| !v.is_empty()).map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/raea-goo"))
}

/// the goo tool's scripts (tools/goo): run.py (MPM) and sph.py (SPlisHSPlasH, a spec's "solver": "sph"); a bake is
/// keyed on both
const GOO_CODE: [&str; 2] = ["run.py", "sph.py"];

fn ensure_goo(script: &Path) -> R<PathBuf> {
    let dir = goo_dir();
    let ready = |d: &Path| d.join(".venv/bin/python").exists() && GOO_CODE.iter().all(|f| d.join(f).exists()) && d.join(".ready").exists();
    // a newer script in the tree than the install's: set up again (it copies them)
    let stale = GOO_CODE.iter().any(|f| {
        std::fs::metadata(script.with_file_name(f)).and_then(|m| m.modified()).ok()
            .zip(std::fs::metadata(dir.join(f)).and_then(|m| m.modified()).ok())
            .is_some_and(|(tree, inst)| tree > inst)
    });
    if ready(&dir) && !stale {
        return Ok(dir);
    }
    if std::env::var("RAEA_AUDIO_AUTOSETUP").as_deref() == Ok("0") && !ready(&dir) {
        return Err(format!("the goo solver isn't installed: run {}", script.display()));
    }
    eprintln!("raea: setting up the goo solver ({} into {})", script.display(), dir.display());
    let st = std::process::Command::new("sh").arg(script).env("RAEA_GOO", &dir).stdout(std::io::stderr()).status().map_err(|e| format!("goo setup: {e}"))?;
    if !st.success() || !ready(&dir) {
        return Err(format!("goo setup didn't finish ({st}): run {}", script.display()));
    }
    Ok(dir)
}

/// `physics.goo`: a goo spec (tools/goo/run.py's) baked to a mesh sequence, once per spec and its files.
pub fn goo(at: &Place, v: &Value) -> R<Value> {
    let mut spec = v.get("spec").cloned().ok_or("physics.goo: spec")?;
    let mut files = Vec::new();
    let obstacles = spec.get_mut("obstacles").and_then(Value::as_array_mut).map(std::mem::take).unwrap_or_default();
    let mut placed = Vec::new();
    for mut o in obstacles {
        if let Some(s) = o.get("src").and_then(Value::as_str) {
            let p = (at.media)(s);
            files.push(file_id(&p)?);
            o["src"] = json!(p.display().to_string());
        }
        placed.push(o);
    }
    if !placed.is_empty() {
        spec["obstacles"] = json!(placed);
    }
    for o in spec.get_mut("objects").and_then(Value::as_array_mut).into_iter().flatten() {
        if let Some(s) = o.get("src").and_then(Value::as_str) {
            let p = (at.media)(s);
            files.push(file_id(&p)?);
            o["src"] = json!(p.display().to_string());
        }
    }
    let setup = &at.goo_setup;
    let code: Vec<String> = GOO_CODE.iter().map(|f| std::fs::read(setup.with_file_name(f)).map(|b| hex_sha(&b)).unwrap_or_default()).collect();
    let key = hex_sha(json!(["goo", 1, spec, files, code]).to_string().as_bytes());
    let dir = &at.store;
    let out = dir.join(format!("{key}.rseq"));
    let stats = dir.join(format!("{key}.rseq.json"));
    let answer = || -> R<Value> {
        let s: Value = std::fs::read(&stats).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or(json!({}));
        Ok(json!({"file": out.display().to_string(), "stats": s}))
    };
    if out.exists() && stats.exists() {
        return answer();
    }
    if std::env::var("RAEA_PHYSICS_NO_BAKE").as_deref() == Ok("1") {
        return Err(format!("physics.goo: not baked yet ({key}) and RAEA_PHYSICS_NO_BAKE=1"));
    }
    let tool = ensure_goo(&at.goo_setup)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let sp = dir.join(format!("{key}.goo.json"));
    std::fs::write(&sp, spec.to_string()).map_err(|e| format!("{}: {e}", sp.display()))?;
    let tmp = dir.join(format!("{key}.rseq.part"));
    let t0 = std::time::Instant::now();
    eprintln!("raea: baking goo ({} objects, {} frames) -> {}", spec["objects"].as_array().map(|a| a.len()).unwrap_or(0), spec["frames"], out.display());
    // (a bake is made once: a render's wait for the frame's commit doesn't count it, track::making)
    let st = making(|| std::process::Command::new(tool.join(".venv/bin/python"))
        .arg("-I").arg(tool.join("run.py")).arg(&sp).arg(&tmp)
        .stdout(std::io::stderr())
        .status())
        .map_err(|e| format!("goo: {e}"))?;
    if !st.success() {
        return Err(format!("goo bake failed ({st}); spec {}", sp.display()));
    }
    std::fs::rename(&tmp, &out).map_err(|e| format!("{}: {e}", out.display()))?;
    let _ = std::fs::rename(tmp.with_extension("part.json"), &stats).or_else(|_| std::fs::rename(dir.join(format!("{key}.rseq.part.json")), &stats));
    if !stats.exists() {
        let _ = std::fs::write(&stats, json!({"seconds": t0.elapsed().as_secs_f64()}).to_string());
    }
    eprintln!("raea: goo baked in {:.1} s", t0.elapsed().as_secs_f64());
    answer()
}
