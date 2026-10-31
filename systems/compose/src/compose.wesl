// Composition passes (compose.rs): which pass drew each pixel, and the frame-wide effects held back from the passes
// that skip them.
//
// The pass a pixel is in is the nearest surface's, as the normal prepass keeps it in its alpha (two bits: 1 for
// "scene", the default and what nothing drawn reads as; 0, 1/3 and 2/3 for the piece's passes 1 to 3; MeshTag gives a
// mesh its pass, prepass_pass in compose.rs writes it).
//   mask_h, mask_v: the passes' coverage (r, g, b: passes 1 to 3), blurred as depth of field blurs the frame (Bevy's
//                   Gaussian: sigma a quarter of each pixel's circle of confusion, two separable passes; its bokeh as an
//                   even blur across the circle's radius), into the mask the looks and the grade read; without depth of
//                   field, each pixel's own pass
//   hold:           a pixel's colour less what its pass holds back from the bloom (it doesn't glow)
//   give:           the colour kept before an effect put back by how much the pixel's pass holds back from it

import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

struct Compose {
    // how much each pass holds back from the effect: x "scene" (0), y z w passes 1 to 3
    hold: vec4<f32>,
    // depth of field: the focal distance (m), focal length, the circle of confusion's scale and its widest (pixels)
    coc: vec4<f32>,
    // the near plane, the farthest depth of field blurs by, and the blur's direction (x, y)
    z: vec4<f32>,
};

@group(0) @binding(0) var<uniform> c: Compose;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var aux: texture_2d<f32>;
@group(0) @binding(3) var depth_tex: texture_depth_2d;

fn load(t: texture_2d<f32>, p: vec2<i32>) -> vec4<f32> {
    let s = vec2<i32>(textureDimensions(t));
    return textureLoad(t, clamp(p, vec2<i32>(0), s - vec2<i32>(1)), 0);
}

// a pixel's pass, one-hot (x scene, y z w passes 1 to 3), from the normal prepass's alpha
fn pass_at(p: vec2<i32>) -> vec4<f32> {
    let k = (u32(round(load(src, p).a * 3.0)) + 1u) % 4u;
    return vec4<f32>(f32(k == 0u), f32(k == 1u), f32(k == 2u), f32(k == 3u));
}

// depth of field's circle of confusion at a pixel, as Bevy's dof.wesl works it out (0 without depth of field)
fn coc_at(p: vec2<i32>) -> f32 {
    if c.coc.z <= 0.0 {
        return 0.0;
    }
    let s = vec2<i32>(textureDimensions(depth_tex));
    let raw = textureLoad(depth_tex, clamp(p, vec2<i32>(0), s - vec2<i32>(1)), 0);
    // Bevy's perspective is reverse-Z to infinity: depth = near / distance
    let d = min(c.z.x / max(raw, 1e-9), c.z.y);
    let k = c.coc.z * abs(d - c.coc.x) / (d * max(c.coc.x - c.coc.y, 1e-6));
    return clamp(k * f32(s.y), 0.0, c.coc.w);
}

// one direction of the blur: `tap` read along the direction, weighted as Bevy's gaussian_blur weights, or for its
// bokeh (`hold.x` 1) evenly across the circle's radius (Bevy's hexagon taken as a square)
fn blur(p: vec2<i32>, horizontal: bool) -> vec4<f32> {
    let dir = vec2<i32>(i32(c.z.z), i32(c.z.w));
    let coc = coc_at(p);
    if c.hold.x > 0.5 {
        let r = min(i32(round(coc * 0.5)), 48);
        var sum = tap(p, horizontal);
        for (var i = 1; i <= r; i++) {
            sum += tap(p + dir * i, horizontal) + tap(p - dir * i, horizontal);
        }
        return sum / f32(1 + 2 * r);
    }
    let sigma = coc * 0.25;
    let support = min(i32(ceil(sigma * 1.5)), 48);
    if support <= 0 {
        return tap(p, horizontal);
    }
    let e = -1.0 / (2.0 * sigma * sigma);
    var sum = tap(p, horizontal);
    var w = 1.0;
    for (var i = 1; i <= support; i++) {
        let k = exp(e * f32(i * i));
        sum += (tap(p + dir * i, horizontal) + tap(p - dir * i, horizontal)) * k;
        w += 2.0 * k;
    }
    return sum / w;
}

fn tap(p: vec2<i32>, horizontal: bool) -> vec4<f32> {
    if horizontal {
        return pass_at(p);
    }
    return load(src, p);
}

@fragment
fn mask_h(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return blur(vec2<i32>(floor(in.position.xy)), true);
}

@fragment
fn mask_v(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // the blurred one-hot's scene share falls away: r g b are passes 1 to 3
    let m = blur(vec2<i32>(floor(in.position.xy)), false);
    return vec4<f32>(m.yzw, 1.0);
}

// how much of this pixel is held back, by the sharp pass (`aux` the normal prepass: before depth of field) or the
// mask (`aux` the mask, sampled where this pixel falls on it: after depth of field, and on the composite's frame)
fn held_sharp(p: vec2<i32>) -> f32 {
    let k = (u32(round(load(aux, p).a * 3.0)) + 1u) % 4u;
    return c.hold[k];
}

fn held_mask(uv: vec2<f32>) -> f32 {
    let s = vec2<f32>(textureDimensions(aux));
    let m = load(aux, vec2<i32>(floor(uv * s))).rgb;
    return clamp(dot(m, c.hold.yzw), 0.0, 1.0);
}

@fragment
fn hold(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let p = vec2<i32>(floor(in.position.xy));
    let col = load(src, p);
    return vec4<f32>(col.rgb * (1.0 - held_sharp(p)), col.a);
}

// `src` the frame after the effect, the kept colour bound at 3's place as a texture (see compose.rs: `kept`)
@group(0) @binding(4) var kept: texture_2d<f32>;

@fragment
fn give_sharp(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let p = vec2<i32>(floor(in.position.xy));
    return mix(load(src, p), load(kept, p), held_sharp(p));
}

@fragment
fn give_mask(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let p = vec2<i32>(floor(in.position.xy));
    return mix(load(src, p), load(kept, p), held_mask(in.uv));
}
