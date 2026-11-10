//! The clipping check's and the lint's results as Findings (docs/direction.feature DR3): the parts each is about by
//! name, the number it turns on, and the move that fixes it. The lint's own list keeps every part's support (what
//! `raea scene explain` prints and a review frames its cameras by); only its faults are findings.

use raea_core_facts::{Finding, Level};
use serde_json::Value;

/// The lint's faults on `frame` as Findings: sunk, floats, air, tilted, clips, a model it can't load or that's over
/// budget, and what it skipped (a note). A part that rests, is mounted or is the ground is no finding.
pub fn lint_findings(found: &[Value], frame: i64, by: &str) -> Vec<Finding> {
    found
        .iter()
        .filter_map(|v| {
            let mut f = Finding::of_value(v, by)?.frames(frame, frame + 1);
            // the lint's fix is a move, said below
            f.fix = None;
            let check = f.check.clone();
            let num = |k: &str| v[k].as_f64();
            if let Some(g) = num("gap").filter(|_| matches!(check.as_str(), "sunk" | "floats")) {
                f = f.measure("gap", g, "m", None);
            } else if let Some(d) = num("depth") {
                f = f.measure("depth", d, "m", None);
            } else if let Some(d) = num("degrees") {
                f = f.measure("lean", d, "degrees", None);
            }
            if let Some((y, dy)) = v.get("fix").and_then(|x| Some((x.get("y")?.as_f64()?, x.get("dy")?.as_f64()?))) {
                f = f.fix(format!("move it {dy:+.4} m in y, to y = {y}"));
            }
            Some(f)
        })
        .collect()
}

/// The clipping check's overlaps (lib.rs `check`) as error Findings over the frames each holds on.
pub fn clip_findings(found: &[Value], by: &str) -> Vec<Finding> {
    found
        .iter()
        .map(|v| {
            let at = |k: &str| v[k].as_i64().unwrap_or(0);
            // the check's "to" is the last frame it clips on; a finding's span ends one past it
            let mut f = Finding::new("clips", Level::Error, v["says"].as_str().unwrap_or_default(), by).frames(at("from"), at("to") + 1);
            for k in ["part", "into"] {
                if let Some(name) = v[k].as_str() {
                    f = f.about(name, None);
                }
            }
            if let Some(d) = v["depth"].as_f64() {
                f = f.measure("depth", d, "m", None);
            }
            if let (Some(axis), Some(step)) = (v.pointer("/move/axis").and_then(Value::as_str), v.pointer("/move/by").and_then(Value::as_f64)) {
                f = f.fix(format!("move it {step:+.3} m along {axis}"));
            }
            f
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_lints_faults_are_findings_and_its_support_lines_are_not() {
        let found = vec![
            json!({"part": "rug", "check": "rests", "level": "ok", "says": "rug rests on floor", "on": "floor"}),
            json!({"part": "lamp", "check": "floats", "level": "error", "says": "lamp floats 12 mm above desk", "on": "desk", "gap": 0.012, "fix": {"y": 0.75, "dy": -0.012}}),
            json!({"part": "vase", "check": "tilted", "level": "warning", "on": "desk", "degrees": 7.5, "says": "vase leans 7.5° off level on desk"}),
        ];
        let fs = lint_findings(&found, 30, "raea scene lint");
        assert_eq!(fs.len(), 2);
        let lamp = &fs[0];
        assert_eq!((lamp.level, lamp.frames, lamp.about.iter().map(|a| a.name.as_str()).collect::<Vec<_>>()), (Level::Error, Some([30, 31]), vec!["lamp", "desk"]));
        assert_eq!((lamp.measure.as_ref().map(|m| m.value), lamp.fix.as_deref()), (Some(0.012), Some("move it -0.0120 m in y, to y = 0.75")));
        assert_eq!((fs[1].level, fs[1].measure.as_ref().map(|m| m.unit.as_str())), (Level::Warn, Some("degrees")));
    }

    #[test]
    fn an_overlap_is_an_error_about_both_parts_over_its_frames() {
        let found = vec![json!({"from": 40, "to": 52, "part": "rider", "into": "seat", "depth": 0.04, "share": 0.2, "move": {"axis": "y", "by": 0.05}, "says": "rider passes 40 mm into seat"})];
        let f = &clip_findings(&found, "raea render")[0];
        // frames 40 to 52, the last included
        assert_eq!(f.line(), "error clips  frames 40-52  rider, seat: rider passes 40 mm into seat (fix: move it +0.050 m along y)");
    }
}
