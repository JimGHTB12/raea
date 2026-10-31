// A simulated volume in a light's shadow map (volume.rs; the volume drawn is volume.wesl). The shadow pass draws the
// volume's box from the light, and each texel marches the light's ray through the field and writes the depth where
// the volume stops the light, so the shadow maps hold the volume as they hold any mesh: its shadow falls on meshes and
// models, and the volumetric fog (march and froxel modes) walks the same maps, so the sun's shafts break round it.
//
// A shadow map holds one depth a texel: lit or not. A texel stops where the volume has stopped half the light (its
// optical depth, as the drawing's, times `shadow`, passes ln 2), so the shadow is the volume's dense body, and the
// light's shadow filter (PCSS with `softShadow`) softens its edge with the distance to what it falls on. (A per-texel
// share of the light, dithered, averages to the volume's own density, but up close a texel is smaller than a pixel and
// the dither shows as grain: measured on the volume-light fixture, so not done.) Under a paint look the texel stops at
// the painted surface, the same eroded threshold the drawing stops at. Everything is from the field and the row's
// time: a frame alone equals that frame in play.

import bevy_pbr::prepass::io::VertexOutput;
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

// the erosion noise, as volume.wesl's (the same tears in the shadow as in the drawing)
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

/// the density at a world point
fn dens(x: vec3<f32>) -> f32 {
    let q = (x - (p.center - p.size * 0.5)) / p.size;
    return textureSampleLevel(field, field_sampler, q, 0.0).r;
}

/// the density at a world point and the erosion there
fn at(x: vec3<f32>) -> vec2<f32> {
    let q = (x - (p.center - p.size * 0.5)) / p.size;
    let f = textureSampleLevel(field, field_sampler, q, 0.0);
    let e = fbm(x / p.erode_size + vec3<f32>(p.seed * 1.7, -p.time * 0.35, p.seed * 0.3));
    return vec2<f32>(f.r, e);
}

struct ShadowOut {
    @builtin(frag_depth) depth: f32,
}

@fragment
fn fragment(in: VertexOutput) -> ShadowOut {
    let x = in.world_position.xyz;
    // the light's ray through this texel: a sun's cascade looks along its forward (orthographic), a spot's or a
    // bulb's face out from the light (perspective)
    var rd: vec3<f32>;
    if view.clip_from_view[3][3] > 0.5 {
        rd = -normalize(view.world_from_view[2].xyz);
    } else {
        rd = normalize(x - view.world_position);
    }
    // the ray's piece in the box, from well before it
    let hs = p.size * 0.5;
    let back = length(p.size) * 2.0;
    let ro = x - rd * back;
    let inv = 1.0 / rd;
    let ta = (p.center - hs - ro) * inv;
    let tb = (p.center + hs - ro) * inv;
    let tn = min(ta, tb);
    let tf = max(ta, tb);
    let t0 = max(max(max(tn.x, tn.y), tn.z), 0.0);
    let t1 = min(min(tf.x, tf.y), tf.z);
    if t1 <= t0 {
        discard;
    }
    // samples at each step's middle, the same for every texel (a per-texel offset would grain the edge)
    // a quarter of the drawing's samples: a shadow map texel needs the depth only roughly, and the field is coarse
    let n = clamp(p.steps * 0.25, 12.0, 32.0);
    let dt = (t1 - t0) / n;
    var hit = 0.0;
    var found = false;
    if p.bands > 0.5 {
        // paint: the painted surface
        var prev = t0;
        for (var i = 0; i < i32(n); i++) {
            let t = t0 + (f32(i) + 0.5) * dt;
            let f = at(ro + rd * t);
            if f.x + f.y * p.erode > p.threshold {
                var a = prev;
                var b = t;
                for (var k = 0; k < 3; k++) {
                    let m = 0.5 * (a + b);
                    let g = at(ro + rd * m);
                    if g.x + g.y * p.erode > p.threshold {
                        b = m;
                    } else {
                        a = m;
                    }
                }
                hit = b;
                found = true;
                break;
            }
            prev = t;
        }
    } else {
        // soft: the depth by which the volume has stopped half the light, its optical depth (density
        // times the drawing's opacity) times `shadow`
        let want = 0.6931472;
        var tau = 0.0;
        for (var i = 0; i < i32(n); i++) {
            let t = t0 + (f32(i) + 0.5) * dt;
            // the density alone: the erosion's noise only frays the drawn edge, and costs most of a sample
            let d = max(dens(ro + rd * t), 0.0) * p.opacity * p.shadow * dt;
            if tau + d >= want {
                // where in the step it crossed
                hit = t - dt + dt * clamp((want - tau) / max(d, 1e-6), 0.0, 1.0);
                found = true;
                break;
            }
            tau += d;
        }
    }
    if !found {
        discard;
    }
    let c = view.clip_from_world * vec4<f32>(ro + rd * max(hit, t0), 1.0);
    var out: ShadowOut;
    // casters nearer the light than the map's near plane are clamped to it, as Bevy's unclipped depth does
    out.depth = clamp(c.z / c.w, 0.0, 1.0);
    return out;
}
