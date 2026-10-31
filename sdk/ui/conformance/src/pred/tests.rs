use super::*;

fn tree() -> Value {
    json!({"frame": 10, "window": {"w": 800, "h": 600}, "nodes": [
        {"id": "feed.list", "role": "list", "rect": {"x": 0, "y": 40, "w": 800, "h": 560}, "visible": true, "scroll": {"top": 0, "max": 900}},
        {"id": "feed.card.0", "role": "button", "rect": {"x": 20, "y": 50, "w": 300, "h": 200}, "visible": true, "hovered": true},
        {"id": "feed.card.1", "role": "button", "rect": {"x": 20, "y": 500, "w": 300, "h": 200}, "visible": true},
        {"id": "feed.search", "role": "searchbox", "rect": {"x": 600, "y": 8, "w": 180, "h": 26}, "visible": true, "value": "ab"},
        {"id": "header.toggle.chat", "role": "button", "rect": {"x": 560, "y": 8, "w": 30, "h": 26}, "visible": true}
    ]})
}

#[test]
fn globs() {
    assert!(glob("feed.card.*", "feed.card.12"));
    assert!(!glob("feed.card.*", "feed.search"));
    assert!(glob("*.badge", "left.feed.intro.badge"));
    assert!(glob("a*b*c", "aXXbYYc"));
    assert!(!glob("a*b*c", "aXXbYY"));
}

#[test]
fn node_fields() {
    let s = Snaps::new();
    assert!(holds(&json!({"id": "feed.card.0", "hovered": true}), &tree(), &s).is_ok());
    assert!(holds(&json!({"id": "feed.card.*", "hovered": true, "count": 1}), &tree(), &s).is_ok());
    assert!(holds(&json!({"id": "review.back", "exists": false}), &tree(), &s).is_ok());
    // a glob: exists = false means no node matches it, exists = true that one does
    assert!(holds(&json!({"id": "chat.message.*", "exists": false}), &tree(), &s).is_ok());
    let e = holds(&json!({"id": "feed.card.*", "exists": false}), &tree(), &s).unwrap_err();
    assert!(e.contains("feed.card.0"), "{e}");
    assert!(holds(&json!({"id": "feed.card.*", "exists": true}), &tree(), &s).is_ok());
    let e = holds(&json!({"id": "chat.message.*", "exists": true}), &tree(), &s).unwrap_err();
    assert!(e.contains("chat.message.*: not in the tree"), "{e}");
    assert!(holds(&json!({"id": "feed.card.0", "exists": false}), &tree(), &s).is_err());
    assert!(holds(&json!({"id": "review.back", "visible": false}), &tree(), &s).is_ok());
    let e = holds(&json!({"id": "feed.card.1", "hovered": true}), &tree(), &s).unwrap_err();
    assert!(e.contains("hovered is null") || e.contains("hovered is false"), "{e}");
    assert!(holds(&json!({"id": "feed.search", "field": "rect.w", "ge": 24}), &tree(), &s).is_ok());
    assert!(holds(&json!({"id": "feed.search", "value_contains": "a"}), &tree(), &s).is_ok());
}

#[test]
fn relative() {
    let mut s = Snaps::new();
    s.insert("last".into(), tree());
    let mut t = tree();
    t["nodes"][0]["scroll"]["top"] = json!(120);
    assert!(holds(&json!({"id": "feed.list", "field": "scroll.top", "increased_by": 100}), &t, &s).is_ok());
    assert!(holds(&json!({"id": "feed.list", "field": "scroll.top", "increased_by": 200}), &t, &s).is_err());
    // capped at the room left
    t["nodes"][0]["scroll"]["top"] = json!(900);
    assert!(holds(&json!({"id": "feed.list", "field": "scroll.top", "increased_by": 5000}), &t, &s).is_ok());
    assert!(holds(&json!({"id": "feed.list", "field": "scroll.top", "eq_field": "scroll.max"}), &t, &s).is_ok());
    assert!(holds(&json!({"id": "feed.card.0", "field": "rect", "unchanged": true}), &t, &s).is_ok());
}

#[test]
fn checks() {
    let s = Snaps::new();
    // card 1 runs past the window's bottom, but it's scroll content: on screen enough
    assert!(holds(&json!({"check": "on_screen"}), &tree(), &s).is_ok());
    assert!(holds(&json!({"check": "no_overlap"}), &tree(), &s).is_ok());
    let e = holds(&json!({"check": "min_size"}), &tree(), &s);
    assert!(e.is_ok(), "{e:?}");
    let mut t = tree();
    t["nodes"][4]["rect"]["x"] = json!(590);
    let e = holds(&json!({"check": "no_overlap"}), &t, &s).unwrap_err();
    assert!(e.contains("feed.search") && e.contains("header.toggle.chat"), "{e}");
    t["nodes"][4]["rect"]["w"] = json!(20);
    assert!(holds(&json!({"check": "min_size"}), &t, &s).is_err());
    t["nodes"][3]["rect"]["x"] = json!(700);
    assert!(holds(&json!({"check": "on_screen"}), &t, &s).unwrap_err().contains("feed.search"));
    assert!(holds(&json!({"check": "inside", "id": "feed.card.0", "within": "feed.list"}), &tree(), &s).is_ok());
}
