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
/// a page sample where the page painted nothing (the 3D's shows there)
pub const NONE: u16 = u16::MAX;

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
    /// each recorded frame's page mask, as runs (NONE where the page painted nothing)
    masks: BTreeMap<i64, Vec<(u16, u16)>>,
    /// each 3D frame's mask (its width, its runs): what shows where the page painted nothing
    three: BTreeMap<i64, (usize, Vec<(u16, u16)>)>,
    /// the composition's folder, where the keys' files are: a use's root is named after its component from there
    src: Option<PathBuf>,
}

impl Recorder {
    pub fn new(path: PathBuf, video: String) -> Recorder {
        Recorder { path, video, size: (0, 0), layers: Vec::new(), seen: Vec::new(), by_key: HashMap::new(), masks: BTreeMap::new(), three: BTreeMap::new(), src: None }
    }

    /// Layers at the root of a use of the edit's components named after the component (`Badge` for the element
    /// `<Badge />` drew outermost), from the sources in `dir`.
    pub fn named_from(mut self, dir: PathBuf) -> Recorder {
        self.src = Some(dir);
        self
    }

    /// The layer as the table lists it: an element or mesh that is the outermost of a use (its parent drawn outside
    /// that use) is named after the component used.
    fn named(&self, l: &Layer) -> Layer {
        let mut l = l.clone();
        let parent_uses = l.parent.as_ref().and_then(|p| self.by_key.get(p)).map(|i| &self.layers[*i as usize - 1].uses);
        let root = l.uses.first().filter(|_| matches!(l.kind, "element" | "mesh") && parent_uses != Some(&l.uses));
        if let Some(name) = root.zip(self.src.as_ref()).and_then(|(u, dir)| crate::names::component_at(dir, u)) {
            l.name = name;
        } else if let Some(name) = self.src.as_ref().filter(|_| l.kind == "mesh" && l.name == "mesh").and_then(|dir| crate::names::component_at(dir, &l.loc)) {
            // an unnamed part: the part it is ("Mesh3d", "Model")
            l.name = name;
        }
        l
    }

    /// Frame `f`'s page: `px` (`w` x `h`, row-major) each pixel's index into `layers` plus one (0: in no layer, NONE:
    /// nothing painted). A frame recorded again replaces what it had (a page drawn twice in a frame). The frames the
    /// page is recorded at are the file's.
    pub fn frame(&mut self, f: i64, w: u32, h: u32, px: &[u16], layers: &[Layer]) {
        self.size = (w, h);
        let global: Vec<u16> = std::iter::once(0).chain(layers.iter().map(|l| self.index(l, f))).collect();
        let (mw, mh) = (w.div_ceil(SCALE), h.div_ceil(SCALE));
        let at = (0..mh).flat_map(|y| (0..mw).map(move |x| ((y * SCALE + SCALE / 2).min(h - 1), (x * SCALE + SCALE / 2).min(w - 1))));
        let mask = at.map(|(fy, fx)| match px.get((fy * w + fx) as usize) {
            Some(&NONE) | None => NONE,
            Some(v) => global.get(*v as usize).copied().unwrap_or(0),
        });
        self.masks.insert(f, runs(mask));
    }

    /// Frame `f`'s 3D, drawn to a target `w` px wide (the frame its top left): `mask` (its quarter, row-major, as
    /// `raster` gives it) each sample's index into `layers` plus one, 0 for none. It shows where the page painted
    /// nothing.
    pub fn frame_3d(&mut self, f: i64, w: u32, mask: &[u16], layers: &[Layer]) {
        let global: Vec<u16> = std::iter::once(0).chain(layers.iter().map(|l| self.index(l, f))).collect();
        self.three.insert(f, (w.div_ceil(SCALE) as usize, runs(mask.iter().map(|v| global.get(*v as usize).copied().unwrap_or(0)))));
    }

    /// A layer's value in the file (its place in the table plus one), added at its first frame.
    fn index(&mut self, l: &Layer, f: i64) -> u16 {
        let i = match self.by_key.get(&l.key) {
            Some(i) => *i,
            None => {
                let l = self.named(l);
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

    /// Frame `f`'s mask as the file has it: the page's, the 3D's where the page painted nothing.
    fn composite(&self, f: i64) -> Vec<(u16, u16)> {
        let page = expand(&self.masks[&f]);
        let (tw, three) = self.three.get(&f).map(|(tw, r)| (*tw, expand(r))).unwrap_or_default();
        let mw = self.size.0.div_ceil(SCALE) as usize;
        let under = |i: usize| Some((i % mw, i / mw)).filter(|(x, _)| *x < tw).and_then(|(x, y)| three.get(y * tw + x)).copied().unwrap_or(0);
        let over = crate::mutant("under");
        runs(page.iter().enumerate().map(|(i, v)| if *v == NONE || (over && under(i) != 0) { under(i) } else { *v }))
    }

    /// The header, as NAME.layers gives it.
    pub fn header(&self, fps: f64) -> serde_json::Value {
        let (w, h) = self.size;
        let keys: Vec<&str> = self.layers.iter().map(|l| l.key.as_str()).collect();
        let key = |i: usize| if crate::mutant("swap") && i < 2 && keys.len() > 1 { keys[1 - i] } else { keys[i] };
        let layers: Vec<_> = self.layers.iter().zip(&self.seen).enumerate().map(|(i, (l, s))| {
            let mut v = json!({"key": key(i), "kind": l.kind, "name": l.name, "parent": l.parent, "loc": l.loc, "use": l.uses, "index": l.index, "frames": s});
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
        let mut before: Option<Vec<(u16, u16)>> = None;
        for f in self.masks.keys() {
            let runs = self.composite(*f);
            if before.as_ref() == Some(&runs) {
                out.extend(0u32.to_le_bytes());
                continue;
            }
            out.extend((runs.len() as u32).to_le_bytes());
            for (v, n) in &runs {
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

/// Values as runs of one value, none past 65535 long.
fn runs(vals: impl Iterator<Item = u16>) -> Vec<(u16, u16)> {
    let mut out: Vec<(u16, u16)> = Vec::new();
    for v in vals {
        match out.last_mut() {
            Some((lv, n)) if *lv == v && *n < u16::MAX => *n += 1,
            _ => out.push((v, 1)),
        }
    }
    out
}

fn expand(runs: &[(u16, u16)]) -> Vec<u16> {
    runs.iter().flat_map(|(v, n)| std::iter::repeat_n(*v, *n as usize)).collect()
}
