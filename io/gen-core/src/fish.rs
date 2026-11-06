//! Fish Audio: an audio.speech step on Fish's text-to-speech (`fish:<model>`, else `fish:s2.1-pro`), with the founder's own
//! key in FISH_API_KEY. raea sends the step's words (`text`), its `voice` as Fish's reference_id (a voice model's id
//! from fish.audio) and its `speed` as Fish's prosody speed, and keeps the MP3 that comes back. Fish bills UTF-8 bytes
//! of text ($15 a million on its price list, 11-05), so a plan quotes the words before anything is sent.
//! RAEA_FISH_API points the requests somewhere else (a test's stand-in).

use std::path::{Path, PathBuf};

use serde_json::{Map, Value as J, json};

use crate::http::{Body, call};
use crate::setting;

/// The model a bare `fish:` names.
pub const DEFAULT_MODEL: &str = "s2.1-pro";

/// Fish's price for a million UTF-8 bytes of text, in cents.
const CENTS_PER_MILLION_BYTES: f64 = 1500.0;

fn model(name: &str) -> &str {
    if name.is_empty() { DEFAULT_MODEL } else { name }
}

/// What a step's words would cost, in cents (whole cents, rounded up; a short line is 1¢).
pub fn quote(text: &str) -> u64 {
    ((text.len() as f64) * CENTS_PER_MILLION_BYTES / 1_000_000.0).ceil().max(1.0) as u64
}

/// The JSON body Fish's /v1/tts reads, from an audio.speech step's inputs.
pub fn body(inputs: &Map<String, J>) -> Result<J, String> {
    let text = inputs.get("text").and_then(J::as_str).filter(|t| !t.trim().is_empty()).ok_or("an audio.speech step on fish needs text")?;
    let mut b = json!({"text": text, "format": "mp3"});
    if let Some(v) = inputs.get("voice").and_then(J::as_str).filter(|v| !v.is_empty()) {
        b["reference_id"] = json!(v);
    }
    if let Some(s) = inputs.get("speed").and_then(J::as_f64) {
        if !(0.5..=2.0).contains(&s) {
            return Err(format!("fish: speed is 0.5 to 2, got {s}"));
        }
        b["prosody"] = json!({"speed": s});
    }
    Ok(b)
}

/// One audio.speech step on Fish: the MP3 written beside `stem`, and what the words cost.
pub fn run(name: &str, op: &str, inputs: &Map<String, J>, stem: &Path) -> Result<(PathBuf, u64), String> {
    if op != "audio.speech" {
        return Err(format!("fish:{name} makes speech (audio.speech), not {op}"));
    }
    let key = setting("FISH_API_KEY").ok_or("fish needs your Fish Audio API key: put FISH_API_KEY=… in ~/.raea/env (fish.audio, API keys)")?;
    let body = body(inputs)?;
    let api = setting("RAEA_FISH_API").unwrap_or_else(|| "https://api.fish.audio".into());
    let out = stem.with_extension("mp3");
    let h = [format!("Authorization: Bearer {key}"), format!("model: {}", model(name))];
    let r = call("POST", &format!("{}/v1/tts", api.trim_end_matches('/')), &h, Body::Json(&body), Some(&out), 600)?;
    if !r.ok() {
        let said = std::fs::read_to_string(&out).unwrap_or_default().chars().take(300).collect::<String>();
        let _ = std::fs::remove_file(&out);
        return Err(format!("fish:{} answered {}: {said}", model(name), r.code));
    }
    if !r.mime.starts_with("audio/") {
        let _ = std::fs::remove_file(&out);
        return Err(format!("fish:{} answered {} rather than audio", model(name), r.mime));
    }
    Ok((out, quote(body["text"].as_str().unwrap_or(""))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_becomes_fishs_body() {
        let i = json!({"text": "Three drops.", "voice": "abc", "speed": 0.85});
        assert_eq!(body(i.as_object().unwrap()).unwrap(), json!({"text": "Three drops.", "format": "mp3", "reference_id": "abc", "prosody": {"speed": 0.85}}));
        assert!(body(json!({"text": " "}).as_object().unwrap()).is_err());
        assert!(body(json!({"text": "a", "speed": 3}).as_object().unwrap()).is_err());
    }

    #[test]
    fn words_are_quoted_by_their_bytes() {
        assert_eq!(quote("short"), 1);
        assert_eq!(quote(&"a".repeat(10_000)), 15);
        // a non-ASCII character is several bytes
        assert_eq!(quote(&"語".repeat(10_000)), 45);
    }
}
