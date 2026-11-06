// Hair on cards, shaded as hair (<Character> puts it on a body's "hair" material, the name projects/psx gives it): the standard surface lit as usual (diffuse, shadows), then the strand highlights added
// after lighting the way games shade hair cards (Scheuermann, "Hair Rendering and Shading", GDC 2004; the lobes as
// Karis's "Physically Based Hair Shading in Unreal", 2016, separates them):
//   R    a white highlight off the strands' surface, a band across them (Kajiya-Kay on the tangent shifted to the root)
//   TRT  a wider band in the hair's own colour, shifted to the tip (light that went in and back out), broken up per strand
//   TT   light through the hair from behind, in its colour (the rim round a backlit head)
// The tangent is the strand's direction (baked into the hair's TANGENT by projects/psx hairlook.py), and the depth shade
// (COLOR_0) dims the highlights inside the hair as it dims its colour.
export const HAIR_WESL = /* wgsl */ `
fn hair_hash(x: f32) -> f32 { return fract(sin(x * 12.9898) * 43758.5453); }
// smooth 1D value noise: strands as stripes across the hair (each a value, eased into the next)
fn hair_strands(x: f32) -> f32 { let i = floor(x); let f = fract(x); let u = f * f * (3.0 - 2.0 * f); return mix(hair_hash(i), hair_hash(i + 1.0), u); }

// Kajiya-Kay: the highlight of a thin cylinder along T, faded where the half vector is behind the strand
fn hair_kk(T: vec3<f32>, H: vec3<f32>, e: f32) -> f32 {
  let th = dot(T, H);
  let s = sqrt(max(0.0, 1.0 - th * th));
  return smoothstep(-1.0, 0.0, th) * pow(s, e);
}

fn lit(in: VertexOutput, pbr: PbrInput, color: vec4<f32>) -> vec4<f32> {
  if (bevy_pbr::render::mesh_view_bindings::lights.n_directional_lights == 0u) { return color; }
  let N = normalize(pbr.N);
  let V = normalize(pbr.V);
  let T = normalize(in.world_tangent.xyz);
  let L = normalize(bevy_pbr::render::mesh_view_bindings::lights.directional_lights[0].direction_to_light);
  let Lc = bevy_pbr::render::mesh_view_bindings::lights.directional_lights[0].color.rgb * view.exposure;
  let H = normalize(L + V);
  // (a strand's value from where it lies across the strands, in the world: stretched along it, varying across it, and
  // the same however a card's UVs run; a 2D hash of the UV sparkled like sand)
  let across = dot(in.world_position.xyz, normalize(cross(N, T)));
  let id = 0.6 * hair_strands(across * u_grain().x) + 0.4 * hair_strands(across * u_grain().y + 17.0);
  let shift = (id - 0.5) * u_jitter();
  let T1 = normalize(T + N * (u_shift_r() + shift));
  let T2 = normalize(T + N * (u_shift_trt() + shift));
  let base = pbr.material.base_color.rgb;
  let occ = in.color.r;
  let wrap = clamp(dot(N, L) * 0.5 + 0.5, 0.0, 1.0);
  // (the white highlight broken up by the texture's own strands: lit evenly it read as a wet patch)
  let lum = dot(base, vec3<f32>(0.3, 0.59, 0.11)) / max(1e-3, max(base.r, max(base.g, base.b)));
  let r = hair_kk(T1, H, u_exp_r()) * u_r() * clamp(lum * lum * 2.0, 0.2, 1.5) * mix(0.5, 1.5, id);
  let trt = hair_kk(T2, H, u_exp_trt()) * u_trt() * mix(0.4, 1.4, id);
  let tt = pow(clamp(dot(-L, V), 0.0, 1.0), 6.0) * u_tt();
  // the first reflection is nearly white in real hair, but across a whole card it read as grey on brown hair: half
  // tinted by the hair's own hue
  let hue = base / max(1e-3, max(base.r, max(base.g, base.b)));
  let add = (r * mix(vec3<f32>(1.0), hue, 0.5) + base * (trt + tt)) * Lc * wrap * occ;
  return vec4<f32>(color.rgb + add, color.a);
}
`;

/** the hair look's settings: each lobe's strength (r, trt, tt), the bands' sharpness (exp_r, exp_trt) and shifts along
 * the strand (shift_r, shift_trt), how much strands differ (jitter) and how fine they are across (grain, per metre) */
export interface HairLook { r: number; trt: number; tt: number; exp_r: number; exp_trt: number; shift_r: number; shift_trt: number; jitter: number; grain: [number, number] }
export const HAIR_LOOK: HairLook = { r: 0.03, trt: 0.07, tt: 0.3, exp_r: 80, exp_trt: 12, shift_r: -0.1, shift_trt: 0.12, jitter: 0.15, grain: [900, 260] };
