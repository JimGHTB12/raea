// paint.rs: a surface lit as an illustrator paints it (the paint and cartoon looks). The suns' light on it (how
// square it faces each, and whether it's in that sun's shadow, cut hard at half) is put into `bands` flat tones: lit
// is the surface's own colour, turned from the light its colour times `mid`, in a cast shadow its colour times
// `shadow` (bands 2: one shade, `shadow`, for both; 4 and more: the lit side stepped from `mid` up by how square it
// faces the light). Point and spot lights only where one asks (paint_lights.wesl). No specular, no ambient
// gradient: chosen colours, not physically blended ones. The rest of the look (ragged edges, grain, the sky, a
// palette) is look.wesl's mode 6.
import bevy_pbr::render::pbr_fragment::pbr_input_from_standard_material;
import bevy_pbr::render::pbr_functions::alpha_discard;
import bevy_pbr::render::pbr_bindings;
import bevy_pbr::render::pbr_types::{STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE, STANDARD_MATERIAL_FLAGS_UNLIT_BIT};
import bevy_pbr::render::mesh_view_bindings as view_bindings;
import bevy_pbr::render::mesh_view_types;
import bevy_pbr::render::shadows;
import bevy_pbr::render::mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT;
import raea_sys_look::paint_lights::light_bands;
@if(PREPASS_PIPELINE) import bevy_pbr::prepass::io::{VertexOutput, FragmentOutput};
@if(PREPASS_PIPELINE) import bevy_pbr::deferred::functions::deferred_output;
@if(!PREPASS_PIPELINE) import bevy_pbr::render::forward_io::{VertexOutput, FragmentOutput};
@if(!PREPASS_PIPELINE) import bevy_pbr::render::pbr_functions::main_pass_post_lighting_processing;

