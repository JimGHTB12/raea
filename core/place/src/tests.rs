use super::*;
use serde_json::json;

fn room() -> serde_json::Value {
    json!({
        "room": {"min": [-2, -2], "max": [2, 2]},
        "objects": [
            {"id": "desk", "size": [1.4, 0.75, 0.7]},
            {"id": "chair", "size": [0.5, 0.9, 0.5]},
            {"id": "lamp", "size": [0.2, 0.45, 0.2]},
            {"id": "bed", "size": [1.0, 0.5, 2.0]},
            {"id": "nightstand", "size": [0.45, 0.55, 0.4]},
            {"id": "wardrobe", "size": [1.2, 2.0, 0.6]}
        ],
        "rules": [
            {"against": "desk", "wall": "+z"},
            {"near": "chair", "of": "desk", "dist": 0.15},
            {"facing": "chair", "of": "desk"},
            {"on": "lamp", "of": "desk"},
            {"against": "bed", "wall": "-x"},
            {"beside": "nightstand", "of": "bed", "side": "+z", "gap": 0.05},
            {"against": "wardrobe", "wall": "+x"},
            {"clear": "wardrobe", "depth": 0.7},
            {"apart": "chair", "of": "bed", "min": 0.3}
        ]
    })
}

fn at(out: &serde_json::Value, id: &str) -> (Vec<f64>, f64) {
    let o = out["objects"].as_array().unwrap().iter().find(|o| o["id"] == id).unwrap();
    (o["at"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect(), o["yaw"].as_f64().unwrap())
}

#[test]
fn a_bedroom_is_laid_out_by_its_rules() {
    let out = solve_json(&room()).unwrap();
    let bad: Vec<&serde_json::Value> = out["found"].as_array().unwrap().iter().filter(|f| f["level"] == "error").collect();
    assert!(bad.is_empty(), "{bad:#?}");
    // the desk's back on the +z wall, facing into the room
    let (desk, yaw) = at(&out, "desk");
    assert!((desk[2] - 1.65).abs() < 2e-3 && (yaw.abs() - 180.0).abs() < 1e-6, "{desk:?} {yaw}");
    // the lamp stands on the desk's top
    let (lamp, _) = at(&out, "lamp");
    assert!((lamp[1] - 0.75).abs() < 1e-9);
    // the chair turned to the desk
    let (chair, cyaw) = at(&out, "chair");
    let want = (desk[0] - chair[0]).atan2(desk[2] - chair[2]).to_degrees();
    assert!((cyaw - want).abs() < 1.0, "{cyaw} vs {want}");
    // the same input, the same layout
    assert_eq!(solve_json(&room()).unwrap(), out);
}

#[test]
fn what_cannot_hold_is_found() {
    let v = json!({"room": {"min": [-2, -2], "max": [2, 2]},
        "objects": [{"id": "a", "size": [3, 1, 0.5]}, {"id": "b", "size": [3, 1, 0.5]}, {"id": "cup", "size": [0.5, 0.1, 0.5]}, {"id": "coaster", "size": [0.1, 0.01, 0.1]}],
        "rules": [{"against": "a", "wall": "+z"}, {"against": "b", "wall": "+z"}, {"on": "cup", "of": "coaster"}]});
    let out = solve_json(&v).unwrap();
    let checks: Vec<&str> = out["found"].as_array().unwrap().iter().filter(|f| f["level"] == "error").map(|f| f["check"].as_str().unwrap()).collect();
    assert!(checks.contains(&"overlap"), "{checks:?}");
    assert!(checks.contains(&"on"), "the cup overhangs the coaster: {checks:?}");
    assert!(solve_json(&json!({"objects": [{"id": "a", "size": [1, 1, 1]}], "rules": [{"on": "a", "of": "z"}]})).unwrap_err().contains("no object z"));
}
