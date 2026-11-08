//! fonts.families (Bevy's ui/text/system_fonts): the system's families, sorted, each once.

#[test]
fn families_are_the_systems_sorted_and_each_once() {
    let v = raea_runtime_calls::call("fonts.families", &serde_json::json!({})).unwrap();
    let names: Vec<&str> = v.as_array().unwrap().iter().map(|n| n.as_str().unwrap()).collect();
    assert!(!names.is_empty(), "no system families");
    assert!(names.windows(2).all(|w| w[0] < w[1]), "not sorted and unique");
    #[cfg(target_vendor = "apple")]
    assert!(names.contains(&"Helvetica"), "no Helvetica among {} families", names.len());
}
