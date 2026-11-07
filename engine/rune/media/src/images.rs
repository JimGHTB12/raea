//! `<img>` sources (R2a, T3d): decoded by Rune's image cache on its worker threads, one load per source. A URL comes
//! through raea's URL cache: headless on the calling thread, so a run is the same every time; in a window on a thread
//! of its own, which wakes the window when the bytes are in (the image is drawn from the next frame).

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rune::image::{Image, ImageCache};

use crate::src::{self, Src};

/// Where a source's image is.
#[derive(Clone)]
pub enum Load {
    /// Rune has its bytes (it may still be decoding them)
    Image(Image),
    /// a URL still being fetched
    Fetching,
    Failed(String),
}

type Fetched = Arc<Mutex<HashMap<String, Result<Vec<u8>, String>>>>;

pub struct Images {
    cache: ImageCache,
    by_src: RefCell<HashMap<String, Load>>,
    /// a window's: fetches on threads, which wake it
    waker: Option<rune::Waker>,
    fetched: Fetched,
    /// the elements the last view drew an image for, and their sources
    shown: RefCell<HashMap<u64, String>>,
    /// the source each element was last told of
    told: RefCell<HashMap<u64, String>>,
}

impl Default for Images {
    /// Headless: decodes wake nothing (the next frame draws them) and a URL is fetched where it's asked for.
    fn default() -> Self {
        Images { cache: ImageCache::new(Arc::new(|| {})), by_src: RefCell::default(), waker: None, fetched: Fetched::default(), shown: RefCell::default(), told: RefCell::default() }
    }
}

impl Images {
    /// A window's images: `waker` (AppSpec::setup's) redraws when a decode or a fetch is done.
    pub fn new(waker: rune::Waker) -> Self {
        Images { cache: ImageCache::new(waker.clone()), waker: Some(waker), ..Images::default() }
    }

    /// Element `id`'s image, for a view drawing it.
    pub fn show(&self, id: u64, src: &str) -> Load {
        self.shown.borrow_mut().insert(id, src.to_string());
        self.get(src)
    }

    /// Before a view: no element drawn yet.
    pub fn unsee(&self) {
        self.shown.borrow_mut().clear();
    }

    /// After a frame: each drawn element's news (`load` or `error`) once its source has some, once per source.
    pub fn news_out(&self) -> Vec<(u64, serde_json::Value)> {
        let shown: Vec<(u64, String)> = self.shown.borrow().iter().map(|(k, v)| (*k, v.clone())).collect();
        let mut out = Vec::new();
        for (id, s) in shown {
            if self.told.borrow().get(&id) == Some(&s) {
                continue;
            }
            if let Some(n) = self.news(&s) {
                self.told.borrow_mut().insert(id, s);
                out.push((id, n));
            }
        }
        out.sort_by_key(|(id, _)| *id);
        out
    }

    /// The image for `src`.
    pub fn get(&self, src: &str) -> Load {
        if let Some(l) = self.by_src.borrow().get(src)
            && !matches!(l, Load::Fetching)
        {
            return l.clone();
        }
        let l = self.load(src);
        self.by_src.borrow_mut().insert(src.to_string(), l.clone());
        l
    }

    /// `src` as its news for @raea/ui (`load` with its size, or `error`), once it has some.
    pub fn news(&self, src: &str) -> Option<serde_json::Value> {
        match self.get(src) {
            Load::Image(i) if i.error().is_some() => Some(serde_json::json!({"type": "error"})),
            Load::Image(i) => i.size().map(|[w, h]| serde_json::json!({"type": "load", "w": w, "h": h})),
            Load::Fetching => None,
            Load::Failed(_) => Some(serde_json::json!({"type": "error"})),
        }
    }

    fn load(&self, s: &str) -> Load {
        let from = match src::resolve(s, src::base()) {
            Ok(f) => f,
            Err(e) => return Load::Failed(e),
        };
        let bytes = match (&from, &self.waker) {
            (Src::Url(url), Some(wake)) => {
                let done = self.fetched.lock().unwrap().remove(url);
                match done {
                    Some(b) => b,
                    None => {
                        self.fetch_beside(url, wake.clone());
                        return Load::Fetching;
                    }
                }
            }
            _ => src::bytes(&from),
        };
        match bytes {
            Ok(b) => Load::Image(self.cache.load(b)),
            Err(e) => Load::Failed(e),
        }
    }

    /// A URL fetched on a thread of its own (once), waking the window when it's in.
    fn fetch_beside(&self, url: &str, wake: rune::Waker) {
        if matches!(self.by_src.borrow().get(url), Some(Load::Fetching)) {
            return;
        }
        let (url, fetched) = (url.to_string(), self.fetched.clone());
        std::thread::spawn(move || {
            let r = src::bytes(&Src::Url(url.clone()));
            fetched.lock().unwrap().insert(url, r);
            wake();
        });
    }
}
