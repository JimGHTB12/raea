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

/// The project an edit belongs to: the nearest folder at or above the edit's holding space.json (an object's folder,
/// docs/sdk.feature S5a, whose edit is src/main.tsx), else the edit's own folder. Remembered per folder, since media
/// paths are resolved every frame.
pub fn project_dir(dir: &Path) -> PathBuf {
    static SEEN: std::sync::Mutex<Option<std::collections::HashMap<PathBuf, PathBuf>>> = std::sync::Mutex::new(None);
    let mut seen = SEEN.lock().unwrap_or_else(|e| e.into_inner());
    let seen = seen.get_or_insert_with(Default::default);
    if let Some(p) = seen.get(dir) {
        return p.clone();
    }
    let p = dir.ancestors().find(|d| d.join("space.json").is_file()).unwrap_or(dir).to_path_buf();
    seen.insert(dir.to_path_buf(), p.clone());
    p
}

/// A media src as a file: a path from staticFile (/media/x: the project's media/, then its assets/, as Bevy's assets
/// folder; the edit's own media/ first when the edit isn't the project's folder), absolute, or relative to the edit.
pub fn media_path(dir: &Path, src: &str) -> PathBuf {
    if let Some(rest) = src.strip_prefix("/media/") {
        let project = project_dir(dir);
        let mut tried = vec![dir.join("media")];
        if project != dir {
            tried.push(project.join("media"));
        }
        tried.push(project.join("assets"));
        if let Some(p) = tried.iter().map(|d| d.join(rest)).find(|p| p.exists()) {
            return p;
        }
    }
    let p = PathBuf::from(src);
    if p.is_absolute() && p.exists() {
        return p;
    }
    dir.join(src.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    // an object's folder (S5a): space.json at its root, the edit in src/, files in assets/ or media/
    #[test]
    fn media_from_the_project() {
        let root = std::env::temp_dir().join(format!("raea-media-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let src = root.join("src");
        for d in [&src, &root.join("assets"), &root.join("media"), &src.join("media")] {
            std::fs::create_dir_all(d).unwrap();
        }
        std::fs::write(root.join("space.json"), "{}").unwrap();
        for f in ["assets/a.png", "assets/b.png", "media/b.png", "src/media/c.png", "assets/c.png"] {
            std::fs::write(root.join(f), "").unwrap();
        }
        assert_eq!(project_dir(&src), root);
        assert_eq!(media_path(&src, "/media/a.png"), root.join("assets/a.png"));
        assert_eq!(media_path(&src, "/media/b.png"), root.join("media/b.png"));
        assert_eq!(media_path(&src, "/media/c.png"), src.join("media/c.png"));
        // missing: where it was always looked for
        assert_eq!(media_path(&src, "/media/d.png"), src.join("media/d.png"));
        // no space.json above: the edit's folder is the project, its assets/ after its media/
        let plain = root.join("plain");
        std::fs::create_dir_all(plain.join("assets")).unwrap();
        std::fs::write(plain.join("assets/e.png"), "").unwrap();
        std::fs::remove_file(root.join("space.json")).unwrap();
        assert_eq!(media_path(&plain, "/media/e.png"), plain.join("assets/e.png"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
