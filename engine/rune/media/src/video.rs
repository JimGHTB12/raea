//! `<video>` (T3d) through Rune's video surface: a Rune `Player` per element, made from its source (a file, or a URL's
//! cached file headless so a run decodes the same frames every time; a window streams the URL), kept while the
//! element is drawn and dropped when it isn't. The element's props are the player's intent: `autoPlay` plays it,
//! `loop` and `muted` set it, and @raea/ui's `__seek` (its currentTime set: a time and a count) seeks it. Headless,
//! Rune decodes the frame at the virtual clock's time, synchronously.
//!
//! The news @raea/ui's HTMLMediaElement hears, as the web host's (runtime/dom media.rs): `loadedmetadata` with the
//! size and duration, then `loadeddata`, `seeked` after a seek, `ended`, and `error`.

use std::cell::RefCell;
use std::collections::HashMap;

use rune::{Player, PlayerState};
use serde_json::{Map, Value, json};

use crate::src::{self, Src};

struct One {
    src: String,
    player: Result<Player, String>,
    /// the `__seek` count last applied
    seeks: u64,
    /// what @raea/ui was last told: 0 nothing, 1 loadedmetadata, 2 loadeddata
    told: u8,
    ended: bool,
    seeked: Option<f64>,
    /// drawn in the last view
    seen: bool,
}

#[derive(Default)]
pub struct Videos {
    by_id: RefCell<HashMap<u64, One>>,
    /// a window's: stream URLs rather than fetch them first
    pub window: bool,
}

impl Videos {
    /// Element `id`'s player for its props (made, or remade when its source changed), with the props applied.
    pub fn player(&self, id: u64, props: &Map<String, Value>) -> Result<Player, String> {
        let s = props.get("src").and_then(Value::as_str).unwrap_or("");
        let mut all = self.by_id.borrow_mut();
        let one = all.entry(id).or_insert_with(|| One { src: String::new(), player: Err(String::new()), seeks: 0, told: 0, ended: false, seeked: None, seen: false });
        if one.src != s || one.player.as_ref().is_err_and(String::is_empty) {
            *one = One { src: s.to_string(), player: self.open(s), seeks: 0, told: 0, ended: false, seeked: None, seen: true };
            if let (Ok(p), true) = (&one.player, flag(props, "autoplay") || flag(props, "autoPlay")) {
                p.play();
            }
        }
        one.seen = true;
        let p = one.player.clone()?;
        p.set_looping(flag(props, "loop"));
        p.set_muted(flag(props, "muted"));
        if let Some([t, n]) = props.get("__seek").and_then(Value::as_array).and_then(|a| Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?]))
            && n as u64 != one.seeks
        {
            one.seeks = n as u64;
            p.seek(t);
            one.seeked = Some(t);
        }
        Ok(p)
    }

    fn open(&self, s: &str) -> Result<Player, String> {
        Ok(match src::resolve(s, src::base())? {
            Src::File(p) if p.exists() => Player::file(p),
            Src::File(p) => return Err(format!("{}: no such file", p.display())),
            Src::Url(u) if self.window => Player::url(u),
            Src::Url(u) => Player::file(raea_io_net::cache::fetch(&u)?),
            Src::Bytes(_) => return Err("a data URL video (the host plays files and URLs)".into()),
        })
    }

    /// Before a view: every element is unseen until the view draws it.
    pub fn unsee(&self) {
        self.by_id.borrow_mut().values_mut().for_each(|o| o.seen = false);
    }

    /// After a frame: the players of elements no longer drawn dropped, and each one's news since the last frame,
    /// by element id.
    pub fn news(&self) -> Vec<(u64, Value)> {
        let mut out = Vec::new();
        let mut all = self.by_id.borrow_mut();
        all.retain(|_, o| o.seen);
        let mut ids: Vec<u64> = all.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            let o = all.get_mut(&id).unwrap();
            let p = match &o.player {
                Ok(p) => p,
                Err(e) if o.told == 0 => {
                    o.told = 3;
                    out.push((id, json!({"type": "error", "message": e})));
                    continue;
                }
                Err(_) => continue,
            };
            let state = p.state();
            if let PlayerState::Failed(e) = &state {
                if o.told < 3 {
                    o.told = 3;
                    out.push((id, json!({"type": "error", "message": e})));
                }
                continue;
            }
            let (size, duration) = (p.natural_size(), p.duration());
            if o.told == 0 && let (Some((w, h)), Some(d)) = (size, duration) {
                o.told = 1;
                out.push((id, json!({"type": "loadedmetadata", "w": w, "h": h, "duration": d})));
            }
            if o.told == 1 && !matches!(state, PlayerState::Loading) {
                o.told = 2;
                let (w, h) = size.unwrap_or_default();
                out.push((id, json!({"type": "loadeddata", "w": w, "h": h, "duration": duration})));
            }
            if o.told >= 2 && let Some(t) = o.seeked.take() {
                out.push((id, json!({"type": "seeked", "time": t})));
            }
            let ended = matches!(state, PlayerState::Ended);
            if ended && !o.ended {
                out.push((id, json!({"type": "ended", "time": p.time()})));
            }
            o.ended = ended;
        }
        out
    }
}

/// A boolean attribute as React passes it: present and not false.
fn flag(props: &Map<String, Value>, k: &str) -> bool {
    !matches!(props.get(k), None | Some(Value::Null | Value::Bool(false)))
}
