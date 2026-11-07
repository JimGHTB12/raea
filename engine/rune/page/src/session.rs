//! A page run by Rune frame by frame, headless (R3, R4): input in through Rune's own pointer, wheel and key paths (so
//! hit-testing, text editing and the page's press rules in events.rs run as they do in a window), and out the messages
//! the JS needs: each element event the page's handlers queued, and after each frame whose layout changed, the layout
//! snapshot (layout.rs).

use rune::{Key, KeyEvent, Mods, Runtime};
use serde_json::{Value, json};

use raea_core_wire::{Ev, named};

use crate::page::{Out, Page, headless};
use crate::view::key;

pub struct Session {
    pub rt: Runtime<Page>,
    pub(crate) viewport: (f32, f32),
    clock: f64,
    frames: u64,
    /// run at the end of every frame with the runtime and the pointer (R8a's encoder: each virtual-clock frame,
    /// those inside a press or a drag too); none by default
    pub on_frame: Option<Box<dyn FnMut(&mut Runtime<Page>, (f32, f32))>>,
}

/// One frame at 60 fps.
pub const FRAME: f64 = 1.0 / 60.0;

impl Session {
    pub fn new(page: Page, viewport: (f32, f32)) -> Self {
        let mut s = Session { rt: headless(page, viewport), viewport, clock: 0.0, frames: 0, on_frame: None };
        s.frame();
        s
    }

    /// A frame: the clock moves on a frame, the view runs if anything changed, layout and motion advance, rects and the
    /// snapshot follow, and a ticking page hears `frame`.
    pub fn frame(&mut self) {
        let dt = if self.frames == 0 { 0.0 } else { FRAME };
        self.clock += dt;
        self.step(dt);
        // every frame reaches the page's `__ui_layout`, "" when its layout didn't change, as the page host's does
        // (runtime/dom elements.rs): it's where @raea/ui runs a frame's requestAnimationFrame callbacks, so a JS
        // animation that only moves transforms or opacity (Motion's) keeps its frames (T3c)
        if !self.rt.state.outbox.iter().any(|o| matches!(o, Out::Layout(_))) {
            self.rt.state.outbox.push(Out::Layout(String::new()));
        }
        self.frames += 1;
        if self.rt.state.ticking {
            self.rt.state.outbox.push(Out::Deliver(named("frame", json!(self.frames)).to_string()));
        }
        if let Some(f) = self.on_frame.as_mut() {
            let p = self.rt.state.pointer;
            f(&mut self.rt, p);
        }
    }

    /// The virtual clock, in seconds: a frame's worth per frame since the first.
    pub fn clock(&self) -> f64 {
        self.clock
    }

    /// Frames run so far.
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Rune's frame over `dt` seconds, then the page takes in its layout.
    fn step(&mut self, dt: f64) {
        self.rt.frame(dt);
        let m = crate::layout::measure(&self.rt.elements(), &self.rt.state.boxes.borrow(), self.viewport);
        self.rt.state.measured(m);
    }

    /// A batch from the JS, laid out at once (the clock stays: a batch answers what happened in this frame, and a
    /// ticking page that renders on every `frame` mustn't make another).
    pub fn apply(&mut self, batch: &str) -> Result<(), String> {
        // through `update`, so Rune knows the state changed and runs the view again. It animates only where the page
        // asked for Rune's motion: view.rs lands every other subtree at once and shows each element whole when it mounts
        let mut r = Ok(());
        self.rt.update(|p, _| r = p.apply(batch));
        r?;
        self.step(0.0);
        Ok(())
    }

    /// What the page asked of the host by `__ui_emit` (input.rs), a frame after it when it moved the focus.
    pub fn emit(&mut self, name: &str, v: &Value) {
        let before = self.rt.state.focus();
        self.rt.update(|p, tx| p.emit(name, v, tx));
        if self.rt.state.focus() != before {
            self.frame();
        }
    }

    /// Element `id`'s box after the last frame.
    pub fn rect(&self, id: u64) -> Option<[f32; 4]> {
        self.rt.state.rects.get(&id).copied()
    }

    /// The element whose `data-testid` is `t`.
    pub fn tid(&self, t: &str) -> Option<u64> {
        self.rt.state.tid(t)
    }

    /// The wire element on top at `(x, y)` after the last frame.
    pub fn at(&self, x: f32, y: f32) -> Option<u64> {
        self.rt.state.at(x, y)
    }

