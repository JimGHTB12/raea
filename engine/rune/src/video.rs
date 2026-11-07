//! The session as video (R8a): every frame on the virtual clock, those inside a press or a drag too, drawn offscreen,
//! a cursor painted where the pointer is, and piped as raw RGBA to `raea media encode`. raea's encoder makes the MP4,
//! lays the sound under it and signs it (C2PA) like every raea export; this crate doesn't link it (the goal's rule:
//! raea only through core/wire and runtime/jsengine). Each frame's FNV-1a hash is kept, so two runs can be compared
//! frame for frame: the MP4's bytes differ by the signature's time, the frames don't.

use std::cell::RefCell;
use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::rc::Rc;

use serde_json::{Value, json};

use crate::js::App;
use crate::script::Step;
use crate::session::{FRAME, Session};
use crate::shot::Shooter;

struct Shared {
    stdin: Option<ChildStdin>,
    shooter: Shooter,
    size: (u32, u32),
    hashes: Vec<u64>,
    /// the cursor is drawn once the pointer has first glided in
    cursor: bool,
    /// the arrow's size: a point of it per this many pixels (a 1080-wide view draws it 2.7 times its 400-wide size)
    scale: f32,
    /// each step: its name and the frames its glide starts, its action starts and it ends
    marks: Vec<Value>,
    /// the last frame before its cursor: what `png` writes (a second Shooter would miss glyphs: both renderers read
    /// one atlas, and the first to prepare marks it uploaded)
    last: Vec<u8>,
    /// where the pointer was and how many frames it has stayed there: the cursor fades out when it rests
    rest: ((f32, f32), u32),
    err: Option<String>,
}

/// Frames the pointer rests before its cursor fades, and the frames the fade takes.
const REST: u32 = 50;
const FADE: u32 = 14;

pub struct Video {
    child: Child,
    shared: Rc<RefCell<Shared>>,
}

/// Frames a glide to a step's element takes: 0.33 s at 60 fps.
const GLIDE: u32 = 20;

/// `session`'s frames from now on into `out` at `size` (the viewport), with `audio` under them when given.
pub fn start(session: &mut Session, out: &Path, size: (u32, u32), audio: Option<&Path>) -> Result<Video, String> {
    let raea = std::env::var_os("RAEA_BIN").unwrap_or_else(|| "raea".into());
    let mut cmd = Command::new(&raea);
    cmd.args(["media", "encode", "--size", &format!("{}x{}", size.0, size.1), "--fps", &format!("{}", 1.0 / FRAME), "-o"]).arg(out);
    if let Some(a) = audio {
        cmd.arg("--audio").arg(a);
    }
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().map_err(|e| format!("{}: {e} (raea on PATH, or RAEA_BIN)", raea.to_string_lossy()))?;
    let stdin = child.stdin.take();
    let shared = Rc::new(RefCell::new(Shared { stdin, shooter: Shooter::new(size.0, size.1), size, hashes: Vec::new(), cursor: false, scale: (size.0.min(size.1) as f32 / 400.0).max(1.0), marks: Vec::new(), last: Vec::new(), rest: ((-1.0, -1.0), 0), err: None }));
    let s = shared.clone();
    session.on_frame = Some(Box::new(move |rt, pointer| {
        let mut s = s.borrow_mut();
        if s.err.is_some() {
            return;
        }
        s.last = s.shooter.pixels(rt);
        let mut px = s.last.clone();
        s.rest = if s.rest.0 == pointer { (pointer, s.rest.1 + 1) } else { (pointer, 0) };
        let shown = 1.0 - (s.rest.1.saturating_sub(REST) as f32 / FADE as f32).min(1.0);
        if s.cursor && shown > 0.0 {
            paint_cursor(&mut px, s.size, pointer, s.scale, shown);
        }
        s.hashes.push(fnv(&px));
        if let Some(w) = s.stdin.as_mut()
            && let Err(e) = w.write_all(&px)
        {
            s.err = Some(format!("raea media encode: {e}"));
        }
    }));
    Ok(Video { child, shared })
}

impl Video {
    /// Before `step`: the pointer glides to its element's middle, eased, a frame at a time (a press, a field typed
    /// into, a wheel; a key or a wait has nowhere to go). The first glide starts low in the middle of the view.
    pub fn glide(&self, app: &mut App, step: &Step) -> Result<(), String> {
        let t = match step {
            Step::Press { press } => press,
            Step::Type { into, .. } => into,
            Step::Wheel { wheel, .. } => wheel,
            Step::Hover { hover } => hover,
            Step::Key { .. } | Step::Frames { .. } | Step::Pick { .. } | Step::Save { .. } | Step::Dialog { .. } | Step::Dark { .. } | Step::Width { .. } => return Ok(()),
        };
        let id = app.session.tid(t).ok_or_else(|| format!("no element with data-testid {t:?}"))?;
        let r = app.session.rect(id).ok_or("the element has no box")?;
        let to = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
        let from = {
            let mut s = self.shared.borrow_mut();
            if !s.cursor {
                s.cursor = true;
                let (w, h) = s.size;
                (w as f32 * 0.5, h as f32 * 0.85)
            } else {
                app.session.rt.state.pointer
            }
        };
        for i in 1..=GLIDE {
            let k = i as f32 / GLIDE as f32;
            let k = k * k * (3.0 - 2.0 * k);
            let (x, y) = (from.0 + (to.0 - from.0) * k, from.1 + (to.1 - from.1) * k);
            app.input(|s| s.pointer_move(x, y))?;
        }
        Ok(())
    }

