//! What the meter finds, as raea's one Finding (docs/direction.feature DR3): a cut that jumps in exposure or drifts in
//! white balance, about its two shots; and `raea scene light`'s text, its findings counted last.

use raea_core_facts::{Finding, Level, finding};
use serde_json::Value;

use crate::{DRIFT_MIRED, JUMP_STOPS};

/// Each cut's jump and drift (`cuts`), a warn about the shots on either side, with the stops or mired it turns on.
pub fn cut_findings(cuts: &[Value]) -> Vec<Finding> {
    let mut found = Vec::new();
    for c in cuts {
        for (k, check, unit, limit) in [("jump", "light-jump", "stops", JUMP_STOPS), ("drift", "light-drift", "mired", DRIFT_MIRED)] {
            if c[k] != true {
                continue;
            }
            let mut f = Finding::new(check, Level::Warn, c["says"].as_str().unwrap_or_default(), "raea scene light");
            for s in [&c["from"], &c["to"]] {
                f = f.about(s.as_str().unwrap_or_default(), None);
            }
            if let Some(v) = c[unit].as_f64() {
                f = f.measure(k, v, unit, Some(limit as f64));
            }
            found.push(f);
        }
    }
    found
}

/// `raea scene light`'s text: each shot's meter at its frames, each cut, then the cuts' findings and their count.
pub fn report(shots: &[(String, Vec<(i64, Value)>)], cuts: &[Value]) -> String {
    let mut t = Vec::new();
    for (c, m) in shots {
        for (f, v) in m {
            t.push(crate::say(if c.is_empty() { "the composition" } else { c }, *f, v));
        }
    }
    for c in cuts {
        t.push(format!("cut {}", c["says"].as_str().unwrap_or("")));
    }
    let found = cut_findings(cuts);
    t.extend(found.iter().map(Finding::line));
    t.push(format!("raea scene light: {}", finding::count(&found)));
    t.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_cut_that_jumps_and_drifts_is_two_warns_about_its_shots() {
        let cuts = vec![
            json!({"from": "wide", "to": "close", "stops": 0.9, "mired": 25.0, "jump": true, "drift": true, "says": "wide to close: the picture +0.90 stops"}),
            json!({"from": "close", "to": "end", "stops": 0.1, "mired": 2.0, "jump": false, "drift": false, "says": "close to end"}),
        ];
        let fs = cut_findings(&cuts);
        assert_eq!(fs.iter().map(|f| f.check.as_str()).collect::<Vec<_>>(), ["light-jump", "light-drift"]);
        assert_eq!((fs[0].about[1].name.as_str(), fs[1].measure.as_ref().map(|m| (m.value, m.unit.as_str()))), ("close", Some((25.0, "mired"))));
        assert!(report(&[], &cuts).ends_with("raea scene light: 0 errors, 2 warns, 0 notes"));
    }
}
