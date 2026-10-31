//! raea's IR: an edit as rows spawned once and keyed per frame. raea builds it from a composition's frames
//! (runtime/export/src/program.rs), html.rs from an HTML page; eval.rs and timeline.rs read every frame off it.

use std::collections::BTreeMap;

/// A value in a column: numbers (booleans are 1 and 0) or strings.
#[derive(Debug, Clone, PartialEq)]
pub enum V {
    N(f64),
    S(String),
}

impl V {
    pub fn num(&self) -> f64 {
        match self {
            V::N(x) => *x,
            V::S(s) => s.parse().unwrap_or(0.0),
        }
    }
    pub fn truthy(&self) -> bool {
        match self {
            V::N(x) => *x != 0.0,
            V::S(s) => !s.is_empty(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub fps: f64,
    pub width: u32,
    pub height: u32,
    pub frames: i64,
    /// (kind, rows of column -> value), in order
    pub spawns: Vec<(String, Vec<BTreeMap<String, V>>)>,
    /// (id, prop) -> [(frame, value, ease)] sorted by frame
    pub keys: BTreeMap<(String, String), Vec<(i64, f64, String)>>,
}
