//! What each side reports after a step (a `Snap`), and how two are compared: layout (every data-testid's box within
//! 0.5 pt), pixels (SSIM over the frame and in every element's box) and behaviour (the event trace, and each element's
//! value, focus, hover and scroll offset).

use std::collections::BTreeMap;

use image::RgbaImage;
use serde_json::{Value, json};

/// The spec's bars (docs/ts-native.feature P1, P2).
pub const LAYOUT_PT: f64 = 0.5;
pub const SSIM_FRAME: f64 = 0.98;
pub const SSIM_BOX: f64 = 0.95;

#[derive(Clone, Debug)]
pub struct Node {
    /// x, y, w, h in viewport points
    pub rect: [f64; 4],
    /// laid out with a box (Chrome: it has client rects; Bevy: it's in the tree with a size)
    pub shown: bool,
    pub value: Value,
    pub focused: bool,
    pub hovered: bool,
    /// a scroller's offset (top)
    pub scroll: Option<f64>,
    /// a field's selection, start and end (the DOM's selectionStart and selectionEnd): where its caret is, checked here
    /// since the pixel compares hide the caret (build.ts)
    pub caret: Option<[u64; 2]>,
}

fn caret(v: &Value) -> Option<[u64; 2]> {
    Some([v.get(0)?.as_u64()?, v.get(1)?.as_u64()?])
}

pub struct Snap {
    pub nodes: BTreeMap<String, Node>,
    pub trace: Vec<Value>,
    pub png: RgbaImage,
    /// each data-testid's accessibility record (P16 cases only: src/a11y.rs)
    pub a11y: Option<crate::a11y::Tree>,
    /// the page's clock (performance.now, ms) when it was read: Bevy's side is read no sooner after its previous read
    /// than Chrome's was, so the timers due by Chrome's read are due by Bevy's (src/bevy.rs, `catch_up`)
    pub now: f64,
}

impl Snap {
    /// Writes the oracle's final state, oracle.json in `dir`: each box's rect and state and the trace, for whoever
    /// fixes the case or screen (and to see a case does what it says).
    pub fn write_oracle(&self, dir: &std::path::Path) {
        let nodes: serde_json::Map<String, Value> = self.nodes.iter().map(|(k, n)| (k.clone(), json!({"rect": n.rect, "shown": n.shown, "value": n.value, "focused": n.focused, "hovered": n.hovered, "scroll": n.scroll}))).collect();
        let _ = std::fs::write(dir.join("oracle.json"), serde_json::to_string_pretty(&json!({"nodes": nodes, "trace": self.trace})).unwrap_or_default());
    }

    /// The centre of an element's box, for a step aimed at it.
    pub fn centre(&self, id: &str) -> Option<(f64, f64)> {
        let n = self.nodes.get(id)?;
        Some((n.rect[0] + n.rect[2] / 2.0, n.rect[1] + n.rect[3] / 2.0))
    }
    pub fn rect(&self, id: &str) -> Option<[f64; 4]> {
        self.nodes.get(id).map(|n| n.rect)
    }
}

/// One step's differences, by check.
#[derive(Default, Debug)]
pub struct Diffs {
    pub layout: Vec<String>,
    pub pixels: Vec<String>,
    pub behaviour: Vec<String>,
    /// the accessibility tree (P16)
    pub a11y: Vec<String>,
    /// the frame's SSIM and the lowest box SSIM (for the report)
    pub ssim: Option<f64>,
    pub ssim_box_min: Option<(String, f64)>,
}

fn r(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

fn fmt_rect(r4: [f64; 4]) -> String {
    format!("[{} {} {}x{}]", r(r4[0]), r(r4[1]), r(r4[2]), r(r4[3]))
}

/// Value equality as the two sides report it: numbers compared as numbers, a missing value as null.
fn same_value(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => (x.as_f64().unwrap_or(0.0) - y.as_f64().unwrap_or(0.0)).abs() < 1e-6,
        _ => a == b,
    }
}

