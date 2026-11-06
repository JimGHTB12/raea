//! HTML compositions' page-independent part (docs/ts-native.feature P27): a the HTML composition format-format page (HTML, CSS, GSAP and
//! its own scripts) prepared for @raea/ui's HTML runtime (sdk/ui/src/html.ts), its probe and its videos read from what
//! the runtime reports. raea-page's HtmlPage (engine/page/src/html.rs) mounts it on a Blitz document; the bevy_ui
//! page that did went with docs/blitz-bevy.feature B5d.
//!
//! The runtime asks the host for files through `html.read` {url, binary?} -> {path, text?}: a path relative to the
//! page's folder, or an http(s) URL fetched once into raea's cache (raea_io_net::cache),
//! so what Blitz fetched is found there.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use bevy::prelude::*;
use serde_json::{Value, json};

use raea_core_wire::tree::VideoAt;

type R<T> = Result<T, String>;

/// What the page reported once loaded.
#[derive(Clone, Debug, Default)]
pub struct Probe {
    pub w: u32,
    pub h: u32,
    pub duration: f64,
    /// each `<video>`: {id, src, start, end, mediaStart, volume, hasAudio, loop, rate}
    pub videos: Vec<Value>,
    pub audio: Vec<Value>,
    pub warnings: Vec<String>,
}

pub fn read_answer(dir: &Path, args: &Value) -> R<Value> {
    let url = args["url"].as_str().ok_or("html.read: no url")?;
    let binary = args["binary"].as_bool().unwrap_or(false);
    let path = if url.starts_with("http://") || url.starts_with("https://") {
        raea_io_net::cache::fetch(url)?
    } else {
        let rel = url.split(['?', '#']).next().unwrap_or(url);
        let p = if Path::new(rel).is_absolute() { PathBuf::from(rel) } else { dir.join(rel) };
        if !p.is_file() {
            return Err(format!("{url}: not found"));
        }
        p
    };
    let mut out = json!({ "path": path.to_string_lossy() });
    if !binary {
        let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        out["text"] = Value::String(String::from_utf8_lossy(&bytes).into_owned());
    }
    Ok(out)
}

/// The runtime's bundle (sdk/ui/src/html.ts), bundled once a process by raea's own bundler (Rolldown, in process),
/// from raea's files where `raea_io_home` finds them: an install's share/raea, else this checkout. `BEVY_TSX_UI` names
/// another sdk/ui/src. (It was bun and this crate's js/build.ts, beside the tree it was built from: a release has
/// neither, so `raea render PAGE.html` failed installed.)
pub fn bundle() -> R<String> {
    static CODE: OnceLock<R<String>> = OnceLock::new();
    CODE.get_or_init(|| {
        let ui = std::env::var_os("BEVY_TSX_UI").map(PathBuf::from).unwrap_or_else(|| raea_io_home::home().join("sdk/ui/src"));
        let entry = ui.join("html.ts");
        if !entry.is_file() {
            return Err(format!("{}: the HTML runtime isn't there (BEVY_TSX_UI names another sdk/ui/src)", entry.display()));
        }
        raea_runtime_bundle::inproc::bundle(&entry, json!({})).map(|(js, _css)| js).map_err(|e| format!("the HTML runtime ({}): {e}", entry.display()))
    })
    .clone()
}

/// The project's files (relative paths), for the runtime's tests of what exists.
fn listing(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(r) = p.strip_prefix(dir) {
                out.push(r.to_string_lossy().into_owned());
            }
            if out.len() > 20000 {
                return out;
            }
        }
    }
    out
}

/// The root composition's size as its markup gives it (data-width / data-height of the first composition), before
/// the page runs.
fn size_of(src: &str) -> (u32, u32) {
    // the first element carrying data-composition-id: an attribute (whitespace before, `=` after), not the name in a
    // stylesheet's selector (`[data-composition-id="main"] > div`), which comes first in a page's <head> and sized a
    // 1080 x 1920 page at the 1920 x 1080 default
    let key = "data-composition-id";
    let at = src
        .match_indices(key)
        .map(|(i, _)| i)
        .find(|&i| i > 0 && src.as_bytes()[i - 1].is_ascii_whitespace() && src[i + key.len()..].trim_start().starts_with('='))
        .unwrap_or(0);
    let tag_start = src[..at].rfind('<').unwrap_or(0);
    let tag_end = src[at..].find('>').map(|i| at + i).unwrap_or(src.len());
    let tag = &src[tag_start..tag_end];
    let num = |k: &str| -> Option<u32> {
        let i = tag.find(k)? + k.len();
        let rest = tag[i..].trim_start().strip_prefix('=')?.trim_start();
        let rest = rest.trim_start_matches(['"', '\'']);
        let n: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        n.parse::<f64>().ok().map(|v| v.round() as u32).filter(|v| *v > 0)
    };
    (num("data-width=").or_else(|| num("data-width")).unwrap_or(1920), num("data-height=").or_else(|| num("data-height")).unwrap_or(1080))
}

