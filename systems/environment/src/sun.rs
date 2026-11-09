//! A panorama's sun (docs/raytracing.feature RT6): a small disk far brighter than the rest of the sky, found when the
//! panorama is decoded, from its own floats (the cubemap's halves clamp at 65504, and a clear sky's sun is brighter).
//! Solari samples it as a directional light (bevy_solari's EnvironmentSun, vendored), so a glossy surface gets its
//! highlight and the ground its shadows under traced lighting; reflection rays alone find a disk that small too rarely
//! for the denoiser to keep. The raster passes don't read it.

use bevy::prelude::*;
use bevy::solari::scene::EnvironmentSun;
use std::f32::consts::PI;

use super::{num, Equirect, FrameRows};

/// Each panorama's sun, in env_srcs' order (None: no sun stands out).
#[derive(Component)]
pub struct Suns(pub Vec<Option<EnvironmentSun>>);

/// The brightest pixel must be this many times the panorama's mean radiance to be a sun: a clear sky's sun is
/// thousands of times it, a lamp in a room or an overcast sky well under.
const STANDOUT: f32 = 50.0;
/// A sun's pixels: within this angle of the brightest, and at least this share of its radiance.
const REACH: f32 = 10.0 * PI / 180.0;
const SHARE: f32 = 0.05;

fn lum(p: [f32; 3]) -> f32 {
    0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2]
}

impl Equirect {
    /// A pixel's world direction (Bevy's: +Y up, -Z ahead; `sample`'s mapping inverted) and its solid angle.
    fn texel(&self, x: usize, y: usize) -> (Vec3, f32) {
        let lon = ((x as f32 + 0.5) / self.w as f32 - 0.5) * 2.0 * PI;
        let theta = (y as f32 + 0.5) / self.h as f32 * PI;
        let d = Vec3::new(theta.sin() * lon.sin(), theta.cos(), -theta.sin() * lon.cos());
        (d, (2.0 * PI / self.w as f32) * (PI / self.h as f32) * theta.sin())
    }

    /// Every pixel within `reach` of `d`, with its direction and solid angle (only the rows that can be).
    fn near(&self, d: Vec3, reach: f32) -> impl Iterator<Item = ([f32; 3], Vec3, f32)> + '_ {
        let theta = d.y.clamp(-1.0, 1.0).acos();
        let row = |t: f32| ((t / PI * self.h as f32) as isize).clamp(0, self.h as isize - 1) as usize;
        let (top, bottom) = (row(theta - reach - 0.01), row(theta + reach + 0.01));
        let cos_reach = reach.cos();
        (top..=bottom).flat_map(move |y| (0..self.w).map(move |x| (x, y))).filter_map(move |(x, y)| {
            let (t, sa) = self.texel(x, y);
            (t.dot(d) >= cos_reach).then(|| (self.px[y * self.w + x], t, sa))
        })
    }

    /// The panorama's sun, if one stands out: its direction (the light-weighted middle of its pixels), its disk (out to
    /// its farthest pixel, half a pixel more), the sky's radiance in a ring around it, and the light above that sky it
    /// adds (what the traced environment map then leaves out of the disk).
    pub(super) fn sun(&self) -> Option<EnvironmentSun> {
        let (mut total, mut peak, mut at) = (0.0f32, 0.0f32, (0, 0));
        for y in 0..self.h {
            let (_, sa) = self.texel(0, y);
            for x in 0..self.w {
                let l = lum(self.px[y * self.w + x]);
                total += l * sa;
                if l > peak {
                    (peak, at) = (l, (x, y));
                }
            }
        }
        let mean = total / (4.0 * PI);
        if !(peak > STANDOUT * mean) || !peak.is_finite() {
            return None;
        }
        let (brightest, _) = self.texel(at.0, at.1);
        let floor = SHARE * peak;
        let mut middle = Vec3::ZERO;
        for (p, d, sa) in self.near(brightest, REACH) {
            if lum(p) >= floor {
                middle += d * lum(p) * sa;
            }
        }
        let direction = middle.try_normalize()?;
        let half_pixel = PI / self.h as f32 * 0.5;
        let radius = self
            .near(direction, REACH)
            .filter(|(p, _, _)| lum(*p) >= floor)
            .map(|(_, d, _)| d.dot(direction).clamp(-1.0, 1.0).acos())
            .fold(0.0f32, f32::max)
            + half_pixel;
        // the sky: the ring from one and a half to three radii out
        let (mut sky, mut ring) = (Vec3::ZERO, 0.0);
        let (inner, outer) = ((1.5 * radius).cos(), (3.0 * radius + 2.0 * half_pixel).cos());
        for (p, d, sa) in self.near(direction, 3.0 * radius + 2.0 * half_pixel) {
            let c = d.dot(direction);
            if c <= inner && c >= outer {
                sky += Vec3::from(p) * sa;
                ring += sa;
            }
        }
        let sky = if ring > 0.0 { sky / ring } else { Vec3::ZERO };
        let mut illuminance = Vec3::ZERO;
        for (p, _, sa) in self.near(direction, radius) {
            illuminance += (Vec3::from(p) - sky).max(Vec3::ZERO) * sa;
        }
        Some(EnvironmentSun { direction, angular_size: 2.0 * radius, illuminance, sky })
    }
}

