//! Solids kept on disk between runs. An expression's mesh is the same every time this build makes it, and making it
//! is the cost: the boolean cuts of a furnished set were a quarter of the seconds its scene took to open, every time
//! it opened (a launch-film set, 11-10). A kept mesh is named by the expression and by this build (the running
//! program's size and time: another build may cut differently), and read back as the bytes that were written, so
//! what is drawn from it is what would have been made. RAEA_SHAPE_CACHE=0 turns it off, =DIR moves it from raea's
//! caches folder (shapes/ in it).

use crate::Tris;
use std::path::PathBuf;

/// raea's caches folder, as raea-io-home names it (this crate is below it): ~/Library/Caches/raea on macOS,
/// %LOCALAPPDATA%\raea\cache on Windows, $XDG_CACHE_HOME/raea or ~/.cache/raea elsewhere.
fn caches() -> Option<PathBuf> {
    let base = PathBuf::from(std::env::var_os(if cfg!(windows) { "LOCALAPPDATA" } else { "HOME" })?);
    if base.as_os_str().is_empty() {
        return None;
    }
    if cfg!(windows) {
        return Some(base.join("raea").join("cache"));
    }
    if cfg!(target_os = "macos") {
        return Some(base.join("Library/Caches/raea"));
    }
    let xdg = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from);
    Some(match xdg {
        Some(d) if d.is_absolute() => d,
        _ => base.join(".cache"),
    }
    .join("raea"))
}

fn this_build() -> (u64, u128) {
    static BUILD: std::sync::OnceLock<(u64, u128)> = std::sync::OnceLock::new();
    *BUILD.get_or_init(|| {
        let m = std::env::current_exe().ok().and_then(|p| std::fs::metadata(p).ok());
        let t = m.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_nanos()).unwrap_or(0);
        (m.map(|m| m.len()).unwrap_or(0), t)
    })
}

/// Where the mesh for `expr` is kept, or None when nothing is (off, or no caches folder here).
pub fn place(expr: &str) -> Option<PathBuf> {
    let root = match std::env::var("RAEA_SHAPE_CACHE") {
        Ok(v) if v == "0" => return None,
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => caches()?.join("shapes"),
    };
    let (len, time) = this_build();
    let mut key: u64 = 0xcbf2_9ce4_8422_2325;
    for b in expr.as_bytes().iter().chain(&len.to_le_bytes()).chain(&time.to_le_bytes()) {
        key = (key ^ *b as u64).wrapping_mul(0x100_0000_01b3);
    }
    Some(root.join(format!("{key:016x}-{:x}.tris", expr.len())))
}

const MAGIC: &[u8; 8] = b"raeatri1";

/// The mesh kept at `p`: None when there is none or it isn't whole (a write that was cut short).
pub fn read(p: &std::path::Path) -> Option<Tris> {
    let b = std::fs::read(p).ok()?;
    let n = u64::from_le_bytes(b.get(8..16)?.try_into().ok()?) as usize;
    if &b[..8] != MAGIC || b.len() != 16 + n * 24 {
        return None;
    }
    let f = |at: usize| f32::from_le_bytes(b[at..at + 4].try_into().unwrap());
    let run = |from: usize| (0..n).map(|i| [f(from + i * 12), f(from + i * 12 + 4), f(from + i * 12 + 8)]).collect::<Vec<_>>();
    Some(Tris { positions: run(16), normals: run(16 + n * 12) })
}

/// `t` kept at `p`, written beside it and moved into place so a reader never sees half of it. Failing is fine: the
/// mesh is made again next time.
pub fn write(p: &std::path::Path, t: &Tris) {
    if t.positions.len() != t.normals.len() {
        return;
    }
    let mut b = Vec::with_capacity(16 + t.positions.len() * 24);
    b.extend_from_slice(MAGIC);
    b.extend_from_slice(&(t.positions.len() as u64).to_le_bytes());
    for v in t.positions.iter().chain(&t.normals) {
        for x in v {
            b.extend_from_slice(&x.to_le_bytes());
        }
    }
    let Some(dir) = p.parent() else { return };
    let part = dir.join(format!(".{}.{}.part", p.file_name().unwrap_or_default().to_string_lossy(), std::process::id()));
    if std::fs::create_dir_all(dir).is_ok() && std::fs::write(&part, b).is_ok() && std::fs::rename(&part, p).is_err() {
        let _ = std::fs::remove_file(&part);
    }
}
