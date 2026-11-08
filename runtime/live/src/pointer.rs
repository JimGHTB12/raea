//! The play's pointer on the page's elements (docs/control.feature FC0a, FC3c): each frame's pointer, a window's or
//! `--input-script`'s "Pointer:X:Y[:down]", hit tested on the page as last laid out, its events sent to the React
//! elements' on* handlers as a browser fires them: pointerOut and pointerOver along the old and new paths when the
//! element under it changes, pointerLeave and pointerEnter on the elements left and entered (innermost left first,
//! outermost entered first), pointerMove when it moves, pointerDown and pointerUp on the button's edges, and click on
//! the elements both the press and the release were over. Each goes to every element up the path (bevy-react runs
//! the ones that have that handler), as `{"t": "uiEvent"}` (runtime/react pointer.rs), before the frame's step.

use serde_json::{Value, json};

/// The page pointer as the last frame left it.
#[derive(Default)]
pub(crate) struct PagePointer {
    at: Option<(f64, f64)>,
    path: Vec<u64>,
    down: Option<Vec<u64>>,
}

impl PagePointer {
    /// This frame's events for a pointer at `(x, y, down)` over `path` (the elements under it, innermost first).
    pub(crate) fn step(&mut self, (x, y, down): (f64, f64, bool), path: Vec<u64>) -> Vec<Value> {
        let mut out = Vec::new();
        let mut send = |ids: &mut dyn Iterator<Item = &u64>, kind: &str| {
            for id in ids {
                out.push(json!({"t": "uiEvent", "event": {"id": id, "kind": kind, "x": x, "y": y, "button": 0}}));
            }
        };
        if self.path.first() != path.first() {
            send(&mut self.path.iter(), "pointerOut");
            send(&mut self.path.iter().filter(|id| !path.contains(id)), "pointerLeave");
            send(&mut path.iter(), "pointerOver");
            send(&mut path.iter().rev().filter(|id| !self.path.contains(id)), "pointerEnter");
        }
        if self.at.is_some_and(|a| a != (x, y)) {
            send(&mut path.iter(), "pointerMove");
        }
        match (down, self.down.take()) {
            (true, None) => {
                send(&mut path.iter(), "pointerDown");
                self.down = Some(path.clone());
            }
            (true, held) => self.down = held,
            (false, Some(pressed)) => {
                send(&mut path.iter(), "pointerUp");
                send(&mut path.iter().filter(|id| pressed.contains(id)), "click");
            }
            (false, None) => {}
        }
        self.at = Some((x, y));
        self.path = path;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(v: &[Value]) -> Vec<(String, u64)> {
        v.iter().map(|m| (m["event"]["kind"].as_str().unwrap().to_string(), m["event"]["id"].as_u64().unwrap())).collect()
    }

    #[test]
    fn a_browsers_order_over_enter_down_up_click() {
        let mut p = PagePointer::default();
        let k = |s: &str, id: u64| (s.to_string(), id);
        // onto a child (5) of 3: over bubbles, enter from the outside in
        assert_eq!(kinds(&p.step((10.0, 10.0, false), vec![5, 3])), vec![k("pointerOver", 5), k("pointerOver", 3), k("pointerEnter", 3), k("pointerEnter", 5)]);
        // pressed on 5, released on its parent 3 alone: the click is 3's
        assert_eq!(kinds(&p.step((10.0, 10.0, true), vec![5, 3])), vec![k("pointerDown", 5), k("pointerDown", 3)]);
        let up = kinds(&p.step((20.0, 10.0, false), vec![3]));
        assert_eq!(up, vec![k("pointerOut", 5), k("pointerOut", 3), k("pointerLeave", 5), k("pointerOver", 3), k("pointerMove", 3), k("pointerUp", 3), k("click", 3)]);
        // still: nothing
        assert!(p.step((20.0, 10.0, false), vec![3]).is_empty());
    }
}
