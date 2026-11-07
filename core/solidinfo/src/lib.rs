//! `raea solid info`: a solid measured, part by part, in scene lint's shape (docs/handoff/solids.md, Next 1; the
//! 11-06 research: numbers before renders for geometry, and a prediction checked rather than a threshold judged). It
//! takes each named part's shape expression (what a Solid knows of itself) and answers volume, area, box, centre of
//! mass, pieces, genus and triangles for each part and for the whole; how each pair of parts meets; and `found`, each
//! finding as `{part, check, level, says}` with `level` ok, note, warning or error. An expect file's predictions are
//! checked (`expect`), and two mutants (the whole's box and its convex hull) test whether those predictions can tell
//! the object from a crude stand-in. No aggregate pass: errors > 0 fails it, as lint.

mod expect;
mod measure;

pub use expect::{ACCEPTED, check_expect, mutants};
pub use measure::{Measured, measure, r6};

use raea_core_shape::Manifold;
use serde_json::{Value, json};

/// The parts built from their expressions; an expression that doesn't build is an error naming its part.
pub fn build(parts: &[(String, String)]) -> Result<Vec<(String, Manifold)>, String> {
    parts.iter().map(|(name, e)| raea_core_shape::solid(e).map(|m| (name.clone(), m)).map_err(|err| format!("{name}: {err}"))).collect()
}

/// The whole: every part unioned.
pub fn whole(parts: &[(String, Manifold)]) -> Manifold {
    parts.iter().map(|(_, m)| m.clone()).reduce(|a, b| a.union(&b)).expect("at least one part")
}

/// How two parts meet: overlapping (by how much), touching, or apart (by how far, searched to 5 cm).
pub fn pair(a: &Manifold, b: &Manifold) -> Value {
    let overlap = a.intersection(b).volume();
    if overlap > 1e-12 {
        return json!({"status": "overlap", "gap": 0.0, "overlap": r6(overlap)});
    }
    let gap = a.min_gap(b, 0.05);
    let status = if gap < 1e-4 { "touching" } else { "apart" };
    json!({"status": status, "gap": r6(gap), "overlap": 0.0})
}

