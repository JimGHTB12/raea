//! raea's sound tools from the command line: the two local models (SAO for foley, TokenSynth for an instrument's notes)
//! and the open sample libraries a Tone.js Sampler can play.
//!
//!   raea sao "PROMPT" [--seconds 2] [--seed 1] [--dir EDIT_DIR] [-o OUT.wav]
//!   raea tokensynth "TIMBRE" [--notes C2,C3,C4] [--seconds 2] [--velocity 100] [--seed 1234] [--precision fp16]
//!                          [--dir EDIT_DIR] [-o OUT_DIR]
//!   raea setup-audio [--check] [sao|tokensynth]
//!   raea sounds [WORDS...]
//!   raea sounds get PACK/INSTRUMENT [--notes C2,C4 | --all] [--dir EDIT_DIR] [-o OUT_DIR]
//!
//! sao and tokensynth make through the same asset store a composition's render does (raea_io_assets::Store beside
//! EDIT_DIR/edit.tsx, default the working folder), with the same parameters, so a sound made here is the one
//! `sao("…", {seconds})` or `Tone.tokensynth("…", notes)` in that edit finds already made, and the reverse. Their
//! defaults are the TypeScript functions' defaults for the same reason. Either model installs itself the first time
//! it's used (raea_io_assets::ensure_installed); setup-audio does it on purpose and says what's there.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use raea_runtime_bundle::flag;

mod library;
pub use library::*;
#[cfg(test)]
mod tests;

type R<T> = Result<T, String>;

/// The words of a command that aren't flags or a flag's value.
fn positional<'a>(args: &'a [String], with_value: &[&str]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if with_value.contains(&a) {
            i += 2;
            continue;
        }
        if !a.starts_with("--") || a == "-" {
            out.push(a);
        }
        i += 1;
    }
    out
}

fn number(args: &[String], name: &str, default: f64) -> R<f64> {
    match flag(args, name) {
        None => Ok(default),
        Some(v) => v.parse::<f64>().ok().filter(|n| n.is_finite()).ok_or(format!("{name} takes a number, got `{v}`")),
    }
}

/// The edit folder whose asset store a make goes through: --dir, else the working folder.
fn store(args: &[String]) -> raea_io_assets::Store {
    let dir = PathBuf::from(flag(args, "--dir").unwrap_or_else(|| ".".into()));
    raea_io_assets::Store::beside(&dir.join("edit.tsx"))
}

/// A number as JavaScript prints it (2, not 2.0), for the TypeScript line that reuses what was made.
fn js_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", n as i64) } else { format!("{n}") }
}

// ---- notes ----------------------------------------------------------------------------------------------------------

const SHARPS: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const FLATS: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

/// A note's MIDI number: "C4" is 60, as in Tone; sharps as `#` or `s` (tonejs-instruments' file names), flats as `b`,
/// or a MIDI number itself.
pub fn midi_of(note: &str) -> Option<i64> {
    let s = note.trim();
    if let Ok(n) = s.parse::<i64>() {
        return (0..=127).contains(&n).then_some(n);
    }
    let mut c = s.chars();
    let pc = match c.next()?.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest: String = c.collect();
    let (acc, oct) = match rest.chars().next() {
        Some('#' | 's') => (1, &rest[1..]),
        Some('b') => (-1, &rest[1..]),
        _ => (0, rest.as_str()),
    };
    let oct: i64 = oct.parse().ok()?;
    let m = (oct + 1) * 12 + pc + acc;
    (0..=127).contains(&m).then_some(m)
}

pub fn note_name(midi: i64, flats: bool) -> String {
    let names = if flats { FLATS } else { SHARPS };
    format!("{}{}", names[midi.rem_euclid(12) as usize], midi.div_euclid(12) - 1)
}

/// A note as a file name: `C#4` is `Cs4` (a `#` in a file name is trouble in a URL).
fn note_file(midi: i64) -> String {
    note_name(midi, false).replace('#', "s")
}

