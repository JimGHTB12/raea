// lib.rs: a pattern over the standard material (Bevy's extended-material fragment, its lighting kept). The pattern is
// taken in the mesh's own space (it rides with the part), on the face's dominant plane for the flat ones (brick, tiles,
// fabric, brushed); its height bumps the normal (surface gradient, screen space); then use: dust on what faces up, dirt
// in the gaps, hollows and the floor line, edges worn lighter.
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::alpha_discard;
import bevy_pbr::render::mesh_functions::get_local_from_world;
@if(PREPASS_PIPELINE) import bevy_pbr::prepass::io::{VertexOutput, FragmentOutput};
@if(PREPASS_PIPELINE) import bevy_pbr::deferred::functions::deferred_output;
@if(!PREPASS_PIPELINE) import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput};
@if(!PREPASS_PIPELINE) import bevy_pbr::render::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing};

struct Proc {
    a: vec4<f32>,
    b: vec4<f32>,
    gap_color: vec4<f32>,
    dust_color: vec4<f32>,
    dirt_color: vec4<f32>,
    // kind, scale, rough_a, rough_b
    k: vec4<f32>,
    // bump, seed, tile_x, tile_y
    t: vec4<f32>,
    // gap, dust, dirt, edges
    w: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> pm: Proc;

fn hash(p: vec3<f32>) -> f32 {
    let q = fract(p * 0.3183099 + vec3(0.1, 0.2, 0.3));
    let r = q * 17.0;
    return fract(r.x * r.y * r.z * (r.x + r.y + r.z));
}

fn vnoise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(mix(hash(i), hash(i + vec3(1.0, 0.0, 0.0)), u.x), mix(hash(i + vec3(0.0, 1.0, 0.0)), hash(i + vec3(1.0, 1.0, 0.0)), u.x), u.y);
    let b = mix(mix(hash(i + vec3(0.0, 0.0, 1.0)), hash(i + vec3(1.0, 0.0, 1.0)), u.x), mix(hash(i + vec3(0.0, 1.0, 1.0)), hash(i + vec3(1.0, 1.0, 1.0)), u.x), u.y);
    return mix(a, b, u.z);
}

fn fbm(p: vec3<f32>) -> f32 {
    var s = 0.0;
    var w = 0.5;
    var q = p;
    for (var k = 0; k < 5; k++) {
        s += w * vnoise(q);
        q = q * 2.03 + vec3(1.7, 9.2, 3.1);
        w *= 0.5;
    }
    return s / 0.96875;
}

// the face's own plane: the two axes across the dominant normal (a wall's x-y, a floor's x-z)
fn plane(p: vec3<f32>, n: vec3<f32>) -> vec2<f32> {
    let a = abs(n);
    if (a.y >= a.x && a.y >= a.z) { return p.xz; }
    if (a.x >= a.z) { return vec2(p.z, p.y); }
    return p.xy;
}

