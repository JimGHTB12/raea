use super::*;

#[test]
fn declarations_match_the_committed_file() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../sdk/raea/src/brief.gen.ts");
    let now = declarations();
    if std::env::var_os("RAEA_WRITE").is_some() {
        std::fs::write(path, &now).unwrap();
    }
    let committed = std::fs::read_to_string(path).unwrap_or_default();
    assert!(committed == now, "sdk/raea/src/brief.gen.ts is out of date: RAEA_WRITE=1 cargo test -p raea-core-brief");
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

const DELIVERY: &str = r#"{"title": "Delivery", "help": "What every final keeps", "groups": [{"title": "Format", "fields": [
    {"label": "Aspect ratio", "type": "choice", "options": ["9:16", "4:5", "1:1"], "value": "9:16"},
    {"label": "Loudness", "type": "number", "value": -14, "unit": "LUFS", "min": -30, "max": 0},
    {"label": "Burn in captions", "type": "toggle", "value": true},
    {"label": "Steps", "type": "workflow", "value": [{"name": "Cut", "by": "@cutter"}, {"name": "Review", "by": "you", "back": "Cut"}]}]}]}"#;

#[test]
fn a_controller_fills_ids_from_titles_and_labels() {
    let c = controller::normalize(&serde_json::from_str(DELIVERY).unwrap()).unwrap();
    assert_eq!(c.id, "delivery");
    assert_eq!(c.groups[0].id, "format");
    let ids: Vec<_> = c.groups[0].fields.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(ids, ["aspect-ratio", "loudness", "burn-in-captions", "steps"]);
    let controller::Kind::Workflow { value, .. } = &c.groups[0].fields[3].kind else { panic!("a workflow") };
    assert_eq!(value[0].by.as_deref(), Some("cutter"));
    // what it writes back reads as the same controller (the page's types are these)
    let again = controller::normalize(&serde_json::to_value(&c).unwrap()).unwrap();
    assert_eq!(again, c);
}

#[test]
fn a_controller_says_what_is_wrong_in_a_sentence_the_agent_can_act_on() {
    let bad = |j: &str| controller::normalize(&serde_json::from_str(j).unwrap()).unwrap_err();
    assert!(bad(r#"{"title": "X", "groups": [{"title": "G", "fields": [{"label": "A", "type": "slider"}]}]}"#).contains("type must be one of"));
    assert!(bad(r#"{"title": "X", "groups": [{"title": "G", "fields": [{"label": "A", "type": "choice", "options": ["one"]}]}]}"#).contains("at least two options"));
    assert!(bad(r#"{"title": "X"}"#).contains("needs groups"));
    assert!(bad(r#"{"groups": []}"#).contains("needs a title"));
    assert!(bad(r#"{"title": "X", "groups": [{"title": "G", "fields": [{"label": "A", "type": "toggle"}, {"label": "a", "type": "toggle"}]}]}"#).contains("two fields are called a"));
}

#[test]
fn a_value_is_checked_against_its_field() {
    let c = controller::normalize(&serde_json::from_str(DELIVERY).unwrap()).unwrap();
    let cs = [c];
    let f = |p: &str| controller::field(&cs, p).unwrap();
    assert!(controller::check_value(f("delivery.format.loudness"), &serde_json::json!(-16)).is_ok());
    assert!(controller::check_value(f("delivery.format.loudness"), &serde_json::json!(4)).unwrap_err().contains("between -30 and 0"));
    assert!(controller::check_value(f("delivery.format.aspect-ratio"), &serde_json::json!("16:9")).unwrap_err().contains("one of: 9:16, 4:5, 1:1"));
    assert!(controller::check_value(f("delivery.format.burn-in-captions"), &serde_json::json!("yes")).is_err());
    assert!(controller::field(&cs, "delivery.format.nope").is_none());
}
