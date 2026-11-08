//! Where ingest keeps a clip's tracks: the store, and the content hash a clip's folder is named by (on every platform: a
//! song's beats and speech spans are kept there too).

use std::path::Path;

/// The default store: $RAEA_INGEST, else ~/.cache/raea-ingest.
pub fn store() -> std::path::PathBuf {
    std::env::var_os("RAEA_INGEST")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/raea-ingest"))
}

pub fn content_hash(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
