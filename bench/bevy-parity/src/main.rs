//! Bevy's own examples with the parity probe (docs/control.feature FC0a). RAEA_PARITY_EXAMPLE=stress_tests/many_cubes
//! picks one; the rest of the command line is the example's own (`-- --benchmark --instance-count 10000`).
//! tools/bevy-parity runs it beside the example's TSX port.

use bevy::prelude::*;

include!(concat!(env!("OUT_DIR"), "/registry.rs"));

/// The example's App: a fixed 60 fps clock (RAEA_PARITY_FPS) and the probe, before the example's own plugins.
pub fn parity_app() -> App {
    let mut app = App::new();
    let fps = std::env::var("RAEA_PARITY_FPS").ok().and_then(|f| f.parse().ok()).unwrap_or(60.0);
    raea_sys_parity::fixed_clock(&mut app, fps);
    raea_sys_parity::install(&mut app);
    app
}

fn main() {
    let ex = std::env::var("RAEA_PARITY_EXAMPLE").unwrap_or_default();
    if !run(&ex) {
        if BEVY_SRC.is_empty() {
            eprintln!("raea-bevy-parity: built without Bevy's examples: clone bevy v0.20.0 to BEVY_SRC (~/.cache/raea-vendor/bevy-0.20.0) and rebuild");
            std::process::exit(2);
        }
        eprintln!("raea-bevy-parity: RAEA_PARITY_EXAMPLE={ex:?} isn't in bench/bevy-parity/examples.list");
        std::process::exit(2);
    }
}
