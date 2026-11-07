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
    /// per row: true when no frame changes any of its values from the base, so a system that applies a row's values
    /// to the World each frame can apply a still row's once
    still: Vec<bool>,
}

impl Timeline {
    pub fn new(prog: &Program, rows: &[Tup]) -> Result<Timeline, String> {
        Timeline::from_cols(prog, Cols::from_tups(rows))
    }

    pub fn from_cols(prog: &Program, base: Cols) -> Result<Timeline, String> {
        let frames = prog.frames.max(0);
        let per: Vec<(HashMap<String, Col>, Vec<usize>)> = parallel(frames as usize, |f| {
            let raw = eval::frame_cols(prog, &base, f as i64);
            let w: HashMap<String, Col> = raw.cols.into_iter().filter(|(c, v)| base.cols.get(c) != Some(v)).collect();
            // the rows this frame moves: a written column's value differing from the base's at that row
            let moved = (0..base.len()).filter(|r| w.iter().any(|(c, col)| !col.same_at(base.cols.get(c), *r))).collect();
            Ok((w, moved))
        })?;
        let mut still = vec![true; base.len()];
        let written = per.into_iter().map(|(w, moved)| {
            for r in moved {
                still[r] = false;
            }
            w
        }).collect();
        Ok(Timeline { frames, base, written, still })
    }

    /// Whether row r's value of any of `cols` differs from its base on some frame (it moves over the edit).
    pub fn moves(&self, r: usize, cols: &[&str]) -> bool {
        self.written.iter().any(|w| cols.iter().any(|c| w.get(*c).is_some_and(|col| col.get(r) != self.base.get(r, c))))
    }

    /// Whether row r's values are the same on every frame (out of range: false).
    pub fn still(&self, r: usize) -> bool {
        self.still.get(r).copied().unwrap_or(false)
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

/// f(0..n) on every core, in order. A browser build (wasm32) has one thread and can't spawn more: in turn there.
fn parallel<T: Send>(n: usize, f: impl Fn(usize) -> Result<T, String> + Sync) -> Result<Vec<T>, String> {
    #[cfg(target_arch = "wasm32")]
    return (0..n).map(f).collect();
    #[cfg(not(target_arch = "wasm32"))]
    parallel_threads(n, f)
}

#[cfg(not(target_arch = "wasm32"))]
fn parallel_threads<T: Send>(n: usize, f: impl Fn(usize) -> Result<T, String> + Sync) -> Result<Vec<T>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, x: f64) -> Tup {
        Tup { kind: "mesh".into(), id: id.into(), vals: [("x".to_string(), V::N(x))].into_iter().collect() }
    }

    #[test]
    fn a_row_no_frame_changes_is_still_and_a_keyed_one_is_not() {
        let mut prog = Program { frames: 4, ..Default::default() };
        prog.keys.insert(("moves".into(), "x".into()), vec![(0, 0.0, "linear".into()), (3, 3.0, "linear".into())]);
        // a key that holds one value on every frame changes nothing
        prog.keys.insert(("held".into(), "x".into()), vec![(0, 2.0, "linear".into()), (3, 2.0, "linear".into())]);
        let t = Timeline::new(&prog, &[row("cube", 1.0), row("moves", 0.0), row("held", 2.0)]).unwrap();
        assert!(t.still(0));
        assert!(!t.still(1));
        assert!(t.still(2));
        assert!(!t.still(3), "a row past the end is never still");
    }
}
