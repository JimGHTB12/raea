//! What raea's other host makes of input beyond Rune's handlers (runtime/dom/src/host/bridge.rs), kept by the page
//! (R3a) so the headless session and a window (R7a) do the same: the focus (a press moves it to the nearest focusable
//! element up from its target, `focus` and `blur` go to the elements that listen, and a ref's `focus()` and `blur()`
//! arrive as emits), keys sent to the focused element when it listens and as the host's own `keydown`/`keyup`
//! otherwise, menus (a click, Enter or Space on a menu item is its `select`; Escape, or a press outside the open menu,
//! is the menu's `close`), and `scroll` when an element's scroll offset moved. Presses come through events.rs's
//! handlers, so they run inside Rune's dispatch wherever Rune runs.

use raea_core_wire::{Ev, named};
use rune::Tx;
use serde_json::{Value, json};

use crate::page::{Out, Page};
use crate::tree::Node;

/// A scroll viewport's numbers after a frame: scroll left, top, width, height, client width, height.
pub type Scroll = [f32; 6];

impl Page {
    /// The wire element on top at `(x, y)`: the last in paint order whose box held it after the last frame.
    pub fn at(&self, x: f32, y: f32) -> Option<u64> {
        self.order.iter().rev().copied().find(|id| {
            let r = self.rect(*id);
            x >= r[0] && y >= r[1] && x < r[0] + r[2] && y < r[1] + r[3]
        })
    }

    /// The nearest element up from `id` (itself included) that `f` holds for.
    pub fn up(&self, id: Option<u64>, f: impl Fn(&Node) -> bool) -> Option<u64> {
        let mut at = id;
        while let Some(i) = at {
            let n = self.tree.get(i)?;
            if f(n) {
                return Some(i);
            }
            at = n.parent;
        }
        None
    }

    fn role_up(&self, id: Option<u64>, role: &str) -> Option<u64> {
        self.up(id, |n| n.props.get("role").and_then(Value::as_str) == Some(role))
    }

    fn is_field(&self, id: u64) -> bool {
        self.tree.get(id).is_some_and(|n| n.kind == "editableText")
    }

    /// The element with the page's focus.
    pub fn focus(&self) -> Option<u64> {
        self.focus
    }

    /// The text field Rune has focused, by wire id.
    pub fn focused_field(&self) -> Option<u64> {
        self.edits.iter().find(|(_, e)| e.focused).map(|(id, _)| *id)
    }

    /// Before a press at `(x, y)`: the open menu closes if the press is outside it.
    pub(crate) fn before_press(&mut self, x: f32, y: f32) {
        let menu = self.role_up(self.focus, "menu");
        if let Some(m) = menu.filter(|_| self.role_up(self.at(x, y), "menu") != menu) {
            self.send(m, Ev::Close, Value::Null);
        }
    }

    /// After a press at `(x, y)`: the focus moves (Chrome focuses on the press), and a menu item is selected.
    pub(crate) fn after_press(&mut self, x: f32, y: f32, tx: &mut Tx) {
        let t = self.at(x, y);
        let to = self.up(t, focusable);
        self.focus_to(to, tx);
        if let Some(item) = self.role_up(t, "menuitem") {
            self.send(item, Ev::Select, Value::Null);
        }
    }

    /// The focus to `to`: `blur` to the element that had it (its value and where the focus went), `focus` to `to`, and
    /// Rune's own focus with it (a field focused, or the field that had it blurred).
    pub fn focus_to(&mut self, to: Option<u64>, tx: &mut Tx) {
        let from = self.focus;
        if from == to {
            return;
        }
        self.focus = to;
        // Chrome's heuristic: a focus a key moved shows, and so does a text field's however it came
        self.focus_visible = to.is_some_and(|t| self.keyed || self.is_field(t));
        if let Some(f) = from {
            let value = if self.is_field(f) { self.edit_mut(f).text.clone() } else { String::new() };
            self.send(f, Ev::Blur, json!({"value": value, "related": to}));
        }
        match to {
            Some(t) if self.is_field(t) => self.edit_mut(t).focused = true,
            _ if self.focused_field().is_some() => tx.blur(),
            _ => {}
        }
        if let Some(t) = to {
            self.send(t, Ev::Focus, Value::Null);
        }
    }

