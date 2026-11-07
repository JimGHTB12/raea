//! R3a: Rune's input back over the wire, driven through Rune's own input paths on the R5 apps' recorded mounts (DOM
//! mode: the root listens for pointer events, some elements for enter and leave, fields send change).

use raea_rune_host::page::{Out, Page};
use raea_rune_host::session::Session;
use serde_json::Value;

fn mounted(app: &str) -> Session {
    let path = format!("{}/tests/recorded/{app}.json", env!("CARGO_MANIFEST_DIR"));
    let rec: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut s = Session::new(Page::default(), (800.0, 600.0));
    s.apply(rec["frames"][0]["batch"].as_str().unwrap()).unwrap();
    s.take_out();
    s
}

fn delivered(out: &[Out]) -> Vec<Value> {
    out.iter().filter_map(|o| if let Out::Deliver(m) = o { Some(serde_json::from_str(m).unwrap()) } else { None }).collect()
}

/// A press on `id`: what was sent, and its downs and ups as (event, element).
fn press(s: &mut Session, id: u64) -> (Vec<Value>, Vec<(String, u64)>) {
    s.click_el(id).unwrap();
    let msgs = delivered(&s.take_out());
    let kinds = msgs.iter().filter(|m| m["t"] == "event" && matches!(m["event"].as_str(), Some("pointerdown" | "pointerup")))
        .map(|m| (m["event"].as_str().unwrap().to_string(), m["id"].as_u64().unwrap())).collect();
    (msgs, kinds)
}

/// As raea's other host does: a press goes to the nearest element up from its target that listens, and only to it.
#[test]
fn a_press_goes_to_the_nearest_listener_at_its_client_point() {
    let mut s = mounted("counter");
    let inc = s.tid("inc").unwrap();
    let r = s.rect(inc).unwrap();
    let (msgs, kinds) = press(&mut s, inc);
    assert_eq!(kinds, [("pointerdown".to_string(), inc), ("pointerup".to_string(), inc)]);
    // the count doesn't listen for presses, so the root hears them
    let count = s.tid("count").unwrap();
    let (_, under) = press(&mut s, count);
    assert_eq!(under, [("pointerdown".to_string(), 0), ("pointerup".to_string(), 0)]);
    let down = msgs.iter().find(|m| m["event"] == "pointerdown").unwrap();
    let (cx, cy) = (down["payload"]["clientX"].as_f64().unwrap() as f32, down["payload"]["clientY"].as_f64().unwrap() as f32);
    assert!((cx - (r[0] + r[2] / 2.0)).abs() < 0.5 && (cy - (r[1] + r[3] / 2.0)).abs() < 0.5, "client ({cx}, {cy}) for {r:?}");
    assert_eq!(down["payload"]["detail"], 1);
}

#[test]
fn hover_enters_and_leaves() {
    let mut s = mounted("counter");
    let count = s.tid("count").unwrap();
    let r = s.rect(count).unwrap();
    s.pointer_move(r[0] + 2.0, r[1] + 2.0);
    s.pointer_move(790.0, 590.0);
    let msgs = delivered(&s.take_out());
    let mine: Vec<&str> = msgs.iter().filter(|m| m["id"] == count).map(|m| m["event"].as_str().unwrap()).collect();
    assert_eq!(mine, ["pointerenter", "pointerleave"]);
}

#[test]
fn typing_in_a_field_sends_its_value_and_the_keys() {
    let mut s = mounted("settings");
    let name = s.tid("name").unwrap();
    s.click_el(name).unwrap();
    assert_eq!(s.focused_field(), Some(name), "a press focuses the field");
    s.key(rune::Key::End);
    s.take_out();
    s.type_text(" B");
    let msgs = delivered(&s.take_out());
    let changes: Vec<&str> = msgs.iter().filter(|m| m["event"] == "change" && m["id"] == name).map(|m| m["payload"].as_str().unwrap()).collect();
    assert_eq!(changes, ["Shane ", "Shane B"]);
    // the field listens for keys, so they're its events (the bridge's hearer), not the host's
    let keys: Vec<(&str, &str)> = msgs.iter().filter(|m| m["id"] == name && m["event"].as_str().unwrap().starts_with("key")).map(|m| (m["event"].as_str().unwrap(), m["payload"]["key"].as_str().unwrap())).collect();
    assert_eq!(keys, [("keydown", " "), ("keyup", " "), ("keydown", "B"), ("keyup", "B")]);
    let first = msgs.iter().find(|m| m["event"] == "keydown").unwrap();
    assert_eq!(first["payload"]["value"], "Shane", "a key's payload carries the field's value before it");
}

