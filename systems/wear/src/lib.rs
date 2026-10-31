//! Worn surfaces: the standard material with use written over it. Paint mottled by noise, its edges chipped back to
//! `chip_color` (the metal under the paint, shinier), grime gathered low and on undersides, and rust run down in
//! streaks. The noise is taken in the mesh's own space, not the world's, so a walking robot's wear stays where it is
//! on each part instead of swimming over it; a part built in rest-pose coordinates (as a rigged build's pieces are)
//! shares one space with its neighbours, so their wear lines up across a seam. Edges are found from how fast the
//! surface's normal turns on screen against how fast its position does (its curvature, 1 / radius), so a rounded edge
//! of a few millimetres chips and a broad face doesn't.
//!
//! A mesh row's columns (all optional but wear_amount): wear_amount (0..1, how worn; on when > 0), wear_scale (metres
//! across a blotch, 0.08), wear_chips, wear_grime, wear_rust (each 0..1 of the amount: 0.6, 0.5, 0.25), wear_seed,
//! wear_chip_color ('#3a3c40'), wear_grime_color ('#3b3226'), wear_rust_color ('#7a3f1c'). `columns` makes them from
//! a material's `wear` prop (raea-runtime-react's scene3d); `of` reads them back where the engine spawns the mesh.

use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use raea_core_program::ir::V;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub type WearMaterial = ExtendedMaterial<StandardMaterial, Wear>;

pub fn install(app: &mut App) {
    bevy::asset::embedded_asset!(app, "wear.wesl");
    app.add_plugins(MaterialPlugin::<WearMaterial>::default());
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct Wear {
    #[uniform(100)]
    p: WearParams,
}

#[derive(ShaderType, Clone, Copy, Default, PartialEq, Debug)]
struct WearParams {
    chip: Vec4,
    grime: Vec4,
    rust: Vec4,
    amount: f32,
    scale: f32,
    chips: f32,
    grimes: f32,
    rusts: f32,
    seed: f32,
}

impl MaterialExtension for Wear {
    fn fragment_shader() -> ShaderRef {
        "embedded://raea_sys_wear/wear.wesl".into()
    }
}

/// The prop's keys and the columns they become; colours pass through as the hex the SDK checked
const NUMS: [(&str, &str); 6] =
    [("amount", "wear_amount"), ("scale", "wear_scale"), ("chips", "wear_chips"), ("grime", "wear_grime"), ("rust", "wear_rust"), ("seed", "wear_seed")];
const COLOURS: [(&str, &str); 3] = [("chipColor", "wear_chip_color"), ("grimeColor", "wear_grime_color"), ("rustColor", "wear_rust_color")];

/// A material's `wear` prop as its row's columns (the first frame's, as a grid's are)
pub fn columns(row: &mut BTreeMap<String, V>, props: &Map<String, Value>) {
    let Some(w) = props.get("wear").and_then(Value::as_object) else { return };
    for (k, col) in NUMS {
        if let Some(n) = w.get(k).and_then(Value::as_f64) {
            row.insert(col.into(), V::N(n));
        }
    }
    for (k, col) in COLOURS {
        if let Some(s) = w.get(k).and_then(Value::as_str) {
            row.insert(col.into(), V::S(s.into()));
        }
    }
}

/// The wear a mesh row asks for, or None (no wear_amount above 0)
pub fn of(vals: &BTreeMap<String, V>) -> Option<Wear> {
    let num = |col: &str| vals.get(col).map(|v| v.num() as f32);
    let amount = num("wear_amount").unwrap_or(0.0).min(1.0);
    if amount <= 0.0 {
        return None;
    }
    let rgb = |col: &str, d: Srgba| {
        let c = match vals.get(col) {
            Some(V::S(s)) => Srgba::hex(s.trim()).unwrap_or(d),
            _ => d,
        };
        let l = LinearRgba::from(c);
        Vec4::new(l.red, l.green, l.blue, 1.0)
    };
    let share = |col: &str, d: f32| num(col).unwrap_or(d).clamp(0.0, 1.0);
    Some(Wear {
        p: WearParams {
            chip: rgb("wear_chip_color", Srgba::rgb(0.227, 0.235, 0.251)),
            grime: rgb("wear_grime_color", Srgba::rgb(0.231, 0.196, 0.149)),
            rust: rgb("wear_rust_color", Srgba::rgb(0.478, 0.247, 0.11)),
            amount,
            scale: num("wear_scale").filter(|s| *s > 0.0).unwrap_or(0.08),
            chips: share("wear_chips", 0.6),
            grimes: share("wear_grime", 0.5),
            rusts: share("wear_rust", 0.25),
            seed: num("wear_seed").unwrap_or(0.0),
        },
    })
}

#[cfg(test)]
mod tests;
