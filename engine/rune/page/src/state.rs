//! What Tailwind's variants test (docs/targets.feature T3b), kept by the page so a window and the headless session do
//! the same. Hover is Rune's: an element with a `hover:` class hears its hover change (events.rs), and the pointer over
//! something inside it counts, as CSS's :hover does. A press is active from its down to its up on the element on top
//! where it began and everything holding it. Focus-visible follows Chrome: a focus that a key moved, or that any key
//! met while it stayed, shows, and so does a text field's however it came. The system's appearance is Rune's
//! (`Cx::system_dark`), and the width is the viewport's.

use std::collections::HashSet;

use serde_json::Value;

use crate::page::Page;
use crate::tree::Node;
use crate::variants::State;

/// The page's state for one view: what each element's `State` is made from.
#[derive(Clone, Copy)]
pub struct Live<'a> {
    pub hovered: &'a HashSet<u64>,
    /// the elements a press holds: the one on top where it began and those up from it
    pub pressed: &'a [u64],
    pub focus: Option<u64>,
    pub focus_visible: bool,
    pub dark: bool,
    pub width: f32,
}

impl Live<'_> {
    /// Element `id`'s (`n`'s) state.
    pub fn of(&self, id: u64, n: &Node) -> State {
        let control = matches!(n.kind.as_str(), "button" | "editableText" | "select" | "textarea");
        State {
            hover: self.hovered.contains(&id),
            active: self.pressed.contains(&id),
            focus: self.focus == Some(id),
            focus_visible: self.focus == Some(id) && self.focus_visible,
            control,
            disabled: control && n.props.get("disabled").is_some_and(|v| v.as_bool().unwrap_or(true)),
            dark: self.dark,
            width: self.width,
        }
    }
}

/// Whether element `n`'s classes have a variant `v:` anywhere in a chain.
pub fn has_variant(n: &Node, v: &str) -> bool {
    n.props.get("className").and_then(Value::as_str).is_some_and(|c| c.split_whitespace().any(|u| u.split(':').rev().skip(1).any(|p| p == v)))
}

impl Page {
    /// The page's state for a view, with the system's appearance as Rune has it.
    pub fn live(&self, dark: bool) -> Live<'_> {
        Live { hovered: &self.hovered, pressed: &self.pressed, focus: self.focus, focus_visible: self.focus_visible, dark, width: self.viewport.0 }
    }

    /// A press began at `(x, y)`: the element on top there and those up from it are active until it ends.
    pub(crate) fn press_began(&mut self, x: f32, y: f32) {
        self.pressed.clear();
        let mut at = self.at(x, y);
        while let Some(i) = at {
            self.pressed.push(i);
            at = self.tree.get(i).and_then(|n| n.parent);
        }
        self.keyed = false;
    }

    /// The press ended.
    pub(crate) fn press_ended(&mut self) {
        self.pressed.clear();
    }
}
