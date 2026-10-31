//! The timeline: every frame of an edit evaluated once, shared by everything that reads frames (the render, the
//! sound's lanes, frame digests). Each frame keeps only the columns that differ from the base, and is a pure function
//! of the edit, so any frame can be read in any order.

use std::collections::HashMap;

use crate::eval::{self, Col, Cols, Tup};
use crate::ir::{Program, V};

pub struct Timeline {
    pub frames: i64,
    pub base: Cols,
    /// per frame: the columns whose values differ from the base (whole columns)
    written: Vec<HashMap<String, Col>>,
}

impl Timeline {
    pub fn new(prog: &Program, rows: &[Tup]) -> Result<Timeline, String> {
        Timeline::from_cols(prog, Cols::from_tups(rows))
    }

    pub fn from_cols(prog: &Program, base: Cols) -> Result<Timeline, String> {
        let frames = prog.frames.max(0);
        let written = parallel(frames as usize, |f| {
            let raw = eval::frame_cols(prog, &base, f as i64);
            Ok(raw.cols.into_iter().filter(|(c, v)| base.cols.get(c) != Some(v)).collect())
        })?;
        Ok(Timeline { frames, base, written })
    }

    /// Whether row r's value of any of `cols` differs from its base on some frame (it moves over the edit).
    pub fn moves(&self, r: usize, cols: &[&str]) -> bool {
        self.written.iter().any(|w| cols.iter().any(|c| w.get(*c).is_some_and(|col| col.get(r) != self.base.get(r, c))))
    }

    /// One frame as columns.
    pub fn cols(&self, f: i64) -> Cols {
        let mut c = self.base.clone();
        if let Some(w) = self.written.get(f as usize) {
            for (k, v) in w {
                c.cols.insert(k.clone(), v.clone());
            }
        }
        c
    }

    /// One frame as rows.
    pub fn tups(&self, f: i64) -> Vec<Tup> {
        self.cols(f).to_tups()
    }

    /// One value at frame f (None: the row doesn't have it).
    pub fn get(&self, f: i64, r: usize, c: &str) -> Option<V> {
        match self.written.get(f as usize).and_then(|w| w.get(c)) {
            Some(col) => col.get(r),
            None => self.base.get(r, c),
        }
    }
}

/// f(0..n) on every core, in order.
fn parallel<T: Send>(n: usize, f: impl Fn(usize) -> Result<T, String> + Sync) -> Result<Vec<T>, String> {
    let cores = std::thread::available_parallelism().map(|c| c.get()).unwrap_or(4).min(n.max(1));
    let chunk = n.div_ceil(cores.max(1)).max(1);
    let parts: Vec<Result<Vec<T>, String>> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..n).step_by(chunk).map(|a| {
            let f = &f;
            s.spawn(move || (a..(a + chunk).min(n)).map(f).collect::<Result<Vec<T>, String>>())
        }).collect();
        hs.into_iter().map(|h| h.join().expect("a timeline thread panicked")).collect()
    });
    let mut out = Vec::with_capacity(n);
    for p in parts {
        out.extend(p?);
    }
    Ok(out)
}
