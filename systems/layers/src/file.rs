//! NAME.layers (v1, agreed with apps/app's player): one gzip stream of "RAEALAYR", a u32 LE header length, the
//! header's JSON, then each frame as a u32 LE run count N and N (u16 value, u16 count) runs over its quarter-size mask,
//! row-major (N = 0: the frame before's mask). Value 0 is no layer, v is the header's layers[v - 1]. A run never
//! passes 65535. Mask px (x, y) is frame px (4x + 2, 4y + 2).

use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::PathBuf;

use serde_json::json;

use crate::Layer;

pub const SCALE: u32 = 4;

pub struct Recorder {
    path: PathBuf,
    /// the export's file name
    video: String,
    /// the frame's size
    size: (u32, u32),
    layers: Vec<Layer>,
    /// each layer's first and last frame
    seen: Vec<[i64; 2]>,
    by_key: HashMap<String, u16>,
    /// each recorded frame's mask, as runs
    masks: BTreeMap<i64, Vec<(u16, u16)>>,
}

impl Recorder {
    pub fn new(path: PathBuf, video: String) -> Recorder {
        Recorder { path, video, size: (0, 0), layers: Vec::new(), seen: Vec::new(), by_key: HashMap::new(), masks: BTreeMap::new() }
    }

    /// Frame `f`'s layers: `px` (`w` x `h`, row-major) each pixel's index into `layers` plus one (0: none). A frame
    /// recorded again replaces what it had (a page drawn twice in a frame).
    pub fn frame(&mut self, f: i64, w: u32, h: u32, px: &[u16], layers: &[Layer]) {
        self.size = (w, h);
        let global: Vec<u16> = std::iter::once(0).chain(layers.iter().map(|l| self.index(l, f))).collect();
        let (mw, mh) = (w.div_ceil(SCALE), h.div_ceil(SCALE));
        let mut runs: Vec<(u16, u16)> = Vec::new();
        for y in 0..mh {
            let fy = (y * SCALE + SCALE / 2).min(h - 1);
            for x in 0..mw {
                let fx = (x * SCALE + SCALE / 2).min(w - 1);
                let v = px.get((fy * w + fx) as usize).and_then(|v| global.get(*v as usize)).copied().unwrap_or(0);
                match runs.last_mut() {
                    Some((lv, n)) if *lv == v && *n < u16::MAX => *n += 1,
                    _ => runs.push((v, 1)),
                }
            }
        }
        self.masks.insert(f, runs);
    }

    /// A layer's value in the file (its place in the table plus one), added at its first frame.
    fn index(&mut self, l: &Layer, f: i64) -> u16 {
        let i = match self.by_key.get(&l.key) {
            Some(i) => *i,
            None => {
                self.layers.push(l.clone());
                self.seen.push([f, f]);
                let i = self.layers.len() as u16;
                self.by_key.insert(l.key.clone(), i);
                i
            }
        };
        let s = &mut self.seen[i as usize - 1];
        *s = [s[0].min(f), s[1].max(f)];
        i
    }

    /// The header, as NAME.layers gives it.
    pub fn header(&self, fps: f64) -> serde_json::Value {
        let (w, h) = self.size;
        let layers: Vec<_> = self.layers.iter().zip(&self.seen).map(|(l, s)| {
            let mut v = json!({"key": l.key, "kind": l.kind, "name": l.name, "parent": l.parent, "loc": l.loc, "use": l.uses, "index": l.index, "frames": s});
            if let Some(src) = &l.src {
                v["src"] = json!(src);
            }
            v
        }).collect();
        let mut head = json!({"version": 1, "video": self.video, "fps": fps, "frames": self.masks.len(), "width": w, "height": h,
            "mask": {"w": w.div_ceil(SCALE), "h": h.div_ceil(SCALE), "scale": SCALE}, "layers": layers});
        // frames not from 0 on, one after another (a render's --at): which they are
        let at: Vec<i64> = self.masks.keys().copied().collect();
        if at.iter().enumerate().any(|(i, f)| *f != i as i64) {
            head["at"] = json!(at);
        }
        head
    }

    /// The file's bytes, before gzip.
    pub fn bytes(&self, fps: f64) -> Vec<u8> {
        let head = self.header(fps).to_string();
        let mut out = b"RAEALAYR".to_vec();
        out.extend((head.len() as u32).to_le_bytes());
        out.extend(head.as_bytes());
        let mut before: Option<&Vec<(u16, u16)>> = None;
        for runs in self.masks.values() {
            if before == Some(runs) {
                out.extend(0u32.to_le_bytes());
                continue;
            }
            out.extend((runs.len() as u32).to_le_bytes());
            for (v, n) in runs {
                out.extend(v.to_le_bytes());
                out.extend(n.to_le_bytes());
            }
            before = Some(runs);
        }
        out
    }

    /// Written to its path: the file's name. None when no frame was recorded.
    pub fn write(self, fps: f64) -> Result<Option<String>, String> {
        if self.masks.is_empty() {
            return Ok(None);
        }
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gz.write_all(&self.bytes(fps)).and_then(|_| gz.finish().and_then(|b| std::fs::write(&self.path, b))).map_err(|e| format!("{}: {e}", self.path.display()))?;
        Ok(self.path.file_name().map(|n| n.to_string_lossy().into_owned()))
    }
}