// (mix between a and b, gap mask, height 0..1)
fn pattern(p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let kind = i32(pm.k.x + 0.5);
    let q = p / max(pm.k.y, 1e-4) + vec3(pm.t.y * 17.3);
    let uv = plane(p, n);
    // wood: rings round the local y axis, wandering, and fine grain along it
    if (kind == 1) {
        let r = length(q.xz + vec2(fbm(q * 0.4), fbm(q * 0.4 + 7.0)) * 0.6) * 4.0;
        let ring = smoothstep(0.0, 0.6, fract(r)) * (1.0 - smoothstep(0.75, 1.0, fract(r)));
        let grain = vnoise(vec3(q.x * 30.0, q.y * 1.5, q.z * 30.0));
        let m = clamp(ring * 0.75 + grain * 0.35, 0.0, 1.0);
        return vec3(m, 0.0, 0.3 * ring + 0.2 * grain);
    }
    // marble: sharp veins through soft clouding
    if (kind == 2) {
        let v = pow(1.0 - abs(sin(q.x * 2.0 + q.y * 0.7 + fbm(q * 0.8) * 7.0)), 10.0);
        let cloud = fbm(q * 0.5) * 0.3;
        return vec3(clamp(v + cloud, 0.0, 1.0), 0.0, 0.05 * v);
    }
    // stone / concrete: mottled, pitted, speckled
    if (kind == 3) {
        let m = fbm(q);
        let pits = smoothstep(0.78, 0.84, vnoise(q * 9.0));
        let speck = step(0.93, hash(floor(q * 30.0)));
        return vec3(clamp(m * 0.8 + speck * 0.4, 0.0, 1.0), pits, m * 0.6 - pits * 0.5);
    }
    // brick (running bond) and tiles: cells on the face's plane with gaps between, each cell its own shade
    if (kind == 4 || kind == 5) {
        let size = vec2(max(pm.t.z, 1e-3), max(pm.t.w, 1e-3));
        var c = uv / size;
        let row = floor(c.y);
        if (kind == 4) { c.x += 0.5 * (row - 2.0 * floor(row / 2.0)); }
        let id = floor(c);
        let f = fract(c);
        let g = vec2(pm.w.x) / size;
        let edge = min(min(f.x, 1.0 - f.x) / max(g.x, 1e-4), min(f.y, 1.0 - f.y) / max(g.y, 1e-4));
        let gap = 1.0 - smoothstep(0.5, 1.0, edge);
        let shade = hash(vec3(id, pm.t.y)) * 0.7 + fbm(q * 2.0) * 0.3;
        let bevel = smoothstep(0.5, 2.0, edge);
        return vec3(shade, gap, bevel * (0.8 + 0.2 * fbm(q * 3.0)));
    }
    // fabric: a plain weave
    if (kind == 6) {
        let f = uv / max(pm.k.y, 1e-4) * 6.2831;
        let warp = 0.5 + 0.5 * sin(f.x) * sign(sin(f.y * 0.5));
        let weft = 0.5 + 0.5 * sin(f.y) * sign(sin(f.x * 0.5 + 1.57));
        // threads finer than a pixel fade to the weave's mean shade, so a distant sofa doesn't shimmer in moire
        let far = smoothstep(0.8, 2.5, length(fwidth(f)));
        let m = mix(max(warp, weft), 0.72, far);
        return vec3(clamp(m * 0.6 + fbm(q * 0.3) * 0.4, 0.0, 1.0), 0.0, m * (1.0 - far));
    }
    // brushed metal: long fine streaks along the face
    if (kind == 7) {
        let sy = uv.y / max(pm.k.y, 1e-4) * 60.0;
        let s = mix(vnoise(vec3(uv.x / max(pm.k.y, 1e-4) * 0.4, sy, pm.t.y)), 0.5, smoothstep(0.5, 1.5, fwidth(sy)));
        return vec3(s * 0.6 + fbm(q * 0.2) * 0.4, 0.0, s * 0.15);
    }
    // plaster: soft trowel marks and fine bumps
    if (kind == 8) {
        let m = fbm(q * 0.6);
        let fine = vnoise(q * 18.0);
        return vec3(m, 0.0, m * 0.5 + fine * 0.2);
    }
    // noise: two colours by fractal noise
    let m = fbm(q);
    return vec3(m, 0.0, m * 0.5);
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    var p = in.world_position.xyz;
    var n = normalize(in.world_normal);
@if(VERTEX_OUTPUT_INSTANCE_INDEX) {
    let m = get_local_from_world(in.instance_index);
    p = (m * vec4(in.world_position.xyz, 1.0)).xyz;
    n = normalize((m * vec4(n, 0.0)).xyz);
}
    let pat = pattern(p, n);
    let t = pat.x;
    let gap = pat.y;

    var c = mix(pm.a.rgb, pm.b.rgb, t);
    var rough = mix(pm.k.z, pm.k.w, t);
    c = mix(c, pm.gap_color.rgb, gap);
    rough = mix(rough, 0.95, gap);

    // bump from the pattern's height (surface gradient: Mikkelsen 2010), in world space
    let h = (pat.z - gap * 0.6) * pm.t.x;
    let sx = dpdx(in.world_position.xyz);
    let sy = dpdy(in.world_position.xyz);
    let nw = normalize(pbr_input.N);
    let r1 = cross(sy, nw);
    let r2 = cross(nw, sx);
    let det = dot(sx, r1);
    let grad = sign(det) * (dpdx(h) * r1 + dpdy(h) * r2);
    let bumped = normalize(abs(det) * nw - grad * 0.02);
    if (pm.t.x > 0.0 && abs(det) > 1e-12) {
        pbr_input.N = bumped;
    }

    // use: dust settles on what faces up; dirt in the gaps, the hollows, undersides and along the floor line; edges
    // worn lighter and smoother where hands and feet go
    let wq = p / max(pm.k.y, 1e-4) * 0.7 + vec3(pm.t.y * 3.1);
    let up = smoothstep(0.55, 0.95, normalize(in.world_normal).y);
    let dust = clamp(up * pm.w.y * (0.55 + 0.45 * fbm(wq * 2.0 + 30.0)), 0.0, 0.9);
    c = mix(c, pm.dust_color.rgb, dust);
    rough = mix(rough, 0.95, dust);
    let floor_line = 1.0 - smoothstep(0.0, 0.25, in.world_position.y);
    let under = clamp(-normalize(in.world_normal).y, 0.0, 1.0);
    let dirt = clamp((gap * 0.8 + under * 0.5 + floor_line * 0.7 + smoothstep(0.6, 0.9, fbm(wq * 0.8 + 50.0)) * 0.4) * pm.w.z, 0.0, 0.85) * (0.6 + 0.4 * fbm(wq * 3.0 + 70.0));
    c = mix(c, pm.dirt_color.rgb, dirt);
    rough = mix(rough, 0.9, dirt);
    let curve = length(fwidth(in.world_normal)) / max(length(fwidth(in.world_position.xyz)), 1e-5);
    let edge = smoothstep(25.0, 90.0, curve) * pm.w.w * (0.5 + 0.5 * fbm(wq * 4.0 + 90.0));
    c = mix(c, min(c * 1.35 + vec3(0.04), vec3(1.0)), edge);
    rough = mix(rough, rough * 0.6, edge);

    pbr_input.material.base_color = vec4(c, pbr_input.material.base_color.a);
    pbr_input.material.perceptual_roughness = clamp(rough, 0.04, 1.0);
    pbr_input.material.base_color = alpha_discard(pbr_input.material.flags, pbr_input.material.alpha_cutoff, pbr_input.material.base_color);

@if(PREPASS_PIPELINE) {
    return deferred_output(in, pbr_input);
} @else {
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
}