fn parse_notes(list: &str) -> R<Vec<(String, i64)>> {
    list.split(',').map(str::trim).filter(|n| !n.is_empty())
        .map(|n| midi_of(n).map(|m| (n.to_string(), m)).ok_or(format!("`{n}` isn't a note (C4, F#3, Bb2 or a MIDI number)")))
        .collect()
}

// ---- sao and tokensynth ---------------------------------------------------------------------------------------------

/// Copy a made asset to where the CLI was asked to put it, with Content Credentials saying a model made it (the store's
/// copy stays as the model left it).
fn deliver(from: &Path, to: &Path) -> R<()> {
    if let Some(d) = to.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    std::fs::copy(from, to).map_err(|e| format!("{}: {e}", to.display()))?;
    Ok(())
}

pub const SAO_USAGE: &str = "raea sao \"PROMPT\" [--seconds 2] [--seed 1] [--dir EDIT_DIR] [-o OUT.wav]";

/// `raea sao "PROMPT" [--seconds 2] [--seed 1] [--dir D] [-o OUT.wav]`: one foley sound, made once.
pub fn sao(args: &[String]) -> R<String> {
    const USAGE: &str = SAO_USAGE;
    let prompt = positional(&args[1..], &["--seconds", "--seed", "--dir", "-o"]).join(" ");
    if prompt.trim().is_empty() {
        return Err(USAGE.into());
    }
    // sao()'s default length in TypeScript, and the seed every composition's make uses
    let seconds = number(args, "--seconds", 2.0)?;
    if !(seconds > 0.0) {
        return Err(format!("--seconds must be positive, got {seconds}"));
    }
    let seed = number(args, "--seed", 1.0)? as u64;
    let s = store(args);
    let params = json!({"seconds": seconds});
    let cached = s.made("sao", &prompt, seed, &params)?.is_some_and(|a| s.path(&a).exists());
    let t0 = std::time::Instant::now();
    let a = s.make("sao", &prompt, seed, &params)?;
    let made = s.path(&a);
    let at = match flag(args, "-o") {
        Some(o) => {
            deliver(&made, Path::new(&o))?;
            o
        }
        None => made.display().to_string(),
    };
    let how = if cached { "already made".to_string() } else { format!("made in {:.1} s", t0.elapsed().as_secs_f64()) };
    let reuse = if seed == 1 { format!("\n         in TSX: sao({prompt:?}, {{ seconds: {} }}) finds it in {}", js_num(seconds), s.dir.display()) } else { String::new() };
    Ok(format!("APPLIED  sao {prompt:?}, {} s, seed {seed} -> {at} ({how}; asset {}){reuse}", js_num(seconds), &a.hash[..12]))
}

pub const TOKENSYNTH_USAGE: &str = "raea tokensynth \"TIMBRE\" [--notes C2,C3,C4] [--seconds 2] [--velocity 100] [--seed 1234] [--precision fp16] [--dir EDIT_DIR] [-o OUT_DIR]";

