//! The accessibility check (docs/ts-native.feature P16): every data-testid's node in the accessibility tree, as a
//! canonical record, compared key for key. Chrome's side is CDP's Accessibility.getPartialAXTree for each element;
//! Bevy's is raea-page-host's `page.a11y`, the AccessKit tree read through accesskit_consumer (systems/a11y),
//! which writes the same record.
//!
//! The record: `role` (Chrome's role names; "hidden" for a node out of the tree: display none, visibility hidden,
//! aria-hidden; "generic" for one that's in it only for its content: a plain div, role=none, an ignored element),
//! `name` (whitespace collapsed; none for a generic), and the states and properties that say something: checked,
//! pressed (true, false, mixed), expanded, selected, level, the value of a range or a field, valuemin and valuemax (of
//! a range), multiline (of a textbox), haspopup, and disabled, focusable, focused, modal, required, readonly,
//! invalid and multiselectable when true. What isn't compared (on purpose): Chrome's settable, editable, orientation,
//! url, live regions' politeness, and its text and list-marker nodes (they're names and content, not elements).

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

pub type Tree = BTreeMap<String, Value>;

/// Roles that report a value and a range.
const RANGES: &[&str] = &["slider", "progressbar", "scrollbar", "meter"];
/// Roles whose value is compared.
const VALUED: &[&str] = &["slider", "progressbar", "scrollbar", "meter", "spinbutton", "textbox", "searchbox", "combobox"];
/// Ignored for one of these, a node is out of the tree; for any other reason it's there for its content (generic).
const HIDDEN_BECAUSE: &[&str] = &["notRendered", "notVisible", "ariaHiddenElement", "ariaHiddenSubtree", "inertElement", "inertSubtree", "activeModalDialog"];

pub fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One of Chrome's AXNodes as the canonical record.
pub fn chrome_record(n: &Value) -> Value {
    if n["ignored"] == json!(true) {
        let hidden = n["ignoredReasons"].as_array().into_iter().flatten().any(|r| HIDDEN_BECAUSE.contains(&r["name"].as_str().unwrap_or("")));
        return json!({"role": if hidden { "hidden" } else { "generic" }});
    }
    let mut role = n["role"]["value"].as_str().unwrap_or("").to_string();
    if role == "none" {
        role = "generic".into();
    }
    let mut r = Map::new();
    let props: BTreeMap<&str, &Value> = n["properties"].as_array().into_iter().flatten().filter_map(|p| Some((p["name"].as_str()?, &p["value"]["value"]))).collect();
    let truthy = |v: &Value| v == &json!(true) || v == &json!("true");
    for k in ["disabled", "focusable", "focused", "modal", "required", "readonly", "multiselectable", "invalid"] {
        if props.get(k).is_some_and(|v| truthy(v)) {
            r.insert(k.into(), json!(true));
        }
    }
    for k in ["checked", "pressed"] {
        if let Some(v) = props.get(k) {
            r.insert(k.into(), json!(v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string())));
        }
    }
    for k in ["expanded", "selected"] {
        if let Some(v) = props.get(k).and_then(|v| v.as_bool()) {
            r.insert(k.into(), json!(v));
        }
    }
    if let Some(v) = props.get("level").and_then(|v| v.as_f64()) {
        r.insert("level".into(), json!(v));
    }
    if let Some(v) = props.get("hasPopup").and_then(|v| v.as_str()) {
        r.insert("haspopup".into(), json!(v));
    }
    if RANGES.contains(&role.as_str()) {
        for k in ["valuemin", "valuemax"] {
            if let Some(v) = props.get(k).and_then(|v| v.as_f64()) {
                r.insert(k.into(), json!(v));
            }
        }
    }
    if role == "textbox" && let Some(v) = props.get("multiline").and_then(|v| v.as_bool()) {
        r.insert("multiline".into(), json!(v));
    }
    if VALUED.contains(&role.as_str()) && let Some(v) = n.get("value").map(|v| &v["value"]).filter(|v| !v.is_null()) {
        r.insert("value".into(), json!(value_str(v)));
    }
    let generic = role == "generic";
    let name = if generic { String::new() } else { collapse(n["name"]["value"].as_str().unwrap_or("")) };
    if generic && r.is_empty() {
        return json!({"role": "generic"});
    }
    r.insert("role".into(), json!(role));
    r.insert("name".into(), json!(name));
    Value::Object(r)
}

/// A value as compared: numbers without a trailing ".0", strings as they are.
pub fn value_str(v: &Value) -> String {
    match v {
        Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.fract() == 0.0 { format!("{}", f as i64) } else { format!("{f}") }
        }
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Every data-testid's backend node id in a CDP DOM.getDocument tree.
pub fn testids(n: &Value, out: &mut Vec<(String, u64)>) {
    let a = n["attributes"].as_array().cloned().unwrap_or_default();
    for p in a.chunks(2) {
        if p[0] == json!("data-testid") && let (Some(id), Some(b)) = (p.get(1).and_then(Value::as_str), n["backendNodeId"].as_u64()) {
            out.push((id.to_string(), b));
        }
    }
    for c in n["children"].as_array().into_iter().flatten() {
        testids(c, out);
    }
}

/// The differences between the oracle's tree and the tree under test, one line each.
pub fn diff(a: &Tree, b: &Tree) -> Vec<String> {
    let mut out = vec![];
    for (id, ra) in a {
        let Some(rb) = b.get(id) else {
            out.push(format!("{id}: a11y missing (chrome {ra})"));
            continue;
        };
        if ra != rb {
            let (oa, ob) = (ra.as_object().cloned().unwrap_or_default(), rb.as_object().cloned().unwrap_or_default());
            let mut keys: Vec<&String> = oa.keys().chain(ob.keys()).collect();
            keys.sort();
            keys.dedup();
            let parts: Vec<String> = keys.into_iter().filter(|k| oa.get(*k) != ob.get(*k)).map(|k| format!("{k} chrome {} bevy {}", oa.get(k).map(Value::to_string).unwrap_or("-".into()), ob.get(k).map(Value::to_string).unwrap_or("-".into()))).collect();
            out.push(format!("{id}: a11y {}", parts.join(", ")));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records() {
        let n = json!({"ignored": false, "role": {"value": "button"}, "name": {"value": "Save  now "}, "properties": [{"name": "focusable", "value": {"value": true}}, {"name": "invalid", "value": {"value": "false"}}, {"name": "pressed", "value": {"value": "true"}}]});
        assert_eq!(chrome_record(&n), json!({"role": "button", "name": "Save now", "focusable": true, "pressed": "true"}));
        let h = json!({"ignored": true, "ignoredReasons": [{"name": "ariaHiddenElement"}]});
        assert_eq!(chrome_record(&h), json!({"role": "hidden"}));
        let g = json!({"ignored": true, "ignoredReasons": [{"name": "uninteresting"}]});
        assert_eq!(chrome_record(&g), json!({"role": "generic"}));
        let s = json!({"ignored": false, "role": {"value": "slider"}, "name": {"value": "vol"}, "value": {"value": 3}, "properties": [{"name": "valuemin", "value": {"value": 0}}, {"name": "valuemax", "value": {"value": 10}}]});
        assert_eq!(chrome_record(&s), json!({"role": "slider", "name": "vol", "value": "3", "valuemin": 0.0, "valuemax": 10.0}));
    }
}
