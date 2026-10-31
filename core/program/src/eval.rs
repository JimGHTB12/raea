//! An edit's rows at a frame: the rows as spawned with the keys' values at that frame. Rows are held as columns (Cols:
//! per column one optional value per row), the shape the World's store gathers into and the timeline is made of; Tup
//! is the row form for interchange.

use std::collections::{BTreeMap, HashMap};

use crate::ir::{self, V};

/// One row of a kind's table.
#[derive(Debug, Clone, PartialEq)]
pub struct Tup {
    pub kind: String,
    pub id: String,
    pub vals: BTreeMap<String, V>,
}

/// One column: numbers as f64 with a presence mask (the typed case K kernels read as slices), or any values (a
/// column that holds text in some row).
#[derive(Debug, Clone, PartialEq)]
pub enum Col {
    Num { v: Vec<f64>, has: Vec<bool> },
    /// shared until written (copying a frame copies no text)
    Any(std::sync::Arc<Vec<Option<V>>>),
}

impl Col {
    fn empty_for(v: &V, n: usize) -> Col {
        match v {
            V::N(_) => Col::Num { v: vec![0.0; n], has: vec![false; n] },
            V::S(_) => Col::Any(std::sync::Arc::new(vec![None; n])),
        }
    }
    pub fn get(&self, r: usize) -> Option<V> {
        match self {
            Col::Num { v, has } => has[r].then(|| V::N(v[r])),
            Col::Any(x) => x[r].clone(),
        }
    }
    pub fn has(&self, r: usize) -> bool {
        match self {
            Col::Num { has, .. } => has[r],
            Col::Any(x) => x[r].is_some(),
        }
    }
    pub fn num(&self, r: usize) -> Option<f64> {
        match self {
            Col::Num { v, has } => has[r].then_some(v[r]),
            Col::Any(x) => x[r].as_ref().map(V::num),
        }
    }
    pub fn set(&mut self, r: usize, x: V) {
        match (&mut *self, x) {
            (Col::Num { v, has }, V::N(n)) => {
                v[r] = n;
                has[r] = true;
            }
            (Col::Num { v, has }, x) => {
                // text arrives: the column holds any values from now on
                let mut any: Vec<Option<V>> = v.iter().zip(has.iter()).map(|(n, h)| h.then_some(V::N(*n))).collect();
                any[r] = Some(x);
                *self = Col::Any(std::sync::Arc::new(any));
            }
            (Col::Any(a), x) => std::sync::Arc::make_mut(a)[r] = Some(x),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Col::Num { v, .. } => v.len(),
            Col::Any(x) => x.len(),
        }
    }
}

/// Rows as columns: each row's kind and id, and per column one value per row (absent: the row doesn't have it).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Cols {
    /// shared between a frame and the base it came from (cloning a frame copies no strings)
    pub kind: std::sync::Arc<Vec<String>>,
    pub id: std::sync::Arc<Vec<String>>,
    pub cols: HashMap<String, Col>,
}

impl Cols {
    pub fn from_tups(rows: &[Tup]) -> Cols {
        let mut c = Cols { kind: std::sync::Arc::new(rows.iter().map(|t| t.kind.clone()).collect()), id: std::sync::Arc::new(rows.iter().map(|t| t.id.clone()).collect()), cols: HashMap::new() };
        for (r, t) in rows.iter().enumerate() {
            for (k, v) in &t.vals {
                c.set(r, k, v.clone());
            }
        }
        c
    }
    pub fn len(&self) -> usize {
        self.id.len()
    }
    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }
    pub fn get(&self, r: usize, c: &str) -> Option<V> {
        self.cols.get(c).and_then(|v| v.get(r))
    }
    pub fn num(&self, r: usize, c: &str) -> Option<f64> {
        self.cols.get(c).and_then(|v| v.num(r))
    }
    pub fn set(&mut self, r: usize, c: &str, v: V) {
        let n = self.id.len();
        match self.cols.get_mut(c) {
            Some(col) => col.set(r, v),
            None => {
                let mut col = Col::empty_for(&v, n);
                col.set(r, v);
                self.cols.insert(c.to_string(), col);
            }
        }
    }
    pub fn rows_of(&self, kind: &str) -> Vec<usize> {
        (0..self.kind.len()).filter(|&r| self.kind[r] == kind).collect()
    }
    pub fn tup(&self, r: usize) -> Tup {
        Tup { kind: self.kind[r].clone(), id: self.id[r].clone(), vals: self.cols.iter().filter_map(|(c, v)| v.get(r).map(|x| (c.clone(), x))).collect() }
    }
    pub fn to_tups(&self) -> Vec<Tup> {
        (0..self.len()).map(|r| self.tup(r)).collect()
    }
}

/// The keys (id, prop) -> [(frame, value, ease)] as of `frame`, eased.
pub fn key_value(pts: &[(i64, f64, String)], frame: i64) -> f64 {
    let ease = |e: &str, p: f64| match e {
        "out" => 1.0 - (1.0 - p).powi(3),
        "in" => p * p * p,
        "inout" | "in-out" => if p < 0.5 { 4.0 * p * p * p } else { 1.0 - (-2.0 * p + 2.0).powi(3) / 2.0 },
        // held until the next key's own frame, which has the new value (`{30: 1, 31: 0} step` is down at frame 30 only)
        "step" => if p >= 1.0 { 1.0 } else { 0.0 },
        _ => p,
    };
    let i = pts.partition_point(|k| k.0 < frame);
    if i == 0 {
        pts[0].1
    } else if i == pts.len() {
        pts[pts.len() - 1].1
    } else {
        let ((f0, v0, _), (f1, v1, e)) = (&pts[i - 1], &pts[i]);
        v0 + (v1 - v0) * ease(e, (frame - f0) as f64 / (f1 - f0) as f64)
    }
}

/// A frame's state: the inserted rows with every key's value at `frame`. Pure: depends only on its inputs.
pub fn frame_cols(prog: &ir::Program, base: &Cols, frame: i64) -> Cols {
    let mut all = base.clone();
    keys_into(prog, &mut all, frame);
    all
}

/// Every key's value at `frame` into its row's column.
pub fn keys_into(prog: &ir::Program, all: &mut Cols, frame: i64) {
    for r in 0..all.len() {
        let id = all.id[r].clone();
        for ((kid, prop), pts) in prog.keys.range((id.clone(), String::new())..) {
            if *kid != id {
                break;
            }
            all.set(r, prop, V::N(key_value(pts, frame)));
        }
    }
}
