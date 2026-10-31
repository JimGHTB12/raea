//! The volume's simulation on the CPU: value noise, the grid stepped in parallel slabs (smoke, fire, dust), and its
//! marks for a render to check.

use super::*;

pub fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

pub fn rand01(a: u32, b: u32) -> f32 {
    (hash(a.wrapping_mul(0x9e37_79b9) ^ hash(b)) & 0xff_ffff) as f32 / 16_777_216.0
}

/// smooth value noise in 0..1 on an integer lattice, seeded
pub fn vnoise(x: f32, y: f32, z: f32, s: u32) -> f32 {
    let (ix, iy, iz) = (x.floor(), y.floor(), z.floor());
    let (fx, fy, fz) = (x - ix, y - iy, z - iz);
    let (ux, uy, uz) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy), fz * fz * (3.0 - 2.0 * fz));
    let h = |a: i32, b: i32, c: i32| rand01((a as u32).wrapping_mul(73_856_093) ^ (b as u32).wrapping_mul(19_349_663) ^ (c as u32).wrapping_mul(83_492_791), s);
    let (x0, y0, z0) = (ix as i32, iy as i32, iz as i32);
    let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
    l(
        l(l(h(x0, y0, z0), h(x0 + 1, y0, z0), ux), l(h(x0, y0 + 1, z0), h(x0 + 1, y0 + 1, z0), ux), uy),
        l(l(h(x0, y0, z0 + 1), h(x0 + 1, y0, z0 + 1), ux), l(h(x0, y0 + 1, z0 + 1), h(x0 + 1, y0 + 1, z0 + 1), ux), uy),
        uz,
    )
}

/// Cells split over threads by z-slab, each written by one thread only: the same result however many threads run.
pub fn par_slabs(nz: usize, slab: usize, out: &mut [f32], f: impl Fn(usize, &mut [f32]) + Sync) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let threads = 6usize.min(nz.max(1));
        let per = nz.div_ceil(threads);
        std::thread::scope(|s| {
            for (k, chunk) in out.chunks_mut(per * slab).enumerate() {
                let f = &f;
                s.spawn(move || {
                    for (j, row) in chunk.chunks_mut(slab).enumerate() {
                        f(k * per + j, row);
                    }
                });
            }
        });
    }
    #[cfg(target_arch = "wasm32")]
    for (z, row) in out.chunks_mut(slab).enumerate() {
        f(z, row);
    }
}

pub struct Sim {
    pub spec: Spec,
    pub n: [usize; 3],
    /// a cell's size, metres
    pub cell: f32,
    pub u: Vec<f32>,
    pub v: Vec<f32>,
    pub w: Vec<f32>,
    pub d: Vec<f32>,
    pub t: Vec<f32>,
    pub step: u32,
    /// the fields as they were at earlier steps, oldest first, so a time before the current one starts from the last
    /// of these before it, not from the start (see `run_to`)
    pub marks: Vec<Mark>,
    /// steps between marks: CHECK, doubled whenever the marks would pass MARK_BYTES
    pub every: u32,
}

/// the fields at one step
pub struct Mark {
    pub step: u32,
    pub u: Vec<f32>,
    pub v: Vec<f32>,
    pub w: Vec<f32>,
    pub d: Vec<f32>,
    pub t: Vec<f32>,
}

/// steps between marks to start with (a second of simulated time)
pub const CHECK: u32 = 30;
/// what a volume's marks may hold; past it every other mark goes and the spacing doubles, so they still reach back
/// to the start, sparser further back
pub const MARK_BYTES: usize = if cfg!(test) { 2 << 20 } else { 96 << 20 };

impl Sim {
    pub fn new(spec: Spec, size: Vec3) -> Sim {
        let cell = size.max_element() / spec.res.clamp(8, 256) as f32;
        let n = [(size.x / cell).ceil().max(4.0) as usize, (size.y / cell).ceil().max(4.0) as usize, (size.z / cell).ceil().max(4.0) as usize];
        let len = n[0] * n[1] * n[2];
        let mut s = Sim { spec, n, cell, u: vec![0.0; len], v: vec![0.0; len], w: vec![0.0; len], d: vec![0.0; len], t: vec![0.0; len], step: 0, marks: Vec::new(), every: CHECK };
        s.start();
        s
    }

