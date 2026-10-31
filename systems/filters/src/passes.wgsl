struct P { a: vec4f, b: vec4f, m: array<vec4f, 5>, o: array<vec4f, 2>, c: array<vec4f, 8>, t: array<vec4f, 64> }
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var other: texture_2d<f32>;
@group(0) @binding(2) var<uniform> p: P;

@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let q = vec2f(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4f(q * 2.0 - 1.0, 0.0, 1.0);
}

// a texel, transparent outside the texture
fn at(t: texture_2d<f32>, q: vec2i) -> vec4f {
    let s = vec2i(textureDimensions(t));
    if any(q < vec2i(0)) || any(q >= s) { return vec4f(0.0); }
    return textureLoad(t, q, 0);
}

// Vello's straight alpha in, premultiplied out
@fragment fn load(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let c = at(src, vec2i(f.xy));
    return vec4f(c.rgb * c.a, c.a);
}

// premultiplied in, straight alpha out (what Vello copies an image from)
@fragment fn store(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let c = at(src, vec2i(f.xy));
    if c.a <= 0.0 { return vec4f(0.0); }
    return vec4f(clamp(c.rgb / c.a, vec3f(0.0), vec3f(1.0)), clamp(c.a, 0.0, 1.0));
}

// a warped layer (B3 H5): each pixel of its page box back through the homography m[0..3] (rows on x, y, w) to the
// layer's texels, sampled bilinearly (premultiplied) across the pixel's footprint there (a layer seen near edge-on is
// squeezed many texels a pixel: up to 16 taps a side); nothing where it lands behind the viewer or off the layer
fn to_layer(x: vec2f) -> vec3f {
    let h = vec3f(x, 1.0);
    return vec3f(dot(p.m[0].xyz, h), dot(p.m[1].xyz, h), dot(p.m[2].xyz, h));
}

fn bilinear(uv: vec2f) -> vec4f {
    let u = uv - 0.5;
    let i = vec2i(floor(u));
    let t = u - floor(u);
    let a = mix(at(src, i), at(src, i + vec2i(1, 0)), t.x);
    let b = mix(at(src, i + vec2i(0, 1)), at(src, i + vec2i(1, 1)), t.x);
    return mix(a, b, t.y);
}

@fragment fn warp(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let q = to_layer(f.xy);
    if q.z <= 0.0 { return vec4f(0.0); }
    let uv = q.xy / q.z;
    // the pixel's footprint in the layer: how far a pixel step right and down moves there
    let qx = to_layer(f.xy + vec2f(1.0, 0.0));
    let qy = to_layer(f.xy + vec2f(0.0, 1.0));
    let dx = select(vec2f(0.0), qx.xy / qx.z - uv, qx.z > 0.0);
    let dy = select(vec2f(0.0), qy.xy / qy.z - uv, qy.z > 0.0);
    let nx = i32(clamp(ceil(length(dx)), 1.0, 16.0));
    let ny = i32(clamp(ceil(length(dy)), 1.0, 16.0));
    var sum = vec4f(0.0);
    for (var j = 0; j < ny; j++) {
        for (var i = 0; i < nx; i++) {
            let o = (vec2f(f32(i), f32(j)) + 0.5) / vec2f(f32(nx), f32(ny)) - 0.5;
            let r = to_layer(f.xy + o);
            if r.z > 0.0 { sum += bilinear(r.xy / r.z); }
        }
    }
    return sum / f32(nx * ny);
}

// feColorMatrix on straight values: rows m[0..4] hold [r g b a] coefficients by output channel, m[4] the offsets
@fragment fn matrix(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let c = at(src, vec2i(f.xy));
    let s = select(vec4f(0.0), vec4f(c.rgb / c.a, c.a), c.a > 0.0);
    let o = clamp(vec4f(dot(p.m[0], s), dot(p.m[1], s), dot(p.m[2], s), dot(p.m[3], s)) + p.m[4], vec4f(0.0), vec4f(1.0));
    return vec4f(o.rgb * o.a, o.a);
}

// one direction of a Gaussian blur: p.a.xy the direction, p.a.z sigma, p.a.w the radius in texels
@fragment fn blur(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let q = vec2i(f.xy);
    let d = vec2i(p.a.xy);
    let r = i32(p.a.w);
    let k = -0.5 / (p.a.z * p.a.z);
    var sum = vec4f(0.0);
    var wsum = 0.0;
    for (var i = -r; i <= r; i++) {
        let w = exp(f32(i * i) * k);
        sum += at(src, q + d * i) * w;
        wsum += w;
    }
    return sum / wsum;
}

// src averaged over p.a.x x p.a.x texel blocks (a blur's downsample)
@fragment fn down(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let n = i32(p.a.x);
    let q = vec2i(f.xy) * n;
    var sum = vec4f(0.0);
    for (var y = 0; y < n; y++) {
        for (var x = 0; x < n; x++) {
            sum += at(src, q + vec2i(x, y));
        }
    }
    return sum / f32(n * n);
}

