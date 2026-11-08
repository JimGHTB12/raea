// How a three shader's colour reaches the screen as three puts it there (docs/r3f.feature R3F11). A ShaderMaterial
// writes the value the canvas shows (three applies no tone mapping or colour space to it unless its GLSL includes
// them), where Bevy tone-maps and encodes every pixel of the frame after the fact. So a "display" colour is decoded
// from sRGB and taken back through the camera's tone mapping: the linear value Bevy's curve turns into it. Slot DISPLAY_SLOT (the last):
// x is 0 for a linear (scene-referred) colour, left as it is, or 1 + the camera's tone mapping (lib.rs's `display`).
// The curves are Bevy 0.20's own (bevy_core_pipeline tonemapping.wesl), inverted by a damped fixed point, so one
// method covers the LUT-based ones too; the LUT is the camera's (TonemappingLuts), at bindings 9 and 10.

@group(GROUP) @binding(9) var three_lut: texture_3d<f32>;
@group(GROUP) @binding(10) var three_lut_sampler: sampler;

fn three_lum(v: vec3<f32>) -> f32 {
    return dot(v, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn three_lut3(p: vec3<f32>, block: f32) -> vec3<f32> {
    return textureSampleLevel(three_lut, three_lut_sampler, p * ((block - 1.0) / block) + 0.5 / block, 0.0).rgb;
}

fn three_log2_norm(color: vec3<f32>, lo: f32, hi: f32) -> vec3<f32> {
    var c = max(vec3(0.0), color);
    c = select(c, 0.00001525878 + c, c < vec3<f32>(0.00003051757));
    c = clamp(log2(c / 0.18), vec3(lo), vec3(hi));
    return (c - lo) / (hi - lo);
}

fn three_aces(color: vec3<f32>) -> vec3<f32> {
    let rgb_to_rrt = mat3x3<f32>(vec3(0.59719, 0.35458, 0.04823), vec3(0.07600, 0.90834, 0.01566), vec3(0.02840, 0.13383, 0.83777));
    let odt_to_rgb = mat3x3<f32>(vec3(1.60475, -0.53108, -0.07367), vec3(-0.10208, 1.10813, -0.00605), vec3(-0.00327, -0.07276, 1.07602));
    var v = color * rgb_to_rrt;
    v = (v * (v + 0.0245786) - 0.000090537) / (v * (0.983729 * v + 0.4329510) + 0.238081);
    return saturate(v * odt_to_rgb);
}

fn three_curve(v: f32) -> f32 {
    return 1.0 - exp(-v);
}

fn three_boring(col: vec3<f32>) -> vec3<f32> {
    let m = mat3x3<f32>(0.2126, 0.7152, 0.0722, -0.1146, -0.3854, 0.5, 0.5, -0.4542, -0.0458);
    let ycbcr = col * m;
    let bt = three_curve(length(ycbcr.yz) * 2.4);
    var desat = max((bt - 0.7) * 0.8, 0.0);
    desat *= desat;
    let desat_col = mix(col, ycbcr.xxx, desat);
    let tm_luma = three_curve(ycbcr.x);
    let tm0 = col * max(0.0, tm_luma / max(1e-5, three_lum(col)));
    let tm1 = vec3(three_curve(desat_col.r), three_curve(desat_col.g), three_curve(desat_col.b));
    return mix(tm0, tm1, bt * bt) * 0.97;
}

fn three_agx(color: vec3<f32>) -> vec3<f32> {
    var p = max(vec3(0.0), color);
    p = vec3(dot(p, vec3(0.84247906, 0.0784336, 0.07922375)), dot(p, vec3(0.04232824, 0.87846864, 0.07916613)), dot(p, vec3(0.04237565, 0.0784336, 0.87914297)));
    p = clamp(three_log2_norm(p, -10.0, 6.5), vec3(0.0), vec3(1.0));
    return three_lut3(p, 32.0);
}

fn three_neutral(color_in: vec3<f32>) -> vec3<f32> {
    let start = 0.8 - 0.04;
    let min_c = min(color_in.r, min(color_in.g, color_in.b));
    let offset = select(0.04, min_c - 6.25 * min_c * min_c, min_c < 0.08);
    let oc = color_in - offset;
    let max_c = max(oc.r, max(oc.g, oc.b));
    if max_c < start {
        return oc;
    }
    let d = 1.0 - start;
    let nm = 1.0 - d * d / (max_c + d - start);
    let c = oc * (nm / max_c);
    let g = 1.0 - 1.0 / (0.15 * (max_c - nm) + 1.0);
    return mix(c, vec3(nm), g);
}

/// Bevy's tone mapping `m` (1 none, 2 Reinhard, 3 Reinhard on luminance, 4 ACES fitted, 5 AgX, 6 somewhat boring,
/// 7 Tony McMapface, 8 Blender filmic, 9 PBR neutral)
fn three_tonemap(x: vec3<f32>, m: u32) -> vec3<f32> {
    let c = max(x, vec3(0.0));
    switch m {
        case 2u: { return c / (1.0 + c); }
        case 3u: { let l = three_lum(c); return c * ((l / (1.0 + l)) / max(l, 1e-6)); }
        case 4u: { return three_aces(c); }
        case 5u: { return three_agx(c); }
        case 6u: { return three_boring(c); }
        case 7u: { return three_lut3(saturate(c / (c + 1.0)), 48.0); }
        case 8u: { return three_lut3(saturate(three_log2_norm(c, -11.0, 12.0)), 64.0); }
        case 9u: { return three_neutral(c); }
        default: { return c; }
    }
}

fn three_srgb_decode(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055) / 1.055, vec3(2.4)), c / 12.92, c <= vec3(0.04045));
}

