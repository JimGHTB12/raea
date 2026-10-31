//! The app's settings and folders under the feed's name. A setting is RAEA_FEED_<name>;
//! the folders are ~/.raea-feed and ~/.cache/raea-feed.

use std::ffi::OsString;
use std::path::PathBuf;

/// RAEA_FEED_<name>.
pub fn var_os(name: &str) -> Option<OsString> {
    std::env::var_os(format!("RAEA_FEED_{name}"))
}

/// var_os as a string (None when it isn't one).
pub fn var(name: &str) -> Option<String> {
    var_os(name).and_then(|v| v.into_string().ok())
}

/// The app's folder: RAEA_FEED_HOME when set, else ~/.raea-feed.
pub fn home() -> PathBuf {
    if let Some(p) = var_os("HOME") {
        return PathBuf::from(p);
    }
    person()
}

/// The person's ~/.raea-feed, whatever RAEA_FEED_HOME says (an eval's home is its own, but its voices are the
/// person's).
pub fn person() -> PathBuf {
    made(user().join(".raea-feed"))
}

/// The app's cache: ~/.cache/raea-feed (models' weights live under it).
pub fn cache() -> PathBuf {
    made(user().join(".cache/raea-feed"))
}

fn user() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

fn made(dir: PathBuf) -> PathBuf {
    let _ = std::fs::create_dir_all(&dir);
    dir
}
