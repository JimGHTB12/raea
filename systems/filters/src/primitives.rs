//! The SVG primitives anyrender's structs carry too little of (`FilterEffect::Svg`): feBlend, feMorphology,
//! feConvolveMatrix, feTurbulence, feDisplacementMap, feComponentTransfer, feTile and the lighting filters, as Chrome draws
//! them, in primitives.wgsl.

use super::passes::{Params, Passes};
use super::*;

use anyrender::filters::svg::{Light, SvgPrimitive, Transfer};
use wgpu::util::DeviceExt;

impl Passes {
    /// `prim` over `input` (and `second`, its in2) in `layer`'s texels.
    pub(crate) fn svg(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, prim: &SvgPrimitive, input: &wgpu::Texture, second: Option<&wgpu::Texture>, layer: &Filtered) -> wgpu::Texture {
        let size = layer.size;
        let s = layer.scale;
        let out = self.work(device, size);
        match prim {
            SvgPrimitive::Blend(mix) => {
                let none = self.work(device, size);
                let p = Params { a: [*mix as u8 as f32, 0.0, 0.0, 0.0], ..Default::default() };
                self.pass(device, encoder, "feblend", input, Some(second.unwrap_or(&none)), p, &out);
            }
            SvgPrimitive::Morphology { dilate, rx, ry } => {
                let (rx, ry) = ((rx * s).round().min(256.0), (ry * s).round().min(256.0));
                let d = if *dilate { 1.0 } else { 0.0 };
                let mid = self.work(device, size);
                self.pass(device, encoder, "morph", input, None, Params { a: [1.0, 0.0, rx, d], ..Default::default() }, &mid);
                self.pass(device, encoder, "morph", &mid, None, Params { a: [0.0, 1.0, ry, d], ..Default::default() }, &out);
            }
            SvgPrimitive::Convolve(c) => {
                let n = (c.columns * c.rows) as usize;
                if n == 0 || n > 256 || c.kernel.len() < n {
                    return input.clone();
                }
                let mut p = Params {
                    a: [c.columns as f32, c.rows as f32, c.target.0 as f32, c.target.1 as f32],
                    b: [c.divisor, c.bias, c.edge as f32, if c.preserve_alpha { 1.0 } else { 0.0 }],
                    ..Default::default()
                };
                for (i, k) in c.kernel[..n].iter().enumerate() {
                    p.t[i / 4][i % 4] = *k;
                }
                self.pass(device, encoder, "convolve", input, None, p, &out);
            }
            SvgPrimitive::Displacement { scale, x, y } => {
                let none = self.work(device, size);
                let p = Params { a: [scale * s, *x as f32, *y as f32, 0.0], ..Default::default() };
                self.pass(device, encoder, "displace", input, Some(second.unwrap_or(&none)), p, &out);
            }
            SvgPrimitive::Transfer(fs) => {
                let mut p = Params::default();
                for (c, f) in fs.iter().enumerate() {
                    p.m[c] = match f {
                        Transfer::Identity => [0.0; 4],
                        Transfer::Table(v) | Transfer::Discrete(v) if v.is_empty() => [0.0; 4],
                        Transfer::Table(v) | Transfer::Discrete(v) => {
                            let v = &v[..v.len().min(64)];
                            for (i, k) in v.iter().enumerate() {
                                p.t[i][c] = *k;
                            }
                            [if matches!(f, Transfer::Table(_)) { 1.0 } else { 2.0 }, v.len() as f32, 0.0, 0.0]
                        }
                        Transfer::Linear { slope, intercept } => [3.0, *slope, *intercept, 0.0],
                        Transfer::Gamma { amplitude, exponent, offset } => [4.0, *amplitude, *exponent, *offset],
                    };
                }
                self.pass(device, encoder, "transfer_fn", input, None, p, &out);
            }
            SvgPrimitive::Turbulence(t) => {
                let (mut fx, mut fy) = t.frequency;
                // the spec's stitching: frequencies whole cycles a tile, and where the lattice wraps
                let mut stitch = [0.0; 4];
                if let Some([x, y, w, h]) = t.stitch {
                    let fit = |f: f32, len: f32| {
                        if f == 0.0 {
                            return f;
                        }
                        let (lo, hi) = ((len * f).floor() / len, (len * f).ceil() / len);
                        if f / lo < hi / f { lo } else { hi }
                    };
                    (fx, fy) = (fit(fx, w), fit(fy, h));
                    let (sw, sh) = ((w * fx + 0.5) as i32, (h * fy + 0.5) as i32);
                    stitch = [sw as f32, ((x * fx + 4096.0) as i32 + sw) as f32, sh as f32, ((y * fy + 4096.0) as i32 + sh) as f32];
                }
                let mut p = Params { a: [fx, fy, t.octaves.min(24) as f32, if t.fractal { 1.0 } else { 0.0 }], b: stitch, ..Default::default() };
                let u = layer.user.inverse().as_coeffs().map(|v| v as f32);
                p.m[2] = [u[0], u[2], u[4], 0.0];
                p.m[3] = [u[1], u[3], u[5], 0.0];
                let lattice = self.lattice(device, encoder, t.seed);
                self.pass(device, encoder, "turbulence", input, Some(&lattice), p, &out);
            }
            SvgPrimitive::Tile { rect } => {
                let (t, u) = (layer.user.as_coeffs().map(|v| v as f32), layer.user.inverse().as_coeffs().map(|v| v as f32));
                let mut p = Params { a: *rect, ..Default::default() };
                p.m = [[t[0], t[2], t[4], 0.0], [t[1], t[3], t[5], 0.0], [u[0], u[2], u[4], 0.0], [u[1], u[3], u[5], 0.0], [0.0; 4]];
                self.pass(device, encoder, "tile", input, None, p, &out);
            }
            SvgPrimitive::Lighting(l) => {
                let to = |x: f32, y: f32| {
                    let q = layer.user * vello::kurbo::Point::new(x as f64, y as f64);
                    [q.x as f32, q.y as f32]
                };
                let mut p = Params {
                    a: [l.surface_scale, l.constant, l.exponent.clamp(1.0, 128.0), if l.specular { 1.0 } else { 0.0 }],
                    b: [0.0, 1.0, -2.0, 0.0],
                    ..Default::default()
                };
                p.c[0] = [l.color[0], l.color[1], l.color[2], 1.0];
                match &l.light {
                    Light::Distant { azimuth, elevation } => {
                        let (az, el) = (azimuth.to_radians(), elevation.to_radians());
                        p.m[0] = [az.cos() * el.cos(), az.sin() * el.cos(), el.sin(), 0.0];
                    }
                    Light::Point { at } => {
                        let [x, y] = to(at[0], at[1]);
                        p.b[0] = 1.0;
                        p.m[0] = [x, y, at[2] * s, 0.0];
                    }
                    Light::Spot { at, points_at, exponent, cone } => {
                        let ([x, y], [px, py]) = (to(at[0], at[1]), to(points_at[0], points_at[1]));
                        let d = [px - x, py - y, (points_at[2] - at[2]) * s];
                        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
                        p.b = [2.0, exponent.clamp(1.0, 128.0), cone.map_or(-2.0, |c| c.abs().min(90.0).to_radians().cos()), 0.0];
                        p.m[0] = [x, y, at[2] * s, 0.0];
                        p.m[1] = [d[0] / len, d[1] / len, d[2] / len, 0.0];
                    }
                }
                self.pass(device, encoder, "lighting", input, None, p, &out);
            }
        }
        out
    }

