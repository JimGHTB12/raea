//! noisy_bevy (the declaration's `"calls": "noisy"`): its CPU noise, the same functions as its WGSL import
//! (`noisy_bevy::simplex_noise_2d`), so a TypeScript program samples on the CPU what a shader samples on the GPU. As
//! noise/ has, a grid comes back in one call.
//!
//!   noisy/sample  {kind: "simplex" | "fbm" | "worley", seed?, frequency?, octaves?, lacunarity?, gain?, jitter?,
//!                  points: [[x, y] | [x, y, z], ...]}                                         -> [value]
//!   noisy/grid    {kind, seed?, frequency?, ..., origin: [x, y], size: [w, h], step?}        -> [[value per x] per y]
//!
//! worley is 2D only and gives its nearest distance (F1); fbm's lacunarity and gain default to 2 and 0.5.

use bevy::prelude::*;
use bevy_remote::BrpResult;
use noisy_bevy::{fbm_simplex_2d_seeded, fbm_simplex_3d_seeded, simplex_noise_2d_seeded, simplex_noise_3d_seeded, worley_2d};
use serde_json::{Value, json};

use crate::{fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "noisy/sample" => sample, "noisy/grid" => grid);
}

#[derive(serde::Deserialize)]
struct Spec {
    kind: String,
    #[serde(default)]
    seed: f32,
    frequency: Option<f32>,
    octaves: Option<usize>,
    lacunarity: Option<f32>,
    gain: Option<f32>,
    jitter: Option<f32>,
}

impl Spec {
    fn at(&self, p: &[f32]) -> Result<f32, bevy_remote::BrpError> {
        let f = self.frequency.unwrap_or(1.0);
        let (oct, lac, gain) = (self.octaves.unwrap_or(4), self.lacunarity.unwrap_or(2.0), self.gain.unwrap_or(0.5));
        let v2 = || Vec2::new(p[0], p[1]) * f;
        let v3 = || Vec3::new(p[0], p[1], p.get(2).copied().unwrap_or(0.0)) * f;
        Ok(match (self.kind.as_str(), p.len()) {
            (_, n) if n < 2 => return Err(fail("a point is [x, y] or [x, y, z]")),
            ("simplex", 2) => simplex_noise_2d_seeded(v2(), self.seed),
            ("simplex", _) => simplex_noise_3d_seeded(v3(), Vec3::splat(self.seed)),
            ("fbm", 2) => fbm_simplex_2d_seeded(v2(), oct, lac, gain, self.seed),
            ("fbm", _) => fbm_simplex_3d_seeded(v3(), oct, lac, gain, Vec3::splat(self.seed)),
            ("worley", 2) => worley_2d(v2(), self.jitter.unwrap_or(1.0)).x,
            ("worley", _) => return Err(fail("worley is 2D: give [x, y]")),
            (k, _) => return Err(fail(format!("noisy kind {k}: simplex, fbm or worley"))),
        })
    }
}

#[derive(serde::Deserialize)]
struct Sample {
    #[serde(flatten)]
    spec: Spec,
    points: Vec<Vec<f32>>,
}

fn sample(In(p): In<Option<Value>>) -> BrpResult {
    let s: Sample = params(p)?;
    Ok(json!(s.points.iter().map(|p| s.spec.at(p)).collect::<Result<Vec<_>, _>>()?))
}

#[derive(serde::Deserialize)]
struct Grid {
    #[serde(flatten)]
    spec: Spec,
    origin: [f32; 2],
    size: [usize; 2],
    step: Option<f32>,
}

fn grid(In(p): In<Option<Value>>) -> BrpResult {
    let g: Grid = params(p)?;
    if g.size[0] * g.size[1] > 4_000_000 {
        return Err(fail("noisy/grid: at most 4M samples a call"));
    }
    let st = g.step.unwrap_or(1.0);
    let rows = (0..g.size[1])
        .map(|y| (0..g.size[0]).map(|x| g.spec.at(&[g.origin[0] + x as f32 * st, g.origin[1] + y as f32 * st])).collect::<Result<Vec<_>, _>>())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!(rows))
}
