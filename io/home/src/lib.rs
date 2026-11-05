//! Where raea's own files are, for the crates that read them (the SDK's TypeScript, the tools that install the local
//! models, the fonts), and where its per-user caches and logs go on each platform.

pub mod sdk;

/// Where raea's own files are (the SDK's TypeScript, the lite runtime and its packages, the shell's sources, the
/// sandbox's modules, the tools that install the local models): $RAEA_HOME; else beside an installed binary,
/// `<prefix>/share/raea` for `<prefix>/bin/raea` (the release's layout, tools/release.sh); else the tree this
/// build came from. RAEA_NO_TREE=1 refuses the last: a release's check that it never reaches back into its build tree.
pub fn home() -> std::path::PathBuf {
    static HOME: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    HOME.get_or_init(|| {
        if let Some(h) = std::env::var_os("RAEA_HOME").filter(|v| !v.is_empty()) {
            return std::path::PathBuf::from(h);
        }
        let installed = std::env::current_exe().ok().and_then(|e| e.canonicalize().ok()).and_then(|e| e.parent()?.parent().map(|p| p.join("share/raea")));
        if let Some(d) = installed.filter(|d| d.join("sdk/raea/src").is_dir()) {
            return d;
        }
        if std::env::var_os("RAEA_NO_TREE").is_some() {
            eprintln!("raea: no share/raea beside {} and RAEA_NO_TREE is set: its own files can't be found", std::env::current_exe().map(|e| e.display().to_string()).unwrap_or_default());
            return std::path::PathBuf::from("/nonexistent/raea");
        }
        let tree = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        tree.canonicalize().unwrap_or(tree)
    })
    .clone()
}

/// raea's per-user cache folder: ~/Library/Caches/raea on macOS, $XDG_CACHE_HOME/raea (~/.cache/raea by default)
/// elsewhere on Unix, %LOCALAPPDATA%\raea\cache on Windows. None when that platform's home isn't set.
pub fn caches() -> Option<std::path::PathBuf> {
    user_dir("Library/Caches/raea", "XDG_CACHE_HOME", ".cache", "cache")
}

/// raea's per-user log folder: ~/Library/Logs/raea on macOS, $XDG_STATE_HOME/raea (~/.local/state/raea by default)
/// elsewhere on Unix, %LOCALAPPDATA%\raea\logs on Windows. None when that platform's home isn't set.
pub fn logs() -> Option<std::path::PathBuf> {
    user_dir("Library/Logs/raea", "XDG_STATE_HOME", ".local/state", "logs")
}

/// macOS's folder under HOME, the XDG base folder (its variable, else its default under HOME) with raea in it, or raea's
/// folder of that kind under %LOCALAPPDATA%. An XDG variable that isn't an absolute path is ignored, as the spec says.
fn user_dir(apple: &str, xdg: &str, xdg_default: &str, windows: &str) -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    if cfg!(windows) {
        return std::env::var_os("LOCALAPPDATA").filter(|d| !d.is_empty()).map(|d| PathBuf::from(d).join("raea").join(windows));
    }
    let home = PathBuf::from(std::env::var_os("HOME").filter(|h| !h.is_empty())?);
    if cfg!(target_vendor = "apple") {
        return Some(home.join(apple));
    }
    let base = std::env::var_os(xdg).map(PathBuf::from).filter(|d| d.is_absolute()).unwrap_or_else(|| home.join(xdg_default));
    Some(base.join("raea"))
}

/// A local file as a page's `src` (an <img>, a page's snapshot): its path on Unix; on Windows a file URL, since the
/// page reads `C:\...` as a URL whose scheme is `c` and loads nothing (PT20). A verbatim `\\?\C:\` path loses its
/// prefix first.
pub fn src(p: &std::path::Path) -> String {
    src_on(&p.display().to_string(), cfg!(windows))
}

fn src_on(s: &str, windows: bool) -> String {
    if !windows {
        return s.to_string();
    }
    let s = s.strip_prefix(r"\\?\").filter(|r| r.as_bytes().get(1) == Some(&b':')).unwrap_or(s);
    if s.as_bytes().get(1) == Some(&b':') { format!("file:///{}", s.replace('\\', "/")) } else { s.to_string() }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_src_is_a_file_url_on_windows() {
        assert_eq!(super::src_on("/Users/a/assets/m.png", false), "/Users/a/assets/m.png");
        assert_eq!(super::src_on(r"C:\Users\a\assets/m.png", true), "file:///C:/Users/a/assets/m.png");
        assert_eq!(super::src_on(r"\\?\C:\a\m.png", true), "file:///C:/a/m.png");
        assert_eq!(super::src_on("assets/m.png", true), "assets/m.png");
    }

    #[test]
    fn caches_and_logs_follow_the_platform() {
        // one test, so nothing else reads these variables while it sets them
        unsafe {
            std::env::set_var("HOME", "/h");
            std::env::set_var("XDG_CACHE_HOME", "relative");
            std::env::set_var("XDG_STATE_HOME", "/state");
        }
        let (c, l) = (super::caches().unwrap(), super::logs().unwrap());
        if cfg!(target_vendor = "apple") {
            assert_eq!((c.to_str().unwrap(), l.to_str().unwrap()), ("/h/Library/Caches/raea", "/h/Library/Logs/raea"));
        } else if cfg!(unix) {
            // a relative XDG_CACHE_HOME is ignored, as the spec says
            assert_eq!((c.to_str().unwrap(), l.to_str().unwrap()), ("/h/.cache/raea", "/state/raea"));
        }
    }
}
