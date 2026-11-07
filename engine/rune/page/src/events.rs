//! Rune's input back over the wire (R3a): each wire element gets Rune handlers only for the events it listens for,
//! so Rune's hit test hands a gesture to the nearest element that listens, as raea's other host does
//! (runtime/dom/src/host/bridge.rs). A handler queues `{t: "event", id, event, payload}` on the page's outbox with the
//! same payload that host sends: for pointers, where in the element as fractions of its size, the client point, the
//! button, the press count and the modifiers. Text fields are Rune text inputs whose edits go back as `change` with the
//! whole value, one per edit, as the other host sends them.

use raea_core_wire::{Ev, event};
use rune::{Drag, El, Mods, Phase, PointerEvent, Tx, WheelEvent};
use serde_json::{Value, json};

use crate::page::{Out, Page};
use crate::tree::Node;

/// `e` with the handlers for what element `id` (`n`) listens for.
pub fn attach(mut e: El, id: u64, n: &Node, root: bool) -> El {
    let hears = |ev: Ev| n.on.contains(&ev);
    // A press is also the page's (input.rs): the open menu's close before it, the focus and a menu item's select after
    // it. So a focusable element or menu item gets the handler even when it doesn't listen, passing the press to its
    // nearest listener up, and the root takes a press on nothing. Fields are left to Rune's text input (their focus
    // follows Rune's: Page::follow_field_focus).
    let field = n.kind == "editableText";
    let presses = hears(Ev::PointerDown) || hears(Ev::PointerUp) || root || (!field && (crate::input::focusable(n) || n.props.get("role").and_then(Value::as_str) == Some("menuitem")));
    if presses {
        e = e.on_drag(move |p: &mut Page, d: &mut Drag, tx: &mut Tx| {
            let r = p.rect(id);
            let (x, y) = (r[0] + d.x, r[1] + d.y);
            match d.phase {
                Phase::Down => {
                    p.press_began(x, y);
                    p.before_press(x, y);
                    p.send_up(id, Ev::PointerDown, x, y, d.clicks, Mods::default());
                }
                Phase::Up => {
                    p.press_ended();
                    p.send_up(id, Ev::PointerUp, x, y, d.clicks, Mods::default());
                    p.after_press(x, y, tx);
                }
                Phase::Move => {}
            }
        });
    }
    if hears(Ev::PointerMove) {
        e = e.on_pointer_move_local(move |p: &mut Page, m: PointerEvent, _: &mut Tx| at(p, id, Ev::PointerMove, &m, 0, 0));
    }
    // a `hover:` class is restyled by Rune's own hover (state.rs), whether or not the element listens
    let styled = crate::state::has_variant(n, "hover");
    let listens = hears(Ev::PointerEnter) || hears(Ev::PointerLeave);
    if listens || styled {
        e = e.on_hover_change(move |p: &mut Page, inside: bool, _: &mut Tx| {
            // a leave clears hover whatever the classes are now: one that lost its `hover:` while the pointer was on it
            // (the row a press opened) mustn't keep it for when it gets one back
            if !inside {
                p.hovered.remove(&id);
            } else if styled {
                p.hovered.insert(id);
            }
            if !listens {
                return;
            }
            let ev = if inside { Ev::PointerEnter } else { Ev::PointerLeave };
            let (x, y) = p.pointer;
            let r = p.rect(id);
            let payload = pointer(p, id, x - r[0], y - r[1], 0, 0, Mods::default());
            p.send(id, ev, payload);
        });
    }
    if hears(Ev::ContextMenu) {
        e = e.on_secondary_press_local(move |p: &mut Page, m: PointerEvent, _: &mut Tx| at(p, id, Ev::ContextMenu, &m, 2, 1));
    }
    if hears(Ev::Wheel) {
        e = e.on_wheel(move |p: &mut Page, w: WheelEvent, _: &mut Tx| {
            let mut payload = pointer(p, id, w.x, w.y, 0, 0, w.mods);
            // the wire's deltas are DOM's: positive scrolls content up and left, the opposite of Rune's
            payload["deltaX"] = json!(-w.dx);
            payload["deltaY"] = json!(-w.dy);
            payload["deltaMode"] = json!(0);
            p.send(id, Ev::Wheel, payload);
            false
        });
    }
    // keys, focus and blur, scroll, and menus' select and close are the page's (input.rs): they follow its focus and
    // scroll offsets, not a Rune element's handlers
    e
}

