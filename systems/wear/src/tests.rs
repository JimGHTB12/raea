use super::*;
use serde_json::json;

fn row(v: Value) -> BTreeMap<String, V> {
    let mut row = BTreeMap::new();
    columns(&mut row, v.as_object().unwrap());
    row
}

#[test]
fn a_prop_becomes_columns() {
    let r = row(json!({ "wear": { "amount": 0.7, "rust": 0.4, "chipColor": "#202020", "bogus": 1 } }));
    assert_eq!(r.len(), 3);
    assert!(matches!(r.get("wear_amount"), Some(V::N(n)) if *n == 0.7));
    assert!(matches!(r.get("wear_chip_color"), Some(V::S(s)) if s == "#202020"));
    assert!(row(json!({ "color": "#fff" })).is_empty());
}

#[test]
fn off_without_an_amount() {
    assert!(of(&BTreeMap::new()).is_none());
    assert!(of(&row(json!({ "wear": { "amount": 0 } }))).is_none());
}

#[test]
fn defaults_and_clamps() {
    let w = of(&row(json!({ "wear": { "amount": 3, "chips": -1, "scale": 0 } }))).unwrap();
    assert_eq!((w.p.amount, w.p.chips, w.p.scale, w.p.grimes), (1.0, 0.0, 0.08, 0.5));
    let red = of(&row(json!({ "wear": { "amount": 0.5, "rustColor": "#ff0000" } }))).unwrap();
    assert_eq!(red.p.rust, Vec4::new(1.0, 0.0, 0.0, 1.0));
}