    /// Tab (Shift+Tab when `back`): the focus to the next (previous) element in the tree's order that is focusable,
    /// shown and not taken out of the order by a negative tabIndex, as Chrome's sequential navigation goes; past the
    /// last (first) it leaves the page. Whether the focus moved (Rune shouldn't then move its own).
    pub fn tab(&mut self, back: bool, tx: &mut Tx) -> bool {
        let mut order = Vec::new();
        let mut stack = vec![crate::tree::ROOT];
        while let Some(id) = stack.pop() {
            let Some(n) = self.tree.get(id) else { continue };
            let skip = n.props.get("tabIndex").and_then(Value::as_i64).is_some_and(|t| t < 0);
            let r = self.rect(id);
            if focusable(n) && !skip && self.rects.contains_key(&id) && r[2] > 0.0 && r[3] > 0.0 {
                order.push(id);
            }
            stack.extend(n.children.iter().rev());
        }
        if back {
            order.reverse();
        }
        let next = match self.focus.and_then(|f| order.iter().position(|o| *o == f)) {
            Some(i) => order.get(i + 1).copied(),
            None => order.first().copied(),
        };
        let moved = next != self.focus;
        self.focus_to(next, tx);
        moved
    }

    /// A field Rune focused itself (a press on it, Tab between fields) taken as the page's focus.
    pub fn follow_field_focus(&mut self, tx: &mut Tx) {
        if let Some(f) = self.focused_field().filter(|f| Some(*f) != self.focus) {
            self.focus_to(Some(f), tx);
        }
    }

    /// What the page asked of the host by `__ui_emit`: `ticks` (a `frame` host event every frame), a ref's `focus`
    /// (by wire id or data-testid) and `blur`. Other names are the JS's own business.
    pub fn emit(&mut self, name: &str, v: &Value, tx: &mut Tx) {
        match name {
            "ticks" => self.ticking = v.as_bool().unwrap_or(false),
            "focus" => {
                let id = v.as_u64().or_else(|| v.as_str().and_then(|t| self.tid(t)));
                if id.is_some() {
                    self.focus_to(id, tx);
                }
            }
            "blur" => self.focus_to(None, tx),
            _ => {}
        }
    }

    /// The element whose `data-testid` is `t`.
    pub fn tid(&self, t: &str) -> Option<u64> {
        self.tree.ids().find(|id| self.tree.get(*id).is_some_and(|n| n.props.get("data-testid").and_then(Value::as_str) == Some(t)))
    }

    /// A key's message: to the focused element if it listens (with a field's value), else the host's own; and a menu
    /// item's `select` or the menu's `close` for Enter, Space and Escape. Whether Rune should still see the key.
    pub fn key_out(&mut self, ev: Ev, mut payload: Value) -> bool {
        let f = self.focus;
        if ev == Ev::KeyDown {
            // a key met by the focus where it is shows it from now on
            self.keyed = true;
            self.focus_visible |= f.is_some();
        }
        if let Some(id) = f.filter(|id| self.is_field(*id)) {
            payload["value"] = json!(self.edit_mut(id).text.clone());
        }
        match f.filter(|id| self.tree.get(*id).is_some_and(|n| n.on.contains(&ev))) {
            Some(id) => self.send(id, ev, payload.clone()),
            None => self.outbox.push(Out::Deliver(named(if ev == Ev::KeyDown { "keydown" } else { "keyup" }, payload.clone()).to_string())),
        }
        if ev != Ev::KeyDown {
            return true;
        }
        let k = payload["key"].as_str().unwrap_or("");
        if matches!(k, "Enter" | " ")
            && let Some(item) = self.role_up(f, "menuitem")
        {
            self.send(item, Ev::Select, Value::Null);
            return false;
        }
        if k == "Escape"
            && let Some(menu) = self.role_up(f, "menu")
        {
            self.send(menu, Ev::Close, Value::Null);
            return false;
        }
        true
    }

    /// After a frame: `scroll` to each element whose scroll offset moved since the last.
    pub(crate) fn scrolled(&mut self, now: Vec<(u64, Scroll)>) {
        for (id, s) in now {
            let before = self.scrolls.insert(id, [s[0], s[1]]);
            if before.is_some_and(|b| b != [s[0], s[1]]) {
                let payload = json!({"scrollLeft": s[0], "scrollTop": s[1], "scrollWidth": s[2], "scrollHeight": s[3], "clientWidth": s[4], "clientHeight": s[5]});
                self.send(id, Ev::Scroll, payload);
            }
        }
    }

    /// A new viewport: the JS hears `resize` (the snapshot after the next frame carries the size).
    pub fn resized(&mut self, w: f32, h: f32) {
        self.viewport = (w, h);
        self.outbox.push(Out::Deliver(named("resize", json!({"w": w, "h": h})).to_string()));
    }
}

/// Focusable as Chrome has it: buttons, fields and links, and anything with a `tabIndex`, unless disabled.
pub fn focusable(n: &Node) -> bool {
    let disabled = n.props.get("disabled").is_some_and(|v| v.as_bool().unwrap_or(true));
    !disabled && (matches!(n.kind.as_str(), "button" | "editableText" | "select" | "textarea") || n.props.contains_key("tabIndex") || (n.kind == "a" && n.props.contains_key("href")))
}