#[test]
fn a_key_nothing_focused_hears_is_the_host_s() {
    let mut s = mounted("cards");
    s.key(rune::Key::Escape);
    let msgs = delivered(&s.take_out());
    let keys: Vec<(&str, &str)> = msgs.iter().filter(|m| m["t"] == "named").map(|m| (m["name"].as_str().unwrap(), m["value"]["key"].as_str().unwrap())).collect();
    assert_eq!(keys, [("keydown", "Escape"), ("keyup", "Escape")]);
}

#[test]
fn a_press_moves_the_focus_with_focus_and_blur() {
    let mut s = mounted("settings");
    let name = s.tid("name").unwrap();
    s.click_el(name).unwrap();
    let msgs = delivered(&s.take_out());
    assert!(msgs.iter().any(|m| m["event"] == "focus" && m["id"] == name), "focus to the field");
    // a press on the page's background: the field blurs with its value, the focus going nowhere
    s.click(790.0, 590.0);
    let msgs = delivered(&s.take_out());
    let blur = msgs.iter().find(|m| m["event"] == "blur").expect("a blur");
    assert_eq!((blur["id"].as_u64(), &blur["payload"]["value"], &blur["payload"]["related"]), (Some(name), &Value::from("Shane"), &Value::Null));
    assert_eq!(s.focused_field(), None, "Rune's focus follows");
}

#[test]
fn a_ref_s_focus_and_blur_move_it() {
    let mut s = mounted("settings");
    s.emit("focus", &Value::from("name"));
    let name = s.tid("name").unwrap();
    assert_eq!(s.focused_field(), Some(name));
    assert!(delivered(&s.take_out()).iter().any(|m| m["event"] == "focus" && m["id"] == name));
    s.emit("blur", &Value::Null);
    assert_eq!(s.focused_field(), None);
    assert!(delivered(&s.take_out()).iter().any(|m| m["event"] == "blur" && m["id"] == name));
}

#[test]
fn a_scroll_reports_its_offset() {
    let mut s = mounted("list");
    let sc = s.tid("scroller").unwrap();
    let r = s.rect(sc).unwrap();
    s.wheel(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0, 120.0);
    for _ in 0..30 {
        s.frame();
    }
    let msgs = delivered(&s.take_out());
    let last = msgs.iter().filter(|m| m["event"] == "scroll" && m["id"] == sc).last().expect("a scroll");
    let p = &last["payload"];
    assert!(p["scrollTop"].as_f64().unwrap() > 0.0, "{p}");
    assert!(p["scrollHeight"].as_f64().unwrap() > p["clientHeight"].as_f64().unwrap(), "{p}");
}

#[test]
fn a_resize_is_heard_and_laid_out() {
    let mut s = mounted("cards");
    s.resize(640.0, 480.0);
    let out = s.take_out();
    assert!(delivered(&out).iter().any(|m| m["name"] == "resize" && m["value"] == serde_json::json!({"w": 640.0, "h": 480.0})));
    let snap = out.iter().rev().find_map(|o| if let Out::Layout(l) = o { Some(serde_json::from_str::<Value>(l).unwrap()) } else { None }).unwrap();
    assert_eq!((snap["vw"].as_f64(), snap["vh"].as_f64()), (Some(640.0), Some(480.0)));
    assert_eq!(s.rect(0).unwrap()[2], 640.0);
}

/// A menu (raea-runtime-js's `menu` and `menuitem` roles): a click or Enter selects an item, Escape or a press
/// outside closes the menu.
#[test]
fn menus_select_and_close() {
    let batch = r#"{"v":1,"ops":[
        {"op":"create","id":1,"kind":"div","props":{"role":"menu","style":{"width":200,"height":80}}},
        {"op":"create","id":2,"kind":"div","props":{"role":"menuitem","tabIndex":-1,"style":{"height":40}}},
        {"op":"create","id":3,"kind":"div","props":{"role":"menuitem","tabIndex":-1,"style":{"height":40}}},
        {"op":"append","parent":0,"child":1},{"op":"append","parent":1,"child":2},{"op":"append","parent":1,"child":3},
        {"op":"listen","id":1,"events":["close"]},{"op":"listen","id":2,"events":["select"]},{"op":"listen","id":3,"events":["select"]}]}"#;
    let mut s = Session::new(Page::default(), (400.0, 300.0));
    s.apply(batch).unwrap();
    let ev = |out: Vec<Out>| delivered(&out).iter().filter(|m| m["t"] == "event").map(|m| (m["event"].as_str().unwrap().to_string(), m["id"].as_u64().unwrap())).collect::<Vec<_>>();
    s.click(100.0, 60.0);
    assert_eq!(ev(s.take_out()), [("select".to_string(), 3)]);
    s.emit("focus", &Value::from(2));
    s.key(rune::Key::Enter);
    assert_eq!(ev(s.take_out()), [("select".to_string(), 2)]);
    s.key(rune::Key::Escape);
    assert_eq!(ev(s.take_out()), [("close".to_string(), 1)]);
    s.click(300.0, 250.0);
    assert_eq!(ev(s.take_out()), [("close".to_string(), 1)]);
}

