//! raea's URL cache: an http(s) URL fetched once into `$RAEA_CACHE` (else ~/Library/Caches/raea), keyed by FNV-1a of
//! the URL. A Google Fonts stylesheet is fetched as Chrome, which it answers with WOFF2 faces, and has `#ua=<len>` in
//! its key. RAEA_OFFLINE refuses what isn't cached.

use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

const CHROME_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const PLAIN_UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/81.0";

/// raea's cache folder: $RAEA_CACHE, else the platform's (raea_io_home::caches).
pub fn dir() -> Option<PathBuf> {
    std::env::var_os("RAEA_CACHE").map(PathBuf::from).or_else(raea_io_home::caches)
}

fn fnv(s: &str) -> String {
    let h = s.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
    format!("{h:016x}")
}

/// An http(s) URL's bytes, from the cache or fetched into it; the cached file's path.
pub fn fetch(url: &str) -> Result<PathBuf, String> {
    let google = url.starts_with("https://fonts.googleapis.com/");
    let key = if google { format!("{url}#ua={}", CHROME_UA.len()) } else { url.to_string() };
    let dir = dir().ok_or("no cache folder (HOME unset)")?;
    let path = dir.join(fnv(&key));
    // the cached file under its URL's extension too (a decoder that goes by it: an .mp4 without one has "no video
    // track"), a hard link beside the entry
    let ext = url.split(['?', '#']).next().unwrap_or("").rsplit('/').next().and_then(|f| f.rsplit_once('.')).map(|(_, e)| e.to_ascii_lowercase()).filter(|e| !e.is_empty() && e.len() <= 5 && e.chars().all(|c| c.is_ascii_alphanumeric()));
    let named = |p: PathBuf| -> PathBuf {
        let Some(e) = &ext else { return p };
        let q = p.with_extension(e);
        if q.is_file() || std::fs::hard_link(&p, &q).is_ok() || std::fs::copy(&p, &q).is_ok() { q } else { p }
    };
    // (an empty entry is a fetch that failed on its way and was kept: fetched again)
    if path.metadata().is_ok_and(|m| m.is_file() && m.len() > 0) {
        return Ok(named(path));
    }
    raea_io_home::offline(&format!("{url} (not in raea's cache)"))?;
    #[cfg(target_arch = "wasm32")]
    return Err(format!("{url}: no HTTP in a browser build (the page fetches it)"));
    #[cfg(not(target_arch = "wasm32"))]
    {
    let agent = ureq::AgentBuilder::new().redirects(10).timeout(Duration::from_secs(300)).build();
    let r = agent.get(url).set("User-Agent", if google { CHROME_UA } else { PLAIN_UA }).call().map_err(|e| format!("{url}: {e}"))?;
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut r.into_reader(), &mut bytes).map_err(|e| format!("{url}: {e}"))?;
    if bytes.is_empty() {
        return Err(format!("{url}: an empty answer (not cached)"));
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let tmp = dir.join(format!("{}.part{}", fnv(&key), std::process::id()));
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(named(path))
    }
}

/// fetch, read.
pub fn read(url: &str) -> Result<Vec<u8>, String> {
    fetch(url).and_then(|p| std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display())))
}

#[cfg(test)]
mod tests {
    #[test]
    fn keys_as_before() {
        // the key engine/html's SyncNet gave, so entries cached before are found
        assert_eq!(super::fnv(""), "cbf29ce484222325");
        assert_eq!(super::fnv("a"), "af63dc4c8601ec8c");
    }
}
