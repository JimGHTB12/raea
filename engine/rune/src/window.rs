//! A TSX app in a native window through Rune's shell (R7a): Rune's `App` owns the window, the GPU, the event loop and
//! the page (`Page`, its state); the JS engine lives beside it on the main thread. Every frame starts with `poll`,
//! which takes in the last frame's layout (layout.rs), moves the JS on to real time (its frame number, its timers due)
//! and pumps it against the page. Input is Rune's: the page's handlers (events.rs, input.rs) do in the window exactly
//! what they do headless. A frame whose layout changed wakes the loop for one more poll, so the JS hears it.

use std::cell::RefCell;
use std::time::Duration;

use raea_core_wire::{Ev, named};
use rune::{App, AppSpec, KeyEvent, Tx, Waker};
use serde_json::{Value, json};

use crate::js::{Host, Js};
use crate::layout::Measured;
use crate::page::{Out, Page, view};

/// The engine, its bundle until the first poll runs it, and what the window's hooks hand the next poll.
#[derive(Default)]
struct Beside {
    js: Option<Js>,
    bundle: Option<String>,
    measured: Option<Measured>,
    viewport: (f32, f32),
    /// the viewport the page last heard of
    heard: (f32, f32),
    waker: Option<Waker>,
    frames: u64,
    error: Option<String>,
}

thread_local! {
    static BESIDE: RefCell<Beside> = RefCell::default();
}

/// The page in the window, as the JS's host for one poll.
struct Win<'a> {
    page: &'a mut Page,
    tx: &'a mut Tx,
    clock: f64,
    changed: bool,
}

impl Host for Win<'_> {
    fn clock(&self) -> f64 {
        self.clock
    }
    fn apply(&mut self, batch: &str) -> Result<(), String> {
        self.changed = true;
        self.page.apply(batch)
    }
    fn emit(&mut self, name: &str, v: &Value) {
        self.changed = true;
        self.page.emit(name, v, self.tx);
    }
    fn take_out(&mut self) -> Vec<Out> {
        self.page.take_out()
    }
}

/// `bundle` in a `size` window titled `title`, until it closes. Errors from the JS end the app and come back here.
pub fn run(bundle: String, title: &str, size: (f32, f32)) -> Result<(), String> {
    begin(bundle, size)?;
    // the window can shrink below Rune's default floor (420x400) when the app asks for less
    let min_size = Some((size.0.min(420.0), size.1.min(400.0)));
    // the app's faces, registered at the start (raea-rune-media's fonts: Rune takes them only then)
    let fonts = raea_rune_media::fonts::registered();
    let spec = AppSpec { setup: Some(setup), poll: Some(poll), on_key: Some(on_key), min_size, fonts, ..AppSpec::new(Box::leak(title.to_string().into_boxed_str()), size, Page::default, view) };
    App::new(spec).on_frame_drawn(drawn).run();
    error(false).map_or(Ok(()), Err)
}

/// The engine made and the bundle kept for the first poll, at `size`: what any window's host does first (Rune's shell
/// here, Bevy's in on_bevy/window.rs).
pub(crate) fn begin(bundle: String, size: (f32, f32)) -> Result<(), String> {
    BESIDE.with_borrow_mut(|b| {
        b.js = Some(Js::new()?);
        b.bundle = Some(bundle);
        b.viewport = size;
        b.heard = size;
        Ok::<_, String>(())
    })
}

/// The error that ended the app (the JS's): taken once by the host that ends on it, or `peek`ed by a host without
/// Rune's `tx.quit()` (Bevy's), which ends when there is one.
pub(crate) fn error(peek: bool) -> Option<String> {
    BESIDE.with_borrow_mut(|b| if peek { b.error.clone() } else { b.error.take() })
}