#[test]
fn a_controlled_field_follows_its_value() {
    let mut s = mounted("settings");
    let name = s.tid("name").unwrap();
    s.apply(&format!(r#"{{"v":1,"ops":[{{"op":"update","id":{name},"set":{{"value":"Ana"}}}}]}}"#)).unwrap();
    assert_eq!(s.rt.state.edits[&name].text, "Ana");
}

#[test]
fn a_wheel_reaches_an_element_that_listens() {
    let mut s = Session::new(Page::default(), (400.0, 300.0));
    s.apply(r#"{"v":1,"ops":[{"op":"create","id":1,"kind":"div","props":{"style":{"width":200,"height":100}}},{"op":"append","parent":0,"child":1},{"op":"listen","id":1,"events":["wheel"]}]}"#).unwrap();
    s.take_out();
    s.wheel(50.0, 50.0, 30.0);
    let msgs = delivered(&s.take_out());
    let w = msgs.iter().find(|m| m["event"] == "wheel").expect("a wheel event");
    assert_eq!((w["id"].as_u64(), w["payload"]["deltaY"].as_f64()), (Some(1), Some(30.0)));
}

#[test]
fn the_layout_snapshot_has_a_record_per_element() {
    let mut s = Session::new(Page::default(), (400.0, 300.0));
    s.apply(r#"{"v":1,"ops":[{"op":"create","id":1,"kind":"div","props":{"style":{"width":200,"height":100,"padding":7,"borderWidth":2}}},{"op":"append","parent":0,"child":1}]}"#).unwrap();
    let snap = s.take_out().into_iter().rev().find_map(|o| if let Out::Layout(j) = o { Some(j) } else { None }).expect("a snapshot after the batch");
    let v: Value = serde_json::from_str(&snap).unwrap();
    let n: Vec<f64> = v["n"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect();
    assert_eq!(n.len() % 18, 0);
    let rec = n.chunks(18).find(|r| r[0] == 1.0).unwrap();
    assert_eq!(&rec[1..5], &[0.0, 0.0, 200.0, 100.0]);
    assert_eq!(&rec[9..17], &[9.0, 9.0, 9.0, 9.0, 2.0, 2.0, 2.0, 2.0], "padding with the border, then the border");
    s.frame();
    // the frame still reaches `__ui_layout` (its rAF callbacks), with "" for an unchanged layout (T3c)
    assert!(!s.take_out().iter().any(|o| matches!(o, Out::Layout(j) if !j.is_empty())), "no snapshot when nothing moved");
}

#[test]
fn frames_tick_when_asked() {
    let mut s = Session::new(Page::default(), (100.0, 100.0));
    s.rt.state.ticking = true;
    s.frame();
    assert!(delivered(&s.take_out()).iter().any(|m| m["t"] == "named" && m["name"] == "frame"));
}

/// A drag as DOM mode follows one: the press, the root's moves all the way, the release where it ended.
#[test]
fn a_drag_s_moves_reach_the_root() {
    let mut s = mounted("cards");
    let card = s.tid("card-a").unwrap();
    let r = s.rect(card).unwrap();
    let from = (r[0] + 10.0, r[1] + 10.0);
    let to = (from.0 + 120.0, from.1 + 40.0);
    s.drag(from, to, 6);
    let msgs = delivered(&s.take_out());
    let seq: Vec<(&str, u64)> = msgs.iter().filter(|m| m["t"] == "event" && !matches!(m["event"].as_str(), Some("pointerenter" | "pointerleave")))
        .map(|m| (m["event"].as_str().unwrap(), m["id"].as_u64().unwrap())).collect();
    assert_eq!(seq.iter().filter(|e| e.0 == "pointerdown").count(), 1, "{seq:?}");
    let down = seq.iter().position(|e| e.0 == "pointerdown").unwrap();
    let up = seq.iter().position(|e| e.0 == "pointerup").unwrap();
    assert!(seq[down..up].iter().filter(|e| *e == &("pointermove", 0)).count() >= 6, "the moves between: {seq:?}");
    let last = msgs.iter().find(|m| m["event"] == "pointerup").unwrap();
    assert_eq!((last["payload"]["clientX"].as_f64(), last["payload"]["clientY"].as_f64()), (Some(to.0 as f64), Some(to.1 as f64)));
}
