//! The local models' make()s: Stable Audio Open (foley) and TokenSynth (an instrument's notes), each run as its own
//! process from its install.

use super::*;

/// Stable Audio Open Small on Core ML ($RAEA_SAO, else ~/.cache/raea-sao; installed on first use), run as its own
/// process: it writes a wav of `seconds` (default 4; the model's window is 11.9 s) for the prompt and seed. run.py is
/// tools/sao's runner; an install from before it has the spike's coreml_run.py, which gives the same bytes.
pub(crate) fn sao(prompt: &str, seed: u64, params: &J, part: &Path) -> Result<Vec<u8>, String> {
    let dir = ensure_installed("sao")?;
    let seconds = params["seconds"].as_f64().unwrap_or(4.0);
    // the runner's ffmpeg picks the container from the extension
    let wav = part.with_extension("wav");
    let runner = if dir.join("run.py").exists() && dir.join(".ready").exists() { "run.py" } else { "coreml_run.py" };
    let mut cmd = std::process::Command::new(dir.join(".venv/bin/python"));
    // the spike's runner imports its sibling sao_test.py, so it can't run isolated (-I drops the script's folder)
    if runner == "run.py" {
        cmd.arg("-I");
    }
    let out = cmd
        .arg(dir.join(runner))
        .args(["--prompt", prompt, "--seed", &seed.to_string(), "--seconds", &seconds.to_string(), "--wav"])
        .arg(&wav)
        .output()
        .map_err(|e| format!("sao: {}: {e}", dir.display()))?;
    let bytes = std::fs::read(&wav);
    let _ = std::fs::remove_file(&wav);
    if !out.status.success() {
        return Err(format!("sao: {}", String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("failed")));
    }
    bytes.map_err(|e| format!("sao: {}: {e}", wav.display()))
}

/// TokenSynth (local; tools/tokensynth/setup.sh installs it in $RAEA_TOKENSYNTH, else ~/.cache/raea-tokensynth, on first
/// use): one note of the instrument the prompt describes, {midi, velocity, seconds} in the params, normalised to -1
/// dBFS. The device is in the params (CPU and MPS give different samples from the same seed), so the cache key holds it;
/// so is `precision` (fp32, fp16, bf16, int8 or coreml: run.py --precision), since each precision samples differently
/// from the same seed. Absent, it's fp16, run.py's default and what TSX's tokensynth() and the CLI always write out.
/// (Until 2026-10-19 an absent precision meant fp32 here; nothing has written a note without one since precisions
/// existed, so no cached key changes meaning in practice.)
pub(crate) fn tokensynth(prompt: &str, seed: u64, params: &J, part: &Path) -> Result<Vec<u8>, String> {
    if params["device"].as_str().unwrap_or("cpu") != "cpu" {
        return Err("tokensynth: only the CPU renders (MPS can't run DAC's decoder on long clips)".into());
    }
    let precision = params["precision"].as_str().unwrap_or(TOKENSYNTH_PRECISION);
    if !TOKENSYNTH_PRECISIONS.contains(&precision) {
        return Err(format!("tokensynth: precision `{precision}`: run.py knows fp32, fp16, bf16, int8 and coreml"));
    }
    let dir = ensure_installed("tokensynth")?;
    let wav = part.with_extension("wav");
    let out = std::process::Command::new(dir.join(".venv/bin/python"))
        .arg("-I")
        .arg(dir.join("run.py"))
        .args(["--timbre", prompt, "--seed", &seed.to_string()])
        .args(["--midi", &params["midi"].as_i64().unwrap_or(60).to_string()])
        .args(["--velocity", &params["velocity"].as_i64().unwrap_or(100).to_string()])
        .args(["--seconds", &params["seconds"].as_f64().unwrap_or(2.0).to_string()])
        .args(["--precision", precision])
        .arg("--weights")
        .arg(dir.join("weights"))
        .arg("--wav")
        .arg(&wav)
        .output()
        .map_err(|e| format!("tokensynth: {}: {e}", dir.display()))?;
    let bytes = std::fs::read(&wav);
    let _ = std::fs::remove_file(&wav);
    if !out.status.success() {
        return Err(format!("tokensynth: {}", String::from_utf8_lossy(&out.stderr).lines().last().unwrap_or("failed")));
    }
    bytes.map_err(|e| format!("tokensynth: {}: {e}", wav.display()))
}
