//! #28 (4): the export cache's key changes when any file in the edit's folder changes, whatever its kind: an .svg the
//! page loads as an <img> was left out of it (only a list of media extensions was), so an edited logo came back from
//! the cache in its old colour.

use std::path::Path;

fn key(tsx: &Path, js: &Path, out: &Path) -> std::path::PathBuf {
    raea_runtime_bundle::export_cache(tsx, js, out, &[]).expect("a cache key")
}

#[test]
fn an_edited_svg_or_jpeg_beside_the_edit_changes_the_export_key() {
    let dir = std::env::temp_dir().join(format!("raea-cache-inputs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("media")).unwrap();
    unsafe { std::env::set_var("RAEA_EXPORT_CACHE", dir.join("exports")) };
    let (tsx, js, out) = (dir.join("edit.tsx"), dir.join("edit.js"), dir.join("out.mp4"));
    std::fs::write(&tsx, "export {}").unwrap();
    std::fs::write(&js, "registerRoot()").unwrap();
    for (file, a, b) in [("logo.svg", "<svg fill='red'/>", "<svg fill='blue'/>"), ("shot.jpeg", "jpeg one", "jpeg two!")] {
        let p = dir.join("media").join(file);
        std::fs::write(&p, a).unwrap();
        let before = key(&tsx, &js, &out);
        assert_eq!(key(&tsx, &js, &out), before, "the same inputs give the same key");
        std::fs::write(&p, b).unwrap();
        assert_ne!(key(&tsx, &js, &out), before, "{file} changed, so the key does");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