/// A text field: Rune's text input over the page's edit for element `id`, its edits going back as `change`.
pub fn field(id: u64, edit: &rune::TextEdit, style: rune::TextStyle, color: rune::Color, placeholder: &str) -> El {
    rune::text_input(edit, style)
        .color(color)
        .single_line()
        .placeholder(placeholder, color.alpha(color.a * 0.5))
        .on_edit(move |p: &mut Page| p.edit_mut(id))
        .on_input_change(move |p: &mut Page, _: &mut Tx| {
            let v = p.edit_mut(id).text.clone();
            p.send(id, Ev::Change, json!(v));
        })
}

/// A pointer payload for element `id` at `(lx, ly)` in its own points.
/// A pointer event at a point local to the element, sent as `ev` with `button` and `detail`
fn at(p: &mut Page, id: u64, ev: Ev, m: &PointerEvent, button: u8, detail: u32) {
    let mut payload = pointer(p, id, m.x, m.y, button, detail, Mods::default());
    if ev == Ev::PointerMove {
        targeted(p, id, &mut payload);
    }
    p.send(id, ev, payload);
}

/// A press's or a move's deepest wire element under the pointer as its `target`, as the other host sends it
/// (runtime/dom/src/host/bridge.rs): sdk/ui's own hit test reads only classes without their variants (a `hidden md:flex`
/// sidebar is hidden to it), so without one a release over it found no element and no click came. It's the topmost
/// element at the point inside `within` (or `within` itself), the element Rune's hit test, which clips, handed the
/// gesture to: the page's own boxes don't clip, and a list's rows scrolled out under its header are still there.
fn targeted(p: &Page, within: u64, payload: &mut Value) {
    let (x, y) = (payload["clientX"].as_f64().unwrap_or(0.0) as f32, payload["clientY"].as_f64().unwrap_or(0.0) as f32);
    let inside = |t: u64| std::iter::successors(Some(t), |i| p.tree.get(*i).and_then(|n| n.parent)).any(|i| i == within);
    let hit = |t: &u64| {
        let r = p.rect(*t);
        x >= r[0] && y >= r[1] && x < r[0] + r[2] && y < r[1] + r[3] && inside(*t)
    };
    if let Some(t) = p.order.iter().rev().copied().find(hit) {
        payload["target"] = json!(t);
    }
}

fn pointer(p: &Page, id: u64, lx: f32, ly: f32, button: u8, detail: u32, m: Mods) -> Value {
    let r = p.rect(id);
    let (cx, cy) = (r[0] + lx, r[1] + ly);
    json!({"x": lx / r[2].max(1.0), "y": ly / r[3].max(1.0), "clientX": cx, "clientY": cy, "button": button, "detail": detail,
        "shiftKey": m.shift, "ctrlKey": m.ctrl, "altKey": m.alt, "metaKey": m.logo})
}

/// A key's payload: DOM's `key` name and the modifiers.
pub fn key_payload(k: &rune::KeyEvent) -> Value {
    use rune::Key;
    let name = match &k.key {
        Key::Text(t) => t.clone(),
        Key::Enter => "Enter".into(),
        Key::Backspace => "Backspace".into(),
        Key::Delete => "Delete".into(),
        Key::Left => "ArrowLeft".into(),
        Key::Right => "ArrowRight".into(),
        Key::Up => "ArrowUp".into(),
        Key::Down => "ArrowDown".into(),
        Key::Home => "Home".into(),
        Key::End => "End".into(),
        Key::PageUp => "PageUp".into(),
        Key::PageDown => "PageDown".into(),
        Key::Escape => "Escape".into(),
        Key::Tab => "Tab".into(),
        other => format!("{other:?}"),
    };
    let m = k.mods;
    json!({"key": name, "shiftKey": m.shift, "ctrlKey": m.ctrl, "altKey": m.alt, "metaKey": m.logo})
}

impl Page {
    /// A press's `ev` at window point `(x, y)` for the nearest element up from `id` that listens, where in it.
    fn send_up(&mut self, id: u64, ev: Ev, x: f32, y: f32, clicks: u32, m: Mods) {
        if let Some(t) = self.up(Some(id), |n| n.on.contains(&ev)) {
            let r = self.rect(t);
            let mut payload = pointer(self, t, x - r[0], y - r[1], 0, clicks, m);
            targeted(self, id, &mut payload);
            self.send(t, ev, payload);
        }
    }

    /// `ev` for element `id`, queued for the JS, if it still listens.
    pub fn send(&mut self, id: u64, ev: Ev, payload: Value) {
        if self.tree.get(id).is_some_and(|n| n.on.contains(&ev)) {
            self.outbox.push(Out::Deliver(event(id, ev, payload).to_string()));
        }
    }
}
