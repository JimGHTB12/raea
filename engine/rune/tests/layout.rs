//! R5a's host fixes, each against the number Chrome gives for the same page (target/parity, js/parity/run.ts): a
//! text's width unrounded, the strut under inline content, and boxes carried by their scroll viewports.

use raea_rune_host::page::Page;
use raea_rune_host::session::Session;

fn page(ops: &str) -> Session {
    let mut s = Session::new(Page::default(), (800.0, 600.0));
    s.apply(&format!(r#"{{"v":1,"ops":[{ops}]}}"#)).unwrap();
    s.frame();
    s
}

#[test]
fn a_box_around_a_text_has_the_text_s_unrounded_width() {
    // Chrome: 125.33 (Inter 600 at 32 px), centred in 360 at 117.34; Rune's own measure rounds it to 126
    let s = page(r#"{"op":"create","id":9,"kind":"div","props":{"style":{"display":"flex","flexDirection":"column","alignItems":"center","width":360}}},
        {"op":"create","id":1,"kind":"span","props":{"style":{"fontSize":32,"fontWeight":600}}},{"op":"create","id":2,"kind":"text","props":{},"text":"Count: 1"},
        {"op":"append","parent":1,"child":2},{"op":"append","parent":9,"child":1},{"op":"append","parent":0,"child":9}"#);
    let r = s.rect(1).unwrap();
    assert!((r[2] - 125.33).abs() < 0.01 && (r[0] - 117.34).abs() < 0.01, "{r:?}");
}

#[test]
fn a_text_given_its_width_still_wraps_where_it_must() {
    let s = page(r#"{"op":"create","id":1,"kind":"div","props":{"style":{"width":80}}},{"op":"create","id":2,"kind":"text","props":{},"text":"a long line that wraps here"},
        {"op":"append","parent":1,"child":2},{"op":"append","parent":0,"child":1}"#);
    assert_eq!(s.rect(1).unwrap()[3], 72.0, "three lines of 24");
}

#[test]
fn inline_text_in_a_block_stands_on_the_block_s_strut() {
    // Chrome: a 14 px span (line 21) in a 16 px block (line 24) makes a 24 pt line box, so the bubble is 24 + 16
    let s = page(r#"{"op":"create","id":1,"kind":"div","props":{"style":{"paddingTop":8,"paddingBottom":8}}},{"op":"create","id":2,"kind":"span","props":{"style":{"fontSize":14}}},
        {"op":"create","id":3,"kind":"text","props":{},"text":"Did the render finish?"},{"op":"append","parent":2,"child":3},{"op":"append","parent":1,"child":2},{"op":"append","parent":0,"child":1}"#);
    assert_eq!(s.rect(1).unwrap()[3], 40.0);
}

#[test]
fn a_scrolled_element_s_box_is_where_it_is_drawn() {
    let mut rows = String::new();
    for i in 0..20 {
        rows.push_str(&format!(r#",{{"op":"create","id":{},"kind":"div","props":{{"style":{{"height":30}}}}}},{{"op":"append","parent":1,"child":{}}}"#, 10 + i, 10 + i));
    }
    let mut s = page(&format!(r#"{{"op":"create","id":1,"kind":"div","props":{{"style":{{"height":100,"overflowY":"scroll"}}}}}},{{"op":"append","parent":0,"child":1}}{rows},{{"op":"listen","id":1,"events":["wheel"]}}"#));
    s.wheel(50.0, 50.0, 90.0);
    for _ in 0..30 {
        s.frame();
    }
    assert_eq!(s.rect(13).unwrap()[1], 0.0, "the fourth row at the top after 90 pt");
    assert_eq!(s.at(50.0, 15.0), Some(13), "and the pointer finds it there");
}
