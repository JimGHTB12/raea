//! HTML compositions on the Blitz page (docs/blitz-bevy.feature B5d (1)): a the HTML composition format-format page (HTML, CSS, GSAP
//! and its own scripts) mounted by the host (host.rs) with @raea/ui's HTML runtime (sdk/ui/src/html.ts, raea-runtime-js's
//! html_page::bundle), so the page's scripts work on a DOM whose elements are a Blitz document's, which stylo styles,
//! Taffy lays out and Vello paints as Chrome would. The same open, seek and frame as raea-runtime-js's HtmlPage (bevy_ui),
//! which this replaces.

use std::path::Path;

use bevy::prelude::*;
use raea_runtime_js::html_page::{self, Prepared, Probe};
use serde_json::{Value, json};

use raea_runtime_dom::host::{self, PageAsks, Roots};
use crate::motion::{BlitzPage, Videos};
use raea_sys_page::{Docs, PageClock};

type R<T> = Result<T, String>;

/// Updates a load or a seek may take before it's given up (each runs the page's JS once).
const MAX_UPDATES: usize = 3000;

pub struct HtmlPage {
    page: BlitzPage,
    root: Entity,
    pub probe: Probe,
    videos: Videos,
    /// videos that wouldn't open (told once)
    bad: std::collections::HashSet<std::path::PathBuf>,
}

impl HtmlPage {
    /// The page at `file` (or a folder's index.html), at its markup's size, with the WOFF2 faces in `fonts`.
    pub fn open(file: &Path, fps: f64, fonts: Option<&Path>) -> R<HtmlPage> {
        Self::open_with(file, fps, fonts, None, None)
    }

    /// `open` at `size` when given, and served from `root` (its root-absolute URLs resolve there) when given.
    pub fn open_with(file: &Path, fps: f64, fonts: Option<&Path>, size: Option<(u32, u32)>, root: Option<&Path>) -> R<HtmlPage> {
        let Prepared { file, dir, w, h, globals } = html_page::prepare(file, fps, fonts, size, root)?;
        let code = html_page::bundle()?;
        let mut page = BlitzPage::new_on(w, h, fonts, None)?;
        let app = page.app();
        // the host's step and layout snapshot (HostStepPlugin's and HostPlugin's systems: the App is already finished)
        app.add_systems(PreUpdate, host::step.in_set(raea_sys_page::PageSystems::Step)).add_systems(PostUpdate, host::layout_out.after(raea_sys_page::paint));
        let world = app.world_mut();
        world.init_resource::<raea_runtime_js::HostCalls>();
        world.resource_mut::<raea_runtime_js::HostCalls>().on("html.read", move |_world, args, answer| {
            let dir = dir.clone();
            // a fetch blocks on a thread of its own, never the frame
            std::thread::spawn(move || match html_page::read_answer(&dir, &args) {
                Ok(v) => answer.ok(v),
                Err(e) => answer.err(e),
            });
        });
        let root = host::mount(world, w, h, &code, &[format!("globalThis.__html = {globals};")], fonts.map(raea_sys_fonts::context))?;
        host::place_videos_itself(page.app().world_mut(), root);
        page.show(root);
        let mut me = HtmlPage { page, root, probe: Probe::default(), videos: Videos::default(), bad: Default::default() };
        let ready = me.wait_for("html.ready", |_| true)?;
        me.probe = html_page::probe_of(&file, &ready, (w, h))?;
        Ok(me)
    }

    /// The page's size as drawn.
    pub fn size(&self) -> (u32, u32) {
        self.page.size()
    }

    /// Updates until the root emits `name` with a value `want` accepts; that value.
    fn wait_for(&mut self, name: &str, want: impl Fn(&Value) -> bool) -> R<Value> {
        let root = self.root;
        for _ in 0..MAX_UPDATES {
            self.page.app().update();
            let mut asks = self.page.app().world_mut().resource_mut::<PageAsks>();
            let mut got = None;
            asks.0.retain(|(page, n, v)| {
                if *page != root {
                    return true;
                }
                if got.is_none() && n == name && want(v) {
                    got = Some(v.clone());
                    false
                } else {
                    n != name
                }
            });
            if let Some(v) = got {
                return Ok(v);
            }
            std::thread::sleep(std::time::Duration::from_micros(200));
        }
        Err(format!("raea-page html: no {name} in {MAX_UPDATES} updates"))
    }

    /// The page at `t` seconds: its scripts' timelines, clips, CSS animations and videos where they are then.
    pub fn seek(&mut self, t: f64) -> R<()> {
        self.page.app().world_mut().resource_mut::<PageClock>().0 = Some(t);
        host::post_to(self.page.app().world_mut(), self.root, "html.seek", json!({ "t": t }));
        let done = self.wait_for("html.seeked", |v| (v["t"].as_f64().unwrap_or(f64::NAN) - t).abs() < 1e-9)?;
        // each video's frame placed now, as the runtime says its time is, not when a stream gets there
        let world = self.page.app().world_mut();
        let roots = world.remove_non_send::<Roots>().ok_or("raea-page html: no roots")?;
        let placed = {
            let mut docs = world.non_send_mut::<Docs>();
            match (docs.0.get_mut(&self.root), roots.0.iter().find(|r| r.page == self.root)) {
                (Some(doc), Some(r)) => html_page::videos_of(&done).into_iter().map(|v| (v.file.clone(), self.videos.place(doc, &r.nodes, vec![v]))).collect(),
                _ => Vec::new(),
            }
        };
        world.insert_non_send(roots);
        // a video that won't open (a template's placeholder src) is left out, as a browser leaves it blank
        for (file, r) in placed {
            if let Err(e) = r {
                if self.bad.insert(file) {
                    warn!("raea-page html: {e}");
                    self.probe.warnings.push(e);
                }
            }
        }
        Ok(())
    }

    /// The page drawn and read back: RGBA rows at its size.
    pub fn frame(&mut self) -> R<Vec<u8>> {
        self.page.frame()
    }

    /// WPT's reftest-wait: updates (the page's timers and animation frames running) until the document's root element
    /// no longer has the class, `max` updates at most, then two more, as a browser's harness takes the shot two frames
    /// after; whether the class went (a page without it has nothing to wait for).
    pub fn settle_reftest(&mut self, max: usize) -> bool {
        let waiting = |p: &mut HtmlPage| {
            let root = p.root;
            let docs = p.page.app().world().non_send::<Docs>();
            docs.0.get(&root).is_some_and(|d| d.root_element().attr(blitz_dom::local_name!("class")).is_some_and(|c| c.split_ascii_whitespace().any(|w| w == "reftest-wait")))
        };
        let mut gone = false;
        for _ in 0..max {
            if !waiting(self) {
                gone = true;
                break;
            }
            self.page.app().update();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        for _ in 0..2 {
            self.page.app().update();
        }
        gone
    }

}