    /// The pointer to `(x, y)`.
    pub fn pointer_move(&mut self, x: f32, y: f32) {
        self.rt.state.pointer = (x, y);
        self.rt.pointer_move(x, y);
        self.frame();
    }

    /// A press and release at `(x, y)`, a frame between.
    pub fn click(&mut self, x: f32, y: f32) {
        self.pointer_move(x, y);
        self.rt.pointer_down(x, y);
        self.frame();
        self.rt.pointer_up(x, y);
        self.frame();
        self.follow_field_focus();
    }

    /// A press at `from`, the pointer moved to `to` in `steps` frames, and a release there.
    pub fn drag(&mut self, from: (f32, f32), to: (f32, f32), steps: u32) {
        self.pointer_move(from.0, from.1);
        self.rt.pointer_down(from.0, from.1);
        self.frame();
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            self.pointer_move(from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
        }
        self.rt.pointer_up(to.0, to.1);
        self.frame();
        self.follow_field_focus();
    }

    /// A press and release in the middle of element `id`.
    pub fn click_el(&mut self, id: u64) -> Option<()> {
        let r = self.rect(id)?;
        self.click(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
        Some(())
    }

    /// A wheel at `(x, y)`: `dy` points, positive scrolling the content up (DOM's sign).
    pub fn wheel(&mut self, x: f32, y: f32, dy: f32) {
        self.pointer_move(x, y);
        self.rt.wheel_xy(x, y, 0.0, -dy);
        self.frame();
    }

    /// Text typed at the focus, a character at a time, a frame after each.
    pub fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            self.key(Key::Text(c.to_string()));
        }
    }

    /// A key at the focus, as raea's other host sends it: `keydown` (input.rs's `key_out`), Rune's own handling (a
    /// field's edit, which sends its `change`; Tab's focus move), then `keyup`.
    pub fn key(&mut self, k: Key) {
        let ev = KeyEvent { key: k, mods: Mods::default() };
        let payload = crate::events::key_payload(&ev);
        if self.rt.state.key_out(Ev::KeyDown, payload.clone()) {
            // Tab is the page's sequential focus (input.rs), Rune's own only where the page has nowhere to go
            let mut moved = false;
            if ev.key == Key::Tab {
                self.rt.update(|p, tx| moved = p.tab(ev.mods.shift, tx));
            }
            if !moved {
                self.rt.key(ev.clone());
            }
        }
        self.frame();
        self.follow_field_focus();
        self.rt.key_up(ev);
        self.rt.state.key_out(Ev::KeyUp, payload);
        self.frame();
    }

    /// A field Rune focused itself is the page's focus too, with a frame for it when it moved.
    fn follow_field_focus(&mut self) {
        if self.rt.state.focused_field().is_some_and(|f| Some(f) != self.rt.state.focus()) {
            self.rt.update(|p, tx| p.follow_field_focus(tx));
            self.frame();
        }
    }

    /// The text field with the focus, by wire id.
    pub fn focused_field(&self) -> Option<u64> {
        self.rt.state.focused_field()
    }

    /// A new viewport: Rune lays out to it, and the JS hears `resize`.
    pub fn resize(&mut self, w: f32, h: f32) {
        self.viewport = (w, h);
        self.rt.set_viewport((w, h), 1.0);
        // through `update`: a breakpoint's classes restyle with the width (T3b)
        self.rt.update(|p, _| p.resized(w, h));
        self.frame();
    }

    /// The viewport in points.
    pub fn viewport(&self) -> (f32, f32) {
        self.viewport
    }

    /// The system's appearance (Tailwind's `dark:`): Rune's environment, as a window has it from the system.
    pub fn set_dark(&mut self, dark: bool) {
        self.rt.set_system_dark(dark);
        self.frame();
    }

    /// The pointer moved to the middle of element `id` and left there (hover).
    pub fn hover_el(&mut self, id: u64) -> Option<()> {
        let r = self.rect(id)?;
        self.pointer_move(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
        // Rune's hover change lands in this frame; the restyle it asks for, in the next
        self.frame();
        Some(())
    }

    /// What the JS hasn't been sent, taken.
    pub fn take_out(&mut self) -> Vec<Out> {
        self.rt.state.take_out()
    }

    /// The Rune element key for wire id `id` (for reading `rt.elements()`).
    pub fn key_of(id: u64) -> String {
        key(id)
    }
}

pub use crate::layout::shown;