    /// Frames filmed so far: the next frame's index.
    pub fn at(&self) -> usize {
        self.shared.borrow().hashes.len()
    }

    /// The last frame, without its cursor, as a PNG at `path` (the headless run's step stills while filming).
    pub fn png(&self, path: &Path) -> Result<(), String> {
        let s = self.shared.borrow();
        crate::shot::write_png(path, s.size.0, s.size.1, &s.last)
    }

    /// A step's timing kept in video.json (what a sound pass cues on).
    pub fn mark(&self, m: Value) {
        self.shared.borrow_mut().marks.push(m);
    }

    /// The pipe closed and the encoder waited for: its report, the frames' hashes and one hash over them all.
    pub fn finish(self, session: &mut Session) -> Result<Value, String> {
        session.on_frame = None;
        let (hashes, marks, err) = {
            let mut s = self.shared.borrow_mut();
            drop(s.stdin.take());
            (std::mem::take(&mut s.hashes), std::mem::take(&mut s.marks), s.err.take())
        };
        let out = self.child.wait_with_output().map_err(|e| format!("raea media encode: {e}"))?;
        if let Some(e) = err {
            return Err(e);
        }
        if !out.status.success() {
            return Err(format!("raea media encode exited with {}", out.status));
        }
        let encoded: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
        let all = fnv(&hashes.iter().flat_map(|h| h.to_le_bytes()).collect::<Vec<_>>());
        Ok(json!({"encoded": encoded, "frames": hashes.len(), "fps": 1.0 / FRAME, "marks": marks, "hash": format!("{all:016x}"), "frame_hashes": hashes.iter().map(|h| format!("{h:016x}")).collect::<Vec<_>>()}))
    }
}

/// FNV-1a over `b`: stable across runs and machines, no dependency. The same hash as systems/tone's tonegraph.rs `hash`
/// (the host can't depend on systems/): a fix to one goes to the other.
fn fnv(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325, |h, &x| (h ^ x as u64).wrapping_mul(0x100_0000_01b3))
}

/// An arrow pointer, tip at the origin, in points.
const ARROW: [(f32, f32); 7] = [(0.0, 0.0), (0.0, 17.0), (4.2, 13.2), (7.0, 19.6), (9.6, 18.5), (6.9, 12.2), (12.2, 12.2)];

/// The arrow at `at` over `px`, `k` pixels a point, at `alpha`: black, a white edge 1.5 points wide, 4 samples a pixel for its edges.
fn paint_cursor(px: &mut [u8], (w, h): (u32, u32), at: (f32, f32), k: f32, alpha: f32) {
    let (x0, y0) = ((at.0 - 2.0 * k).floor().max(0.0) as u32, (at.1 - 2.0 * k).floor().max(0.0) as u32);
    let (x1, y1) = (((at.0 + 15.0 * k).ceil() as u32).min(w), ((at.1 + 22.0 * k).ceil() as u32).min(h));
    for y in y0..y1 {
        for x in x0..x1 {
            let (mut ink, mut edge) = (0.0f32, 0.0f32);
            for (sx, sy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                let p = ((x as f32 + sx - at.0) / k, (y as f32 + sy - at.1) / k);
                if inside(p) {
                    ink += 0.25 * alpha;
                } else if near(p, 1.5) {
                    edge += 0.25 * alpha;
                }
            }
            if ink + edge == 0.0 {
                continue;
            }
            let i = (y * w + x) as usize * 4;
            for c in 0..3 {
                let under = px[i + c] as f32;
                px[i + c] = (under * (1.0 - ink - edge) + 255.0 * edge).round() as u8;
            }
        }
    }
}

fn inside(p: (f32, f32)) -> bool {
    let mut odd = false;
    for i in 0..ARROW.len() {
        let (a, b) = (ARROW[i], ARROW[(i + 1) % ARROW.len()]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (p.1 - a.1) / (b.1 - a.1) * (b.0 - a.0) {
            odd = !odd;
        }
    }
    odd
}

fn near(p: (f32, f32), r: f32) -> bool {
    (0..ARROW.len()).any(|i| {
        let (a, b) = (ARROW[i], ARROW[(i + 1) % ARROW.len()]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let t = (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        let (ex, ey) = (a.0 + dx * t - p.0, a.1 + dy * t - p.1);
        ex * ex + ey * ey <= r * r
    })
}