pub(crate) fn setup(page: &mut Page, w: Waker) {
    // a window's media: a decode or a URL's fetch wakes it, and a video streams its URL (T3d)
    page.images = crate::images::Images::new(w.clone());
    page.videos.window = true;
    BESIDE.with_borrow_mut(|b| b.waker = Some(w));
}

/// After each frame: its layout, for the next poll, and a wake for that poll when it changed.
pub(crate) fn drawn(rt: &rune::Runtime<Page>) {
    // what a handler queued in this frame (a press, an edit, a hover) waits for the JS: a poll now
    if !rt.state.outbox.is_empty() {
        wake();
    }
    let m = crate::layout::measure(&rt.elements(), &rt.state.boxes.borrow(), rt.viewport);
    if m.snap == rt.state.last_layout {
        return;
    }
    BESIDE.with_borrow_mut(|b| {
        b.measured = Some(m);
        b.viewport = rt.viewport;
    });
    wake();
}

/// A poll soon, from outside one (the loop's waker).
fn wake() {
    if let Some(w) = BESIDE.with_borrow(|b| b.waker.clone()) {
        w();
    }
}

/// The start of a frame, `t` seconds in: the layout taken in, a field Rune focused followed, and the JS moved on.
pub(crate) fn poll(page: &mut Page, t: f64, tx: &mut Tx) -> bool {
    let (measured, resized) = BESIDE.with_borrow_mut(|b| {
        let resized = (b.viewport != b.heard).then_some(b.viewport);
        b.heard = b.viewport;
        (b.measured.take(), resized)
    });
    let focus = page.focus();
    if let Some((w, h)) = resized {
        page.resized(w, h);
    }
    // every frame reaches the page's `__ui_layout` ("" when unchanged), where @raea/ui runs rAF callbacks (T3c)
    if !measured.is_some_and(|m| page.measured(m)) {
        page.outbox.push(Out::Layout(String::new()));
    }
    page.follow_field_focus(tx);
    page.media_news();
    if page.ticking {
        let n = BESIDE.with_borrow(|b| b.frames);
        page.outbox.push(Out::Deliver(named("frame", json!(n)).to_string()));
    }
    let (mut js, bundle, n) = BESIDE.with_borrow_mut(|b| {
        b.frames += 1;
        (b.js.take(), b.bundle.take(), b.frames)
    });
    let Some(engine) = js.as_mut() else { return false };
    let mut win = Win { page, tx, clock: t, changed: false };
    let next = match bundle {
        Some(src) => engine.run(&src, &mut win).and_then(|()| engine.timers()),
        None => engine.frame(n, &mut win),
    };
    // a new width restyles what a breakpoint is on (T3b), whether or not the JS draws again
    let changed = win.changed || win.page.focus() != focus || resized.is_some();
    let ticking = win.page.ticking;
    let wrote = win.changed;
    match next {
        Ok(next) => {
            if let Some(ms) = next {
                tx.wake_after(Duration::from_secs_f64(ms / 1e3));
            }
            // a page that wrote this frame gets the next one: a rAF animation (Motion's) writes every frame it moves
            if ticking || wrote {
                tx.wake_after(Duration::from_millis(16));
            }
        }
        Err(e) => {
            eprintln!("raea-rune-host: {e}");
            BESIDE.with_borrow_mut(|b| b.error = Some(e));
            tx.quit();
        }
    }
    BESIDE.with_borrow_mut(|b| b.js = js);
    changed
}

/// A key Rune's focused control didn't take (or any key while nothing has Rune's focus): the page's keydown, and
/// whether the page took it from Rune (a menu's Enter, Space or Escape).
pub(crate) fn on_key(page: &mut Page, k: &KeyEvent, tx: &mut Tx) -> bool {
    let mut taken = !page.key_out(Ev::KeyDown, crate::events::key_payload(k));
    // Tab is the page's sequential focus (input.rs)
    if !taken && k.key == rune::Key::Tab {
        taken = page.tab(k.mods.shift, tx);
    }
    wake();
    taken
}
