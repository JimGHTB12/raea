//! The page laid out with nothing drawn (docs/direction.feature DR1): the render's own page (raea-runtime-page's
//! BevyPage) synced to the frame's React tree and laid out by Blitz, its text and boxes read as a render's manifest
//! reads them. Made at the first frame that asks, once a read. When it can't be made, the reason is said once and the
//! frames' text carries `needs: "render"`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::World;
use serde_json::Value;

pub(crate) struct Page {
    on: bool,
    size: (u32, u32),
    dir: PathBuf,
    page: Option<raea_runtime_page::BevyPage>,
    failed: bool,
}

impl Page {
    pub(crate) fn new(on: bool, w: u32, h: u32, dir: &Path) -> Page {
        Page { on, size: (w, h), dir: dir.to_path_buf(), page: None, failed: false }
    }

    /// Frame `f`'s page as laid out: the render manifest's frame entry ({videos, text}); None when the page isn't
    /// laid out (not asked for, or it couldn't be).
    pub(crate) fn laid_out(&mut self, react: &mut World, f: i64, fps: f64) -> Option<Value> {
        if !self.on || self.failed {
            return None;
        }
        let got = (|| {
            if self.page.is_none() {
                self.page = Some(raea_runtime_page::BevyPage::new(self.size.0, self.size.1, &self.dir)?);
            }
            let page = self.page.as_mut().ok_or("no page")?;
            page.at_frame(f, fps);
            // the elements holding 3D keep their own props: they change what's painted under them, not the layout
            page.sync(react, &HashMap::new(), None)?;
            page.lay_out();
            Ok::<Value, String>(page.manifest_frame(f))
        })();
        got.map_err(|e| {
            eprintln!("raea world: the page wasn't laid out ({e}), so its text has no box or size (needs: \"render\")");
            self.failed = true;
        })
        .ok()
    }
}
