// A simulated volume (volume.rs): each pixel of its box marches the ray through the 3D field (r density, g the sun's
// light reaching the cell through the volume, b heat). Under a paint look (`bands` > 0) the march stops where the
// density, eroded by noise, crosses `threshold`: the surface. There the light picks one of the look's flat tones (lit,
// mid, dark), torn at its boundaries by the same noise, and heat above `fire_at` picks the fire's three colours: an
// illustrator's smoke, flat shapes with ragged edges. Without one the volume is drawn soft: absorbed and lit.
// Everything is from the field and the row's time: a frame alone equals that frame in play.

import bevy_pbr::render::forward_io::VertexOutput;
import bevy_pbr::render::mesh_view_bindings::view;

struct VolumeParams {
    center: vec3<f32>,
    time: f32,
    size: vec3<f32>,
    steps: f32,
    lit: vec4<f32>,
    mid: vec4<f32>,
    dark: vec4<f32>,
    fire0: vec4<f32>,
    fire1: vec4<f32>,
    fire2: vec4<f32>,
    fire_at: vec4<f32>,
    bands: f32,
    threshold: f32,
    erode: f32,
    erode_size: f32,
    opacity: f32,
    seed: f32,
    shadow: f32,
    _b: f32,
    sun: vec4<f32>,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> p: VolumeParams;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var field: texture_3d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var field_sampler: sampler;

fn grad(c: vec3<i32>) -> vec3<f32> {
    var v = bitcast<vec3<u32>>(c) * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v = v ^ (v >> vec3<u32>(16u));
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    return vec3<f32>(v) * (2.0 / 4294967295.0) - 1.0;
}

fn noise(x: vec3<f32>) -> f32 {
    let i = vec3<i32>(floor(x));
    let f = fract(x);
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let a = dot(grad(i), f);
    let b = dot(grad(i + vec3<i32>(1, 0, 0)), f - vec3<f32>(1.0, 0.0, 0.0));
    let c = dot(grad(i + vec3<i32>(0, 1, 0)), f - vec3<f32>(0.0, 1.0, 0.0));
    let d = dot(grad(i + vec3<i32>(1, 1, 0)), f - vec3<f32>(1.0, 1.0, 0.0));
    let e = dot(grad(i + vec3<i32>(0, 0, 1)), f - vec3<f32>(0.0, 0.0, 1.0));
    let g = dot(grad(i + vec3<i32>(1, 0, 1)), f - vec3<f32>(1.0, 0.0, 1.0));
    let h = dot(grad(i + vec3<i32>(0, 1, 1)), f - vec3<f32>(0.0, 1.0, 1.0));
    let k = dot(grad(i + vec3<i32>(1, 1, 1)), f - vec3<f32>(1.0, 1.0, 1.0));
    return mix(mix(mix(a, b, u.x), mix(c, d, u.x), u.y), mix(mix(e, g, u.x), mix(h, k, u.x), u.y), u.z);
}

fn fbm(x0: vec3<f32>) -> f32 {
    let m = mat3x3<f32>(0.00, 0.80, 0.60, -0.80, 0.36, -0.48, -0.60, -0.48, 0.64);
    var x = x0;
    var s = 0.0;
    var a = 0.6;
    for (var i = 0; i < 3; i++) {
        s += a * noise(x);
        x = m * x * 2.1;
        a *= 0.5;
    }
    return s;
}

/// the field at a world point (rgb: density, light, heat), and the erosion there
fn at(x: vec3<f32>) -> vec4<f32> {
    let q = (x - (p.center - p.size * 0.5)) / p.size;
    let f = textureSampleLevel(field, field_sampler, q, 0.0);
    let e = fbm(x / p.erode_size + vec3<f32>(p.seed * 1.7, -p.time * 0.35, p.seed * 0.3));
    return vec4<f32>(f.rgb, e);
}

/// the density at a world point (no erosion)
fn dens(x: vec3<f32>) -> f32 {
    let q = (x - (p.center - p.size * 0.5)) / p.size;
    return textureSampleLevel(field, field_sampler, q, 0.0).r;
}

/// a surface point's flat tone: `x` the point, `rd` the ray. The light is how square the surface (the density's
/// gradient) faces the sun and how much sun reaches a little way in (its own shadow), torn by the erosion noise; heat
/// a little way in picks the fire's colours.
fn tone(x: vec3<f32>, rd: vec3<f32>) -> vec3<f32> {
    let f = at(x);
    let cell = max(max(p.size.x, p.size.y), p.size.z) / 64.0;
    let h = max(cell * 2.0, 0.15);
    let g = vec3<f32>(
        dens(x + vec3<f32>(h, 0.0, 0.0)) - dens(x - vec3<f32>(h, 0.0, 0.0)),
        dens(x + vec3<f32>(0.0, h, 0.0)) - dens(x - vec3<f32>(0.0, h, 0.0)),
        dens(x + vec3<f32>(0.0, 0.0, h)) - dens(x - vec3<f32>(0.0, 0.0, h)),
    );
    var n = -rd;
    if length(g) > 1e-4 {
        n = -normalize(g);
    }
    let inside = at(x + rd * h * 2.0);
    let heat = max(f.b, inside.b) + f.a * 0.15;
    if p.fire_at.w > 0.0 && heat > p.fire_at.x {
        if heat > p.fire_at.z {
            return p.fire2.rgb;
        }
        if heat > p.fire_at.y {
            return p.fire1.rgb;
        }
        return p.fire0.rgb;
    }
    // the light, torn at its tone boundaries
    let ndl = dot(n, p.sun.xyz);
    let l = 0.5 * inside.g + 0.5 * clamp(ndl * 0.6 + 0.4, 0.0, 1.0) + f.a * 0.25;
    let k = round(p.bands);
    if k < 1.5 {
        return p.lit.rgb;
    }
    if k < 2.5 {
        return select(p.dark.rgb, p.lit.rgb, l > 0.45);
    }
    if l > 0.58 {
        return p.lit.rgb;
    }
    if l > 0.3 {
        return p.mid.rgb;
    }
    return p.dark.rgb;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ro = view.world_position;
    let rd = normalize(in.world_position.xyz - ro);
    let hs = p.size * 0.5;
    let lo = p.center - hs;
    let hi = p.center + hs;
    let inv = 1.0 / rd;
    let ta = (lo - ro) * inv;
    let tb = (hi - ro) * inv;
    let tn = min(ta, tb);
    let tf = max(ta, tb);
    let t0 = max(max(max(tn.x, tn.y), tn.z), 0.0);
    let t1 = min(min(tf.x, tf.y), tf.z);
    if t1 <= t0 {
        discard;
    }
    let n = max(p.steps, 1.0);
    let dt = (t1 - t0) / n;
    let j = fract(52.9829189 * fract(dot(floor(in.position.xy), vec2<f32>(0.06711056, 0.00583715))));
    if p.bands > 0.5 {
        // paint: the first point past the surface, found by marching then halving back to it
        var prev = t0;
        for (var i = 0; i < i32(n); i++) {
            let t = t0 + (f32(i) + j) * dt;
            let f = at(ro + rd * t);
            if f.r + f.a * p.erode > p.threshold {
                var a = prev;
                var b = t;
                for (var k = 0; k < 5; k++) {
                    let m = 0.5 * (a + b);
                    let g = at(ro + rd * m);
                    if g.r + g.a * p.erode > p.threshold {
                        b = m;
                    } else {
                        a = m;
                    }
                }
                return vec4<f32>(tone(ro + rd * b, rd), 1.0);
            }
            prev = t;
        }
        discard;
    }
    // soft: absorbed and lit, front to back
    var col = vec3<f32>(0.0);
    var trans = 1.0;
    for (var i = 0; i < i32(n); i++) {
        let t = t0 + (f32(i) + j) * dt;
        let f = at(ro + rd * t);
        let d = max(f.r + f.a * p.erode * 0.5, 0.0);
        if d > 0.001 {
            let a = 1.0 - exp(-d * p.opacity * dt);
            var c = mix(p.dark.rgb, p.lit.rgb, f.g);
            if p.fire_at.w > 0.0 && f.b > p.fire_at.x {
                c += mix(p.fire0.rgb, p.fire2.rgb, smoothstep(p.fire_at.x, p.fire_at.z, f.b)) * 4.0;
            }
            col += c * a * trans;
            trans *= 1.0 - a;
            if trans < 0.01 {
                break;
            }
        }
    }
    if trans > 0.99 {
        discard;
    }
    return vec4<f32>(col / max(1.0 - trans, 1e-4), 1.0 - trans);
}