/// The whole report for the parts (and the expect file's predictions, if any).
pub fn info(parts: &[(String, String)], expect: Option<&Value>) -> Result<Value, String> {
    if parts.is_empty() {
        return Err("solid info: the file made no solid (export default a Solid, an object of Solids, or a function of the params returning either)".into());
    }
    let built = build(parts)?;
    let all = whole(&built);
    let whole_m = measure(&all);
    let mut found: Vec<Value> = Vec::new();
    let mut part_rows = Vec::new();
    for (name, m) in &built {
        let pm = measure(m);
        if pm.empty {
            found.push(json!({"part": name, "check": "empty", "level": "error", "says": format!("{name} is empty: its boolean left nothing")}));
        } else if pm.pieces > 1 {
            found.push(json!({"part": name, "check": "disjoint", "level": "warning", "says": format!("{name} is {} pieces (the largest {} m³, the smallest {} m³)", pm.pieces, pm.piece_volumes.first().copied().unwrap_or(0.0), pm.piece_volumes.last().copied().unwrap_or(0.0))}));
        }
        // the part's measures beside its name, as the solid's are (agent-tool-surface.md's shape)
        let mut row = json!({"part": name});
        if let (Some(r), Some(m)) = (row.as_object_mut(), pm.json().as_object()) {
            r.extend(m.clone());
        }
        part_rows.push(row);
    }
    let mut pairs = Vec::new();
    for i in 0..built.len() {
        for j in i + 1..built.len() {
            let (a, b) = (&built[i], &built[j]);
            let mut p = pair(&a.1, &b.1);
            let small = a.1.volume().min(b.1.volume());
            if p["status"] == "overlap" && small > 0.0 && p["overlap"].as_f64().unwrap_or(0.0) > 0.5 * small {
                found.push(json!({"part": b.0, "check": "contains", "level": "warning", "says": format!("{} and {} overlap by more than half of the smaller: one is mostly inside the other", a.0, b.0)}));
            }
            p["a"] = json!(a.0);
            p["b"] = json!(b.0);
            pairs.push(p);
        }
    }
    if whole_m.empty {
        found.push(json!({"check": "empty", "level": "error", "says": "the solid is empty"}));
    } else if whole_m.pieces > 1 {
        // which parts sit apart from everything else
        let alone: Vec<&str> = built.iter().filter(|(n, _)| !pairs.iter().any(|p| (p["a"] == n.as_str() || p["b"] == n.as_str()) && p["status"] != "apart")).map(|(n, _)| n.as_str()).collect();
        let says = if alone.is_empty() || built.len() == 1 { format!("the solid is in {} pieces", whole_m.pieces) } else { format!("the solid is in {} pieces: {} reach no other part", whole_m.pieces, alone.join(", ")) };
        found.push(json!({"check": "disjoint", "level": "error", "says": says, "pieces": whole_m.piece_volumes}));
    } else {
        found.push(json!({"check": "disjoint", "level": "ok", "says": "one piece"}));
    }
    let mut out = json!({"solid": whole_m.json(), "parts": part_rows, "pairs": pairs});
    if let Some(e) = expect {
        let (checks, summary) = check_expect(e, &all, &whole_m)?;
        found.extend(checks);
        out["expect"] = summary;
        out["mutants"] = json!(mutants(e, &all, &whole_m)?);
        for m in out["mutants"].as_array().into_iter().flatten() {
            if m["status"] == "survived" {
                found.push(json!({"check": "mutant", "level": "warning", "says": m["says"]}));
            }
        }
    } else {
        found.push(json!({"check": "skipped", "level": "note", "says": "no expect file: nothing predicted, so nothing checked against a prediction (--expect E.json)"}));
    }
    let errors = found.iter().filter(|f| f["level"] == "error").count();
    let warnings = found.iter().filter(|f| f["level"] == "warning").count();
    out["errors"] = json!(errors);
    out["warnings"] = json!(warnings);
    out["found"] = json!(found);
    Ok(out)
}

/// What changed between two whole solids' measures (`--set` against the defaults).
pub fn delta(from: &str, was: &Value, now: &Value) -> Value {
    let pct = |a: f64, b: f64| if a.abs() > 1e-15 { r6((b - a) / a * 100.0) } else { 0.0 };
    json!({"from": from,
        "volume": {"was": was["volume"], "now": now["volume"], "pct": pct(was["volume"].as_f64().unwrap_or(0.0), now["volume"].as_f64().unwrap_or(0.0))},
        "size": {"was": was["box"]["size"], "now": now["box"]["size"]},
        "pieces": {"was": was["pieces"], "now": now["pieces"]},
        "genus": {"was": was["genus"], "now": now["genus"]}})
}

/// A walk's verdict for one parameter moved to one end: broken (empty, or more pieces than at the default), inert
/// (neither volume nor box moved), or ok.
pub fn walk_verdict(base: &Value, now: &Value) -> (&'static str, Option<String>) {
    if now["empty"] == true {
        return ("broken", Some("the solid is empty".into()));
    }
    let (bp, np) = (base["pieces"].as_u64().unwrap_or(1), now["pieces"].as_u64().unwrap_or(1));
    if np > bp {
        return ("broken", Some(format!("{np} pieces (at the default {bp})")));
    }
    let same_v = (base["volume"].as_f64().unwrap_or(0.0) - now["volume"].as_f64().unwrap_or(0.0)).abs() <= 1e-12;
    if same_v && base["box"] == now["box"] {
        return ("inert", Some("neither volume nor box moves: it reaches no geometry".into()));
    }
    ("ok", None)
}

#[cfg(test)]
mod tests;
