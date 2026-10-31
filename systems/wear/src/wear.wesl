// lib.rs: wear over the standard material (Bevy's extended-material fragment, its lighting kept). Noise in the mesh's
// own space; edges from screen-space curvature; the paint's colour, roughness and metalness changed before the light.
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::alpha_discard;
import bevy_pbr::render::mesh_functions::get_local_from_world;
@if(PREPASS_PIPELINE) import bevy_pbr::prepass::io::{VertexOutput, FragmentOutput};
@if(PREPASS_PIPELINE) import bevy_pbr::deferred::functions::deferred_output;
@if(!PREPASS_PIPELINE) import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput};
@if(!PREPASS_PIPELINE) import bevy_pbr::render::pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing};

struct Wear {
    chip: vec4<f32>,
    grime: vec4<f32>,
    rust: vec4<f32>,
    amount: f32,
    scale: f32,
    chips: f32,
    grimes: f32,
    rusts: f32,
    seed: f32,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> wear: Wear;

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

// four octaves, 0..1
fn fbm(p: vec3<f32>) -> f32 {
    var s = 0.0;
    var w = 0.5;
    var q = p;
    for (var k = 0; k < 4; k++) {
        s += w * vnoise(q);
        q = q * 2.03 + vec3(1.7, 9.2, 3.1);
        w *= 0.5;
    }
    return s / 0.9375;
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
    // the mesh's own space, so the wear rides with the part
    let m = get_local_from_world(in.instance_index);
    p = (m * vec4(in.world_position.xyz, 1.0)).xyz;
    n = normalize((m * vec4(n, 0.0)).xyz);
}
    let q = p / wear.scale + vec3(wear.seed * 13.1);

    // curvature (1 / metres): how fast the normal turns against how fast the position moves, on screen
    let curve = length(fwidth(in.world_normal)) / max(length(fwidth(in.world_position.xyz)), 1e-5);
    let edge = smoothstep(25.0, 90.0, curve) + 0.5 * smoothstep(0.12, 0.35, 1.0 - max(abs(n.x), max(abs(n.y), abs(n.z))));
    let a = wear.amount;

    var c = pbr_input.material.base_color.rgb;
    var rough = pbr_input.material.perceptual_roughness;
    var metal = pbr_input.material.metallic;

    // paint: mottled a little, and duller where it's thin
    let mottle = fbm(q * 0.6);
    c *= 1.0 + (mottle - 0.5) * 0.22 * a;
    rough = clamp(rough + (fbm(q * 1.7 + 5.0) - 0.5) * 0.3 * a, 0.04, 1.0);

    // grime: low, in hollows and undersides, broken up
    let low = clamp(-n.y, 0.0, 1.0) * 0.6 + smoothstep(0.55, 0.85, fbm(q * 0.35 + 20.0));
    let g = clamp(low * wear.grimes * a, 0.0, 0.85) * smoothstep(0.3, 0.7, fbm(q * 2.3 + 40.0) + 0.25);
    c = mix(c, wear.grime.rgb, g);
    rough = mix(rough, 0.9, g);

    // rust: streaks run down from blotches (noise squashed along local y)
    let streak = fbm(vec3(q.x * 3.0, q.y * 0.25, q.z * 3.0) + 60.0);
    let blotch = smoothstep(0.6, 0.8, fbm(q * 0.5 + 80.0));
    let r = clamp((blotch + smoothstep(0.62, 0.8, streak) * 0.6) * wear.rusts * a, 0.0, 0.9);
    c = mix(c, wear.rust.rgb * (0.7 + 0.6 * vnoise(q * 6.0)), r);
    rough = mix(rough, 0.95, r);
    metal = mix(metal, 0.0, r);

    // chips: the paint gone at edges and a few scattered spots, the metal under it showing, shinier
    // an edge lowers the bar a long way, so at full wear most edges chip and only the odd spot on a broad face does
    let s = wear.chips * a;
    let field = fbm(q * 2.5 + 100.0) + edge * 0.5 - 0.2;
    let chip = smoothstep(1.0 - s * 0.55, 1.04 - s * 0.55, field);
    c = mix(c, wear.chip.rgb, chip);
    rough = mix(rough, 0.32, chip);
    metal = mix(metal, 0.85, chip);

    pbr_input.material.base_color = vec4(c, pbr_input.material.base_color.a);
    pbr_input.material.perceptual_roughness = rough;
    pbr_input.material.metallic = metal;
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