    /// feTurbulence's lattice for `seed` (the spec's init): 514 x 4 texels, gradient x and y and the selector.
    fn lattice(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, seed: i32) -> wgpu::Texture {
        if let Some(t) = self.lattices.get(&seed) {
            return t.clone();
        }
        const B: usize = 256;
        const M: i64 = 2147483647;
        let random = |s: i64| {
            let r = 16807 * (s % 127773) - 2836 * (s / 127773);
            if r <= 0 { r + M } else { r }
        };
        let mut s = seed as i64;
        if s <= 0 {
            s = -(s % (M - 1)) + 1;
        }
        s = s.min(M - 1);
        let mut selector = [0usize; B + B + 2];
        let mut gradient = [[[0.0f64; 2]; B + B + 2]; 4];
        for g in gradient.iter_mut() {
            for (i, sel) in selector.iter_mut().enumerate().take(B) {
                *sel = i;
                for j in 0..2 {
                    s = random(s);
                    g[i][j] = ((s % (B as i64 * 2)) - B as i64) as f64 / B as f64;
                }
                let n = (g[i][0] * g[i][0] + g[i][1] * g[i][1]).sqrt();
                g[i] = [g[i][0] / n, g[i][1] / n];
            }
        }
        for i in (1..B).rev() {
            s = random(s);
            selector.swap(i, (s % B as i64) as usize);
        }
        for i in 0..B + 2 {
            selector[B + i] = selector[i];
            for g in gradient.iter_mut() {
                g[B + i] = g[i];
            }
        }
        // rows padded to 256 bytes for the copy
        let row = ((B + B + 2) * 16).next_multiple_of(256);
        let mut bytes = vec![0u8; row * 4];
        for (k, g) in gradient.iter().enumerate() {
            for i in 0..B + B + 2 {
                let texel = [g[i][0] as f32, g[i][1] as f32, selector[i] as f32, 0.0];
                for (c, v) in texel.iter().enumerate() {
                    let at = k * row + i * 16 + c * 4;
                    bytes[at..at + 4].copy_from_slice(&v.to_le_bytes());
                }
            }
        }
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("turbulence"), contents: &bytes, usage: wgpu::BufferUsages::COPY_SRC });
        let size = wgpu::Extent3d { width: (B + B + 2) as u32, height: 4, depth_or_array_layers: 1 };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("turbulence"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo { buffer: &buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row as u32), rows_per_image: Some(4) } },
            texture.as_image_copy(),
            size,
        );
        self.lattices.insert(seed, texture.clone());
        texture
    }
}