/// `a` is the oracle (Chrome), `b` the side under test.
pub fn diff(a: &Snap, b: &Snap) -> Diffs {
    let mut d = Diffs::default();
    // layout
    for (id, na) in &a.nodes {
        match b.nodes.get(id) {
            None if na.shown => d.layout.push(format!("{id}: missing (chrome {})", fmt_rect(na.rect))),
            None => {}
            Some(nb) if !na.shown => {
                if nb.shown {
                    d.layout.push(format!("{id}: shown {} but chrome lays it out with no box", fmt_rect(nb.rect)));
                }
            }
            Some(nb) => {
                let off = (0..4).map(|i| (na.rect[i] - nb.rect[i]).abs()).fold(0.0, f64::max);
                if off > LAYOUT_PT {
                    d.layout.push(format!("{id}: chrome {} bevy {} (off {} pt)", fmt_rect(na.rect), fmt_rect(nb.rect), r(off)));
                }
            }
        }
    }
    for (id, nb) in &b.nodes {
        if !a.nodes.contains_key(id) && nb.shown {
            d.layout.push(format!("{id}: extra element {} (not in chrome)", fmt_rect(nb.rect)));
        }
    }
    // behaviour: the trace, then each element's state
    if a.trace != b.trace {
        let at = a.trace.iter().zip(&b.trace).position(|(x, y)| x != y).unwrap_or(a.trace.len().min(b.trace.len()));
        d.behaviour.push(format!(
            "trace differs at entry {at}: chrome {} bevy {} (chrome has {}, bevy {})",
            a.trace.get(at).map(Value::to_string).unwrap_or("(end)".into()),
            b.trace.get(at).map(Value::to_string).unwrap_or("(end)".into()),
            a.trace.len(),
            b.trace.len()
        ));
    }
    for (id, na) in &a.nodes {
        let Some(nb) = b.nodes.get(id) else { continue };
        if !same_value(&na.value, &nb.value) {
            d.behaviour.push(format!("{id}: value chrome {} bevy {}", na.value, nb.value));
        }
        if na.focused != nb.focused {
            d.behaviour.push(format!("{id}: focused chrome {} bevy {}", na.focused, nb.focused));
        }
        // the caret of the field with the focus
        if na.focused && let Some(ca) = na.caret && nb.caret != Some(ca) {
            d.behaviour.push(format!("{id}: caret chrome {ca:?} bevy {:?}", nb.caret));
        }
        if na.shown && na.hovered != nb.hovered {
            d.behaviour.push(format!("{id}: hovered chrome {} bevy {}", na.hovered, nb.hovered));
        }
        if let Some(sa) = na.scroll {
            match nb.scroll {
                None => d.behaviour.push(format!("{id}: chrome scrolls ({} pt) but bevy has no scroller", r(sa))),
                Some(sb) if (sa - sb).abs() > LAYOUT_PT => d.behaviour.push(format!("{id}: scroll chrome {} bevy {}", r(sa), r(sb))),
                _ => {}
            }
        }
    }
    // the accessibility tree, where the case reads it
    match (&a.a11y, &b.a11y) {
        (Some(ta), Some(tb)) => d.a11y = crate::a11y::diff(ta, tb),
        (Some(_), None) => d.a11y.push("no accessibility tree from the side under test".into()),
        _ => {}
    }
    // pixels
    match crate::ssim::map(&a.png, &b.png) {
        Err(e) => d.pixels.push(e),
        Ok(m) => {
            let s = m.mean();
            d.ssim = Some(s);
            if s < SSIM_FRAME {
                d.pixels.push(format!("frame SSIM {:.4} < {SSIM_FRAME}", s));
            }
            for (id, na) in &a.nodes {
                if !na.shown || na.rect[2] < 1.0 || na.rect[3] < 1.0 {
                    continue;
                }
                let Some(sb) = m.mean_in(na.rect[0], na.rect[1], na.rect[2], na.rect[3]) else { continue };
                if d.ssim_box_min.as_ref().is_none_or(|(_, v)| sb < *v) {
                    d.ssim_box_min = Some((id.clone(), sb));
                }
                if sb < SSIM_BOX {
                    d.pixels.push(format!("{id}: box SSIM {:.4} < {SSIM_BOX}", sb));
                }
            }
        }
    }
    d
}

/// A Chrome snapshot's node from the page's JSON (src/chrome.rs STATE_JS).
pub fn chrome_node(v: &Value) -> Node {
    let rc = &v["rect"];
    let f = |k: &str| rc[k].as_f64().unwrap_or(0.0);
    Node {
        rect: [f("x"), f("y"), f("w"), f("h")],
        shown: v["shown"].as_bool().unwrap_or(false),
        value: v.get("value").cloned().unwrap_or(Value::Null),
        focused: v["focused"].as_bool().unwrap_or(false),
        hovered: v["hovered"].as_bool().unwrap_or(false),
        scroll: v["scroll"].as_f64(),
        caret: caret(&v["caret"]),
    }
}

/// A Bevy tree's node (void.ui: apps/legacy/shell-bevy/src/tree.rs).
pub fn bevy_node(v: &Value) -> Node {
    let rc = &v["rect"];
    let f = |k: &str| rc[k].as_f64().unwrap_or(0.0);
    let rect = [f("x"), f("y"), f("w"), f("h")];
    Node {
        rect,
        shown: rect[2] > 0.0 || rect[3] > 0.0,
        value: v.get("value").cloned().unwrap_or(Value::Null),
        focused: v["focused"].as_bool().unwrap_or(false),
        hovered: v["hovered"].as_bool().unwrap_or(false),
        scroll: v["scroll"]["top"].as_f64(),
        caret: caret(&v["caret"]),
    }
}

/// The report's view of a step's differences.
pub fn diffs_json(step: &str, d: &Diffs) -> Value {
    json!({
        "step": step,
        "layout": d.layout,
        "pixels": d.pixels,
        "behaviour": d.behaviour,
        "a11y": d.a11y,
        "ssim": d.ssim.map(|s| (s * 10000.0).round() / 10000.0),
        "ssim_box_min": d.ssim_box_min.as_ref().map(|(id, s)| json!({"id": id, "ssim": (s * 10000.0).round() / 10000.0})),
    })
}
