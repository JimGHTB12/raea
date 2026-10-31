//! noise and fastnoise-lite (the declaration's `"calls": "noise"`): plain Rust libraries with no Bevy types, so
//! TypeScript reaches them through methods. A grid of samples comes back in one call, so a heightmap or a texture is one
//! round trip.
//!
//!   noise/sample  {kind: "perlin" | "simplex" | "open_simplex" | "value" | "worley" | "fbm" | "ridged" | "billow",
//!                  seed?, frequency?, octaves?, points: [[x, y] | [x, y, z], ...]}           -> [value]
//!   noise/grid    {kind, seed?, frequency?, octaves?, origin: [x, y], size: [w, h], step?}  -> [[value per x] per y]
//!
//! "fast_*" kinds (fast_open_simplex2, fast_perlin, fast_cellular, fast_value, fast_value_cubic) use fastnoise-lite.

use bevy::prelude::*;
use bevy_remote::BrpResult;
use noise::{Billow, Fbm, MultiFractal, NoiseFn, OpenSimplex, Perlin, RidgedMulti, Seedable, Simplex, Value as ValueNoise, Worley};
use serde_json::{Value, json};

use crate::{fail, params};

pub fn calls(app: &mut App) {
    crate::methods!(app, "noise/sample" => sample, "noise/grid" => grid);
}

#[derive(serde::Deserialize)]
struct Spec {
    kind: String,
    #[serde(default)]
    seed: u32,
    frequency: Option<f64>,
    octaves: Option<usize>,
}

type Sampler = Box<dyn Fn(&[f64]) -> f64>;

fn sampler(s: &Spec) -> Result<Sampler, bevy_remote::BrpError> {
    let f = s.frequency.unwrap_or(1.0);
    let oct = s.octaves.unwrap_or(4);
    fn of<N: NoiseFn<f64, 2> + NoiseFn<f64, 3> + 'static>(n: N, f: f64) -> Sampler {
        Box::new(move |p: &[f64]| match p.len() {
            2 => NoiseFn::<f64, 2>::get(&n, [p[0] * f, p[1] * f]),
            _ => NoiseFn::<f64, 3>::get(&n, [p[0] * f, p[1] * f, p.get(2).copied().unwrap_or(0.0) * f]),
        })
    }
    Ok(match s.kind.as_str() {
        "perlin" => of(Perlin::new(s.seed), f),
        "simplex" => of(Simplex::new(s.seed), f),
        "open_simplex" => of(OpenSimplex::new(s.seed), f),
        "value" => of(ValueNoise::new(s.seed), f),
        "worley" => of(Worley::new(s.seed), f),
        "fbm" => of(Fbm::<Perlin>::new(s.seed).set_octaves(oct), f),
        "ridged" => of(RidgedMulti::<Perlin>::new(s.seed).set_octaves(oct), f),
        "billow" => of(Billow::<Perlin>::new(s.seed).set_octaves(oct), f),
        k if k.starts_with("fast_") => {
            use fastnoise_lite::{FastNoiseLite, NoiseType};
            let t = match k {
                "fast_open_simplex2" => NoiseType::OpenSimplex2,
                "fast_perlin" => NoiseType::Perlin,
                "fast_cellular" => NoiseType::Cellular,
                "fast_value" => NoiseType::Value,
                "fast_value_cubic" => NoiseType::ValueCubic,
                _ => return Err(fail(format!("noise kind {k}"))),
            };
            let mut n = FastNoiseLite::with_seed(s.seed as i32);
            n.set_noise_type(Some(t));
            n.set_frequency(Some(f as f32));
            Box::new(move |p: &[f64]| match p.len() {
                2 => n.get_noise_2d(p[0] as _, p[1] as _) as f64,
                _ => n.get_noise_3d(p[0] as _, p[1] as _, p.get(2).copied().unwrap_or(0.0) as _) as f64,
            })
        }
        k => return Err(fail(format!("noise kind {k}: perlin, simplex, open_simplex, value, worley, fbm, ridged, billow or fast_*"))),
    })
}

#[derive(serde::Deserialize)]
struct Sample {
    #[serde(flatten)]
    spec: Spec,
    points: Vec<Vec<f64>>,
}

fn sample(In(p): In<Option<Value>>) -> BrpResult {
    let s: Sample = params(p)?;
    let n = sampler(&s.spec)?;
    Ok(json!(s.points.iter().map(|p| n(p)).collect::<Vec<_>>()))
}

#[derive(serde::Deserialize)]
struct Grid {
    #[serde(flatten)]
    spec: Spec,
    origin: [f64; 2],
    size: [usize; 2],
    step: Option<f64>,
}

fn grid(In(p): In<Option<Value>>) -> BrpResult {
    let g: Grid = params(p)?;
    if g.size[0] * g.size[1] > 4_000_000 {
        return Err(fail("noise/grid: at most 4M samples a call"));
    }
    let n = sampler(&g.spec)?;
    let st = g.step.unwrap_or(1.0);
    let rows: Vec<Vec<f64>> = (0..g.size[1]).map(|y| (0..g.size[0]).map(|x| n(&[g.origin[0] + x as f64 * st, g.origin[1] + y as f64 * st])).collect()).collect();
    Ok(json!(rows))
}
