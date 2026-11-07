//! R2: each app of the R5 set, at the end of its recorded session, drawn by Rune headless: every wire element is a
//! Rune element with a box, and what couldn't be mapped is reported, not dropped.

use raea_rune_host::page::{Page, headless};
use raea_rune_host::view::key;
use serde_json::Value;

const APPS: [&str; 5] = ["counter", "settings", "list", "cards", "chat"];

fn replayed(app: &str, upto: Option<usize>) -> Page {
    let path = format!("{}/tests/recorded/{app}.json", env!("CARGO_MANIFEST_DIR"));
    let rec: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut page = Page::default();
    for f in rec["frames"].as_array().unwrap().iter().take(upto.unwrap_or(usize::MAX)) {
        page.apply(f["batch"].as_str().unwrap()).unwrap();
    }
    page
}

#[test]
fn every_element_is_drawn_and_the_rest_reported() {
    for app in APPS {
        let mut rt = headless(replayed(app, None), (800.0, 600.0));
        rt.frame(0.0);
        let els = rt.elements();
        let page = &rt.state;
        let mut missing = Vec::new();
        let mut ids: Vec<u64> = Vec::new();
        collect(&page.tree, 0, &mut ids);
        for id in &ids {
            let n = page.tree.get(*id).unwrap();
            if n.kind == "text" {
                continue;
            }
            if !els.iter().any(|e| e.key.as_deref() == Some(key(*id).as_str())) {
                missing.push(format!("{} {}", n.kind, id));
            }
        }
        assert!(missing.is_empty(), "{app}: no Rune element for {missing:?}");
        let texts = els.iter().filter(|e| e.role == "text").count();
        eprintln!("{app}: {} wire elements, {} Rune elements ({texts} texts)", ids.len(), els.len());
        for (prop, s) in page.report.borrow().all() {
            eprintln!("  unmapped {prop}: {} at {} ({})", s.value, s.path, s.why);
        }
    }
}

fn collect(t: &raea_rune_host::tree::Tree, id: u64, out: &mut Vec<u64>) {
    out.push(id);
    for c in &t.get(id).unwrap().children {
        collect(t, *c, out);
    }
}

#[test]
fn an_image_draws_its_pixels_at_its_aspect() {
    // an http src is fetched through raea's URL cache: offline, one that isn't cached fails, and is reported
    // SAFETY: set before this test reads it; no other test here fetches
    unsafe { std::env::set_var("RAEA_OFFLINE", "1") };
    const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let mut page = Page::default();
    let batch = format!(
        r#"{{"v":1,"ops":[{{"op":"create","id":1,"kind":"img","props":{{"src":"data:image/png;base64,{PNG}","style":{{"width":40}}}}}},
        {{"op":"create","id":2,"kind":"img","props":{{"src":"https://example.com/a.png"}}}},{{"op":"append","parent":0,"child":1}},{{"op":"append","parent":0,"child":2}}]}}"#
    );
    page.apply(&batch).unwrap();
    let raea_rune_host::images::Load::Image(i) = page.images.get(&format!("data:image/png;base64,{PNG}")) else { panic!("the data URL loads") };
    i.wait().unwrap();
    let mut rt = headless(page, (400.0, 300.0));
    rt.frame(0.0);
    rt.frame(1.0 / 60.0);
    let r = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("w1")).unwrap().target;
    assert_eq!([r[2], r[3]], [40.0, 40.0], "the height from the image's 1:1 aspect");
    assert!(rt.state.report.borrow().all().any(|(p, s)| p == "img src" && s.value.starts_with("https://")), "an http src that doesn't load (offline) is reported");
}
