// <Fog> on the camera as Bevy has it (<Camera3d><Fog ... /></Camera3d>: Bevy's DistanceFog, reflected), and
// FogFalloff with Bevy's own constructors (bevy_pbr fog.rs), in f32 as Bevy works them out: docs/control.feature FC0a
// (atmospheric_fog needs the atmospheric falloff and the sun's colour, which the scene's fog row has neither of).
import { BevyComponent, type FogColor, type FogFalloff as Falloff, type FogProps } from "../3d";
import { LinearRgba, Srgba } from "./all";

const f = Math.fround;
const isLinear = (c: FogColor): c is LinearRgba => typeof c === "object" && "to_srgba" in c;
const srgba = (c: FogColor): Srgba => (typeof c === "string" ? Srgba.hex(c) : isLinear(c) ? c.to_srgba() : new Srgba(c.red, c.green, c.blue, c.alpha));
/** bevy_color's Srgba::gamma_function, in f32 as Bevy runs it */
const gamma = (v: number) => (v <= 0 ? v : v <= 0.04045 ? f(v / f(12.92)) : f(f(f(v + f(0.055)) / f(1.055)) ** f(2.4)));
/** LinearRgba::from(Color) as f32 channels: a LinearRgba as it is, an Srgba through gamma_function */
function linear(c: FogColor): [number, number, number, number] {
  if (isLinear(c)) return [c.red, c.green, c.blue, c.alpha].map(f) as [number, number, number, number];
  const s = srgba(c);
  return [gamma(f(s.red)), gamma(f(s.green)), gamma(f(s.blue)), f(s.alpha)];
}
/** core::f32::consts::E */
const E = f(Math.E);
/** Color as reflect's JSON reads it: an Srgba stays sRGB, a LinearRgba linear */
function reflected(c: FogColor) {
  if (isLinear(c)) return { LinearRgba: { red: c.red, green: c.green, blue: c.blue, alpha: c.alpha } };
  const s = srgba(c);
  return { Srgba: { red: s.red, green: s.green, blue: s.blue, alpha: s.alpha } };
}

/** FogFalloff's constructors: the visibility distance (5% contrast left at it by default) to a density */
export const FogFalloff = {
  /** Koschmieder's 5% contrast threshold, revised from 2% */
  REVISED_KOSCHMIEDER_CONTRAST_THRESHOLD: 0.05,
  /** the extinction coefficient for a visibility `v` at contrast `c_t`: -ln(c_t) / v */
  koschmieder: (v: number, c_t: number) => f(f(-Math.log(f(c_t))) / f(v)),
  from_visibility: (visibility: number): Falloff => FogFalloff.from_visibility_contrast(visibility, FogFalloff.REVISED_KOSCHMIEDER_CONTRAST_THRESHOLD),
  from_visibility_contrast: (visibility: number, contrast_threshold: number): Falloff => ({ Exponential: { density: FogFalloff.koschmieder(visibility, contrast_threshold) } }),
  from_visibility_squared: (visibility: number): Falloff => FogFalloff.from_visibility_contrast_squared(visibility, FogFalloff.REVISED_KOSCHMIEDER_CONTRAST_THRESHOLD),
  from_visibility_contrast_squared: (visibility: number, contrast_threshold: number): Falloff => ({ ExponentialSquared: { density: f(Math.sqrt(f(FogFalloff.koschmieder(visibility, contrast_threshold) / f(visibility)))) } }),
  from_visibility_color: (visibility: number, color: FogColor): Falloff => FogFalloff.from_visibility_contrast_colors(visibility, FogFalloff.REVISED_KOSCHMIEDER_CONTRAST_THRESHOLD, color, color),
  from_visibility_colors: (visibility: number, extinction: FogColor, inscattering: FogColor): Falloff => FogFalloff.from_visibility_contrast_colors(visibility, FogFalloff.REVISED_KOSCHMIEDER_CONTRAST_THRESHOLD, extinction, inscattering),
  from_visibility_contrast_color: (visibility: number, contrast_threshold: number, color: FogColor): Falloff => FogFalloff.from_visibility_contrast_colors(visibility, contrast_threshold, color, color),
  /** Atmospheric: extinction (1 - colour) ^ e and inscattering colour ^ e (linear), each times the coefficient and
   *  its alpha ^ e */
  from_visibility_contrast_colors(visibility: number, contrast_threshold: number, extinction: FogColor, inscattering: FogColor): Falloff {
    const k = FogFalloff.koschmieder(visibility, contrast_threshold);
    const [re, ge, be, ae] = linear(extinction);
    const [ri, gi, bi, ai] = linear(inscattering);
    const ea = f(ae ** E);
    const ia = f(ai ** E);
    const ext = [re, ge, be].map((c) => f(f(f(f(1 - c) ** E) * k) * ea)) as [number, number, number];
    const ins = [ri, gi, bi].map((c) => f(f(f(c ** E) * k) * ia)) as [number, number, number];
    return { Atmospheric: { extinction: ext, inscattering: ins } };
  },
};

/** a <Fog> inside the <Camera3d>: Bevy's DistanceFog on it, Bevy's defaults where a field is left out (near and far a
 * Linear falloff, density an Exponential one) */
export function cameraFog(p: FogProps) {
  const exp = p.directional_light_exponent ?? 8;
  if (!(exp >= 0 && Number.isFinite(exp))) throw new RangeError(`Fog: directional_light_exponent is 0 or more, got ${exp}`);
  const old = p.near !== undefined || p.far !== undefined || p.density !== undefined;
  if (old && p.falloff) throw new TypeError("Fog: falloff or near, far and density, not both");
  const falloff: Falloff = p.falloff ?? (p.density !== undefined ? { Exponential: { density: p.density } } : { Linear: { start: p.near ?? 0, end: p.far ?? 100 } });
  if (typeof falloff !== "object" || Object.keys(falloff).length !== 1) throw new TypeError(`Fog: falloff is a FogFalloff ({ Linear }, { Exponential }, FogFalloff.from_visibility(...)), got ${JSON.stringify(falloff)}`);
  const value = {
    color: reflected(p.color ?? new Srgba(1, 1, 1, 1)),
    directional_light_color: reflected(p.directional_light_color ?? new LinearRgba(0, 0, 0, 0)),
    directional_light_exponent: exp,
    falloff,
  };
  return <BevyComponent type="bevy_pbr::fog::DistanceFog" value={value} />;
}
