//! Web Audio's decodeAudioData under the platform layer (sdk/ui/src/platform.ts, docs/ts-native.feature P17), as a
//! host call every root has unless the app registers its own:
//!
//!   audio.decode {bytes: base64, rate} -> {rate, length, channels: [base64]}
//!
//! The file's bytes (what the page fetched) decoded by raea-media's backends, in the order a media element tries them,
//! each channel resampled to the context's `rate` as Chrome resamples to its context's, and sent back as little-endian
//! f32. A file no backend reads is refused (the page's EncodingError). The decode runs on a thread of its own.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use serde_json::{Value, json};

use crate::calls::HostCalls;

/// The call, onto `calls` (HostCalls's default has it).
pub fn install(calls: &mut HostCalls) {
    calls.on("audio.decode", |_w, args, answer| {
        std::thread::spawn(move || match decode(&args) {
            Ok(v) => answer.ok(v),
            Err(e) => answer.err(e),
        });
    });
}

fn decode(args: &Value) -> Result<Value, String> {
    let bytes = B64.decode(args["bytes"].as_str().ok_or("no bytes")?).map_err(|e| e.to_string())?;
    let rate = args["rate"].as_f64().filter(|r| (3000.0..=768000.0).contains(r)).ok_or("no rate")?;
    // the backends read files: the bytes written aside under a name that says what they are
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("raea-runtime-js-decode-{}-{n}.{}", std::process::id(), sniff(&bytes)));
    std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    let mut errs = vec![];
    let mut audio = None;
    for b in raea_io_media::preferred() {
        match b.decode_audio(&path) {
            Ok(a) if a.len() > 0 => {
                audio = Some(a);
                break;
            }
            Ok(_) => errs.push(format!("{}: no sound", b.name())),
            Err(e) => errs.push(format!("{}: {e}", b.name())),
        }
    }
    let _ = std::fs::remove_file(&path);
    let audio = audio.ok_or_else(|| format!("Unable to decode audio data ({})", errs.join("; ")))?;
    let channels: Vec<Vec<f32>> = audio.channels.iter().map(|c| resample(c, audio.rate as f64, rate)).collect();
    let length = channels.first().map_or(0, Vec::len);
    let out: Vec<String> = channels.iter().map(|c| B64.encode(c.iter().flat_map(|s| s.to_le_bytes()).collect::<Vec<u8>>())).collect();
    Ok(json!({ "rate": rate, "length": length, "channels": out }))
}

/// What a file's first bytes say it is, as the extension a backend goes by.
fn sniff(b: &[u8]) -> &'static str {
    match b {
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'A', b'V', b'E', ..] => "wav",
        [b'f', b'L', b'a', b'C', ..] => "flac",
        [b'O', b'g', b'g', b'S', ..] => "ogg",
        [b'I', b'D', b'3', ..] => "mp3",
        [0xFF, x, ..] if x & 0xE0 == 0xE0 && x & 0x06 != 0 => "mp3",
        [0xFF, x, ..] if x & 0xF6 == 0xF0 => "aac",
        [_, _, _, _, b'f', b't', b'y', b'p', ..] => "m4a",
        [0x1A, 0x45, 0xDF, 0xA3, ..] => "webm",
        _ => "bin",
    }
}

/// `x` at `from` Hz resampled to `to` Hz, linearly between its neighbours; the length is the decoded duration at `to`.
fn resample(x: &[f32], from: f64, to: f64) -> Vec<f32> {
    if (from - to).abs() < 1e-9 || x.is_empty() {
        return x.to_vec();
    }
    let n = ((x.len() as f64) * to / from).round() as usize;
    let step = from / to;
    (0..n)
        .map(|i| {
            let t = i as f64 * step;
            let j = t.floor() as usize;
            let f = (t - j as f64) as f32;
            let a = x[j.min(x.len() - 1)];
            let b = x[(j + 1).min(x.len() - 1)];
            a + (b - a) * f
        })
        .collect()
}

#[cfg(test)]
mod tests {
    /// apps/app's listen bed (40 s of AAC), decoded as the page asks: how long the call takes (cargo test -- --ignored)
    #[test]
    #[ignore]
    fn decode_time() {
        use base64::Engine as _;
        let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../apps/app/public/media/music-bed-v2.m4a");
        let bytes = super::B64.encode(std::fs::read(p).unwrap());
        let t = std::time::Instant::now();
        let v = super::decode(&serde_json::json!({ "bytes": bytes, "rate": 44100.0 })).unwrap();
        eprintln!("decoded {} frames in {:?}, {} base64 bytes back", v["length"], t.elapsed(), v["channels"][0].as_str().unwrap().len());
    }
}
