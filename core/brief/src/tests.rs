use super::*;

#[test]
fn declarations_match_the_committed_file() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/brief.d.ts");
    let now = declarations();
    if std::env::var_os("RAEA_WRITE").is_some() {
        std::fs::write(path, &now).unwrap();
    }
    let committed = std::fs::read_to_string(path).unwrap_or_default();
    assert!(committed == now, "core/brief/brief.d.ts is out of date: RAEA_WRITE=1 cargo test -p raea-core-brief");
}

#[test]
fn a_brief_reads_and_its_doctor_asks_for_what_is_missing() {
    let b: Brief = read("brief.ts", r#"{"title": "A walkthrough", "mode": ["explain"], "assumed": ["voice"], "voice": "one calm narrator",
        "deliver": [{"name": "Walkthrough", "specs": ["60 s", "16:9"]}],
        "stages": [{"name": "", "tasks": [{"text": "Storyboard it", "agents": ["director"], "mark": "done"}]}]}"#).unwrap();
    assert_eq!(b.stages[0].tasks[0].mark, brief::Mark::Done);
    let d = brief::doctor(&b);
    assert!(d.iter().any(|m| m.contains("`believe`")), "{d:?}");
    assert!(d.iter().any(|m| m.contains("no script")), "{d:?}");
    assert!(d.iter().any(|m| m.contains("`voice` is assumed")), "{d:?}");
}

#[test]
fn a_misspelt_field_is_an_error_that_names_it() {
    let e = read::<Brief>("brief.ts", r#"{"title": "x", "belive": "y"}"#).unwrap_err();
    assert!(e.contains("belive"), "{e}");
}

#[test]
fn a_board_and_a_screenplay_are_one_script() {
    let board: Script = read("script.ts", r#"{"title": "Walkthrough", "scenes": [
        {"beats": [{"onScreen": "the brief page, a cursor typing one line", "lines": [{"who": "narrator", "says": "It starts with one line."}]}], "payoff": "a brief exists"}]}"#).unwrap();
    let play: Script = read("script.ts", r#"{"title": "The night before", "scenes": [
        {"heading": "INT. SENATE - NIGHT", "beats": [{"onScreen": "Torches gutter.", "camera": "push in, keeps moving", "cut": "match on motion",
          "lines": [{"who": "CASSIUS", "says": "Tomorrow.", "how": "quietly"}], "cues": ["strings drop out"], "onWord": "Tomorrow"}]}]}"#).unwrap();
    assert_eq!(board.lines().next().unwrap().who, "narrator");
    assert_eq!(play.lines().next().unwrap().how.as_deref(), Some("quietly"));
}

#[test]
fn taste_checks_are_typed_values() {
    let t: Taste = read("motion.ts", r#"{"title": "Taste: motion", "sections": [{"heading": "Picture", "rules": [
        {"text": "Text sits well clear of the bottom of the frame.", "check": {"form": "text-above", "value": 54}},
        {"text": "The look is neutral and bright."}]}]}"#).unwrap();
    let c: Vec<_> = t.checks().collect();
    assert_eq!(c.len(), 1);
    assert_eq!((c[0].1.form(), c[0].1.value()), ("text-above", 54.0));
    assert!(read::<Taste>("motion.ts", r#"{"title": "t", "sections": [{"heading": "h", "rules": [{"text": "x", "check": {"form": "glow-max", "value": 1}}]}]}"#).is_err());
}

#[test]
fn a_plan_leaves_nothing_without_a_material() {
    let ok: Plan = read("plan.ts", r#"{"concept": {"what": "flight feather", "style": "realistic", "size": 0.25, "seenFrom": 0.3, "material": "keratin"},
        "parts": {
          "rachis": {"role": "shaft", "level": "primary"},
          "vane": {"role": "vane", "level": "secondary", "parent": "rachis", "count": 2, "style": "barred brown"},
          "barb": {"role": "barb", "level": "tertiary", "parent": "vane", "count": {"min": 300, "max": 500}, "near": "separate barbs, a few split gaps"}}}"#).unwrap();
    assert_eq!(plan::check(&ok), Vec::<String>::new());
    assert_eq!(plan::resolve(&ok, "barb").unwrap(), plan::Resolved { material: "keratin".into(), style: "barred brown".into() });

    let mut bad = ok.clone();
    bad.concept.material = None;
    bad.parts.get_mut("rachis").unwrap().parent = Some("calamus".into());
    let e = plan::check(&bad);
    assert!(e.iter().any(|m| m.contains("`calamus` isn't planned")), "{e:?}");
    assert!(e.iter().any(|m| m.contains("no root")), "{e:?}");
}

#[test]
fn a_close_view_needs_tertiary_detail() {
    let p: Plan = read("plan.ts", r#"{"concept": {"what": "toy rocking horse", "style": "painted wood", "size": 0.9, "seenFrom": 0.5, "material": "beech"},
        "parts": {"body": {"role": "body", "level": "primary"}, "rocker": {"role": "rocker", "level": "secondary", "parent": "body", "count": 2}}}"#).unwrap();
    assert!(plan::check(&p).iter().any(|m| m.contains("tertiary")));
}