    pub fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        (z * self.n[1] + y) * self.n[0] + x
    }

    /// a cell's centre in metres from the box's foot (x and z centred)
    pub fn pos(&self, x: usize, y: usize, z: usize) -> [f32; 3] {
        let c = self.cell;
        [(x as f32 + 0.5) * c - self.n[0] as f32 * c * 0.5, (y as f32 + 0.5) * c, (z as f32 + 0.5) * c - self.n[2] as f32 * c * 0.5]
    }

    /// the puffs, each lumpy (value noise on its edge)
    pub fn start(&mut self) {
        let [nx, ny, nz] = self.n;
        let seed = self.spec.seed;
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let p = self.pos(x, y, z);
                    let mut dens: f32 = 0.0;
                    for b in &self.spec.puffs {
                        let r = ((p[0] - b[0]).powi(2) + (p[1] - b[1]).powi(2) + (p[2] - b[2]).powi(2)).sqrt();
                        let lump = vnoise(p[0] / (b[3] * 0.5), p[1] / (b[3] * 0.5), p[2] / (b[3] * 0.5), seed) * 0.5 + 0.75;
                        dens = dens.max(((1.0 - r / (b[3] * lump)) * 3.0).clamp(0.0, 1.0));
                    }
                    let i = self.idx(x, y, z);
                    self.d[i] = dens;
                }
            }
        }
    }

    pub fn sample(f: &[f32], n: [usize; 3], x: f32, y: f32, z: f32) -> f32 {
        let cl = |a: f32, m: usize| a.clamp(0.0, (m - 1) as f32);
        let (x, y, z) = (cl(x, n[0]), cl(y, n[1]), cl(z, n[2]));
        let (x0, y0, z0) = (x.floor() as usize, y.floor() as usize, z.floor() as usize);
        let (x1, y1, z1) = ((x0 + 1).min(n[0] - 1), (y0 + 1).min(n[1] - 1), (z0 + 1).min(n[2] - 1));
        let (fx, fy, fz) = (x - x0 as f32, y - y0 as f32, z - z0 as f32);
        let at = |a: usize, b: usize, c: usize| f[(c * n[1] + b) * n[0] + a];
        let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
        l(
            l(l(at(x0, y0, z0), at(x1, y0, z0), fx), l(at(x0, y1, z0), at(x1, y1, z0), fx), fy),
            l(l(at(x0, y0, z1), at(x1, y0, z1), fx), l(at(x0, y1, z1), at(x1, y1, z1), fx), fy),
            fz,
        )
    }

    /// `f` carried along the velocity for dt (semi-Lagrangian: each cell takes what was where its flow came from)
    pub fn advect(&self, f: &[f32], dt: f32) -> Vec<f32> {
        let n = self.n;
        let k = dt / self.cell;
        let mut out = vec![0.0; f.len()];
        let (u, v, w) = (&self.u, &self.v, &self.w);
        par_slabs(n[2], n[0] * n[1], &mut out, |z, slab| {
            for y in 0..n[1] {
                for x in 0..n[0] {
                    let i = (z * n[1] + y) * n[0] + x;
                    slab[y * n[0] + x] = Sim::sample(f, n, x as f32 - u[i] * k, y as f32 - v[i] * k, z as f32 - w[i] * k);
                }
            }
        });
        out
    }

    pub fn step_once(&mut self) {
        let dt = 1.0 / RATE;
        let n = self.n;
        let [nx, ny, nz] = n;
        let time = self.step as f32 * dt;
        let c = self.cell;
        let seed = self.spec.seed;
        // sources: density, heat and a push, in a jittered ball (a contrail's swept from where it was a step ago)
        let sources = self.spec.sources.clone();
        for (si, s) in sources.iter().enumerate() {
            let at = |t: f32| -> Option<[f32; 3]> {
                match s.to {
                    None => Some(s.at),
                    Some(to) => {
                        if t < s.t0 || t > s.t1 {
                            return None;
                        }
                        let u = ((t - s.t0) / (s.t1 - s.t0).max(1e-3)).clamp(0.0, 1.0);
                        Some([s.at[0] + (to[0] - s.at[0]) * u, s.at[1] + (to[1] - s.at[1]) * u, s.at[2] + (to[2] - s.at[2]) * u])
                    }
                }
            };
            let (Some(a), b) = (at(time), at((time - dt).max(0.0))) else { continue };
            let b = b.unwrap_or(a);
            let r = s.radius.max(c * 0.75);
            let sub = (((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() / (r * 0.5)).ceil().max(1.0) as usize;
            for q in 0..sub {
                let t = (q as f32 + 0.5) / sub as f32;
                let jit = |k: u32| (rand01(self.step * 31 + si as u32 * 7 + k, seed + q as u32) - 0.5) * r * 0.6;
                let p = [b[0] + (a[0] - b[0]) * t + jit(0), b[1] + (a[1] - b[1]) * t + jit(1), b[2] + (a[2] - b[2]) * t + jit(2)];
                let lo = |k: usize, m: usize| (((p[k] + if k == 1 { 0.0 } else { m as f32 * c * 0.5 }) - r) / c).floor().max(0.0) as usize;
                let hi = |k: usize, m: usize| ((((p[k] + if k == 1 { 0.0 } else { m as f32 * c * 0.5 }) + r) / c).ceil() as usize).min(m);
                for z in lo(2, nz)..hi(2, nz) {
                    for y in lo(1, ny)..hi(1, ny) {
                        for x in lo(0, nx)..hi(0, nx) {
                            let cp = self.pos(x, y, z);
                            let dd = ((cp[0] - p[0]).powi(2) + (cp[1] - p[1]).powi(2) + (cp[2] - p[2]).powi(2)).sqrt() / r;
                            if dd >= 1.0 {
                                continue;
                            }
                            let fall = (1.0 - dd * dd) / sub as f32;
                            let i = self.idx(x, y, z);
                            self.d[i] += s.rate * dt * fall;
                            self.t[i] += s.heat * dt * fall;
                            let k = fall * sub as f32;
                            self.u[i] += (s.speed[0] - self.u[i]) * k * 0.5;
                            self.v[i] += (s.speed[1] - self.v[i]) * k * 0.5;
                            self.w[i] += (s.speed[2] - self.w[i]) * k * 0.5;
                        }
                    }
                }
            }
        }
        // forces: buoyancy, the wind, a drifting turbulence
        let sp = &self.spec;
        let tur = sp.turbulence;
        let ts = sp.turbulence_size.max(c);
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let i = (z * ny + y) * nx + x;
                    self.v[i] += (sp.buoyancy * self.t[i] - sp.weight * self.d[i]) * dt;
                    let pull = (sp.wind_pull * dt).min(1.0) * (self.d[i] * 4.0).min(1.0);
                    // the wind is weaker near the ground, full from `wind_height` up
                    let shear = if sp.wind_height > 0.0 { ((y as f32 + 0.5) * c / sp.wind_height).clamp(0.1, 1.0) } else { 1.0 };
                    self.u[i] += (sp.wind[0] * shear - self.u[i]) * pull;
                    self.v[i] += (sp.wind[1] * shear - self.v[i]) * pull * 0.3;
                    self.w[i] += (sp.wind[2] * shear - self.w[i]) * pull;
                    if tur > 0.0 && self.d[i] > 0.01 {
                        let p = self.pos(x, y, z);
                        let drift = time * 0.15;
                        let (qx, qy, qz) = (p[0] / ts + drift, p[1] / ts, p[2] / ts - drift);
                        self.u[i] += (vnoise(qx, qy, qz, seed + 11) - 0.5) * 2.0 * tur * dt;
                        self.v[i] += (vnoise(qx, qy, qz, seed + 12) - 0.5) * 2.0 * tur * dt;
                        self.w[i] += (vnoise(qx, qy, qz, seed + 13) - 0.5) * 2.0 * tur * dt;
                    }
                }
            }
        }
        // vorticity confinement: a push along N x w, N the direction toward stronger curl
        if sp.vorticity > 0.0 {
            let len = nx * ny * nz;
            let (mut ox, mut oy, mut oz, mut om) = (vec![0.0; len], vec![0.0; len], vec![0.0; len], vec![0.0; len]);
            let g = |f: &[f32], x: usize, y: usize, z: usize, dx: i32, dy: i32, dz: i32| {
                let a = |o: i32, m: usize, b: usize| ((b as i32 + o).clamp(0, m as i32 - 1)) as usize;
                let hi = f[(a(dz, nz, z) * ny + a(dy, ny, y)) * nx + a(dx, nx, x)];
                let lo = f[(a(-dz, nz, z) * ny + a(-dy, ny, y)) * nx + a(-dx, nx, x)];
                (hi - lo) * 0.5 / c
            };
            for z in 0..nz {
                for y in 0..ny {
                    for x in 0..nx {
                        let i = (z * ny + y) * nx + x;
                        ox[i] = g(&self.w, x, y, z, 0, 1, 0) - g(&self.v, x, y, z, 0, 0, 1);
                        oy[i] = g(&self.u, x, y, z, 0, 0, 1) - g(&self.w, x, y, z, 1, 0, 0);
                        oz[i] = g(&self.v, x, y, z, 1, 0, 0) - g(&self.u, x, y, z, 0, 1, 0);
                        om[i] = (ox[i] * ox[i] + oy[i] * oy[i] + oz[i] * oz[i]).sqrt();
                    }
                }
            }
            let e = sp.vorticity * c;
            for z in 0..nz {
                for y in 0..ny {
                    for x in 0..nx {
                        let i = (z * ny + y) * nx + x;
                        let (gx, gy, gz) = (g(&om, x, y, z, 1, 0, 0), g(&om, x, y, z, 0, 1, 0), g(&om, x, y, z, 0, 0, 1));
                        let l = (gx * gx + gy * gy + gz * gz).sqrt() + 1e-5;
                        let (nx_, ny_, nz_) = (gx / l, gy / l, gz / l);
                        self.u[i] += e * (ny_ * oz[i] - nz_ * oy[i]) * dt;
                        self.v[i] += e * (nz_ * ox[i] - nx_ * oz[i]) * dt;
                        self.w[i] += e * (nx_ * oy[i] - ny_ * ox[i]) * dt;
                    }
                }
            }
        }
        // the air's drag, and no faster than a few cells a step (the semi-Lagrangian step stays stable)
        let keep_v = 1.0 - (self.spec.drag * dt).min(1.0);
        let vmax = 3.0 * c / dt;
        for i in 0..self.u.len() {
            let (mut a, mut b, mut w) = (self.u[i] * keep_v, self.v[i] * keep_v, self.w[i] * keep_v);
            let m = (a * a + b * b + w * w).sqrt();
            if m > vmax {
                let k = vmax / m;
                (a, b, w) = (a * k, b * k, w * k);
            }
            (self.u[i], self.v[i], self.w[i]) = (a, b, w);
        }
        // carry the velocity along itself, then make it divergence-free
        let (nu, nv, nw) = (self.advect(&self.u, dt), self.advect(&self.v, dt), self.advect(&self.w, dt));
        (self.u, self.v, self.w) = (nu, nv, nw);
        self.project();
        // carry density and heat; lose some
        let keep_d = 1.0 - (self.spec.dissipation * dt).min(1.0);
        let keep_t = 1.0 - (self.spec.cooling * dt).min(1.0);
        self.d = self.advect(&self.d, dt).into_iter().map(|x| x * keep_d).collect();
        self.t = self.advect(&self.t, dt).into_iter().map(|x| x * keep_t).collect();
        self.step += 1;
    }

    /// Jacobi pressure solve with open sides and a closed floor
    pub fn project(&mut self) {
        let [nx, ny, nz] = self.n;
        let len = nx * ny * nz;
        let mut div = vec![0.0f32; len];
        let at = |f: &[f32], x: i32, y: i32, z: i32| -> f32 {
            if x < 0 || z < 0 || x >= nx as i32 || y >= ny as i32 || z >= nz as i32 {
                return 0.0;
            }
            let y = y.max(0);
            f[(z as usize * ny + y as usize) * nx + x as usize]
        };
        for z in 0..nz as i32 {
            for y in 0..ny as i32 {
                for x in 0..nx as i32 {
                    let i = (z as usize * ny + y as usize) * nx + x as usize;
                    // the floor: no flow through it
                    let vb = if y == 0 { -at(&self.v, x, 0, z) } else { at(&self.v, x, y - 1, z) };
                    div[i] = 0.5 * (at(&self.u, x + 1, y, z) - at(&self.u, x - 1, y, z) + at(&self.v, x, y + 1, z) - vb + at(&self.w, x, y, z + 1) - at(&self.w, x, y, z - 1));
                }
            }
        }
        let mut p = vec![0.0f32; len];
        for _ in 0..self.spec.pressure_iters.clamp(4, 200) {
            let prev = p.clone();
            let div = &div;
            par_slabs(nz, nx * ny, &mut p, |z, slab| {
                let z = z as i32;
                for y in 0..ny as i32 {
                    for x in 0..nx as i32 {
                        let pb = if y == 0 { at(&prev, x, 0, z) } else { at(&prev, x, y - 1, z) };
                        let s = at(&prev, x + 1, y, z) + at(&prev, x - 1, y, z) + at(&prev, x, y + 1, z) + pb + at(&prev, x, y, z + 1) + at(&prev, x, y, z - 1);
                        slab[y as usize * nx + x as usize] = (s - div[(z as usize * ny + y as usize) * nx + x as usize]) / 6.0;
                    }
                }
            });
        }
        for z in 0..nz as i32 {
            for y in 0..ny as i32 {
                for x in 0..nx as i32 {
                    let i = (z as usize * ny + y as usize) * nx + x as usize;
                    let pb = if y == 0 { at(&p, x, 0, z) } else { at(&p, x, y - 1, z) };
                    self.u[i] -= 0.5 * (at(&p, x + 1, y, z) - at(&p, x - 1, y, z));
                    self.v[i] -= 0.5 * (at(&p, x, y + 1, z) - pb);
                    self.w[i] -= 0.5 * (at(&p, x, y, z + 1) - at(&p, x, y, z - 1));
                }
            }
        }
    }

    /// to time t (seconds): forward from here, or, if t is earlier, forward from the last mark at or before it (from
    /// the start without one). Each step depends only on the fields and the step number, so a volume reached from a
    /// mark is the one reached from the start. (Before the marks, a clock that ran back a step re-ran the whole
    /// simulation: a pull-back that slowed its clouds' time past still ran it backward, and each frame's step back cost
    /// ~9 s for three clouds, 10-22.)
    pub fn run_to(&mut self, t: f32, size: Vec3) {
        let want = (t.max(0.0) * RATE).round() as u32;
        if want < self.step {
            match self.marks.iter().rposition(|m| m.step <= want) {
                Some(i) => {
                    self.marks.truncate(i + 1);
                    let m = &self.marks[i];
                    self.step = m.step;
                    self.u.clone_from(&m.u);
                    self.v.clone_from(&m.v);
                    self.w.clone_from(&m.w);
                    self.d.clone_from(&m.d);
                    self.t.clone_from(&m.t);
                }
                None => *self = Sim::new(self.spec.clone(), size),
            }
        }
        while self.step < want {
            if self.step % self.every == 0 && self.marks.last().is_none_or(|m| m.step < self.step) {
                self.mark();
            }
            self.step_once();
        }
    }

    /// the fields kept at this step
    pub fn mark(&mut self) {
        let bytes = self.d.len() * 5 * 4;
        if (self.marks.len() + 1) * bytes > MARK_BYTES {
            self.every *= 2;
            let every = self.every;
            self.marks.retain(|m| m.step % every == 0);
            if self.step % every != 0 || (self.marks.len() + 1) * bytes > MARK_BYTES {
                return;
            }
        }
        self.marks.push(Mark { step: self.step, u: self.u.clone(), v: self.v.clone(), w: self.w.clone(), d: self.d.clone(), t: self.t.clone() });
    }

    /// the 3D texture: density, the sun's light reaching each cell through the density toward it, and heat
    pub fn texels(&self, sun: Vec3) -> Vec<u8> {
        let [nx, ny, nz] = self.n;
        let mut out = vec![0u8; nx * ny * nz * 4];
        let k = self.spec.absorb * self.cell;
        let step = sun.normalize_or(Vec3::Y);
        let (d, n) = (&self.d, self.n);
        let longest = nx.max(ny).max(nz) as i32;
        let slab = nx * ny * 4;
        // heat as a fraction of the hottest burner's (the fire's colours start at fractions of it)
        let heat_ref = self.spec.sources.iter().map(|s| s.heat).fold(1.0f32, f32::max) * 0.3;
        // as bytes, a slab at a time
        #[cfg(not(target_arch = "wasm32"))]
        let threads = 6usize.min(nz.max(1));
        #[cfg(target_arch = "wasm32")]
        let threads = 1usize;
        let per = nz.div_ceil(threads);
        let fill = |z0: usize, chunk: &mut [u8]| {
            for (j, row) in chunk.chunks_mut(slab).enumerate() {
                let z = z0 + j;
                for y in 0..ny {
                    for x in 0..nx {
                        let i = (z * ny + y) * nx + x;
                        let mut tau = 0.0;
                        let (mut px, mut py, mut pz) = (x as f32, y as f32, z as f32);
                        for _ in 0..longest {
                            px += step.x;
                            py += step.y;
                            pz += step.z;
                            if px < 0.0 || py < 0.0 || pz < 0.0 || px > (nx - 1) as f32 || py > (ny - 1) as f32 || pz > (nz - 1) as f32 {
                                break;
                            }
                            tau += Sim::sample(d, n, px, py, pz);
                        }
                        let o = (y * nx + x) * 4;
                        row[o] = (d[i].clamp(0.0, 1.0) * 255.0) as u8;
                        row[o + 1] = ((-tau * k).exp() * 255.0) as u8;
                        row[o + 2] = ((self.t[i] / heat_ref).clamp(0.0, 1.0) * 255.0) as u8;
                        row[o + 3] = 255;
                    }
                }
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::scope(|s| {
            for (c, chunk) in out.chunks_mut(per * slab).enumerate() {
                let fill = &fill;
                s.spawn(move || fill(c * per, chunk));
            }
        });
        #[cfg(target_arch = "wasm32")]
        fill(0, &mut out);
        out
    }
}