/// An HTML composition ready to mount: the page file, its folder (the runtime's reads resolve there), the size it's
/// drawn at and the runtime's `globalThis.__html`.
pub struct Prepared {
    pub file: PathBuf,
    pub dir: PathBuf,
    pub w: u32,
    pub h: u32,
    pub globals: Value,
}

/// `file` (a page, or a folder's index.html) read for @raea/ui's HTML runtime (html.ts, `bundle`), on either page.
pub fn prepare(file: &Path, fps: f64, fonts: Option<&Path>, size: Option<(u32, u32)>, root: Option<&Path>) -> R<Prepared> {
    let file = if file.is_dir() { file.join("index.html") } else { file.to_path_buf() };
    let source = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let file = file.canonicalize().unwrap_or(file);
    // served: the page's folder is `root` and the page a path under it (`/x` is then root's x, as a server rooted
    // there serves it); the project listing is left out (a WPT checkout is too big to list, and only a
    // composition's sub-compositions use it)
    let served = root.and_then(|r| r.canonicalize().ok()).filter(|r| file.starts_with(r));
    let (dir, name, files) = match &served {
        Some(r) => (r.clone(), file.strip_prefix(r).unwrap_or(&file).to_string_lossy().into_owned(), vec![]),
        None => {
            let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
            let files = listing(&dir);
            (dir, file.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_else(|| "index.html".into()), files)
        }
    };
    let (w, h) = size.unwrap_or_else(|| size_of(&source));
    // the bundled faces the composition format's deterministic fonts stand in for system families with (html/fonts.ts)
    let font_files: Vec<String> = fonts
        .and_then(|d| std::fs::read_dir(d).ok())
        .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".woff2")).collect())
        .unwrap_or_default();
    let globals = json!({
        "fonts": fonts.map(|d| d.to_string_lossy().into_owned()),
        "fontFiles": font_files,
        "source": source,
        "file": name,
        "dir": dir.to_string_lossy(),
        "files": files,
        "served": served.is_some(),
        "fps": fps,
    });
    Ok(Prepared { file, dir, w, h, globals })
}

/// What the runtime's `html.ready` said, as a Probe (an error it reported is the open's).
pub fn probe_of(file: &Path, ready: &Value, (w, h): (u32, u32)) -> R<Probe> {
    if let Some(e) = ready["error"].as_str() {
        return Err(format!("{}: {e}", file.display()));
    }
    let probe = Probe {
        w: ready["w"].as_f64().unwrap_or(w as f64).round() as u32,
        h: ready["h"].as_f64().unwrap_or(h as f64).round() as u32,
        duration: ready["duration"].as_f64().unwrap_or(0.0),
        videos: ready["videos"].as_array().cloned().unwrap_or_default(),
        audio: ready["audio"].as_array().cloned().unwrap_or_default(),
        warnings: ready["warnings"].as_array().map(|a| a.iter().filter_map(|w| w.as_str().map(String::from)).collect()).unwrap_or_default(),
    };
    if (probe.w, probe.h) != (w, h) {
        warn!("html: {} is {}x{} (its markup said {w}x{h}); drawn at {w}x{h}", file.display(), probe.w, probe.h);
    }
    Ok(probe)
}

/// A seeked page's videos (`html.seeked`'s): each element's file and source time.
pub fn videos_of(done: &Value) -> Vec<VideoAt> {
    done["videos"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| Some(VideoAt { id: v["id"].as_u64()?, file: PathBuf::from(v["file"].as_str()?), at: v["at"].as_f64()?, fit: None })).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #[test]
    fn size_comes_from_the_root_element_not_a_selector() {
        let page = r#"<html><head><style>[data-composition-id="main"] > div { position: absolute; }</style></head>
<body><div id="root" data-composition-id="main" data-start="0" data-width="1080" data-height="1920"></div></body></html>"#;
        assert_eq!(super::size_of(page), (1080, 1920));
        assert_eq!(super::size_of("<div data-composition-id='m'></div>"), (1920, 1080));
    }
}
