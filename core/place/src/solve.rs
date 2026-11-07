//! The solver: heights from the `on` chains, yaws from `against` and `facing`, then rounds of projection (each rule moves
//! the free objects it names just enough to hold), each round ending with overlaps pushed apart along their shallower
//! axis and everything kept in the room or on its support. What still doesn't hold after the rounds is found.

use serde_json::{Value, json};

use crate::rules::{Obj, Room, Rule};

const ROUNDS: usize = 400;
const TOL: f64 = 2e-3;

fn front(yaw: f64) -> [f64; 2] {
    let r = yaw.to_radians();
    [r.sin(), r.cos()]
}

/// Half the footprint's extent in x and z, turned by the yaw.
fn half(o: &Obj) -> [f64; 2] {
    let r = o.yaw.to_radians();
    let (c, s) = (r.cos().abs(), r.sin().abs());
    [(c * o.size[0] + s * o.size[2]) / 2.0, (s * o.size[0] + c * o.size[2]) / 2.0]
}

/// The footprint gap between two objects (0 when they touch or overlap) and their overlap depth on x and z.
fn gap_and_pen(a: &Obj, b: &Obj) -> (f64, [f64; 2]) {
    let (ha, hb) = (half(a), half(b));
    let d = [b.at[0] - a.at[0], b.at[2] - a.at[2]];
    let sep = [d[0].abs() - ha[0] - hb[0], d[1].abs() - ha[1] - hb[1]];
    let gap = (sep[0].max(0.0).powi(2) + sep[1].max(0.0).powi(2)).sqrt();
    (gap, [-sep[0], -sep[1]])
}

/// A clear zone in front of an object, as an object of its own (its footprint only).
fn zone(o: &Obj, depth: f64) -> Obj {
    let f = front(o.yaw);
    let hd = (half(o)[0] * f[0].abs() + half(o)[1] * f[1].abs()).max(1e-6);
    let across = o.size[0];
    let c = [o.at[0] + f[0] * (hd + depth / 2.0), o.at[2] + f[1] * (hd + depth / 2.0)];
    Obj { id: format!("{}'s clear zone", o.id), size: [across, 0.01, depth], at: [c[0], o.at[1], c[1]], yaw: o.yaw, fixed: true, turned: true }
}

fn support_of(rules: &[Rule], n: usize) -> Vec<Option<usize>> {
    let mut s = vec![None; n];
    for r in rules {
        if let Rule::On { a, b } = r {
            s[*a] = Some(*b);
        }
    }
    s
}

fn push(objs: &mut [Obj], i: usize, dx: f64, dz: f64) {
    if !objs[i].fixed {
        objs[i].at[0] += dx;
        objs[i].at[2] += dz;
    }
}

/// Move a and b apart (or together) by `d` along `n`, split by which are free.
fn split(objs: &mut [Obj], a: usize, b: usize, n: [f64; 2], d: f64) {
    let (fa, fb) = (!objs[a].fixed, !objs[b].fixed);
    let (wa, wb) = match (fa, fb) {
        (true, true) => (0.5, 0.5),
        (true, false) => (1.0, 0.0),
        (false, true) => (0.0, 1.0),
        _ => return,
    };
    push(objs, a, -n[0] * d * wa, -n[1] * d * wa);
    push(objs, b, n[0] * d * wb, n[1] * d * wb);
}

