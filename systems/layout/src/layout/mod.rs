//! How a frame is laid out (Perception.swift › layout(from:)): people, captions, screens, panes and splits, in the
//! previz layout format (previz.rs, deleted with engine/html in f1b4a17e). The inputs are what Vision saw (faces, text lines, rectangles, the
//! person mask, whether it's a screenshot) and grey copies of the frame and the next frames (240 px wide); the Vision
//! side moves behind a trait later (docs/goal.feature A3). The arithmetic follows the Swift's order, and its ties (Swift's
//! `max(by:)` keeps the first maximum).

use serde::{Deserialize, Serialize};

mod read;

pub use read::{inputs_from_json, layout};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect { x, y, w, h }
    }
    pub fn x1(&self) -> f64 {
        self.x + self.w
    }
    pub fn y1(&self) -> f64 {
        self.y + self.h
    }
    pub fn area(&self) -> f64 {
        self.w.max(0.0) * self.h.max(0.0)
    }
    pub fn union(&self, o: &Rect) -> Rect {
        let (a, b) = (self.x.min(o.x), self.y.min(o.y));
        Rect::new(a, b, self.x1().max(o.x1()) - a, self.y1().max(o.y1()) - b)
    }
    pub fn inter(&self, o: &Rect) -> f64 {
        (self.x1().min(o.x1()) - self.x.max(o.x)).max(0.0) * (self.y1().min(o.y1()) - self.y.max(o.y)).max(0.0)
    }
    pub fn contains(&self, o: &Rect) -> bool {
        o.area() > 0.0 && self.inter(o) / o.area() >= 0.8
    }
    pub fn clamped(&self) -> Rect {
        let (a, b) = (self.x.max(0.0), self.y.max(0.0));
        Rect::new(a, b, self.x1().min(1.0) - a, self.y1().min(1.0) - b)
    }
    fn zone(&self, role: &str, face: Option<Rect>, words: Option<usize>, label: Option<String>) -> Zone {
        Zone {
            role: role.into(),
            x: r3(self.x),
            y: r3(self.y),
            w: r3(self.w),
            h: r3(self.h),
            face: face.map(|f| Boxed { x: r3(f.x), y: r3(f.y), w: r3(f.w), h: r3(f.h) }),
            words,
            label,
        }
    }
}

pub fn r3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Boxed {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub role: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face: Option<Boxed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Split {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub ratio: String,
    pub split: Split,
    pub zones: Vec<Zone>,
}

/// A small grey copy of a frame: reads clamp to the edges.
#[derive(Clone, Debug)]
pub struct Grey {
    pub w: usize,
    pub h: usize,
    pub v: Vec<f32>,
}

impl Grey {
    fn px(&self, x: i64, y: i64) -> f32 {
        self.v[(y.clamp(0, self.h as i64 - 1) as usize) * self.w + x.clamp(0, self.w as i64 - 1) as usize]
    }
    fn at(&self, x: f64, y: f64) -> f32 {
        self.px(x.round() as i64, y.round() as i64)
    }
}

/// What the layout is read from.
#[derive(Clone, Debug)]
pub struct Inputs {
    pub w: f64,
    pub h: f64,
    pub screenshot: bool,
    pub faces: Vec<Rect>,
    pub lines: Vec<(Rect, String)>,
    pub rects: Vec<Rect>,
    pub mask: Option<Vec<f32>>,
    pub mw: usize,
    pub mh: usize,
    pub g: Grey,
    pub next: Vec<Grey>,
}

type Seg = (f64, f64);
type R4 = (f64, f64, f64, f64);

/// Swift's `max(by:)`: the first of the largest.
fn first_max<T: Copy>(v: impl IntoIterator<Item = T>, key: impl Fn(&T) -> f64) -> Option<T> {
    let mut best: Option<T> = None;
    for e in v {
        if best.as_ref().is_none_or(|b| key(b) < key(&e)) {
            best = Some(e);
        }
    }
    best
}

/// Swift's `stride(from:to:by:)` over doubles: start + i * step while below `to`.
fn stride_to(from: f64, to: f64, by: f64) -> impl Iterator<Item = f64> {
    (0..).map(move |i| from + i as f64 * by).take_while(move |&v| v < to)
}

fn stride_through(from: f64, through: f64, by: f64) -> impl Iterator<Item = f64> {
    (0..).map(move |i| from + i as f64 * by).take_while(move |&v| v <= through)
}

/// Share of a straight line's length that crosses a hard edge; stretches under `skip` (grey px) are left out.
fn coverage(g: &Grey, p: Seg, q: Seg, step: f32, skip: &[Rect]) -> f64 {
    let (dx, dy) = (q.0 - p.0, q.1 - p.1);
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 10.0 {
        return 0.0;
    }
    let (nx, ny) = (-dy / len * 2.0, dx / len * 2.0);
    let n = len as i64;
    let (mut hit, mut seen) = (0i64, 0i64);
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let (x, y) = (p.0 + dx * t, p.1 + dy * t);
        if skip.iter().any(|r| x >= r.x - 2.0 && x <= r.x1() + 2.0 && y >= r.y - 3.0 && y <= r.y1() + 3.0) {
            continue;
        }
        seen += 1;
        if (g.at(x + nx, y + ny) - g.at(x - nx, y - ny)).abs() > step {
            hit += 1;
        }
    }
    if seen * 2 >= n + 1 { hit as f64 / seen as f64 } else { 0.0 }
}

