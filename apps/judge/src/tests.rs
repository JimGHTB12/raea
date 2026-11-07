use super::*;
use serde_json::json;

fn scores(v: f64) -> serde_json::Map<String, Value> {
    rubric::SCORES.iter().map(|s| (s.0.to_string(), json!(v))).collect()
}

#[test]
fn the_pass_is_code_and_a_blocking_defect_holds_it() {
    assert!(rubric::pass(&scores(7.5), &[]).0);
    assert!(!rubric::pass(&scores(6.5), &[]).0, "mean under 7");
    let mut s = scores(8.0);
    s.insert("form".into(), json!(5.0));
    let (ok, why) = rubric::pass(&s, &[]);
    assert!(!ok && why.iter().any(|w| w.contains("form")));
    let d = vec![json!({"defect": "floating", "present": true, "level": "blocking"})];
    assert!(!rubric::pass(&scores(9.0), &d).0);
    // the questions hold no pass line
    let q = format!("{}{}", rubric::questions("colour"), rubric::questions("clay"));
    assert!(!q.contains("mean") && !q.contains(">= 6"));
}

#[test]
fn answers_are_read_and_rounds_say_done_and_stall() {
    let a = json!({"score_form": {"score": 3.5}, "defect_floating": {"noul": 0.8}});
    let (s, d) = rubric::read(&a);
    assert_eq!(s["form"], 7.0);
    assert_eq!(d[0]["present"], true);
    let r = |p: bool, m: f64| json!({"pass": p, "mean": m, "defects": []});
    assert_eq!(progress(&[r(false, 5.0), r(true, 7.2), r(true, 7.4)])["done"], true);
    assert_eq!(progress(&[r(false, 5.0), r(false, 5.3), r(false, 5.6)])["stall"], true);
    assert_eq!(progress(&[r(false, 4.0), r(false, 5.3), r(false, 6.1)])["stall"], false);
}

#[test]
fn calibration_maps_a_lenient_judge_onto_the_eye() {
    // the judge says 8 to everything the eye gives 5, and 9 to what the eye gives 8
    let row = |j: f64, e: f64, p: bool| json!({"judge": scores(j), "eye": scores(e), "eye_pass": p, "defects": []});
    let rows: Vec<Value> = (0..12).map(|i| if i % 2 == 0 { row(8.0, 5.0, false) } else { row(9.0, 8.0, true) }).collect();
    let cal = eye::fit_rows(&rows);
    assert_eq!(cal["agreement"], 1.0);
    assert_eq!(cal["gate"], true);
    assert!((cal["maps"]["form"]["a"].as_f64().unwrap() - 3.0).abs() < 1e-6);
    let few = eye::fit_rows(&rows[..4]);
    assert_eq!(few["gate"], false, "too few rounds to gate");
    // an eye that failed everything agrees with a judge that fails everything, which proves nothing
    let fails: Vec<Value> = (0..12).map(|_| row(3.0, 2.0, false)).collect();
    let all_fail = eye::fit_rows(&fails);
    assert_eq!(all_fail["agreement"], 1.0);
    assert_eq!(all_fail["gate"], false, "no pass among the eye's rounds");
}

#[test]
fn an_agent_verdict_is_read_from_the_end_of_its_thinking() {
    // an agent thinks aloud, may draft the JSON twice, and ends with the one it means
    let text = r#"• Looking at the sheets... draft: {"scores": {"form": 1}} then the answer:
{"blind": {"place": "office", "objects": [], "photo": "no", "tells": []},
 "scores": {"composition": 6, "form": 5.5, "assembly": 7, "materials": 4, "light": 6, "detail": 5, "world": 6, "vibes": 9},
 "defects": [{"tile": "iso", "where": "left", "wrong": "the desks are plain boxes", "real": "a desk has legs", "fix": "model legs", "level": "blocking"},
             {"tile": "low", "wrong": "", "level": "detail"}],
 "show_as_is": "no"}"#;
    let v = crate::agent::last_verdict(text).unwrap();
    let (scores, defects, photo) = crate::agent::read(&v);
    assert_eq!(scores.len(), 7, "only the seven categories, each 0-10");
    assert_eq!(scores["form"], serde_json::json!(5.5));
    assert_eq!(defects.len(), 1, "a defect with nothing wrong named is dropped");
    assert_eq!(defects[0]["level"], "blocking");
    assert_eq!(photo, serde_json::json!(0.0));
    let (pass, why) = crate::rubric::pass(&scores, &defects);
    assert!(!pass && why[0].starts_with("blocking"));
    assert!(crate::agent::last_verdict("no json here").is_none());
}
