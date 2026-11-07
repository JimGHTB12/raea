//! A material's emissive power in Bevy's units (docs/control.feature FC1f): a Bevy StandardMaterial's `emissive_nits`
//! as written, else a 3d.tsx material's relative emissiveIntensity times EMISSIVE, so it glows as three.js's does.

use raea_core_scene::{EMISSIVE, emissive_power};
use serde_json::json;

#[test]
fn bevy_nits_as_written() {
    // bloom_3d's white spheres: LinearRgba 1000, whatever relative intensity rides along
    assert_eq!(emissive_power(json!({ "emissive_nits": 1000.0, "emissiveIntensity": 4.0 }).as_object().unwrap()), 1000.0);
    assert_eq!(emissive_power(json!({ "emissive_nits": 0.0 }).as_object().unwrap()), 0.0);
}

#[test]
fn relative_intensity_is_three_js_brightness() {
    assert_eq!(emissive_power(json!({ "emissiveIntensity": 4.0 }).as_object().unwrap()), 4.0 * EMISSIVE);
    assert_eq!(emissive_power(json!({}).as_object().unwrap()), EMISSIVE);
    // Bevy's default exposure shows EMISSIVE cd/m² as 1.0: 1.2 x 2^9.7
    assert!((EMISSIVE - 1.2 * 2f64.powf(9.7)).abs() < 0.1);
}
