//! `raea gen …` (raea-io-gen's verbs), with raea's own local models as the flow's `local:` backend: sao for sound
//! effects, made through the edit's asset store as a render would.

use std::path::{Path, PathBuf};

use raea_io_gen::flow::Step;
use serde_json::json;

struct Local;

impl raea_io_gen::run::Local for Local {
    fn names(&self) -> Vec<String> {
        vec!["sao".to_string()]
    }

    fn make(&self, name: &str, step: &Step, dir: &Path, key: &str) -> Result<PathBuf, String> {
        let text = |k: &str| step.inputs.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let (prompt, params) = match name {
            "sao" => (text("prompt"), json!({"seconds": step.inputs.get("seconds").and_then(|v| v.as_f64()).unwrap_or(2.0)})),
            _ => return Err(format!("no local model `{name}` (this host runs sao)")),
        };
        let store = raea_io_assets::Store { dir: dir.join("assets") };
        let seed = step.inputs.get("seed").and_then(|v| v.as_u64()).unwrap_or(1);
        let a = store.make(name, &prompt, seed, &params)?;
        let out = dir.join(format!("{key}.{}", a.ext));
        std::fs::copy(store.path(&a), &out).map_err(|e| format!("{}: {e}", out.display()))?;
        Ok(out)
    }
}

pub fn generate(args: &[String]) -> Result<String, String> {
    raea_io_gen::cli::command(&args[1..], Some(&Local))
}

#[cfg(test)]
mod tests {
    // GN14: a flow's sfx step on this Mac's sao, through `raea gen run` as the CLI calls it. It runs the model (about 2
    // GB, installed on first use), so it's ignored by default: cargo test -p raea-app-tools generate -- --ignored
    #[test]
    #[ignore]
    fn sfx_step_runs_on_sao() {
        let dir = std::env::temp_dir().join(format!("raea-gen-sao-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let flow = dir.join("flow.json");
        std::fs::write(&flow, r#"{"flow":1,"steps":{"knock":{"op":"audio.sfx","in":{"prompt":"three firm knocks on a wooden door, close, dry room","seconds":2}}},"out":["knock"]}"#).unwrap();
        // as engine/cli hands it over: the verb first
        let args: Vec<String> = ["gen", "run", flow.to_str().unwrap(), "-o", dir.to_str().unwrap()].iter().map(|s| s.to_string()).collect();
        let out = super::generate(&args).unwrap();
        assert!(out.contains("local:sao"), "{out}");
        let wav = dir.join("knock.wav");
        assert!(std::fs::metadata(&wav).unwrap().len() > 10_000, "{}", wav.display());
    }
}
