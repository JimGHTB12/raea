//! A composition's 3D as data: the elements read from its React tree (Node3), each part placed in the world (Item),
//! parts matched across frames by identity (tracks), the props they carry, and the files they name. The scene's rows,
//! its contact and its light are all read from these.

use std::collections::BTreeMap;

use bevy::math::Mat4;
use serde_json::{Map, Value};

mod media;
mod parts;
mod props;
pub use media::*;
pub use parts::*;
pub use props::*;

/// A 3D element and its children.
#[derive(Clone, Debug)]
pub struct Node3 {
    /// kind and shape shared too, kept per part with its props (ZeroHeap: each was copied into a String a frame)
    pub kind: std::sync::Arc<str>,
    /// shared: a part's props are parsed once per change of its data-props and handed to each frame's scene, walk and
    /// rows by reference (ZeroHeap: a live 3D frame parsed and copied every part's props)
    pub props: std::sync::Arc<Map<String, Value>>,
    pub shape: Option<std::sync::Arc<str>>,
    /// shared: a subtree no commit touched is the last frame's, handed on for an Arc's count (docs/control.feature
    /// FC1b: many_cubes' 10,000 parts were re-read each frame for a camera's move). Build from a Vec with `.into()`
    pub kids: std::sync::Arc<[Node3]>,
}

/// A part of a frame's 3D, placed: its kind, props and shape, a material child, and its world transform.
#[derive(Clone)]
pub struct Item {
    pub kind: &'static str,
    pub props: std::sync::Arc<Map<String, Value>>,
    pub shape: Option<std::sync::Arc<str>>,
    pub material: Option<Node3>,
    pub world: Mat4,
}

/// What a part is called across frames: its name, else what it draws, counted in tree order.
pub fn base_ident(it: &Item) -> &str {
    if let Some(Value::String(n)) = it.props.get("name") {
        return n;
    }
    match it.kind {
        "mesh" => it.shape.as_deref().unwrap_or_default(),
        "model" | "panel" | "plate" | "splat" | "meshseq" => it.props.get("src").and_then(Value::as_str).unwrap_or(""),
        "text" => it.props.get("text").and_then(Value::as_str).unwrap_or(""),
        _ => "",
    }
}

/// `tracks` over items already walked: only the parts of these kinds are copied
pub fn tracks_of(all: &[Vec<Item>], kinds: &[&str]) -> Vec<(String, Vec<Option<Item>>)> {
    let mut order: Vec<String> = Vec::new();
    let mut by: BTreeMap<String, Vec<Option<Item>>> = BTreeMap::new();
    let frames = all;
    for (i, its) in all.iter().enumerate() {
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for it in its.iter().filter(|it| kinds.contains(&it.kind)).cloned() {
            let base = format!("{}|{}", it.kind, base_ident(&it));
            let k = seen.entry(base.clone()).or_insert(0);
            let id = format!("{base}#{k}");
            *k += 1;
            let slot = by.entry(id.clone()).or_insert_with(|| {
                order.push(id.clone());
                vec![None; frames.len()]
            });
            slot[i] = Some(it);
        }
    }
    order.into_iter().map(|id| {
        let v = by.remove(&id).unwrap();
        (id, v)
    }).collect()
}

/// Each frame's item, a missing frame holding the nearest present one (a hidden part stays where it was).
pub fn filled(track: &[Option<Item>]) -> Vec<Item> {
    let first = track.iter().flatten().next().cloned().expect("a track has a frame");
    let mut last = first;
    track.iter().map(|t| {
        if let Some(it) = t {
            last = it.clone();
        }
        last.clone()
    }).collect()
}
