//! An expect file: predictions about the solid made before it is measured, each checked (`expect.<key>` findings, an
//! error when wrong). The file's ranges are the author's own guesses, so they are no target for a judge. Mutants: the
//! whole's box and its convex hull, each checked against the same predictions; a mutant the predictions can't tell
//! from the object "survives", which says the predictions are too loose to mean anything.

use manifold_rust::linalg::Vec3;
use raea_core_shape::Manifold;
use serde_json::{Value, json};

use crate::measure::{Measured, measure, r6};

pub const ACCEPTED: &str = "volume, area, size, box, centerOfMass, pieces, genus, solidPoints, emptyPoints";

fn range(key: &str, v: &Value) -> Result<(f64, f64), String> {
    match v.as_array().map(|a| a.iter().map(Value::as_f64).collect::<Option<Vec<f64>>>()) {
        Some(Some(a)) if a.len() == 2 && a[0] <= a[1] => Ok((a[0], a[1])),
        _ => Err(format!("expect: {key} is a range [low, high], got {v}")),
    }
}

/// Whether there is material at the point: a cube a thousandth of the solid's size there, intersected.
fn solid_at(m: &Manifold, span: f64, p: [f64; 3]) -> bool {
    let s = (span * 1e-3).max(1e-5);
    let probe = Manifold::cube(Vec3::new(s, s, s), true).translate(Vec3::new(p[0], p[1], p[2]));
    m.intersection(&probe).volume() > 0.5 * s * s * s
}

/// Each prediction checked against the solid: (findings, {passed, failed}).
pub fn check_expect(e: &Value, m: &Manifold, me: &Measured) -> Result<(Vec<Value>, Value), String> {
    let obj = e.as_object().ok_or("expect: the file is a JSON object of predictions")?;
    let mut out = Vec::new();
    let mut push = |key: &str, ok: bool, expected: Value, measured: Value, says: String| {
        out.push(json!({"check": format!("expect.{key}"), "level": if ok { "ok" } else { "error" }, "expected": expected, "measured": measured, "says": says}));
    };
    let size = me.size();
    let span = size.iter().cloned().fold(0.0, f64::max).max(1e-6);
    let axes = ["x", "y", "z"];
    for (key, v) in obj {
        match key.as_str() {
            "volume" | "area" => {
                let (lo, hi) = range(key, v)?;
                let got = if key == "volume" { me.volume } else { me.area };
                let ok = (lo..=hi).contains(&got);
                push(key, ok, v.clone(), json!(r6(got)), format!("{key} {} {} [{lo}, {hi}]", r6(got), if ok { "within" } else { "outside" }));
            }
            "size" | "centerOfMass" => {
                let o = v.as_object().ok_or_else(|| format!("expect: {key} is {{\"x\": [lo, hi], \"y\": ..., \"z\": ...}}"))?;
                for (ax, rv) in o {
                    let k = axes.iter().position(|a| a == ax).ok_or_else(|| format!("expect: {key}.{ax}: the axes are x, y, z"))?;
                    let (lo, hi) = range(&format!("{key}.{ax}"), rv)?;
                    let got = if key == "size" { size[k] } else { me.center_of_mass[k] };
                    let ok = (lo..=hi).contains(&got);
                    push(&format!("{key}.{ax}"), ok, rv.clone(), json!(r6(got)), format!("{key} {ax} {} {} [{lo}, {hi}]", r6(got), if ok { "within" } else { "outside" }));
                }
            }
            "box" => {
                let tol = v.get("tol").and_then(Value::as_f64).unwrap_or(0.002);
                for (side, got) in [("min", me.min), ("max", me.max)] {
                    if let Some(want) = v.get(side) {
                        let want = raea_core_place::xyz(want, &format!("expect: box.{side}"))?;
                        let ok = (0..3).all(|k| (want[k] - got[k]).abs() <= tol);
                        push(&format!("box.{side}"), ok, json!(want), json!(got.map(r6)), format!("box {side} {:?} {} {:?} (±{tol})", got.map(r6), if ok { "at" } else { "not at" }, want));
                    }
                }
            }
            "pieces" | "genus" => {
                let want = v.as_i64().ok_or_else(|| format!("expect: {key} is a whole number"))?;
                let got = if key == "pieces" { me.pieces as i64 } else { me.genus as i64 };
                push(key, got == want, json!(want), json!(got), format!("{key} {got}{}", if got == want { " as expected".to_string() } else { format!(", expected {want}") }));
            }
            "solidPoints" | "emptyPoints" => {
                let want_solid = key == "solidPoints";
                for (i, pv) in v.as_array().ok_or_else(|| format!("expect: {key} is a list of [x, y, z]"))?.iter().enumerate() {
                    let p = raea_core_place::xyz(pv, &format!("expect: a point of {key}"))?;
                    let got = !me.empty && solid_at(m, span, p);
                    let ok = got == want_solid;
                    push(&format!("{key}[{i}]"), ok, json!(p), json!(if got { "solid" } else { "empty" }), format!("{p:?} is {}{}", if got { "solid" } else { "empty" }, if ok { "" } else { if want_solid { ", expected material there" } else { ", expected no material there" } }));
                }
            }
            "mutants" | "note" => {}
            o => return Err(format!("expect: no prediction {o}: {ACCEPTED}")),
        }
    }
    let failed = out.iter().filter(|f| f["level"] == "error").count();
    let summary = json!({"passed": out.len() - failed, "failed": failed});
    Ok((out, summary))
}

/// The box and the hull, each checked against the predictions.
pub fn mutants(e: &Value, m: &Manifold, me: &Measured) -> Result<Vec<Value>, String> {
    if me.empty {
        return Ok(Vec::new());
    }
    let s = me.size();
    let block = Manifold::cube(Vec3::new(s[0].max(1e-6), s[1].max(1e-6), s[2].max(1e-6)), false).translate(Vec3::new(me.min[0], me.min[1], me.min[2]));
    let hull = Manifold::hull_manifolds(std::slice::from_ref(m));
    let mut out = Vec::new();
    for (name, mm) in [("block", block), ("hull", hull)] {
        let mm_me = measure(&mm);
        let (checks, _) = check_expect(e, &mm, &mm_me)?;
        let by: Vec<Value> = checks.iter().filter(|c| c["level"] == "error").map(|c| c["check"].clone()).collect();
        out.push(if by.is_empty() {
            json!({"name": name, "status": "survived", "says": format!("the expect file can't tell the solid from its {}: add an emptyPoints probe where the {name} has material and the object has none (a cavity, a gap, under a handle), or a tighter volume", if name == "block" { "bounding box" } else { "convex hull" })})
        } else {
            json!({"name": name, "status": "killed", "by": by})
        });
    }
    Ok(out)
}
