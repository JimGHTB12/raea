//! raea-render as a library: raea's render engine. An edit is a Program (rows spawned and keyed per frame, ir.rs), made by
//! raea from a composition (or by raea-runtime-encode from an HTML page); it's drawn by Bevy (engine.rs). Renders to
//! files, the direct GPU path and the sound are raea-runtime-encode's; the media verbs are raea-app-media's.

// the image codecs raea-render links (PNG, JPEG), shared with dependents (the media verbs among them) without a second copy
pub use image;
pub mod engine;

use std::path::{Path, PathBuf};

use raea_core_program::{eval, ir};

use eval::Tup;
use ir::{Program, V};

/// A program's rows as spawned, in order.
pub fn inserted_of(p: &Program) -> Vec<Tup> {
    let mut out = Vec::new();
    for (kind, rows) in &p.spawns {
        for r in rows {
            let id = match r.get("id") {
                Some(V::S(s)) => s.clone(),
                _ => format!("{kind}{}", out.len()),
            };
            out.push(Tup { kind: kind.clone(), id, vals: r.iter().filter(|(k, _)| *k != "id").map(|(k, v)| (k.clone(), v.clone())).collect() });
        }
    }
    out
}

pub fn dir_of(path: &str) -> PathBuf {
    std::fs::canonicalize(Path::new(path).parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."))).unwrap_or_else(|_| PathBuf::from("."))
}

pub fn save_png(path: &Path, rgba: &[u8], w: u32, h: u32) -> Result<(), String> {
    image::save_buffer(path, rgba, w, h, image::ColorType::Rgba8).map_err(|e| format!("{}: {e}", path.display()))
}
