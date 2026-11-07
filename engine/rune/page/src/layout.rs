//! What a frame's layout means for the page, whoever ran the frame (the headless session, or a window's
//! `on_frame_drawn`): each wire element's box as getBoundingClientRect has it, their paint order (for hit tests), each
//! scroll viewport's numbers, and DOM mode's layout snapshot (sdk/ui dom/core.ts: per element its id, box, scroll box,
//! padding, border and z, 18 numbers). `Page::measured` takes it in.

use std::collections::HashMap;

use rune::ElementInfo;
use serde_json::json;

use crate::input::Scroll;
use crate::page::{Out, Page};

/// One frame's layout, measured.
#[derive(Default)]
pub struct Measured {
    pub rects: HashMap<u64, [f32; 4]>,
    /// wire ids in Rune's paint order
    pub order: Vec<u64>,
    pub scrolls: Vec<(u64, Scroll)>,
    pub snap: String,
}

/// The wire id an element of the view stands for (its key is `w<id>`).
pub fn wire_id(e: &ElementInfo) -> Option<u64> {
    e.key.as_deref()?.strip_prefix('w')?.parse().ok()
}

/// An element's box where it's drawn at rest: its layout box carried by its scroll viewports' offsets (Rune's `target`
/// is before them), as getBoundingClientRect has it.
pub fn shown(e: &ElementInfo) -> [f32; 4] {
    [e.target[0] - e.scrolled[0], e.target[1] - e.scrolled[1], e.target[2], e.target[3]]
}

/// `els` (Rune's elements after a frame) measured; `boxes` are the padding and border widths the view mapped.
pub fn measure(els: &[ElementInfo], boxes: &HashMap<u64, [f32; 8]>, viewport: (f32, f32)) -> Measured {
    let mut m = Measured::default();
    let mut n: Vec<f64> = Vec::new();
    for (i, e) in els.iter().enumerate() {
        let Some(id) = wire_id(e) else { continue };
        if e.exiting {
            continue;
        }
        let r = shown(e);
        m.rects.insert(id, r);
        m.order.push(id);
        // a scroll viewport's offset is what it carries its children by; its extent, theirs
        let kids: Vec<&ElementInfo> = els.iter().filter(|k| k.parent == Some(i)).collect();
        let b = boxes.get(&id).copied().unwrap_or_default();
        let (mut sl, mut st, mut sw, mut sh) = (0.0, 0.0, r[2], r[3]);
        if let Some(k) = kids.first() {
            (sl, st) = (k.scrolled[0] - e.scrolled[0], k.scrolled[1] - e.scrolled[1]);
        }
        for k in &kids {
            sw = sw.max(k.target[0] + k.target[2] - e.target[0] + b[2]);
            sh = sh.max(k.target[1] + k.target[3] - e.target[1] + b[3]);
        }
        if sw > r[2] + 0.5 || sh > r[3] + 0.5 {
            m.scrolls.push((id, [sl, st, sw, sh, r[2] - b[4] - b[6], r[3] - b[5] - b[7]]));
        }
        n.extend([id as f32, r[0], r[1], r[2], r[3], sl, st, sw, sh].map(f64::from));
        n.extend(b.map(f64::from));
        n.push(0.0);
    }
    m.snap = json!({"vw": viewport.0, "vh": viewport.1, "dpr": 1, "n": n}).to_string();
    m
}

impl Page {
    /// A frame's layout taken in: the rects and paint order, `scroll` to each element whose offset moved, and the
    /// snapshot queued for the JS when it changed. Whether it did.
    pub fn measured(&mut self, m: Measured) -> bool {
        self.rects = m.rects;
        self.order = m.order;
        self.scrolled(m.scrolls);
        if m.snap == self.last_layout {
            return false;
        }
        self.outbox.push(Out::Layout(m.snap.clone()));
        self.last_layout = m.snap;
        true
    }
}
