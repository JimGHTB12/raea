//! A light's power in Bevy's units (docs/control.feature FC1f): a Bevy light's own field as written, else a 3d.tsx
//! light's relative intensity times the scale.

use raea_core_scene::{AMBIENT, LUMENS, LUX, light_power};
use serde_json::json;

#[test]
fn own_units_win_over_intensity() {
    // the 3d.tsx default intensity of 1 rides along; the Bevy field is what counts
    assert_eq!(light_power("directional", json!({ "lux": 100000.0, "intensity": 1.0 }).as_object().unwrap()), 100000.0);
    assert_eq!(light_power("point", json!({ "lumens": 123456.0, "intensity": 1.0 }).as_object().unwrap()), 123456.0);
    assert_eq!(light_power("spot", json!({ "lumens": 0.0, "intensity": 1.0 }).as_object().unwrap()), 0.0);
    assert_eq!(light_power("rect", json!({ "lumens": 900.0 }).as_object().unwrap()), 900.0);
    assert_eq!(light_power("ambient", json!({ "nits": 77.0, "intensity": 1.0 }).as_object().unwrap()), 77.0);
}

#[test]
fn intensity_is_relative_without_them() {
    assert_eq!(light_power("directional", json!({ "intensity": 2.0 }).as_object().unwrap()), 2.0 * LUX);
    assert_eq!(light_power("point", json!({}).as_object().unwrap()), LUMENS);
    assert_eq!(light_power("ambient", json!({ "intensity": 0.5 }).as_object().unwrap()), 0.5 * AMBIENT);
    // a directional light ignores a stray lumens
    assert_eq!(light_power("directional", json!({ "lumens": 5.0 }).as_object().unwrap()), LUX);
}