/// Both sides of a line carry detail.
fn textured(g: &Grey, p: Seg, q: Seg) -> bool {
    let (dx, dy) = (q.0 - p.0, q.1 - p.1);
    let len = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = (-dy / len, dx / len);
    let mut side = [0f32; 2];
    let mut n = 0i64;
    let mut i = 0i64;
    while i <= len as i64 {
        let t = i as f64 / len;
        let (x, y) = (p.0 + dx * t, p.1 + dy * t);
        for (k, s) in [(0usize, 1.0f64), (1, -1.0)] {
            for d in stride_through(4.0, 14.0, 2.0) {
                let (ax, ay) = (x + nx * d * s, y + ny * d * s);
                side[k] += (g.at(ax, ay) - g.at(x + nx * (d + 1.0) * s + 1.0, y + ny * (d + 1.0) * s)).abs() + (g.at(ax, ay) - g.at(ax + 1.0, ay)).abs();
            }
        }
        n += 6;
        i += 3;
    }
    side.iter().all(|v| v / n.max(1) as f32 > 0.025)
}

/// How much the picture carries on across a line: correlation of the detail along it either side.
fn carries_over(g: &Grey, p: Seg, q: Seg) -> f64 {
    let (dx, dy) = (q.0 - p.0, q.1 - p.1);
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 10.0 {
        return 1.0;
    }
    let (ux, uy) = (dx / len, dy / len);
    let (nx, ny) = (-uy, ux);
    let mut best: Option<f64> = None;
    for off in [4.0, 8.0, 12.0] {
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for i in 1..(len as i64 - 1) {
            let (x, y) = (p.0 + ux * i as f64, p.1 + uy * i as f64);
            let along = |o: f64| (g.at(x + nx * o + ux, y + ny * o + uy) - g.at(x + nx * o - ux, y + ny * o - uy)) as f64;
            a.push(along(off));
            b.push(along(-off));
        }
        let n = a.len() as f64;
        let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
        let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
        for i in 0..a.len() {
            sab += (a[i] - ma) * (b[i] - mb);
            saa += (a[i] - ma) * (a[i] - ma);
            sbb += (b[i] - mb) * (b[i] - mb);
        }
        let c = if saa > 1e-9 && sbb > 1e-9 { sab / (saa * sbb).sqrt() } else { 0.0 };
        best = Some(best.map_or(c, |b: f64| if c > b { c } else { b }));
    }
    best.unwrap_or(1.0)
}

