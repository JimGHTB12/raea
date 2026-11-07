//! `<img>` sources (R2a): a local file (a path or `file://`) or a `data:` URL, decoded by Rune's image cache on its
//! worker threads, one load per source. Any other URL needs the host's media (R4) and is reported until then.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use rune::image::{Image, ImageCache};

pub struct Images {
    cache: ImageCache,
    by_src: RefCell<HashMap<String, Result<Image, String>>>,
}

impl Default for Images {
    /// Images whose decodes wake nothing: a headless run draws them on its next frame.
    fn default() -> Self {
        Self::new(Arc::new(|| {}))
    }
}

impl Images {
    /// `waker` is the window's (AppSpec::setup's), so a decoded image redraws.
    pub fn new(waker: rune::Waker) -> Self {
        Images { cache: ImageCache::new(waker), by_src: RefCell::new(HashMap::new()) }
    }

    /// The image for `src`, or why there is none.
    pub fn get(&self, src: &str) -> Result<Image, String> {
        if let Some(r) = self.by_src.borrow().get(src) {
            return r.clone();
        }
        let r = self.load(src);
        self.by_src.borrow_mut().insert(src.to_string(), r.clone());
        r
    }

    fn load(&self, src: &str) -> Result<Image, String> {
        if let Some(data) = src.strip_prefix("data:") {
            let (meta, body) = data.split_once(',').ok_or("a data URL without a comma")?;
            if !meta.ends_with(";base64") {
                return Err("only base64 data URLs are read".into());
            }
            let bytes = base64::engine::general_purpose::STANDARD.decode(body.trim()).map_err(|e| format!("its base64: {e}"))?;
            return Ok(self.cache.load(bytes));
        }
        let path = src.strip_prefix("file://").unwrap_or(src);
        if path.starts_with('/') {
            let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
            return Ok(self.cache.load(bytes));
        }
        Err("a URL the host doesn't load yet (http, relative: the host's media, R4)".into())
    }
}
