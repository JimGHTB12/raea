//! The layer pass (docs/review.feature R2b, R2c): which layer each pixel of a frame shows, so raea's player can select
//! any layer (a 3D part, an element, a text, an image) by where it is clicked. A layer is a stamped thing (engine/bundle
//! stamp.rs): its key is where its JSX is written, its place in a list and the uses of the edit's components it was
//! drawn in (`key_of`). Each frame gives a quarter-size mask of layer indices (mask px (x, y) is frame px (4x+2,
//! 4y+2)); `Recorder` keeps them with the table of layers and writes NAME.layers (file.rs).

mod file;
#[cfg(feature = "page")]
mod page;
#[cfg(feature = "page")]
mod scene;

pub use file::Recorder;
#[cfg(feature = "page")]
pub use page::page_ids;

use std::cell::RefCell;
use std::path::PathBuf;

/// "file:line:col[#index]|use|use…": a stamped element or part's key from its props, `data-raea-loc`,
/// `#data-raea-idx` when it has one, then `|data-raea-use` when it was drawn inside the edit's components. None for an
/// unstamped one.
pub fn key_of(loc: Option<&str>, idx: Option<&str>, uses: Option<&str>) -> Option<String> {
    let loc = loc.filter(|l| !l.is_empty())?;
    let mut k = loc.to_string();
    if let Some(i) = idx.filter(|i| !i.is_empty()) {
        k.push('#');
        k.push_str(i);
    }
    if let Some(u) = uses.filter(|u| !u.is_empty()) {
        k.push('|');
        k.push_str(u);
    }
    Some(k)
}

/// One layer of the table: what NAME.layers' header lists for it.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub key: String,
    /// mesh | element | text | image | video | group
    pub kind: &'static str,
    pub name: String,
    /// the key of the nearest stamped thing it is inside
    pub parent: Option<String>,
    pub loc: String,
    pub uses: Vec<String>,
    /// its .map()'s key
    pub index: Option<String>,
    /// an image or video's source
    pub src: Option<String>,
}

thread_local! {
    /// `raea render --layers`: the render's recorder (one render per thread)
    static ON: RefCell<Option<Recorder>> = const { RefCell::new(None) };
}

/// With `on` (`raea render --layers`), layers recorded from now on for the export `out` of `tsx`, written by `finish`
/// (path_for); off otherwise. The render.json beside the layers that names them, when on.
pub fn start(on: bool, out: &std::path::Path, tsx: &std::path::Path) -> Option<PathBuf> {
    let path = on.then(|| path_for(out, tsx));
    // a folder of frames is no video: its name stands for it
    let video = out.file_name().unwrap_or_default().to_string_lossy().into_owned();
    ON.with(|r| *r.borrow_mut() = path.clone().map(|p| Recorder::new(p, video)));
    path.map(|p| p.with_extension("render.json"))
}

/// Whether this render records its layers.
pub fn on() -> bool {
    ON.with(|r| r.borrow().is_some())
}

/// The recorder, when on.
pub fn with<T>(f: impl FnOnce(&mut Recorder) -> T) -> Option<T> {
    ON.with(|r| r.borrow_mut().as_mut().map(f))
}

/// NAME.layers written at `fps`, the recorder stopped: the file's name, beside render.json, for its "layers". None when
/// off or nothing was recorded.
pub fn finish(fps: f64) -> Option<String> {
    let rec = ON.with(|r| r.borrow_mut().take())?;
    match rec.write(fps) {
        Ok(name) => name,
        Err(e) => {
            eprintln!("raea: the layers weren't written: {e}");
            None
        }
    }
}

/// The NAME.layers beside an export `out`: OUT.layers for a video or still, DIR/NAME.layers for a folder of frames
/// (NAME the composition's).
pub fn path_for(out: &std::path::Path, tsx: &std::path::Path) -> PathBuf {
    match out.extension() {
        Some(_) => out.with_extension("layers"),
        None => out.join(tsx.file_stem().unwrap_or_default()).with_extension("layers"),
    }
}

#[cfg(test)]
mod tests;