pub fn solve(mut objs: Vec<Obj>, rules: &[Rule], room: Room) -> Value {
    let n = objs.len();
    let sup = support_of(rules, n);
    // a start for the free ones given no place: spread on a ring round the room's middle, so none coincide
    let mid = room.map_or([0.0, 0.0], |(lo, hi)| [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0]);
    let spread = room.map_or(1.0, |(lo, hi)| ((hi[0] - lo[0]).min(hi[1] - lo[1]) / 4.0).max(0.1));
    for (i, o) in objs.iter_mut().enumerate() {
        if !o.fixed && o.at == [0.0; 3] {
            let a = i as f64 * 2.399963;
            o.at = [mid[0] + spread * a.cos(), 0.0, mid[1] + spread * a.sin()];
        }
    }
    for (i, s) in sup.iter().enumerate() {
        if let Some(b) = s {
            if !objs[i].fixed {
                objs[i].at[0] = objs[*b].at[0] + (i as f64 * 0.37).sin() * 1e-3;
                objs[i].at[2] = objs[*b].at[2];
            }
        }
    }
    let heights = |objs: &mut Vec<Obj>| {
        for _ in 0..n {
            for (i, s) in sup.iter().enumerate() {
                if let Some(b) = s {
                    objs[i].at[1] = objs[*b].at[1] + objs[*b].size[1];
                }
            }
        }
    };
    let mut wall_err = Vec::new();
    for _ in 0..ROUNDS {
        heights(&mut objs);
        for r in rules {
            match *r {
                Rule::Against { a, wall } => {
                    if !objs[a].turned || !objs[a].fixed {
                        objs[a].yaw = (-wall[0]).atan2(-wall[1]).to_degrees();
                        objs[a].turned = true;
                    }
                    let Some((lo, hi)) = room else {
                        if wall_err.is_empty() {
                            wall_err.push(objs[a].id.clone());
                        }
                        continue;
                    };
                    let h = half(&objs[a]);
                    if wall[0] != 0.0 {
                        let x = if wall[0] > 0.0 { hi[0] - h[0] } else { lo[0] + h[0] };
                        let dx = x - objs[a].at[0];
                        push(&mut objs, a, dx, 0.0);
                    } else {
                        let z = if wall[1] > 0.0 { hi[1] - h[1] } else { lo[1] + h[1] };
                        let dz = z - objs[a].at[2];
                        push(&mut objs, a, 0.0, dz);
                    }
                }
                Rule::Facing { a, b } => {
                    let d = [objs[b].at[0] - objs[a].at[0], objs[b].at[2] - objs[a].at[2]];
                    if d[0].hypot(d[1]) > 1e-9 && !objs[a].fixed {
                        objs[a].yaw = d[0].atan2(d[1]).to_degrees();
                        objs[a].turned = true;
                    }
                }
                Rule::Beside { a, b, side, gap } => {
                    let (ha, hb) = (half(&objs[a]), half(&objs[b]));
                    let want = [objs[b].at[0] + side[0] * (ha[0] + hb[0] + gap), objs[b].at[2] + side[1] * (ha[1] + hb[1] + gap)];
                    // along the side exactly; across it, lined up with b
                    let w = [if side[0] != 0.0 { want[0] } else { objs[b].at[0] }, if side[1] != 0.0 { want[1] } else { objs[b].at[2] }];
                    let d = [w[0] - objs[a].at[0], w[1] - objs[a].at[2]];
                    if objs[a].fixed {
                        push(&mut objs, b, -d[0], -d[1]);
                    } else {
                        push(&mut objs, a, d[0], d[1]);
                    }
                }
                Rule::Near { a, b, dist } => {
                    let (gap, _) = gap_and_pen(&objs[a], &objs[b]);
                    if gap > dist {
                        let d = [objs[b].at[0] - objs[a].at[0], objs[b].at[2] - objs[a].at[2]];
                        let l = d[0].hypot(d[1]).max(1e-9);
                        split(&mut objs, a, b, [-d[0] / l, -d[1] / l], (gap - dist) * 0.5);
                    }
                }
                Rule::Apart { a, b, min } => {
                    let (gap, _) = gap_and_pen(&objs[a], &objs[b]);
                    if gap < min {
                        let d = [objs[b].at[0] - objs[a].at[0], objs[b].at[2] - objs[a].at[2]];
                        let l = d[0].hypot(d[1]);
                        let nrm = if l > 1e-9 { [d[0] / l, d[1] / l] } else { [1.0, 0.0] };
                        split(&mut objs, a, b, nrm, (min - gap) * 0.5);
                    }
                }
                Rule::Align { a, b, axis } => {
                    let d = objs[b].at[axis] - objs[a].at[axis];
                    if axis == 0 { split(&mut objs, a, b, [-1.0, 0.0], d * 0.5) } else { split(&mut objs, a, b, [0.0, -1.0], d * 0.5) }
                }
                Rule::On { .. } | Rule::Clear { .. } => {}
            }
        }
        // overlaps among things on the same support, and clear zones kept free
        let zones: Vec<(usize, Obj)> = rules.iter().filter_map(|r| if let Rule::Clear { a, depth } = r { Some((*a, zone(&objs[*a], *depth))) } else { None }).collect();
        for i in 0..n {
            for j in i + 1..n {
                if sup[i] != sup[j] || sup[i] == Some(j) || sup[j] == Some(i) {
                    continue;
                }
                let (_, pen) = gap_and_pen(&objs[i], &objs[j]);
                if pen[0] > 0.0 && pen[1] > 0.0 {
                    let (k, d) = if pen[0] < pen[1] { (0, pen[0]) } else { (1, pen[1]) };
                    let c = if k == 0 { objs[j].at[0] - objs[i].at[0] } else { objs[j].at[2] - objs[i].at[2] };
                    let s = if c.abs() > 1e-9 { c.signum() } else if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
                    let nrm = if k == 0 { [s, 0.0] } else { [0.0, s] };
                    split(&mut objs, i, j, nrm, d + 1e-4);
                }
            }
            for (owner, z) in &zones {
                if *owner == i || sup[i] != sup[*owner] || sup[i] == Some(*owner) {
                    continue;
                }
                let (_, pen) = gap_and_pen(z, &objs[i]);
                if pen[0] > 0.0 && pen[1] > 0.0 {
                    let (k, d) = if pen[0] < pen[1] { (0, pen[0]) } else { (1, pen[1]) };
                    let c = if k == 0 { objs[i].at[0] - z.at[0] } else { objs[i].at[2] - z.at[2] };
                    let s = if c.abs() > 1e-9 { c.signum() } else { 1.0 };
                    push(&mut objs, i, if k == 0 { s * (d + 1e-4) } else { 0.0 }, if k == 1 { s * (d + 1e-4) } else { 0.0 });
                }
            }
        }
        // in the room, or on the support's top
        for i in 0..n {
            let h = half(&objs[i]);
            let bounds = match sup[i] {
                Some(b) => {
                    let hb = half(&objs[b]);
                    Some(([objs[b].at[0] - hb[0], objs[b].at[2] - hb[1]], [objs[b].at[0] + hb[0], objs[b].at[2] + hb[1]]))
                }
                None => room,
            };
            if let Some((lo, hi)) = bounds {
                for (k, ax) in [(0usize, 0usize), (1, 2)] {
                    let (a, b) = (lo[k] + h[k], hi[k] - h[k]);
                    let x = objs[i].at[ax];
                    let c = if a > b { (lo[k] + hi[k]) / 2.0 } else { x.clamp(a, b) };
                    if (c - x).abs() > 0.0 {
                        let mut d = [0.0, 0.0];
                        d[k] = c - x;
                        push(&mut objs, i, d[0], d[1]);
                    }
                }
            }
        }
    }
    heights(&mut objs);
    let found = crate::solve::check(&objs, rules, room, &sup, &wall_err);
    let errors = found.iter().filter(|f| f["level"] == "error").count();
    let r = |x: f64| (x * 1e5).round() / 1e5;
    let placed: Vec<Value> = objs.iter().map(|o| json!({"id": o.id, "at": o.at.map(r), "yaw": r(o.yaw), "size": o.size})).collect();
    json!({"objects": placed, "found": found, "errors": errors})
}

