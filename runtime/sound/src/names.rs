//! A frame's sound items resolved to sounds: sources, buses and generated files by name.

use super::*;

/// `sounds_of` into `out` (a list kept between frames), from the elements as they're read
pub fn sounds_into<'a>(items: impl IntoIterator<Item = (&'a str, Option<&'a Map<String, Value>>, Option<u64>)>, dir: &Path, media_path: impl Fn(&Path, &str) -> PathBuf, made: &mut HashMap<String, Option<PathBuf>>, out: &mut Vec<Sound>) {
    let num = |p: &Map<String, Value>, k: &str| p.get(k).and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
    // a generated sound's key, written into one string a call (a format! a sound a frame before; ZeroHeap)
    let mut key = String::new();
    for (kind, props, node) in items {
        if kind != "audio" && kind != "video" {
            continue;
        }
        // no props: nothing to play (as an empty map had no data-source-time)
        let Some(props) = props else { continue };
        if props.get("data-muted").and_then(|v| v.as_str()) == Some("true") || props.get("data-ended").and_then(|v| v.as_str()) == Some("true") {
            continue;
        }
        let volume = num(props, "data-gain-db").map(|db| 10f64.powf(db / 20.0)).or_else(|| num(props, "data-volume")).unwrap_or(1.0);
        let Some(at) = num(props, "data-source-time") else { continue };
        let file = match (props.get("data-generate").and_then(|v| v.as_str()), props.get("data-prompt").and_then(|v| v.as_str())) {
            (Some(model), Some(prompt)) => {
                let seconds = num(props, "data-seconds").unwrap_or(4.0);
                let options = props.get("data-options").and_then(|v| v.as_str());
                // found before: its path, shared (ZeroHeap: the key and the path were made again each frame)
                let found = SOUND_NAMES.with_borrow_mut(|n| {
                    use std::fmt::Write as _;
                    // another edit folder: its store, its files
                    if n.dir != dir {
                        n.dir = dir.to_path_buf();
                        n.files.clear();
                        n.made.clear();
                    }
                    n.made_key.clear();
                    let _ = write!(n.made_key, "{model}\u{1}{prompt}\u{1}{seconds}\u{1}{}", options.unwrap_or(""));
                    n.made.get(n.made_key.as_str()).cloned()
                });
                if let Some(f) = found {
                    f
                } else {
                // looked up once per sound (the store's index was read and parsed every frame: 0.4 ms of a step)
                key.clear();
                {
                    use std::fmt::Write as _;
                    let _ = write!(key, "{model}\u{1}{prompt}\u{1}{seconds}\u{1}{}", options.unwrap_or(""));
                }
                if !made.contains_key(key.as_str()) {
                    let store = raea_io_assets::Store::beside(&dir.join("edit.tsx"));
                    let (params, seed) = generated_params(seconds, options);
                    let path = match store.made(model, prompt, seed, &params) {
                        Ok(Some(a)) => Some(store.path(&a)),
                        _ => None,
                    };
                    made.insert(key.clone(), path);
                }
                let found = &made[key.as_str()];
                let file: std::sync::Arc<Path> = match found.clone() {
                    Some(p) => p.into(),
                    // not made yet (no render has made it): a local model's sound is made on a worker now and plays
                    // from the frame it's there; a paid one waits for a render
                    None => match made_live(dir, model, prompt, seconds, options) {
                        Some(p) => {
                            made.insert(key.clone(), Some(p.clone()));
                            p.into()
                        }
                        None => continue,
                    },
                };
                SOUND_NAMES.with_borrow_mut(|n| {
                    if n.made.len() > 1024 {
                        n.made.clear();
                    }
                    n.made.insert(key.clone(), file.clone());
                });
                file
                }
            }
            _ => match props.get("src").and_then(|v| v.as_str()) {
                Some(src) => SOUND_NAMES.with_borrow_mut(|n| n.file(dir, src, &media_path)),
                None => continue,
            },
        };
        if volume <= 0.0 {
            continue;
        }
        let (key, bus) = SOUND_NAMES.with_borrow_mut(|n| (n.key(node, &file), props.get("data-bus").and_then(|v| v.as_str()).map(|b| n.bus(b))));
        out.push(Sound {
            key,
            file,
            at,
            volume: volume as f32,
            rate: num(props, "data-playback-rate").unwrap_or(1.0),
            fade_in: num(props, "data-fade-in").unwrap_or(0.0),
            fade_out: num(props, "data-fade-out").unwrap_or(0.0),
            bus,
        });
    }
}

thread_local! {
    /// sounds_of's names, kept between frames (ZeroHeap: a sound's key, path and bus were made again each frame)
    static SOUND_NAMES: std::cell::RefCell<SoundNames> = std::cell::RefCell::default();
}

/// Each source's resolved path (for one edit folder), each element's key and each bus name, shared
#[derive(Default)]
pub(crate) struct SoundNames {
    pub(crate) dir: PathBuf,
    pub(crate) files: HashMap<String, std::sync::Arc<Path>>,
    pub(crate) nodes: HashMap<u64, std::sync::Arc<str>>,
    pub(crate) paths: HashMap<std::sync::Arc<Path>, std::sync::Arc<str>>,
    pub(crate) buses: HashMap<String, std::sync::Arc<str>>,
    /// generated sounds' files by what they were made from, and the key written for a lookup
    pub(crate) made: HashMap<String, std::sync::Arc<Path>>,
    pub(crate) made_key: String,
}

impl SoundNames {
    pub(crate) fn file(&mut self, dir: &Path, src: &str, media_path: &impl Fn(&Path, &str) -> PathBuf) -> std::sync::Arc<Path> {
        if self.dir != dir || self.files.len() > 4096 {
            if self.dir != dir {
                self.made.clear();
            }
            self.dir = dir.to_path_buf();
            self.files.clear();
        }
        if let Some(f) = self.files.get(src) {
            return f.clone();
        }
        let f: std::sync::Arc<Path> = media_path(dir, src).into();
        self.files.insert(src.to_string(), f.clone());
        f
    }

    /// "n{node}", or the file's path for a sound without a node
    pub(crate) fn key(&mut self, node: Option<u64>, file: &std::sync::Arc<Path>) -> std::sync::Arc<str> {
        if self.nodes.len() > 4096 {
            self.nodes.clear();
        }
        if self.paths.len() > 4096 {
            self.paths.clear();
        }
        match node {
            Some(n) => self.nodes.entry(n).or_insert_with(|| format!("n{n}").into()).clone(),
            None => self.paths.entry(file.clone()).or_insert_with(|| file.display().to_string().into()).clone(),
        }
    }

    pub(crate) fn bus(&mut self, b: &str) -> std::sync::Arc<str> {
        if let Some(a) = self.buses.get(b) {
            return a.clone();
        }
        let a: std::sync::Arc<str> = b.into();
        if self.buses.len() < 256 {
            self.buses.insert(b.to_string(), a.clone());
        }
        a
    }
}
