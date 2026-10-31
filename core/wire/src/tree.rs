//! What a renderer's page shares (docs/ts-native.feature P26, P27): a tree of elements the renderer describes each
//! frame (`Want`s), diffed against the last into the wire's ops (created, updated, removed, children put in order), so
//! a frame depends only on its tree. raea's page describes its React tree (runtime/page/src/lib.rs), raea-render's 2D
//! rows describe themselves (runtime/view/src/rows2d.rs); raea-page's Blitz page applies the ops to a Blitz document
//! (engine/page/src/motion.rs). The bevy_ui page that drew them went with docs/blitz-bevy.feature B5d.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde_json::{Map, Value};

use crate::Op;

/// One element of the page this frame. Ids are the renderer's (0 is the page's root); a text node is kind "text" with
/// `text` set.
#[derive(Clone, Debug)]
pub struct Want {
    pub id: u64,
    pub parent: u64,
    pub kind: String,
    pub props: Map<String, Value>,
    pub text: Option<String>,
    /// its place among its parent's elements, from the page's root (`boxes` reports by it)
    pub path: String,
}

/// A video element's picture this frame: its element, file and time in the file.
#[derive(Clone, Debug)]
pub struct VideoAt {
    pub id: u64,
    pub file: PathBuf,
    pub at: f64,
    /// the part of the frame the element shows, when it's framed (<Video framing="face" punchIn>); None: all of it
    pub fit: Option<VideoFit>,
}

/// A framed video's crop (raea-page's `fit_rect`): the frame covers the element's box at `scale` (1 = cover, punchIn
/// multiplies it), placed so the point `at` of the frame (shares of its picture: the eyes) lands on `to` in the box
/// (shares: 0.5, 0.38), as near as the frame still fills the box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VideoFit {
    pub at: (f32, f32),
    pub to: (f32, f32),
    pub scale: f32,
}

/// What the page was last told about an element.
#[derive(Clone, Debug, PartialEq)]
struct Sent {
    kind: String,
    props: Map<String, Value>,
    text: Option<String>,
}

/// What a root was last told: its elements, each parent's children in order, each element's path.
#[derive(Default)]
pub struct Tree {
    sent: HashMap<u64, Sent>,
    kids: HashMap<u64, Vec<u64>>,
    paths: HashMap<u64, String>,
    parents: HashMap<u64, u64>,
    /// each element's place in the last `want` (parents before children, siblings in order: document order)
    seq: HashMap<u64, usize>,
    /// the ids in the last `want` (kept between diffs for its room: ZeroHeap)
    seen: HashSet<u64>,
}

/// `dst` made equal to `src` in place, reusing what it holds: an object's unchanged entries, a string's buffer, a
/// number overwritten (ZeroHeap: an element's style with one value moved costs nothing new)
pub fn sync_value(dst: &mut Value, src: &Value) {
    match (dst, src) {
        (Value::Object(d), Value::Object(s)) => sync_map(d, s),
        (Value::String(d), Value::String(s)) => d.clone_from(s),
        (d, s) => {
            if *d != *s {
                *d = s.clone();
            }
        }
    }
}

/// [`sync_value`] for a map: its keys that went removed, the rest made equal one by one
pub fn sync_map(dst: &mut Map<String, Value>, src: &Map<String, Value>) {
    dst.retain(|k, _| src.contains_key(k));
    for (k, v) in src {
        match dst.get_mut(k) {
            Some(d) => sync_value(d, v),
            None => {
                dst.insert(k.clone(), v.clone());
            }
        }
    }
}

impl Tree {
    /// Each element's path (its place among its parent's elements, from the root), but text elements'.
    pub fn element_paths(&self) -> impl Iterator<Item = (u64, &str)> {
        self.paths.iter().filter(|(id, _)| self.sent.get(id).is_none_or(|s| s.kind != "text")).map(|(id, p)| (*id, p.as_str()))
    }

    /// The ops that make the root match `want` (parents before children) with each parent's children in `order`'s
    /// order, and the ids that went. An element as it was sent costs nothing (ZeroHeap: nothing is cloned for it).
    pub fn diff(&mut self, want: &[Want], order: &HashMap<u64, Vec<u64>>) -> (Vec<Op>, Vec<u64>) {
        let mut ops = Vec::new();
        self.seen.clear();
        self.seen.extend(want.iter().map(|w| w.id));
        let gone: Vec<u64> = self.sent.keys().copied().filter(|id| !self.seen.contains(id)).collect();
        self.seq.clear();
        for (i, w) in want.iter().enumerate() {
            self.seq.insert(w.id, i);
            let sent = || Sent { kind: w.kind.clone(), props: w.props.clone(), text: w.text.clone() };
            match self.sent.get_mut(&w.id) {
                None => {
                    ops.push(Op::Create { id: w.id, kind: w.kind.clone(), props: w.props.clone(), text: w.text.clone() });
                    self.sent.insert(w.id, sent());
                }
                Some(was) if was.kind != w.kind => {
                    // another kind under the same id: made again
                    ops.push(Op::Remove { id: w.id });
                    ops.push(Op::Create { id: w.id, kind: w.kind.clone(), props: w.props.clone(), text: w.text.clone() });
                    *was = sent();
                    self.kids.remove(&w.parent);
                }
                Some(was) if was.props != w.props || was.text != w.text => {
                    let set: Map<String, Value> = w.props.iter().filter(|(k, v)| was.props.get(*k) != Some(*v)).map(|(k, v)| (k.clone(), v.clone())).collect();
                    let unset: Vec<String> = was.props.keys().filter(|k| !w.props.contains_key(*k)).cloned().collect();
                    let text = (was.text != w.text).then(|| w.text.clone().unwrap_or_default());
                    // what was sent brought up to date in place: only what changed is copied, not the element
                    sync_map(&mut was.props, &w.props);
                    if text.is_some() {
                        was.text.clone_from(&w.text);
                    }
                    ops.push(Op::Update { id: w.id, set: (!set.is_empty()).then_some(set), unset: (!unset.is_empty()).then_some(unset), text });
                }
                Some(_) => {}
            }
            if self.paths.get(&w.id) != Some(&w.path) {
                self.paths.insert(w.id, w.path.clone());
            }
            self.parents.insert(w.id, w.parent);
        }
        for (parent, list) in order {
            if self.kids.get(parent) != Some(list) {
                ops.extend(list.iter().map(|c| Op::Append { parent: *parent, child: *c }));
                match self.kids.get_mut(parent) {
                    Some(k) => k.clone_from(list),
                    None => {
                        self.kids.insert(*parent, list.clone());
                    }
                }
            }
        }
        self.kids.retain(|k, _| order.contains_key(k));
        for id in &gone {
            ops.push(Op::Remove { id: *id });
            self.sent.remove(id);
            self.paths.remove(id);
            self.parents.remove(id);
        }
        (ops, gone)
    }
}
