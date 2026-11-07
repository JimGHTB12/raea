use super::*;

fn mug(handle_x: f64) -> Vec<(String, String)> {
    vec![
        ("body".into(), "move(rotate(minus(cylinder(0.045, 0.1), move(cylinder(0.041, 0.1), 0, 0, 0.004)), -90, 0, 0), 0, 0.05, 0)".into()),
        ("handle".into(), format!("move(rotate(torus(0.025, 0.006), 90, 0, 0), {handle_x}, 0.05, 0)")),
    ]
}

#[test]
fn a_mug_is_measured_and_a_detached_handle_is_an_error() {
    let ok = info(&mug(0.057), None).unwrap();
    assert_eq!(ok["errors"], 0, "{ok:#}");
    assert_eq!(ok["solid"]["pieces"], 1);
    assert_eq!(ok["solid"]["watertight"], true);
    assert_eq!(ok["solid"]["cavities"], 0);
    assert!(ok["parts"][0]["volume"].as_f64().unwrap() > 0.0, "a part's measures sit beside its name");
    let size = ok["solid"]["box"]["size"].as_array().unwrap();
    assert!((size[1].as_f64().unwrap() - 0.1).abs() < 1e-3, "{size:?}");
    assert_eq!(ok["pairs"][0]["status"], "overlap");
    // the handle moved off the body: two pieces, and the handle named
    let off = info(&mug(0.1), None).unwrap();
    assert_eq!(off["errors"], 1);
    let f = off["found"].as_array().unwrap().iter().find(|f| f["check"] == "disjoint").unwrap().clone();
    assert!(f["says"].as_str().unwrap().contains("handle"), "{f}");
    assert_eq!(off["pairs"][0]["status"], "apart");
}

#[test]
fn predictions_are_checked_and_loose_ones_let_mutants_survive() {
    // a cup: the inside is empty, the wall is solid
    let tight = serde_json::json!({"pieces": 1, "size": {"y": [0.099, 0.101]}, "solidPoints": [[0.043, 0.05, 0]], "emptyPoints": [[0, 0.06, 0]]});
    let r = info(&mug(0.057), Some(&tight)).unwrap();
    assert_eq!(r["errors"], 0, "{r:#}");
    assert_eq!(r["expect"]["failed"], 0);
    // the empty point inside the cup kills both the block and the hull
    assert!(r["mutants"].as_array().unwrap().iter().all(|m| m["status"] == "killed"), "{:#}", r["mutants"]);
    let loose = serde_json::json!({"pieces": 1});
    let r = info(&mug(0.057), Some(&loose)).unwrap();
    assert!(r["mutants"].as_array().unwrap().iter().all(|m| m["status"] == "survived"));
    assert!(r["warnings"].as_u64().unwrap() >= 2);
    // a wrong prediction is an error; an unknown key says what is accepted
    let wrong = serde_json::json!({"genus": 3});
    assert_eq!(info(&mug(0.057), Some(&wrong)).unwrap()["errors"], 1);
    assert!(info(&mug(0.057), Some(&serde_json::json!({"colour": 1}))).unwrap_err().contains("volume, area"));
}

#[test]
fn walks_and_deltas_say_what_moved() {
    let a = info(&mug(0.057), None).unwrap()["solid"].clone();
    let b = info(&mug(0.1), None).unwrap()["solid"].clone();
    assert_eq!(walk_verdict(&a, &b).0, "broken");
    assert_eq!(walk_verdict(&a, &a).0, "inert");
    let d = delta("handle=0.1", &a, &b);
    assert_eq!(d["pieces"]["now"], 2);
    // centre of mass of a unit box at the origin's corner
    let m = measure(&raea_core_shape::solid("move(box(1, 1, 1), 0.5, 0.5, 0.5)").unwrap());
    assert!(m.center_of_mass.iter().all(|c| (c - 0.5).abs() < 1e-6), "{:?}", m.center_of_mass);
}

#[test]
fn a_sealed_hollow_is_a_cavity() {
    let hollow = info(&[("ball".to_string(), "minus(sphere(1), sphere(0.5))".to_string())], None).unwrap();
    assert_eq!(hollow["solid"]["pieces"], 1);
    assert_eq!(hollow["solid"]["cavities"], 1, "{}", hollow["solid"]);
}