/// `raea tokensynth "TIMBRE" [--notes C2,C3,C4] ... [-o OUT_DIR]`: an instrument's root notes, one WAV each.
pub fn tokensynth(args: &[String]) -> R<String> {
    const USAGE: &str = TOKENSYNTH_USAGE;
    let timbre = positional(&args[1..], &["--notes", "--seconds", "--velocity", "--seed", "--precision", "--dir", "-o"]).join(" ");
    if timbre.trim().is_empty() {
        return Err(USAGE.into());
    }
    // Tone.tokensynth()'s defaults (sdk/asset/src/assets.ts tokensynthNote), so the params and the cache key are the same
    let notes = parse_notes(&flag(args, "--notes").unwrap_or_else(|| "C2,C3,C4".into()))?;
    if notes.is_empty() {
        return Err(USAGE.into());
    }
    let seconds = number(args, "--seconds", 2.0)?;
    if !(seconds > 0.0 && seconds < 5.0) {
        return Err(format!("--seconds is the note's held length, under 5 (TokenSynth reads notes that end before 5 s); got {seconds}"));
    }
    let velocity = number(args, "--velocity", 100.0)?.round() as i64;
    if !(1..=127).contains(&velocity) {
        return Err(format!("--velocity is 1 to 127, got {velocity}"));
    }
    let seed = number(args, "--seed", 1234.0)? as u64;
    let precision = flag(args, "--precision").unwrap_or_else(|| raea_io_assets::TOKENSYNTH_PRECISION.into());
    if !raea_io_assets::TOKENSYNTH_PRECISIONS.contains(&precision.as_str()) {
        return Err(format!("--precision is one of {}", raea_io_assets::TOKENSYNTH_PRECISIONS.join(", ")));
    }
    let s = store(args);
    let out = flag(args, "-o").map(PathBuf::from);
    let mut lines = Vec::new();
    for (name, midi) in &notes {
        let params = json!({"seconds": seconds, "midi": midi, "velocity": velocity, "seed": seed, "device": "cpu", "precision": precision});
        let cached = s.made("tokensynth", &timbre, seed, &params)?.is_some_and(|a| s.path(&a).exists());
        let t0 = std::time::Instant::now();
        let a = s.make("tokensynth", &timbre, seed, &params)?;
        let made = s.path(&a);
        let at = match &out {
            Some(d) => {
                let to = d.join(format!("{}.wav", note_file(*midi)));
                deliver(&made, &to)?;
                to.display().to_string()
            }
            None => made.display().to_string(),
        };
        let how = if cached { "already made".to_string() } else { format!("made in {:.1} s", t0.elapsed().as_secs_f64()) };
        lines.push(format!("  {name:<4} (MIDI {midi}) -> {at} ({how})"));
    }
    let names: Vec<String> = notes.iter().map(|(n, _)| format!("{n:?}")).collect();
    let mut opts = vec![format!("seconds: {}", js_num(seconds))];
    if velocity != 100 {
        opts.push(format!("velocity: {velocity}"));
    }
    if seed != 1234 {
        opts.push(format!("seed: {seed}"));
    }
    if precision != raea_io_assets::TOKENSYNTH_PRECISION {
        opts.push(format!("precision: {precision:?}"));
    }
    Ok(format!(
        "APPLIED  tokensynth {timbre:?}: {} note(s), {} s held, velocity {velocity}, seed {seed}, {precision}\n{}\n         in TSX: new Tone.Sampler({{ urls: Tone.tokensynth({timbre:?}, [{}], {{ {} }}) }}) finds them in {}",
        notes.len(),
        js_num(seconds),
        lines.join("\n"),
        names.join(", "),
        opts.join(", "),
        s.dir.display()
    ))
}

/// `raea setup-audio [--check] [sao|tokensynth]`: installs whichever local model is missing (--check only
/// says), then where each lives and how much disk it takes.
pub fn setup_audio(args: &[String]) -> R<String> {
    let check = args.iter().any(|a| a == "--check");
    let wanted: Vec<&str> = match positional(&args[1..], &[]).first() {
        Some(&m) if m == "sao" || m == "tokensynth" => vec![m],
        Some(m) => return Err(format!("raea setup-audio [--check] [sao|tokensynth]: no model `{m}`")),
        None => vec!["tokensynth", "sao"],
    };
    let mut lines = Vec::new();
    let mut missing = 0;
    for m in wanted {
        let dir = raea_io_assets::install_dir(m);
        let was = raea_io_assets::installed(m, &dir);
        if !was && !check {
            raea_io_assets::ensure_installed(m)?;
        }
        let now = raea_io_assets::installed(m, &dir);
        missing += !now as usize;
        let what = if m == "sao" { "foley (Stable Audio Open Small, Core ML)" } else { "music: an instrument's notes (TokenSynth)" };
        let state = match (was, now) {
            (true, _) => "installed".to_string(),
            (false, true) => "installed now".to_string(),
            (false, false) => format!("not installed: {} installs it", raea_io_assets::setup_script(m).display()),
        };
        lines.push(format!("  {m:<10} {state}; {what}\n             {} ({})", dir.display(), if now { disk(&dir) } else { "-".into() }));
    }
    let head = if check { if missing > 0 { "CHECKED  local audio models (some missing)" } else { "CHECKED  local audio models" } } else { "APPLIED  local audio models ready" };
    Ok(format!("{head}\n{}", lines.join("\n")))
}

