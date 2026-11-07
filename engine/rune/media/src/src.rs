//! Where a media source is: a `data:` URL's bytes, a file (a path, `file://`, or a path relative to the app's folder),
//! or an http(s) URL, which comes through raea's URL cache (raea-io-net's cache.rs: fetched once into $RAEA_CACHE,
//! RAEA_OFFLINE refusing what isn't there).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static BASE: OnceLock<PathBuf> = OnceLock::new();

/// The folder relative sources resolve against: the app's (its .tsx's), set once at start.
pub fn set_base(dir: &Path) {
    let _ = BASE.set(dir.to_path_buf());
}

pub fn base() -> Option<&'static Path> {
    BASE.get().map(PathBuf::as_path)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Src {
    Bytes(Vec<u8>),
    File(PathBuf),
    Url(String),
}

/// `src`, read as a browser would on a page in `base`'s folder.
pub fn resolve(src: &str, base: Option<&Path>) -> Result<Src, String> {
    let src = src.trim();
    if src.is_empty() {
        return Err("no source".into());
    }
    if src.starts_with("data:") {
        return raea_io_net::data_url(src).map(|(_, b)| Src::Bytes(b)).ok_or_else(|| "a data URL the host can't read".into());
    }
    if src.starts_with("http://") || src.starts_with("https://") {
        return Ok(Src::Url(src.to_string()));
    }
    if let Some(p) = src.strip_prefix("file://") {
        return Ok(Src::File(raea_io_net::percent_decode_str(p).into()));
    }
    if src.starts_with('/') {
        return Ok(Src::File(src.into()));
    }
    if src.contains(':') {
        return Err("a URL scheme the host doesn't load (data:, file:, http(s): and paths are)".into());
    }
    let base = base.ok_or("a relative source with no app folder to resolve it against")?;
    Ok(Src::File(base.join(raea_io_net::percent_decode_str(src.split(['?', '#']).next().unwrap_or(src)))))
}

/// A source's bytes, fetching a URL through the cache.
pub fn bytes(src: &Src) -> Result<Vec<u8>, String> {
    match src {
        Src::Bytes(b) => Ok(b.clone()),
        Src::File(p) => std::fs::read(p).map_err(|e| format!("{}: {e}", p.display())),
        Src::Url(u) => raea_io_net::cache::read(u),
    }
}
