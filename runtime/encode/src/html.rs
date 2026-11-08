//! An HTML composition on raea-render's GPU path. The Blitz page (raea-page's html.rs), its
//! scripts on @raea/ui's DOM, draws the whole page, its `<video>`s included, as the overlay over transparency, a frame at a time;
//! its `<audio>`, and each `<video>` with sound, become sound rows.

use std::path::{Path, PathBuf};

pub use raea_page::html::HtmlPage;

use raea_core_program::eval::Tup;
use raea_core_program::ir::{Program, V};

/// The composition, lowered: raea-render's program and rows, and the page that draws the overlay.
pub struct Lowered {
    pub prog: Program,
    pub rows: Vec<Tup>,
    pub page: HtmlPage,
}

fn row(kind: &str, id: &str, vals: &[(&str, V)]) -> Tup {
    Tup { kind: kind.into(), id: id.into(), vals: vals.iter().map(|(k, v)| (k.to_string(), v.clone())).collect() }
}

/// The page's frame rate: its root's `data-fps`, else 30.
fn fps_of(path: &Path) -> f64 {
    let file = if path.is_dir() { path.join("index.html") } else { path.to_path_buf() };
    let html = std::fs::read_to_string(file).unwrap_or_default();
    html.find("data-fps=")
        .and_then(|i| html[i + 9..].trim_start_matches(['"', '\'']).split(|c: char| !(c.is_ascii_digit() || c == '.')).next().and_then(|n| n.parse().ok()))
        .filter(|f: &f64| *f > 0.0)
        .unwrap_or(30.0)
}

/// A media element's source as a local file: a path relative to the page's folder, or an http(s) URL fetched once
/// into raea's cache.
fn local(dir: &Path, src: &str) -> Result<String, String> {
    if src.starts_with("http://") || src.starts_with("https://") {
        return Ok(raea_io_net::cache::fetch(src)?.to_string_lossy().into_owned());
    }
    let src = src.strip_prefix("file://").unwrap_or(src);
    Ok(dir.join(src).to_string_lossy().into_owned())
}

/// Lower an HTML composition: the page opened and probed for its length and sound, over a white canvas as a browser
/// paints one (CSS Backgrounds 3, the canvas background: the root's or body's background covers it when set).
pub fn lower(path: &Path) -> Result<Lowered, String> {
    let fps = fps_of(path);
    let page = HtmlPage::open(path, fps, Some(&raea_io_home::home().join("core/fonts")))?;
    let dir = if path.is_dir() { path.to_path_buf() } else { path.parent().map(Path::to_path_buf).unwrap_or_default() };
    let duration = page.probe.duration;
    if duration <= 0.0 {
        return Err("the composition's length is unknown (no data-duration)".into());
    }
    let (w, h) = page.size();
    let frames = (duration * fps - 1e-6).ceil().max(1.0) as i64;
    let n = |x: f64| V::N(x);
    let s = |x: &str| V::S(x.to_string());
    let mut rows = vec![row("camera", "cam", &[("bg", s("#ffffff"))])];
    // the first frame at or after t (a clip shows on frames whose time is inside its window)
    let frame = |t: f64| (t * fps - 1e-6).ceil();
    let num = |v: &serde_json::Value, k: &str, d: f64| v[k].as_f64().unwrap_or(d);
    let sounding = page.probe.videos.iter().filter(|v| v["hasAudio"].as_bool().unwrap_or(false)).chain(page.probe.audio.iter());
    for (i, a) in sounding.enumerate() {
        let id = a["id"].as_str().map(str::to_string).unwrap_or_else(|| format!("sound{i}"));
        if a["loop"].as_bool().unwrap_or(false) || (num(a, "rate", 1.0) - 1.0).abs() > 1e-9 {
            return Err(format!("{id}: looping or a playback rate"));
        }
        let Some(src) = a["src"].as_str() else { continue };
        let (start, end) = (num(a, "start", 0.0), num(a, "end", duration).min(duration));
        rows.push(row("audio", &id, &[
            ("src", s(&local(&dir, src)?)),
            ("src_in", n(num(a, "mediaStart", 0.0) + frame(start) / fps - start)),
            ("from", n(frame(start))),
            ("to", n(frame(end).min(frames as f64))),
            ("gain", n(20.0 * num(a, "volume", 1.0).max(1e-6).log10())),
        ]));
    }
    let prog = Program { fps, width: w, height: h, frames, ..Default::default() };
    Ok(Lowered { prog, rows, page })
}

/// Render an HTML composition on the GPU path (see the module): OUT as for `render_file`.
pub fn render_file(edit: &str, out: &Path, opts: &crate::RenderOpts) -> Result<(), String> {
    let path = PathBuf::from(edit);
    let dir = crate::dir_of(edit);
    let mut l = lower(&path)?;
    // --alpha: the canvas clear, so the PNGs are transparent where the page draws nothing (#32)
    if opts.alpha {
        l.rows[0].vals.insert("bg".into(), V::S("#00000000".into()));
    }
    crate::render_with(&l.prog, &l.rows, &dir, out, opts, Some(l.page))
}
