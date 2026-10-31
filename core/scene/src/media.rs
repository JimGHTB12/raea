//! The files 3D parts name, found from the composition's folder.

use std::path::{Path, PathBuf};

thread_local! {
    /// The edit's folder, for the files 3D parts name (set by whoever builds the rows on this thread: a render, or
    /// live play's thread building its 3D), so the rows carry paths that hold wherever raea-render runs.
    static MEDIA_DIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Files named by 3D parts are found from `dir` (the edit's folder) on this thread from now.
pub fn set_media_dir(dir: &Path) {
    MEDIA_DIR.with(|d| *d.borrow_mut() = Some(dir.to_path_buf()));
}

/// A /media/... URL (staticFile) as a file: absolute from the edit's folder when that's known and the file is there,
/// else a path from the edit's folder, its segments decoded.
pub fn src_path(src: &str) -> String {
    // a picture inline (data:image/png;base64,..., data:image/svg+xml;utf8,<svg>): raea-render decodes it in memory, as written
    if src.starts_with("data:") {
        return src.to_string();
    }
    let rel = decoded(src);
    MEDIA_DIR.with(|d| d.borrow().as_ref().map(|dir| media_path(dir, &format!("/{rel}"))).filter(|p| p.exists()).map(|p| p.to_string_lossy().to_string()))
        .unwrap_or(rel)
}

pub fn decoded(src: &str) -> String {
    let s = src.trim_start_matches('/');
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(x) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(x);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// A media src as a file: a path from staticFile (/media/x under the edit's folder), absolute, or relative to it.
pub fn media_path(dir: &Path, src: &str) -> PathBuf {
    if let Some(rest) = src.strip_prefix("/media/") {
        let p = dir.join("media").join(rest);
        if p.exists() {
            return p;
        }
    }
    let p = PathBuf::from(src);
    if p.is_absolute() && p.exists() {
        return p;
    }
    dir.join(src.trim_start_matches('/'))
}