fn bx(o: &Obj) -> Value {
    let h = half(o);
    json!({"min": [o.at[0] - h[0], o.at[1], o.at[2] - h[1]], "max": [o.at[0] + h[0], o.at[1] + o.size[1], o.at[2] + h[1]]})
}

/// What holds and what doesn't, after the rounds.
pub(crate) fn check(objs: &[Obj], rules: &[Rule], room: Room, sup: &[Option<usize>], wall_err: &[String]) -> Vec<Value> {
    let mut out = Vec::new();
    let mut f = |part: &str, check: &str, ok: bool, says: String, o: &Obj| out.push(json!({"part": part, "check": check, "level": if ok { "ok" } else { "error" }, "says": says, "box": bx(o)}));
    for id in wall_err {
        let o = objs.iter().find(|o| &o.id == id).unwrap();
        f(id, "against", false, format!("{id} is against a wall, but there's no room to have walls (give room: {{min, max}})"), o);
    }
    for r in rules {
        match *r {
            Rule::On { a, b } => {
                let (ha, hb) = (half(&objs[a]), half(&objs[b]));
                let inside = (objs[a].at[0] - objs[b].at[0]).abs() + ha[0] <= hb[0] + TOL && (objs[a].at[2] - objs[b].at[2]).abs() + ha[1] <= hb[1] + TOL;
                f(&objs[a].id, "on", inside, if inside { format!("{} on {}", objs[a].id, objs[b].id) } else { format!("{} is wider than {}'s top: it overhangs", objs[a].id, objs[b].id) }, &objs[a]);
            }
            Rule::Against { a, wall } => {
                if let Some((lo, hi)) = room {
                    let h = half(&objs[a]);
                    let d = if wall[0] > 0.0 { hi[0] - (objs[a].at[0] + h[0]) } else if wall[0] < 0.0 { objs[a].at[0] - h[0] - lo[0] } else if wall[1] > 0.0 { hi[1] - (objs[a].at[2] + h[1]) } else { objs[a].at[2] - h[1] - lo[1] };
                    f(&objs[a].id, "against", d.abs() <= TOL, format!("{} is {:.3} m off its wall", objs[a].id, d.abs()), &objs[a]);
                }
            }
            Rule::Beside { a, b, side, gap } => {
                let (ha, hb) = (half(&objs[a]), half(&objs[b]));
                let along = if side[0] != 0.0 { (objs[a].at[0] - objs[b].at[0]) * side[0] - ha[0] - hb[0] } else { (objs[a].at[2] - objs[b].at[2]) * side[1] - ha[1] - hb[1] };
                let ok = (along - gap).abs() <= TOL;
                f(&objs[a].id, "beside", ok, format!("{} is {:.3} m beside {} (asked {gap})", objs[a].id, along, objs[b].id), &objs[a]);
            }
            Rule::Near { a, b, dist } => {
                let (g, _) = gap_and_pen(&objs[a], &objs[b]);
                f(&objs[a].id, "near", g <= dist + TOL, format!("{} is {:.3} m from {} (asked within {dist})", objs[a].id, g, objs[b].id), &objs[a]);
            }
            Rule::Apart { a, b, min } => {
                let (g, _) = gap_and_pen(&objs[a], &objs[b]);
                f(&objs[a].id, "apart", g + TOL >= min, format!("{} is {:.3} m from {} (asked at least {min})", objs[a].id, g, objs[b].id), &objs[a]);
            }
            Rule::Facing { a, b } => {
                let d = [objs[b].at[0] - objs[a].at[0], objs[b].at[2] - objs[a].at[2]];
                let want = d[0].atan2(d[1]).to_degrees();
                let off = ((objs[a].yaw - want + 540.0) % 360.0 - 180.0).abs();
                f(&objs[a].id, "facing", off <= 1.0, format!("{} faces {:.0}° off {}", objs[a].id, off, objs[b].id), &objs[a]);
            }
            Rule::Align { a, b, axis } => {
                let d = (objs[a].at[axis] - objs[b].at[axis]).abs();
                f(&objs[a].id, "align", d <= TOL, format!("{} is {:.3} m out of line with {}", objs[a].id, d, objs[b].id), &objs[a]);
            }
            Rule::Clear { a, depth } => {
                let z = zone(&objs[a], depth);
                let inside: Vec<&str> = (0..objs.len()).filter(|&i| i != a && sup[i] == sup[a] && sup[i] != Some(a)).filter(|&i| { let (_, p) = gap_and_pen(&z, &objs[i]); p[0] > TOL && p[1] > TOL }).map(|i| objs[i].id.as_str()).collect();
                f(&objs[a].id, "clear", inside.is_empty(), if inside.is_empty() { format!("{depth} m clear in front of {}", objs[a].id) } else { format!("{} stand in front of {}", inside.join(", "), objs[a].id) }, &objs[a]);
            }
        }
    }
    for i in 0..objs.len() {
        for j in i + 1..objs.len() {
            if sup[i] != sup[j] || sup[i] == Some(j) || sup[j] == Some(i) {
                continue;
            }
            let (_, p) = gap_and_pen(&objs[i], &objs[j]);
            if p[0] > TOL && p[1] > TOL {
                f(&objs[i].id, "overlap", false, format!("{} and {} overlap by {:.3} x {:.3} m: the rules leave no room for both", objs[i].id, objs[j].id, p[0], p[1]), &objs[i]);
            }
        }
        if sup[i].is_none() {
            if let Some((lo, hi)) = room {
                let h = half(&objs[i]);
                let out_of = objs[i].at[0] - h[0] < lo[0] - TOL || objs[i].at[0] + h[0] > hi[0] + TOL || objs[i].at[2] - h[1] < lo[1] - TOL || objs[i].at[2] + h[1] > hi[1] + TOL;
                if out_of {
                    f(&objs[i].id, "room", false, format!("{} doesn't fit in the room", objs[i].id), &objs[i]);
                }
            }
        }
    }
    out
}