/// the camera's exposure against raea's default (EV100 9.7, core/scene's EV100): 1 at the default, 2 a stop up. Bevy's
/// view.exposure is physical (1/(1.2 * 2^ev100)), for lights in lux; three's is a plain multiplier
fn three_exposure() -> f32 {
    return view.exposure * 1.2 * exp2(9.7);
}

/// the linear colour Bevy's tone mapping turns into what `c` asks to be on screen
fn three_untone(c: vec4<f32>) -> vec4<f32> {
    // a camera without HDR tone-maps in its materials' own shaders (Bevy's pbr_functions), which this one isn't:
    // nothing maps it after, so the canvas's value is only decoded to the sRGB target's linear
    let mode = u32(three_slots.s[DISPLAY_SLOT].x + 0.5);
    let display = three_slots.s[DISPLAY_SLOT].y > 0.5;
    @if(TONEMAP_IN_SHADER) {
        // and a linear (scene-light) colour is mapped here, as Bevy's own materials map theirs
        if !display {
            // scene light takes the camera's exposure, as three's renderer scales it before its tone mapping
            let lit = c.rgb * three_exposure();
            if mode == 0u {
                return vec4(lit, c.a);
            }
            return vec4(three_tonemap(lit, mode - 1u), c.a);
        }
        return vec4(three_srgb_decode(clamp(c.rgb, vec3(0.0), vec3(1.0))), c.a);
    }
    if !display {
        return vec4(c.rgb * three_exposure(), c.a);
    }
    if mode == 0u {
        return c;
    }
    let want = min(three_srgb_decode(clamp(c.rgb, vec3(0.0), vec3(1.0))), vec3(0.995));
    let m = mode - 1u;
    if m <= 1u {
        return vec4(want, c.a);
    }
    var x = want / (1.0 - want);
    if m == 2u {
        return vec4(x, c.a);
    }
    for (var i = 0; i < 24; i++) {
        let y = three_tonemap(x, m);
        x = clamp(x * pow((want + 1e-4) / (y + 1e-4), vec3(0.75)), vec3(0.0), vec3(512.0));
    }
    return vec4(x, c.a);
}
