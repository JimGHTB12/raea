// A screen's picture as a CRT shows it (screen.rs): the source's texels read exactly (textureLoad), each row a beam
// whose width grows with its brightness (scanlines), the beam's colours a little apart (convergence), seen through an
// aperture grille or a shadow mask, with a phosphor halo, the barrel of the glass, a vignette and a flicker seeded by
// the film frame. Scanlines and the mask fade where they would be finer than the frame's pixels, so the tube reads
// as a tube from across the room without moiré and as phosphor up close. Emissive: it lights the bloom; the glass has
// its own colour and a sheen toward grazing angles. Nothing here reads a clock.

import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct CrtParams {
    map_a: vec4<f32>,
    map_b: vec4<f32>,
    res: vec4<f32>,
    a: vec4<f32>,
    b: vec4<f32>,
    c: vec4<f32>,
    glass: vec4<f32>,
    d: vec4<f32>,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> p: CrtParams;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var src: texture_2d<f32>;

fn hash(x: f32) -> f32 {
    return fract(sin(x * 12.9898 + 78.233) * 43758.5453);
}

fn hash2(q: vec2<f32>) -> f32 {
    return fract(sin(dot(q, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn vnoise(q: vec2<f32>) -> f32 {
    let i = floor(q);
    let f = fract(q);
    let u = f * f * (3.0 - 2.0 * f);
    let x0 = mix(hash2(i), hash2(i + vec2<f32>(1.0, 0.0)), u.x);
    let x1 = mix(hash2(i + vec2<f32>(0.0, 1.0)), hash2(i + vec2<f32>(1.0, 1.0)), u.x);
    return mix(x0, x1, u.y);
}

// A fingerprint on the glass at c (glass units: the face's height is 1), r its size, turned by a: whorled ridges in a
// soft, uneven oval, the ridges faded where they'd be finer than the frame's pixels.
fn fingerprint(g: vec2<f32>, c: vec2<f32>, r: f32, a: f32, seed: f32) -> f32 {
    let d0 = g - c;
    let cs = vec2<f32>(cos(a), sin(a));
    let d = vec2<f32>(d0.x * cs.x + d0.y * cs.y, d0.y * cs.x - d0.x * cs.y) / vec2<f32>(r * 0.78, r);
    let rr = length(d) + (vnoise(d * 2.5 + seed) - 0.5) * 0.3;
    let env = smoothstep(1.0, 0.3, rr) * (0.45 + 0.55 * vnoise(d * 1.7 + seed * 1.37));
    let ph = (rr * r / 0.011 + 0.4 * d.x) * 6.2831853;
    let aa = clamp(1.0 - fwidth(ph) * 0.3, 0.0, 1.0);
    return env * mix(0.55, 0.5 + 0.5 * sin(ph), 0.85 * aa);
}

// A wiped arc: a band of fine streaks along a circle (centre c, radius big_r, half-width w) between angles a0 and a1.
fn swipe(g: vec2<f32>, c: vec2<f32>, big_r: f32, w: f32, a0: f32, a1: f32, seed: f32) -> f32 {
    let d = g - c;
    let rad = length(d);
    let ang = atan2(d.y, d.x);
    let along = smoothstep(a0, a0 + 0.12, ang) * smoothstep(a1, a1 - 0.3, ang);
    let across = smoothstep(w, w * 0.2, abs(rad - big_r + (vnoise(vec2<f32>(ang * 5.0, seed)) - 0.5) * w));
    let ph = rad * 700.0 + vnoise(vec2<f32>(ang * 30.0, seed + 5.0)) * 5.0;
    let aa = clamp(1.0 - fwidth(ph) * 0.3, 0.0, 1.0);
    return along * across * mix(0.6, 0.5 + 0.5 * sin(ph), 0.7 * aa) * (0.4 + 0.6 * vnoise(vec2<f32>(ang * 8.0, seed + 9.0)));
}

// Dust specks: a few cells in each of two grids hold a dot.
fn specks(g: vec2<f32>, seed: f32) -> f32 {
    var sum = 0.0;
    for (var k = 0; k < 2; k++) {
        let scale = select(85.0, 190.0, k == 1);
        let q = g * scale + seed * 3.1;
        let i = floor(q);
        let h = hash2(i + seed);
        let o = vec2<f32>(hash2(i + seed + 7.1), hash2(i + seed + 13.3)) * 0.6 + 0.2;
        let rad = 0.07 + 0.09 * hash2(i + seed + 3.3);
        sum += step(0.955, h) * smoothstep(rad, rad * 0.25, length(fract(q) - o)) * (0.5 + 0.5 * hash2(i + seed + 21.7));
    }
    return sum;
}

// The glass's smudging at g: two fingerprints and a part print by the lower left, one by the lower right, a wiped arc
// at each lower corner and a faint film, all fixed by the seed. Returns (smear, dust).
fn smudging(g: vec2<f32>, asp: f32, seed: f32) -> vec2<f32> {
    let j = vec2<f32>(hash(seed + 1.0), hash(seed + 2.0)) - 0.5;
    var sm = fingerprint(g, vec2<f32>(0.15 * asp, 0.83) + j * 0.04, 0.075, 0.5 + j.x, seed);
    sm += 0.7 * fingerprint(g, vec2<f32>(0.24 * asp, 0.92) - j * 0.03, 0.05, 1.1, seed + 11.0);
    sm += 0.85 * fingerprint(g, vec2<f32>(0.86 * asp, 0.79) + j.yx * 0.04, 0.068, -0.6 - j.y, seed + 23.0);
    sm += 0.8 * swipe(g, vec2<f32>(-0.15 * asp, 1.35), 0.62, 0.035, -1.25, -0.35, seed + 31.0);
    sm += 0.6 * swipe(g, vec2<f32>(1.2 * asp, 1.3), 0.5, 0.03, -2.75, -1.9, seed + 41.0);
    sm += 0.35 * smoothstep(0.35, 1.0, g.y) * vnoise(g * 5.0 + seed);
    return vec2<f32>(clamp(sm, 0.0, 1.5), specks(g, seed));
}

fn texel(x: i32, y: i32) -> vec3<f32> {
    let d = vec2<i32>(textureDimensions(src));
    let q = clamp(vec2<i32>(x, y), vec2<i32>(0), d - 1);
    return textureLoad(src, q, 0).rgb;
}

// one row of the source at x (texels), a little soft between texels as a CRT's beam is
fn row(x: f32, y: i32) -> vec3<f32> {
    let fx = x - 0.5;
    let i = i32(floor(fx));
    let t = clamp((fx - floor(fx) - 0.5) * 2.5 + 0.5, 0.0, 1.0);
    return mix(texel(i, y), texel(i + 1, y), t);
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}

fn to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// the picture at (x, y) in texels: each row a beam, its width growing with its brightness
fn beam(x: f32, y: f32, scan: f32) -> vec3<f32> {
    let r0 = i32(floor(y - 0.5));
    let c0 = row(x, r0);
    let c1 = row(x, r0 + 1);
    let d0 = y - (f32(r0) + 0.5);
    let d1 = d0 - 1.0;
    let s0 = mix(0.26, 0.42, sqrt(clamp(luma(c0), 0.0, 1.0)));
    let s1 = mix(0.26, 0.42, sqrt(clamp(luma(c1), 0.0, 1.0)));
    let w0 = exp(-(d0 * d0) / (s0 * s0)) / (1.7725 * s0);
    let w1 = exp(-(d1 * d1) / (s1 * s1)) / (1.7725 * s1);
    let lined = c0 * w0 + c1 * w1;
    let flat = mix(c0, c1, clamp(d0, 0.0, 1.0));
    return mix(flat, lined, scan);
}

// a tube's own scanlines at `ly` (in lines): each line a beam whose width grows with the light it carries, averaging
// 1 over a line, so the picture keeps its brightness
fn lines_at(ly: f32, l: f32) -> f32 {
    let d = fract(ly) - 0.5;
    let sg = mix(0.26, 0.42, sqrt(clamp(l, 0.0, 1.0)));
    var w = 0.0;
    for (var k = -1; k <= 1; k++) {
        let e = d + f32(k);
        w += exp(-(e * e) / (sg * sg));
    }
    return w / (1.7725 * sg);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    var s = vec2<f32>(dot(p.map_a.xy, uv) + p.map_b.x, dot(p.map_a.zw, uv) + p.map_b.y);
    let crt = p.c.w > 0.5;
    let res = p.res.xy;
    let frame = p.res.z;
    let power = clamp(p.res.w, 0.0, 1.0);
    // the glass and its sheen toward grazing angles
    let n = normalize(in.world_normal);
    let v = normalize(view.world_position.xyz - in.world_position.xyz);
    let fres = pow(1.0 - clamp(abs(dot(n, v)), 0.0, 1.0), 4.0);
    // smudges and dust: seen in the sheen and as a faint lift of the dark glass, barely over the picture
    let mud = smudging(vec2<f32>(s.x * res.x / res.y, s.y), res.x / res.y, p.d.y) * p.d.x;
    let haze = mud.x * (0.1 + fres * 2.0 * p.c.y) + mud.y * (0.3 + fres * 0.5);
    let glass = p.glass.rgb * (0.6 + 0.4 * p.c.y) + vec3<f32>(fres * 0.18 * p.c.y) + vec3<f32>(1.0, 0.97, 0.92) * haze;
    let dim = 1.0 - 0.1 * mud.x - 0.3 * mud.y;
    // overscan, then the barrel
    var q = (s * 2.0 - 1.0) / (1.0 + p.c.z);
    q = q * (1.0 + p.a.x * dot(q, q) * 0.25);
    // warming up: a line that opens to the picture
    let open = max(power * power * 1.05 - 0.05, 0.0);
    if (crt && power < 1.0) {
        q.y = q.y / max(open, 0.004);
    }
    s = q * 0.5 + 0.5;
    if (any(s < vec2<f32>(0.0)) || any(s > vec2<f32>(1.0))) {
        return vec4<f32>(glass, 1.0);
    }
    let px = s * res;
    var col: vec3<f32>;
    if (!crt) {
        // flat: the page's texel at its own colour, unlit. The page is premultiplied in sRGB (the paint's own blending),
        // read here through an sRGB view: back to sRGB, its alpha divided out, then to linear; a see-through screen
        // (d.w) is blended by that alpha, an opaque one is drawn over its glass as before
        let d = vec2<i32>(textureDimensions(src));
        let t = textureLoad(src, clamp(vec2<i32>(floor(px)), vec2<i32>(0), d - 1), 0);
        if (p.d.w > 0.5) {
            if (t.a <= 0.0) {
                discard;
            }
            let straight = to_linear(min(to_srgb(t.rgb) / t.a, vec3<f32>(1.0)));
            return vec4<f32>(straight * p.c.x * dim, t.a);
        }
        // no glass under it: the glass's own colour would lift every colour of the page
        return vec4<f32>(t.rgb * p.c.x * dim, 1.0);
    } else {
        // the grid the scanlines and the mask follow: the picture's texels, or the tube's own `lines` (a sharp picture
        // on a tube of a few hundred lines, whose texels would be finer than the frame's pixels)
        let lines = p.d.z;
        let grid = select(px, s * vec2<f32>(lines * res.x / res.y, lines), lines > 0.0);
        // how many frame pixels a line spans: scanlines and the mask fade out where they'd be finer than a pixel
        let dy = max(fwidth(grid.y), 1e-4);
        let scan = p.a.y * (1.0 - smoothstep(0.3, 0.6, dy));
        let conv = p.b.y * (0.35 + abs(q.x));
        if (lines > 0.0) {
            let y = i32(floor(px.y));
            let c = vec3<f32>(row(px.x + conv, y).r, row(px.x, y).g, row(px.x - conv, y).b);
            col = c * mix(1.0, lines_at(grid.y, luma(c)), scan);
        } else {
            let r = beam(px.x + conv, px.y, scan).r;
            let g = beam(px.x, px.y, scan).g;
            let b = beam(px.x - conv, px.y, scan).b;
            col = vec3<f32>(r, g, b);
        }
        // phosphor halo: a wide soft average around the point
        if (p.b.x > 0.0) {
            var halo = vec3<f32>(0.0);
            var wsum = 0.0;
            for (var j = -2; j <= 2; j++) {
                for (var i = -3; i <= 3; i++) {
                    let w = exp(-f32(i * i) / 6.0 - f32(j * j) / 2.5);
                    halo += texel(i32(floor(px.x)) + i * 2, i32(floor(px.y)) + j * 2) * w;
                    wsum += w;
                }
            }
            col += halo / wsum * p.b.x * 0.45;
        }
        // the mask
        let k = p.a.w;
        if (k > 0.5 && p.a.z > 0.0) {
            let mx = grid.x * p.glass.w * 3.0;
            let dm = max(fwidth(mx), 1e-4);
            let fade = 1.0 - smoothstep(0.35, 0.8, dm);
            var ph = mx;
            if (k > 1.5) {
                // shadow mask: each other row of triads half a triad along
                ph = mx + 1.5 * f32(i32(floor(grid.y * p.glass.w * 1.5)) & 1);
            }
            let cell = fract(ph / 3.0) * 3.0;
            let lit = vec3<f32>(
                smoothstep(0.0, 0.25, cell) * (1.0 - smoothstep(0.75, 1.0, cell)),
                smoothstep(1.0, 1.25, cell) * (1.0 - smoothstep(1.75, 2.0, cell)),
                smoothstep(2.0, 2.25, cell) * (1.0 - smoothstep(2.75, 3.0, cell)),
            );
            let m = mix(vec3<f32>(1.0), lit * 2.2 + 0.25, p.a.z * fade);
            col *= m;
        }
        // vignette: the beam's light falls off at the tube's edges and corners
        let e = s * (1.0 - s);
        col *= pow(clamp(e.x * e.y * 16.0, 0.0, 1.0), p.b.z * 0.6);
        // a soft rounded edge to the lit area
        let edge = smoothstep(0.0, 0.012, min(min(s.x, 1.0 - s.x), min(s.y, 1.0 - s.y)));
        col *= edge;
        // flicker, seeded by the frame, and a faint hum bar rolling down
        let fl = 1.0 - p.b.w * hash(frame * 0.6180339 + 0.37);
        let bar = 1.0 - p.b.w * 0.6 * smoothstep(0.0, 0.5, sin((s.y - frame * 0.011) * 6.2831853) * 0.5 + 0.5) * 0.35;
        col *= fl * bar;
        // warming up: the line glows hot before it opens
        if (power < 1.0) {
            col = col * power + vec3<f32>((1.0 - power) * smoothstep(0.0, 0.15, power) * 3.0) * smoothstep(0.2, 0.0, abs(q.y) * max(open, 0.004));
        }
    }
    return vec4<f32>(glass + col * p.c.x * dim, 1.0);
}