/// XY-cut into panes along seams crossing edge to edge; the panes in grey px and the first cut's direction.
fn panes(g: &Grey, skip: &[Rect]) -> (Vec<R4>, Option<&'static str>) {
    fn cut(g: &Grey, skip: &[Rect], r: R4, depth: usize, first: &mut Option<&'static str>) -> Vec<R4> {
        let (x0, y0, x1, y1) = r;
        let (w, h) = (x1 - x0, y1 - y0);
        if !(depth < 3 && w > g.w as f64 * 0.2 && h > g.h as f64 * 0.12) {
            return vec![r];
        }
        let mut best: (f64, &'static str, f64) = (0.0, "", 0.0);
        for y in stride_to(y0 + h * 0.12, y1 - h * 0.12, 1.0) {
            let c = coverage(g, (x0 + 2.0, y), (x1 - 3.0, y), 0.07, skip);
            if c > best.0 && c >= 0.9 && carries_over(g, (x0 + 2.0, y), (x1 - 3.0, y)) < 0.4 {
                best = (c, "row", y);
            }
        }
        for x in stride_to(x0 + w * 0.12, x1 - w * 0.12, 1.0) {
            let c = coverage(g, (x, y0 + 2.0), (x, y1 - 3.0), 0.07, skip);
            if c > best.0 && c >= 0.9 && carries_over(g, (x, y0 + 2.0), (x, y1 - 3.0)) < 0.4 {
                best = (c, "column", x);
            }
        }
        if best.0 < 0.9 {
            return vec![r];
        }
        if depth == 0 {
            *first = Some(best.1);
        }
        let at = best.2;
        let mut out = Vec::new();
        if best.1 == "row" {
            out.extend(cut(g, skip, (x0, y0, x1, at), depth + 1, first));
            out.extend(cut(g, skip, (x0, at, x1, y1), depth + 1, first));
        } else {
            out.extend(cut(g, skip, (x0, y0, at, y1), depth + 1, first));
            out.extend(cut(g, skip, (at, y0, x1, y1), depth + 1, first));
        }
        out
    }
    let mut first = None;
    let all = cut(g, skip, (0.0, 0.0, g.w as f64, g.h as f64), 0, &mut first);
    (all, first)
}

/// Mean step between neighbouring pixels in a box.
fn detail(g: &Grey, r: R4) -> f32 {
    let (mut s, mut n) = (0f32, 0i64);
    let mut y = r.1 as i64 + 1;
    while y < r.3 as i64 - 1 {
        let mut x = r.0 as i64 + 1;
        while x < r.2 as i64 - 1 {
            s += (g.px(x + 1, y) - g.px(x, y)).abs() + (g.px(x, y + 1) - g.px(x, y)).abs();
            n += 1;
            x += 2;
        }
        y += 2;
    }
    if n > 0 { s / n as f32 } else { 0.0 }
}

fn is_picture(g: &Grey, r: R4) -> bool {
    let (w, h) = ((r.2 - r.0) / g.w as f64, (r.3 - r.1) / g.h as f64);
    w * h >= 0.1 && w.min(h) >= 0.25 && detail(g, r) > 0.006
}

/// The best diagonal seam through the middle band: (angle as found, place 0..1, coverage).
fn diagonal(g: &Grey) -> Option<(f64, f64, f64)> {
    let (ww, hh) = (g.w as f64, g.h as f64);
    let mut best: Option<(f64, f64, f64)> = None;
    for a in stride_through(10.0, 60.0, 1.0) {
        for sign in [-1.0, 1.0] {
            let t = (a * sign * std::f64::consts::PI / 180.0).tan();
            for y0 in stride_through(hh * 0.3, hh * 0.7, 1.0) {
                let mut p = (0.0, y0 - ww / 2.0 * t);
                let mut q = (ww - 1.0, y0 + ww / 2.0 * t);
                if p.1 < 0.0 || p.1 > hh - 1.0 {
                    let e = if p.1 < 0.0 { 0.0 } else { hh - 1.0 };
                    p = (ww / 2.0 + (e - y0) / t, e);
                }
                if q.1 < 0.0 || q.1 > hh - 1.0 {
                    let e = if q.1 < 0.0 { 0.0 } else { hh - 1.0 };
                    q = (ww / 2.0 + (e - y0) / t, e);
                }
                let c = coverage(g, p, q, 0.07, &[]);
                if c >= 0.95 && c > best.map_or(0.0, |b| b.2) && textured(g, p, q) && carries_over(g, p, q) < 0.4 {
                    best = Some((a * sign, y0 / hh, c));
                }
            }
        }
    }
    best
}

/// A sharp band between two flat or blurred ones with hard straight edges: (top, bottom) as 0..1.
fn framed_band(g: &Grey) -> Option<(f64, f64)> {
    let mut row = vec![0.0f64; g.h];
    for y in 0..g.h.saturating_sub(1) {
        let mut s = 0f32;
        for x in 0..g.w.saturating_sub(1) {
            let (x, yy) = (x as i64, y as i64);
            s += (g.px(x + 1, yy) - g.px(x, yy)).abs() + (g.px(x, yy + 1) - g.px(x, yy)).abs();
        }
        row[y] = s as f64 / g.w as f64;
    }
    let mut pre = vec![0.0f64];
    for v in &row {
        pre.push(pre.last().unwrap() + v);
    }
    let mean = |a: usize, b: usize| if b > a { (pre[b] - pre[a]) / (b - a) as f64 } else { 0.0 };
    let gh = g.h as i64;
    let mut best: (f64, i64, i64) = (0.0, 0, 0);
    let step_rows = (gh / 100).max(1);
    let mut a = (gh as f64 * 0.08) as i64;
    while a < (gh as f64 * 0.5) as i64 {
        let mut b = a + (gh as f64 * 0.25) as i64;
        while b < (gh as f64 * 0.92) as i64 {
            let (au, bu) = (a as usize, b as usize);
            let outside = (mean(0, au) * a as f64 + mean(bu, g.h) * (gh - b) as f64) / (a + gh - b) as f64;
            let r = mean(au, bu) / (outside + 0.002);
            if r > best.0 {
                best = (r, a, b);
            }
            b += step_rows;
        }
        a += step_rows;
    }
    if best.0 <= 3.0 {
        return None;
    }
    let edge = |y: i64, down: bool| {
        let a = mean((y - 6).max(0) as usize, y as usize);
        let b = mean(y as usize, (y + 6).min(gh) as usize);
        if down { b - a } else { a - b }
    };
    let span = (gh as f64 * 0.12) as i64;
    let top = first_max((best.1 - span).max(1)..(best.1 + span).min(gh - 1), |&y| edge(y, true)).unwrap_or(best.1);
    let bottom = first_max((best.2 - span).max(1)..(best.2 + span).min(gh - 1), |&y| edge(y, false)).unwrap_or(best.2);
    let ww = g.w as f64;
    let hard = |y: i64| (y - 1..=y + 1).any(|v| coverage(g, (2.0, v as f64), (ww - 3.0, v as f64), 0.04, &[]) >= 0.6);
    if !(hard(top) && hard(bottom)) {
        return None;
    }
    Some((top as f64 / gh as f64, bottom as f64 / gh as f64))
}

fn split_key(g: &Grey, skip: &[Rect]) -> (&'static str, Vec<f64>) {
    let content: Vec<R4> = panes(g, skip).0.into_iter().filter(|r| is_picture(g, *r)).collect();
    if content.len() > 1 {
        let mut v: Vec<f64> = content.iter().flat_map(|r| [r.0, r.1, r.2, r.3]).collect();
        v.sort_by(f64::total_cmp);
        return ("panes", v);
    }
    if let Some(d) = diagonal(g) {
        return ("diagonal", vec![d.0, d.1 * g.h as f64]);
    }
    ("none", vec![])
}

fn split_holds(g: &Grey, next: &[Grey], skip: &[Rect]) -> bool {
    let k = split_key(g, skip);
    if k.0 == "none" {
        return true;
    }
    for n in next {
        let change = g.v.iter().zip(&n.v).fold(0f32, |s, (a, b)| s + (a - b).abs()) / g.v.len().max(1) as f32;
        if change < 0.01 {
            continue;
        }
        let m = split_key(n, skip);
        if !(m.0 == k.0 && m.1.len() == k.1.len() && m.1.iter().zip(&k.1).all(|(a, b)| (a - b).abs() <= 1.5)) {
            return false;
        }
    }
    true
}

/// Text lines into blocks (union-find): stacked close, overlapping across, similar size. Blocks in order of their
/// first line (the Swift groups through a Dictionary, whose order isn't fixed; compare zones as a set).
fn text_blocks(lines: &[(Rect, String)]) -> Vec<Vec<(Rect, String)>> {
    let mut parent: Vec<usize> = (0..lines.len()).collect();
    fn find(p: &[usize], mut i: usize) -> usize {
        while p[i] != i {
            i = p[i];
        }
        i
    }
    for i in 0..lines.len() {
        for j in i + 1..lines.len() {
            let (a, b) = (&lines[i].0, &lines[j].0);
            let gap = a.y.max(b.y) - a.y1().min(b.y1());
            let across = (a.x1().min(b.x1()) - a.x.max(b.x)).max(0.0) / a.w.min(b.w).max(1e-6);
            let size = a.h.max(b.h) / a.h.min(b.h).max(1e-6);
            if gap < 0.8 * a.h.min(b.h) && across > 0.3 && size < 1.6 {
                let (fj, fi) = (find(&parent, j), find(&parent, i));
                parent[fj] = fi;
            }
        }
    }
    let mut groups: Vec<(usize, Vec<(Rect, String)>)> = Vec::new();
    for i in 0..lines.len() {
        let r = find(&parent, i);
        match groups.iter_mut().find(|g| g.0 == r) {
            Some(g) => g.1.push(lines[i].clone()),
            None => groups.push((r, vec![lines[i].clone()])),
        }
    }
    groups.into_iter().map(|g| g.1).collect()
}