struct Paint {
    shadow: vec4<f32>,
    mid: vec4<f32>,
    ao: vec4<f32>,
    foam: vec4<f32>,
    patch_: vec4<f32>,
    // bands, ao height, ground y, water y
    k0: vec4<f32>,
    // foam height, ssao threshold, shadow-edge raggedness, breakup amount
    k1: vec4<f32>,
    // breakup scale, edge scale, wash, wash patch size (uv)
    k2: vec4<f32>,
    // stroke amount, stroke length, saturation, foliage
    k3: vec4<f32>,
    // foliage height, calm from, calm to, calm amount
    k4: vec4<f32>,
    calm: vec4<f32>,
    // texel (m: surfaces as pixel art, 0 for none), each cell's shade jitter, cloud shadows' drift (x, z, m)
    k5: vec4<f32>,
    // cloud shadows: cover (0 none to 1), size (m); this material sways in the wind (1: a cut-out leaf or blade), -
    k6: vec4<f32>,
    // wind: how far a blade's tip bends (m, 0 none), its direction (x, z), the time it has blown (s)
    k7: vec4<f32>,
    // haze: how thick (per metre), how fast it thins with height (m), where it starts (m), its most (0 to 1); its
    // colour (a: on), and its colour toward the sun (a: on)
    k8: vec4<f32>,
    haze: vec4<f32>,
    haze_sun: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> paint: Paint;

fn hash3(p: vec3<f32>) -> f32 {
    var q = bitcast<vec3<u32>>(vec3<i32>(floor(p)));
    q = q * 1664525u + 1013904223u;
    q.x += q.y * q.z;
    q.y += q.z * q.x;
    q.z += q.x * q.y;
    q = q ^ (q >> vec3<u32>(16u));
    return f32(q.x + q.y + q.z) * (1.0 / 4294967295.0);
}

fn vnoise(x: vec3<f32>) -> f32 {
    let i = floor(x);
    let f = fract(x);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(mix(hash3(i), hash3(i + vec3(1.0, 0.0, 0.0)), u.x), mix(hash3(i + vec3(0.0, 1.0, 0.0)), hash3(i + vec3(1.0, 1.0, 0.0)), u.x), u.y);
    let b = mix(mix(hash3(i + vec3(0.0, 0.0, 1.0)), hash3(i + vec3(1.0, 0.0, 1.0)), u.x), mix(hash3(i + vec3(0.0, 1.0, 1.0)), hash3(i + vec3(1.0, 1.0, 1.0)), u.x), u.y);
    return mix(a, b, u.z);
}

/// 0..1, three octaves of world-space noise
fn fbm3(x: vec3<f32>) -> f32 {
    return 0.55 * vnoise(x) + 0.3 * vnoise(x * 2.13 + 17.1) + 0.15 * vnoise(x * 4.37 - 5.3);
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material.flags, pbr_input.material.alpha_cutoff, pbr_input.material.base_color);

@if(PREPASS_PIPELINE) {
    return deferred_output(in, pbr_input);
} @else {
    var out: FragmentOutput;
    var c = pbr_input.material.base_color;
    // texels: with `texel` metres set, every surface is drawn as pixel art fixed to the world. The point is moved to
    // the centre of its cell in a world grid `texel` across (nudged a quarter cell off the surface first, so a face
    // lying on a grid plane doesn't flicker between cells), and everything the shading reads from the point (the
    // texture, the shadow, strokes, grades) is read there: each cell one colour, the frame still full resolution
    let texel = paint.k5.x;
    let fine = pbr_input.world_position.xyz;
    var wp = fine;
    if texel > 0.0 {
        wp = (floor((fine + pbr_input.world_normal * texel * 0.25) / texel) + 0.5) * texel;
    }
    // unlit (a MeshBasicMaterial, `unlit`): its own colour, as drawn
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) != 0u {
        out.color = main_pass_post_lighting_processing(pbr_input, vec4<f32>(c.rgb + pbr_input.material.emissive.rgb * view_bindings::view.exposure, c.a));
        return out;
    }
@if(VERTEX_UVS_A) {
    // the texture read at the cell's centre: the uv there found from how uv and the world move across the screen
    // (derivatives taken before any branch, as WGSL needs)
    let uv0 = (pbr_input.material.uv_transform * vec3<f32>(in.uv, 1.0)).xy;
    let px = dpdx(fine);
    let py = dpdy(fine);
    let ux = dpdx(uv0);
    let uy = dpdy(uv0);
@if(!BINDLESS) {
    if texel > 0.0 && (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u {
        let d = wp - fine;
        // the screen offset (a, b) whose world step px * a + py * b is nearest d (least squares)
        let xx = dot(px, px);
        let xy = dot(px, py);
        let yy = dot(py, py);
        let det = xx * yy - xy * xy;
        if abs(det) > 1e-20 {
            let a = (yy * dot(px, d) - xy * dot(py, d)) / det;
            let b = (xx * dot(py, d) - xy * dot(px, d)) / det;
            let t = textureSampleLevel(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv0 + ux * a + uy * b, 0.0);
            c = vec4<f32>(t.rgb * pbr_bindings::material.base_color.rgb, c.a);
        }
    }
}
    // the wash: a textured surface takes its colour as the texture's average over a patch around the point (a spiral
    // of 12 taps), so a scanned model reads as painted colour rather than photographic detail; its alpha (a leaf's
    // cut-out) stays fine
    let wash = paint.k2.z;
    if wash > 0.0 && (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT) != 0u {
        let uv = (pbr_input.material.uv_transform * vec3<f32>(in.uv, 1.0)).xy;
        let r = paint.k2.w;
        var sum = vec3<f32>(0.0);
        var wsum = 0.0;
        for (var k = 0; k < 12; k = k + 1) {
            let a = f32(k) * 2.3998;
            let d = r * sqrt((f32(k) + 0.5) / 12.0);
            let tap = textureSampleLevel(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv + d * vec2<f32>(cos(a), sin(a)), 0.0);
            // taps in the cut-out (a leaf's transparent margin) don't count
            sum += tap.rgb * tap.a;
            wsum += tap.a;
        }
        let coarse = sum / max(wsum, 1e-3);
        @if(BINDLESS)
        let tint = c.rgb;
        @else
        let tint = pbr_bindings::material.base_color.rgb;
        c = vec4<f32>(mix(c.rgb, coarse * tint, wash), c.a);
    }
}
    // foliage: a cut-out leaf or blade (any material with alpha: masked, blended or to coverage) is shaded as part of its plant's mass, as a painter shades a
    // clump of grass: its normal bent up toward the sky, so the clump takes the light as one shape, and its colour
    // graded from a dark root to a light tip over `foliage height` metres
    let alpha_mode = pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if paint.k3.w > 0.0 && alpha_mode != STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE {
        pbr_input.N = normalize(mix(pbr_input.N, vec3<f32>(0.0, 1.0, 0.0), paint.k3.w));
        let h = clamp((wp.y - paint.k0.z) / max(paint.k4.x, 0.01), 0.0, 1.0);
        c = vec4<f32>(c.rgb * mix(1.0, mix(0.55, 1.3, h), paint.k3.w), c.a);
    }
    // brush strokes: the colour lightened and darkened in short strokes laid along one direction in the world
    if paint.k3.x > 0.0 {
        let q = wp / paint.k3.y;
        let sk = fbm3(vec3<f32>(q.x * 0.35 + q.z * 0.2, q.y * 1.6, q.z * 0.35 - q.x * 0.2));
        c = vec4<f32>(c.rgb * (1.0 + (sk - 0.5) * 1.2 * paint.k3.x), c.a);
    }
    // the mesh's breakup: patches of noise over it, in its patch colour, and the same noise wandering its tone
    // boundaries (a mottled field, rust on a container, a painted wall's unevenness)
    let breakup = paint.k1.w;
    var wobble = 0.0;
    if breakup > 0.0 {
        let b = fbm3(wp / paint.k2.x);
        wobble = (b - 0.5) * breakup;
        if paint.patch_.w > 0.0 && b > 1.0 - 0.5 * breakup {
            c = vec4<f32>(mix(c.rgb, paint.patch_.rgb, paint.patch_.w), c.a);
        }
    }
    // the shadows' cut wanders by noise: a torn edge, not a ruled one
    let edge = (fbm3(wp / paint.k2.y + vec3<f32>(3.1, 0.0, 7.7)) - 0.5) * paint.k1.z;
    let view_z = dot(vec4<f32>(
        view_bindings::view.view_from_world[0].z,
        view_bindings::view.view_from_world[1].z,
        view_bindings::view.view_from_world[2].z,
        view_bindings::view.view_from_world[3].z
    ), pbr_input.world_position);
    let suns = view_bindings::lights.n_directional_lights;
    // how square the surface faces the suns (`ndl`), and whether a sun that it faces reaches it (`lit`)
    var ndl = 0.0;
    var lit = false;
    if suns == 0u {
        ndl = 1.0;
        lit = true;
    }
    for (var i: u32 = 0u; i < suns; i = i + 1u) {
        let light = &view_bindings::lights.directional_lights[i];
        let d = dot(pbr_input.N, (*light).direction_to_light) + wobble;
        if d <= 0.03 {
            continue;
        }
        var s = 1.0;
        if (pbr_input.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) != 0u
                && ((*light).flags & mesh_view_types::DIRECTIONAL_LIGHT_FLAGS_SHADOWS_ENABLED_BIT) != 0u {
            // the cell's shadow (at its centre, lifted a cell off the surface so the surface doesn't shadow itself)
            var sp = pbr_input.world_position;
            if texel > 0.0 {
                sp = vec4<f32>(wp + pbr_input.world_normal * texel, 1.0);
            }
            s = shadows::fetch_directional_shadow(i, sp, pbr_input.world_normal, view_z, pbr_input.frag_coord.xy);
        }
        // the shadow's filtered edge cut at half: a shadow is a shape, not a gradient
        if s >= 0.5 + edge {
            lit = true;
            ndl = max(ndl, d);
        } else if !lit {
            ndl = -1.0;
        }
    }
    // the tones, as an illustrator sorts them: lit; turned from the light (form shadow, `mid`); facing it but shaded
    // by something else (cast shadow, `shadow`). Two bands: lit and one shade (`shadow`) for both. Four and more: the
    // lit side in bands - 2 steps from `mid` to the surface's own colour, by how square it faces the light.
    let n = max(round(paint.k0.x), 1.0);
    var tint = vec3<f32>(1.0);
    if n > 1.5 && !lit {
        let shaded = ndl < 0.0;
        if n < 2.5 || shaded {
            tint = paint.shadow.rgb;
        } else {
            tint = paint.mid.rgb;
        }
    } else if n > 3.5 && lit {
        let steps = n - 2.0;
        let j = min(steps, 1.0 + floor(clamp(ndl, 0.0, 1.0) * steps));
        tint = mix(paint.mid.rgb, vec3<f32>(1.0), j / steps);
    }
    // cloud shadows: big soft-edged shapes of shade drifting over the land, cut hard like the cast shadows
    if lit && paint.k6.x > 0.0 {
        let q = vec2<f32>(wp.x - paint.k5.z, wp.z - paint.k5.w) / max(paint.k6.y, 1.0);
        // one broad octave and a finer one for the shapes' edges; the noise keeps to about 0.25 to 0.55 (measured on
        // the meadow), so the cut runs through that: cover 0.5 shades about half the land, 1 nearly all
        let cn = vnoise(vec3<f32>(q.x, 0.5, q.y)) * 0.8 + vnoise(vec3<f32>(q.x * 3.1, 0.5, q.y * 3.1)) * 0.2;
        if cn > 0.55 - 0.3 * paint.k6.x {
            tint = paint.shadow.rgb;
        }
    }
    // the point and spot lights that ask for paint, in bands of their own colour (paint_lights.wesl)
    tint = light_bands(tint, lit, pbr_input.world_position, pbr_input.N, pbr_input.world_normal, pbr_input.frag_coord, view_z, pbr_input.is_orthographic, pbr_input.flags, wobble, edge);
    let up = abs(pbr_input.N.y);
    // ambient occlusion as one band: walls near the ground (ao height, ragged) and creases the SSAO finds
    if paint.ao.w > 0.0 {
        let n2 = fbm3(wp * 1.7 + vec3<f32>(11.0, 0.0, 5.0));
        // the ground's bounce band is on the shade side: where the sun falls it's the sun's light
        var occluded = !lit && paint.k0.y > 0.0 && up < 0.7 && wp.y - paint.k0.z < paint.k0.y * (0.6 + 0.8 * n2);
        if paint.k1.y > 0.0 && pbr_input.diffuse_occlusion.r < paint.k1.y + (n2 - 0.5) * 0.15 {
            occluded = true;
        }
        if occluded {
            tint = tint * paint.ao.rgb;
        }
    }
    var rgb = c.rgb * tint + pbr_input.material.emissive.rgb * view_bindings::view.exposure;
    // foam where things stand in the water: broken white along the waterline of anything not facing up
    if paint.foam.w > 0.0 && up < 0.8 {
        let h = wp.y - paint.k0.w;
        let f = fbm3(vec3<f32>(wp.x * 3.0, wp.y * 1.2, wp.z * 3.0) + 2.0);
        if h > -0.05 && h < paint.k1.x * (0.2 + 1.4 * f) {
            rgb = paint.foam.rgb;
        }
    }
    // calm: low foliage (cut-out blades and petals) settled toward one colour with distance, so grass and flowers
    // finer than a pixel read as a field rather than speckle; trunks, rocks and the ground keep theirs
    if paint.calm.w > 0.0 && alpha_mode != STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE {
        let far = smoothstep(paint.k4.y, paint.k4.z, length(wp - view_bindings::view.world_position.xyz));
        let low = 1.0 - smoothstep(paint.k4.x * 2.0, paint.k4.x * 3.0, wp.y - paint.k0.z);
        rgb = mix(rgb, paint.calm.rgb * tint, far * low * paint.k4.w);
    }
    // each texel a shade of its own, lighter or darker by up to `jitter`: a flat colour reads as hand-placed pixels
    if texel > 0.0 && paint.k5.y > 0.0 {
        rgb = rgb * (1.0 + (hash3(wp / texel + vec3<f32>(0.37)) - 0.5) * 2.0 * paint.k5.y);
    }
    // saturation: painted colour is fuller than photographed colour
    let grey = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    rgb = max(vec3<f32>(grey) + (rgb - vec3<f32>(grey)) * paint.k3.z, vec3<f32>(0.0));
    // haze: air lying low over the land, thick with distance and thinning with height, so far ground and the feet
    // of things dissolve into light while their tops stand clear (a painter's aerial perspective, without a box's
    // edges); read at the true point, not the texel
    // As a painter (and an atmosphere) does it: the air takes the sky's colour, never white; it starts some way off
    // (`start`), so the foreground keeps its full colour; it never quite covers anything (`most`); it takes a
    // thing's saturation and contrast before its value, so the distance goes cool and soft rather than pale; and
    // toward the sun it glows the sun's colour
    if paint.haze.w > 0.0 {
        let eye = view_bindings::view.world_position.xyz;
        let d = max(length(fine - eye) - paint.k8.z, 0.0);
        let lift = max(fine.y - paint.k0.z, 0.0);
        let a = clamp((1.0 - exp(-d * paint.k8.x)) * exp(-lift / max(paint.k8.y, 0.01)), 0.0, 1.0) * paint.k8.w;
        var air = paint.haze.rgb;
        if paint.haze_sun.w > 0.0 && view_bindings::lights.n_directional_lights > 0u {
            let toward = max(dot(normalize(fine - eye), view_bindings::lights.directional_lights[0].direction_to_light), 0.0);
            air = mix(air, paint.haze_sun.rgb, pow(toward, 6.0));
        }
        // first the colour drains toward the air's (half again as fast as the value moves), then the value follows
        let lum = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        let drained = mix(rgb, vec3<f32>(lum) * air / max(dot(air, vec3<f32>(0.2126, 0.7152, 0.0722)), 1e-3), clamp(a * 1.5, 0.0, 1.0));
        rgb = mix(drained, air, a);
    }
    out.color = vec4<f32>(rgb, c.a);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
}
