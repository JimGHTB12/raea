//! A page as a Rune app's state: the tree the wire built, its text fields' edits, its images, what couldn't be
//! mapped, and what goes back to the JS (R3). `view` is the Rune view function (`fn(&S, &mut Cx<S>) -> El`).

use std::cell::RefCell;
use std::collections::HashMap;

use rune::{Cx, El, Env, Runtime, TextEdit};
use serde_json::Value;

use crate::images::Images;
use crate::report::Report;
use crate::tree::Tree;
use crate::view::Ctx;

/// What goes back to the JS.
#[derive(Clone, Debug, PartialEq)]
pub enum Out {
    /// a message for `__ui_deliver` (an element's event, a host event)
    Deliver(String),
    /// a layout snapshot for `__ui_layout` (DOM mode's geometry)
    Layout(String),
}

#[derive(Default)]
pub struct Page {
    pub tree: Tree,
    pub images: Images,
    /// each drawn `<video>`'s player (T3d)
    pub videos: raea_rune_media::video::Videos,
    pub report: RefCell<Report>,
    /// each text field's edit, by wire id, its text kept equal to the field's `value` prop
    pub edits: HashMap<u64, TextEdit>,
    /// what the JS hasn't been sent yet
    pub outbox: Vec<Out>,
    /// each element's box in window points after the last frame, by wire id
    pub rects: HashMap<u64, [f32; 4]>,
    /// wire ids in Rune's paint order after the last frame (input.rs's hit test)
    pub order: Vec<u64>,
    /// the element with the page's focus (input.rs)
    pub(crate) focus: Option<u64>,
    /// each scroll viewport's offset after the last frame
    pub(crate) scrolls: HashMap<u64, [f32; 2]>,
    /// the last layout snapshot the JS was sent
    pub last_layout: String,
    /// each element's padding and border widths (l t r b, l t r b) as the last view mapped them
    pub boxes: RefCell<HashMap<u64, [f32; 8]>>,
    /// the pointer in window points, as last seen
    pub pointer: (f32, f32),
    /// the page asked for a `frame` host event every frame (`__ui_emit("ticks", true)`)
    pub ticking: bool,
    /// unrounded text widths for the view
    pub measure: crate::measure::Measure,
    /// the elements with a `hover:` class the pointer is over (state.rs)
    pub hovered: std::collections::HashSet<u64>,
    /// the elements a press holds until its release (state.rs)
    pub pressed: Vec<u64>,
    /// the focus is one the page shows (:focus-visible)
    pub(crate) focus_visible: bool,
    /// the last input was a key (Chrome's focus-visible heuristic)
    pub(crate) keyed: bool,
    /// the viewport in points: a breakpoint's width
    pub viewport: (f32, f32),
}

impl Page {
    /// A batch from the JS, applied to the tree; each text field's edit follows its `value` (a controlled input: what
    /// React decides the value is wins over what was typed).
    pub fn apply(&mut self, json: &str) -> Result<(), String> {
        self.tree.apply_json(json)?;
        let fields: Vec<(u64, String)> = self.tree.ids().filter_map(|id| {
            let n = self.tree.get(id)?;
            (n.kind == "editableText").then(|| (id, n.props.get("value").and_then(Value::as_str).unwrap_or("").to_string()))
        }).collect();
        self.edits.retain(|id, _| fields.iter().any(|(f, _)| f == id));
        for (id, v) in fields {
            let e = self.edits.entry(id).or_insert_with(|| TextEdit::new(""));
            if e.text != v {
                *e = TextEdit::new(v);
            }
        }
        Ok(())
    }

    /// Element `id`'s box after the last frame (zero before its first).
    pub fn rect(&self, id: u64) -> [f32; 4] {
        self.rects.get(&id).copied().unwrap_or_default()
    }

    /// Element `id`'s edit (made empty if the page had none).
    pub fn edit_mut(&mut self, id: u64) -> &mut TextEdit {
        self.edits.entry(id).or_insert_with(|| TextEdit::new(""))
    }

    /// What the JS hasn't been sent, taken.
    pub fn take_out(&mut self) -> Vec<Out> {
        std::mem::take(&mut self.outbox)
    }

    /// After a frame: what @raea/ui hears of the page's images and videos since the last (`media`: load, error,
    /// loadedmetadata, loadeddata, seeked, ended), and the start's font notes in the report (T3d).
    pub fn media_news(&mut self) {
        let mut news = self.images.news_out();
        news.extend(self.videos.news());
        for (id, mut v) in news {
            v["id"] = id.into();
            self.outbox.push(Out::Deliver(raea_core_wire::named("media", v).to_string()));
        }
        let mut r = self.report.borrow_mut();
        for (what, why) in raea_rune_media::fonts::notes() {
            r.note("/", "font", what, why);
        }
    }
}

/// Rune's view of the page.
pub fn view(p: &Page, cx: &mut Cx<Page>) -> El {
    let mut report = p.report.borrow_mut();
    let mut boxes = p.boxes.borrow_mut();
    let live = p.live(cx.system_dark());
    p.images.unsee();
    p.videos.unsee();
    crate::view::build(&p.tree, &mut Ctx { report: &mut report, images: &p.images, videos: &p.videos, edits: &p.edits, boxes: &mut boxes, measure: &p.measure, marks: 0, live })
}

/// A headless Rune runtime for `page` at `viewport` points, with Rune's text (Inter, embedded) so text has its size,
/// and the app's faces (raea-rune-media's fonts, registered at the start).
pub fn headless(mut page: Page, viewport: (f32, f32)) -> Runtime<Page> {
    page.viewport = viewport;
    let text = rune::text::TextSystem::new(raea_rune_media::fonts::registered(), ("Inter", "Inter"));
    Runtime::with_text(page, view, viewport, Env::new(false, 1.0), text)
}
