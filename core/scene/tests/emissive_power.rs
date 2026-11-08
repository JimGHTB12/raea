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
    // a StandardMaterial's emissive skips the exposure (emissive_exposure_weight 0): three.js's 1 is Bevy's 1.0, not
    // 1.2 x 2^9.7 cd/m², which drew an emissiveIntensity of 0.25 white
    assert_eq!(EMISSIVE, 1.0);
}