// a texel, the nearest edge's outside the texture
fn edge(t: texture_2d<f32>, q: vec2i) -> vec4f {
    return textureLoad(t, clamp(q, vec2i(0), vec2i(textureDimensions(t)) - 1), 0);
}

// src (p.a.x times smaller) sampled bilinearly back up (a blur's upsample)
@fragment fn up(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let u = f.xy / p.a.x - 0.5;
    let i = vec2i(floor(u));
    let t = u - floor(u);
    let top = mix(edge(src, i), edge(src, i + vec2i(1, 0)), t.x);
    let bottom = mix(edge(src, i + vec2i(0, 1)), edge(src, i + vec2i(1, 1)), t.x);
    return mix(top, bottom, t.y);
}

// src moved by p.a.xy texels
@fragment fn offset(@builtin(position) f: vec4f) -> @location(0) vec4f {
    return at(src, vec2i(f.xy) - vec2i(p.a.xy));
}

// src's alpha in colour p.b (premultiplied), moved by p.a.xy: a drop shadow before its blur
@fragment fn tint(@builtin(position) f: vec4f) -> @location(0) vec4f {
    return p.b * at(src, vec2i(f.xy) - vec2i(p.a.xy)).a;
}

// src composited with other (SVG's feComposite, premultiplied): p.a.x over, in, out, atop, xor, arithmetic (k in p.b)
@fragment fn composite(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let a = at(src, vec2i(f.xy));
    let b = at(other, vec2i(f.xy));
    let m = i32(p.a.x);
    if m == 5 {
        let c = clamp(p.b.x * a * b + p.b.y * a + p.b.z * b + vec4f(p.b.w), vec4f(0.0), vec4f(1.0));
        return vec4f(min(c.rgb, vec3f(c.a)), c.a);
    }
    let fa = select(select(select(1.0, b.a, m == 1 || m == 3), 1.0 - b.a, m == 2 || m == 4), 1.0, m == 0);
    let fb = select(0.0, 1.0 - a.a, m == 0 || m == 3 || m == 4);
    return a * fa + b * fb;
}

fn lin(c: f32) -> f32 {
    return select(pow((c + 0.055) / 1.055, 2.4), c / 12.92, c <= 0.04045);
}

fn srgb(c: f32) -> f32 {
    return select(1.055 * pow(c, 1.0 / 2.4) - 0.055, c * 12.92, c <= 0.0031308);
}

// premultiplied sRGB to premultiplied linear light
@fragment fn tolin(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let c = at(src, vec2i(f.xy));
    if c.a <= 0.0 { return vec4f(0.0); }
    let s = c.rgb / c.a;
    return vec4f(vec3f(lin(s.r), lin(s.g), lin(s.b)) * c.a, c.a);
}

// premultiplied linear light to premultiplied sRGB
@fragment fn tosrgb(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let c = at(src, vec2i(f.xy));
    if c.a <= 0.0 { return vec4f(0.0); }
    let s = clamp(c.rgb / c.a, vec3f(0.0), vec3f(1.0));
    return vec4f(vec3f(srgb(s.r), srgb(s.g), srgb(s.b)) * c.a, c.a);
}

// src over other
@fragment fn over(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let s = at(src, vec2i(f.xy));
    return s + at(other, vec2i(f.xy)) * (1.0 - s.a);
}

fn stop(i: i32) -> f32 {
    return p.o[i / 4][i % 4];
}

// a linear gradient as Chrome draws it (Linear): p.a start and end, p.b.x the stops, p.b.y repeat, p.m[0..2] the
// texel-to-gradient affine; straight alpha out, as Vello's own drawing
@fragment fn linear(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let q = f.xy;
    let g = vec2f(p.m[0].x * q.x + p.m[0].z * q.y + p.m[1].x, p.m[0].y * q.x + p.m[0].w * q.y + p.m[1].y);
    let d = p.a.zw - p.a.xy;
    var t = dot(g - p.a.xy, d) / max(dot(d, d), 1e-12);
    if p.b.y > 0.5 { t = fract(t); }
    let n = i32(p.b.x);
    var c = p.c[0];
    if t > stop(0) {
        c = p.c[n - 1];
        for (var i = 0; i < n - 1; i++) {
            let a = stop(i);
            let b = stop(i + 1);
            if t <= b {
                c = select(mix(p.c[i], p.c[i + 1], (t - a) / max(b - a, 1e-12)), p.c[i], b <= a);
                break;
            }
        }
    }
    return select(vec4f(0.0), vec4f(c.rgb / c.a, c.a), c.a > 0.0);
}

@fragment fn flood(@builtin(position) f: vec4f) -> @location(0) vec4f {
    return p.b;
}
