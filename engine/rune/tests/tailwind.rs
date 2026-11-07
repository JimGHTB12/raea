//! R2c: Tailwind utilities resolve with Tailwind's own theme into the style props css.rs reads; inline style wins;
//! utilities the host doesn't know are reported.

use raea_rune_host::css::{Len, parse};
use raea_rune_host::page::{Page, headless};
use raea_rune_host::tailwind::style_in;
use raea_rune_host::variants::State;

fn style(class: &str) -> (serde_json::Map<String, serde_json::Value>, Vec<String>) {
    style_in(class, &State::default())
}
use serde_json::json;

#[test]
fn utilities_resolve_with_the_theme() {
    let (s, unknown) = style("flex flex-col items-center justify-between gap-4 px-3 py-1.5 w-1/2 h-full max-w-sm rounded-lg shadow-md bg-blue-500/50 text-sm font-semibold text-white border-b-2 border-gray-200 flex-1 overflow-y-auto");
    assert!(unknown.is_empty(), "{unknown:?}");
    assert_eq!(s["gap"], json!(16.0));
    assert_eq!((s["paddingLeft"].clone(), s["paddingTop"].clone()), (json!(12.0), json!(6.0)));
    assert_eq!(s["width"], json!("50%"));
    assert_eq!(s["maxWidth"], json!(384.0));
    assert_eq!((s["fontSize"].clone(), s["fontWeight"].clone()), (json!(14.0), json!(600.0)));
    assert_eq!(s["borderRadius"], json!(8.0));
    assert_eq!(s["borderBottomWidth"], json!(2.0));
    let (c, bad) = parse(&s);
    assert!(bad.is_empty(), "every prop the classes set is one css.rs reads: {bad:?}");
    assert!((c.background.unwrap().a - 0.5).abs() < 1e-6, "the /50 modifier");
    assert_eq!(c.line_height, Some(Len::Pct(1.25 / 0.875 * 100.0)));
    assert_eq!(c.shadows.len(), 2);
}

#[test]
fn unknown_utilities_come_back() {
    let (_, unknown) = style("group-hover:bg-red-500 first:flex -mt-2 !p-4 space-x-2 tracking-wide p-4");
    assert_eq!(unknown, ["group-hover:bg-red-500", "first:flex", "-mt-2", "!p-4", "space-x-2", "tracking-wide"]);
}

#[test]
fn inline_style_wins_and_the_rest_is_reported() {
    let mut page = Page::default();
    page.apply(
        r#"{"v":1,"ops":[{"op":"create","id":1,"kind":"div","props":{"className":"flex pl-8 p-2 w-10 group-hover:bg-red-500","style":{"padding":4,"width":30}}},
        {"op":"create","id":2,"kind":"div","props":{"className":"size-4"}},{"op":"append","parent":0,"child":1},{"op":"append","parent":1,"child":2}]}"#,
    )
    .unwrap();
    let mut rt = headless(page, (400.0, 300.0));
    rt.frame(0.0);
    let rect = |k: &str| rt.elements().into_iter().find(|e| e.key.as_deref() == Some(k)).unwrap().target;
    // the inline padding (4) overrides the classes' pl-8 and p-2; the inline width (30) the class's w-10
    assert_eq!(rect("w2"), [4.0, 4.0, 16.0, 16.0]);
    assert_eq!(rect("w1")[2], 30.0);
    let r = rt.state.report.borrow();
    assert!(r.all().any(|(p, s)| p == "className" && s.value == "group-hover:bg-red-500"));
}