/// What a folder takes on disk, as du says it (clones of another install count in full here, as du counts them).
fn disk(dir: &Path) -> String {
    std::process::Command::new("du").arg("-sh").arg(dir).output().ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).split_whitespace().next().map(str::to_string))
        .unwrap_or_else(|| "?".into())
}

// ---- the open sample libraries --------------------------------------------------------------------------------------

/// sdk/raea/library/sounds/catalog.json (tools/sounds/catalog.py writes it): the libraries, their licences and every
/// instrument's files, built into the binary so a search needs no network.
const CATALOG: &str = include_str!("../../../sdk/raea/library/sounds/catalog.json");

fn catalog() -> R<Value> {
    serde_json::from_str(CATALOG).map_err(|e| format!("the sounds catalog: {e}"))
}

/// An instrument's files: (MIDI note or None, file within the instrument, bytes when known). FluidR3 has every note
/// A0-C8, named with flats; the others list their files, a pitched one's note in its name.
fn files_of(pack: &Value, inst: &Value) -> Vec<(Option<i64>, String, u64)> {
    if pack["id"] == "fluid" {
        return (21..=108).map(|m| (Some(m), format!("{}.mp3", note_name(m, true)), 0)).collect();
    }
    inst["files"].as_array().into_iter().flatten().map(|f| {
        let (path, size) = match f {
            Value::Array(a) => (a.first().and_then(|v| v.as_str()).unwrap_or_default().to_string(), a.get(1).and_then(|v| v.as_u64()).unwrap_or(0)),
            other => (other.as_str().unwrap_or_default().to_string(), 0),
        };
        (note_in(&path), path, size)
    }).collect()
}

/// The note a sample's file name carries: a token like `B2`, `A#0` or `Cs4` between underscores, spaces or dashes.
fn note_in(path: &str) -> Option<i64> {
    let base = path.rsplit('/').next().unwrap_or(path);
    let stem = base.rsplit_once('.').map(|(s, _)| s).unwrap_or(base);
    stem.split(['_', ' ', '-']).rev().find_map(|t| {
        let ok = t.len() >= 2 && t.len() <= 4 && t.starts_with(|c: char| ('A'..='G').contains(&c)) && t.ends_with(|c: char| c.is_ascii_digit());
        if ok { midi_of(t) } else { None }
    })
}

/// How loud a take is, from its name, so a fetch can take the middle dynamic: pp..ff, soft/med/loud, vlN or vN.
fn dynamic(path: &str) -> Option<i64> {
    let base = path.rsplit('/').next().unwrap_or(path).to_lowercase();
    let stem = base.rsplit_once('.').map(|(s, _)| s.to_string()).unwrap_or(base);
    stem.split(['_', ' ', '-']).find_map(|t| match t {
        "pp" => Some(10),
        "p" | "soft" => Some(20),
        "mp" => Some(30),
        "mf" | "med" | "medium" => Some(40),
        "f" | "loud" => Some(50),
        "ff" => Some(60),
        t if t.starts_with("vl") => t[2..].parse::<i64>().ok(),
        t if t.starts_with('v') && t.len() <= 3 => t[1..].parse::<i64>().ok(),
        _ => None,
    })
}

fn is_first_take(path: &str) -> bool {
    let l = path.to_lowercase();
    !l.contains("rr") || l.contains("rr1")
}
