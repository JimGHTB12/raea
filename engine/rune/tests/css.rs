//! R2b: the style reader. Values it understands become typed values; anything else is reported with why.

use raea_rune_host::css::{self, Align, Len, parse};
use raea_rune_host::page::{Page, headless};
use serde_json::{Map, Value, json};

fn style(v: Value) -> Map<String, Value> {
    v.as_object().unwrap().clone()
}

#[test]
fn reads_what_react_sends() {
    let (c, bad) = parse(&style(json!({
        "display": "flex", "flexDirection": "column", "alignItems": "center", "justifyContent": "space-between",
        "width": 180, "height": "50%", "minHeight": 0, "padding": 6, "paddingLeft": "10px", "gap": 8,
        "borderWidth": 1, "borderTopWidth": 2, "borderRadius": 8, "lineHeight": 1.25, "fontWeight": "bold",
        "boxShadow": "0 4px 12px rgba(0,0,0,0.5), inset 1px 1px #fff",
        "backgroundImage": "linear-gradient(180deg, #f97316, #16181d 80%)"
    })));
    assert!(bad.is_empty(), "{bad:?}");
    assert_eq!((c.column, c.align_items, c.justify), (Some(true), Some(Align::Center), Some(Align::Between)));
    assert_eq!((c.width, c.height, c.min_height), (Some(Len::Px(180.0)), Some(Len::Pct(50.0)), Some(Len::Px(0.0))));
    assert_eq!(c.padding, [Some(6.0), Some(6.0), Some(6.0), Some(10.0)]);
    assert_eq!(c.border_width, [Some(2.0), Some(1.0), Some(1.0), Some(1.0)]);
    assert_eq!((c.line_height, c.font_weight), (Some(Len::Pct(125.0)), Some(700.0)));
    assert_eq!(c.shadows.len(), 2);
    assert!((c.shadows[0].color.a - 0.5).abs() < 1e-6 && c.shadows[1].inset && c.shadows[0].blur == 12.0);
    let (angle, stops) = c.gradient.unwrap();
    assert!((angle - std::f32::consts::PI).abs() < 1e-5);
    assert_eq!(stops.iter().map(|s| s.0).collect::<Vec<_>>(), vec![0.0, 0.8]);
}

#[test]
fn colours() {
    let w = css::color("#fff").unwrap();
    assert_eq!((w.r, w.a), (1.0, 1.0));
    assert_eq!(css::color("rgb(255 255 255 / 0.25)").unwrap().a, 0.25);
    assert_eq!(css::color("transparent").unwrap().a, 0.0);
    assert!(css::color("hsl(0 0% 0%)").is_none());
}

#[test]
fn reports_what_it_cant_map() {
    let (_, bad) = parse(&style(json!({"transform": "skewX(4deg)", "width": "calc(100% - 4px)", "flexDirection": "row-reverse", "backgroundImage": "url(a.png)"})));
    let mut got: Vec<(&str, &str)> = bad.iter().map(|u| (u.prop.as_str(), u.why)).collect();
    got.sort();
    assert_eq!(got.len(), 4, "{got:?}");
    assert!(got.iter().any(|(p, w)| *p == "transform" && w.contains("no Rune equivalent")));
    assert!(got.iter().any(|(p, w)| *p == "width" && w.contains("can't read")));
}

#[test]
fn the_page_reports_once_with_the_element_s_path() {
    let mut page = Page::default();
    page.apply(
        r#"{"v":1,"ops":[{"op":"create","id":1,"kind":"div","props":{"style":{"transform":"skewX(2deg)","width":"40%"}}},
        {"op":"create","id":2,"kind":"div","props":{"style":{"transform":"skewY(3deg)"},"title":"hi"}},
        {"op":"append","parent":0,"child":1},{"op":"append","parent":1,"child":2}]}"#,
    )
    .unwrap();
    let mut rt = headless(page, (400.0, 300.0));
    rt.frame(0.0);
    rt.frame(1.0 / 60.0);
    let mut r = rt.state.report.borrow_mut();
    let all: Vec<(String, String)> = r.all().map(|(p, s)| (p.to_string(), s.path.clone())).collect();
    assert!(all.contains(&("transform".into(), "/div[0]".into())), "{all:?}");
    assert!(all.contains(&("width".into(), "/div[0]".into())), "{all:?}");
    assert!(all.contains(&("title".into(), "/div[0]/div[0]".into())), "{all:?}");
    assert_eq!(all.iter().filter(|(p, _)| p == "transform").count(), 1, "said once per page");
    assert_eq!(r.take_fresh().len(), all.len());
}