/// Each frame's sun: the one of the panorama the camera's row picks, on the camera while its environment is on.
pub fn sync_suns(mut commands: Commands, frame: Res<FrameRows>, q: Query<(Entity, &Suns, Option<&EnvironmentSun>)>) {
    for (e, suns, has) in &q {
        let Some(v) = frame.vals(e) else { continue };
        let at = num(&v, "env_i", 0.0).round();
        let want = (at > -0.5).then(|| suns.0.get((at.max(0.0) as usize).min(suns.0.len().saturating_sub(1))).copied().flatten()).flatten();
        match (want, has) {
            (Some(s), Some(h)) if s == *h => {}
            (Some(s), _) => {
                commands.entity(e).insert(s);
            }
            (None, Some(_)) => {
                commands.entity(e).remove::<EnvironmentSun>();
            }
            (None, None) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat sky with one small hot disk straight up and a little to the north.
    fn sky_with_sun(sun: f32) -> Equirect {
        let (w, h) = (512, 256);
        let mut px = vec![[0.5, 0.6, 0.9]; w * h];
        let toward = Vec3::new(0.0, 0.8, -0.6).normalize();
        let eq = Equirect { w, h, px: px.clone() };
        for y in 0..h {
            for x in 0..w {
                if eq.texel(x, y).0.dot(toward) > (1.5f32).to_radians().cos() {
                    px[y * w + x] = [sun, sun, sun];
                }
            }
        }
        Equirect { w, h, px }
    }

    #[test]
    fn a_hot_disk_is_the_sun() {
        let s = sky_with_sun(50_000.0).sun().expect("a sun");
        assert!(s.direction.dot(Vec3::new(0.0, 0.8, -0.6).normalize()) > 0.9995, "{:?}", s.direction);
        assert!((s.angular_size - 3.0f32.to_radians()).abs() < 1.5f32.to_radians(), "{}", s.angular_size.to_degrees());
        assert!((s.sky - Vec3::new(0.5, 0.6, 0.9)).length() < 1e-3, "{:?}", s.sky);
        // the disk's light above the sky: (50000 - sky) over its solid angle
        let disk = 2.0 * PI * (1.0 - (1.5f32).to_radians().cos());
        assert!((s.illuminance.x / ((50_000.0 - 0.5) * disk) - 1.0).abs() < 0.15, "{:?}", s.illuminance);
    }

    #[test]
    fn an_even_sky_has_none() {
        assert!(sky_with_sun(5.0).sun().is_none());
    }
}

