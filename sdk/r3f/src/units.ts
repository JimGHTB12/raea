// Light in raea's units and in three.js's, both ways. The SDK's lights take an intensity of 1 as a sun of 8000 lux, a
// bulb of 900,000 lumens and an ambient of 400 cd/m² (core/scene/src/props.rs), and Bevy draws them through the camera's
// exposure, EV100 9.7 by default. three.js lights are in its physical units with no exposure (React Three Fiber's
// default since v8), so a three light is what Bevy puts on screen: lux and candela (lumens over 4π) times the exposure,
// and ambient times π (three's ambient is irradiance, lit by the Lambert 1/π; Bevy's is the radiance itself). The
// playable host's parity check measured this mapping (docs/playables.feature PA9).
export const LUX = 8000;
export const LUMENS = 9.0e5;
export const AMBIENT = 400;
/** Bevy's default exposure, EV100 9.7, as a multiplier */
export const EXPOSURE = 1 / (1.2 * 2 ** 9.7);

export type LightKind = "DirectionalLight" | "PointLight" | "SpotLight" | "AmbientLight" | "HemisphereLight" | "RectAreaLight";

/** a raea light's intensity as three.js's for the same light on screen */
export function toThree(kind: LightKind, raea: number, area = 1): number {
  switch (kind) {
    case "DirectionalLight": return raea * LUX * EXPOSURE;
    case "PointLight":
    case "SpotLight": return (raea * LUMENS * EXPOSURE) / (4 * Math.PI);
    // three's RectAreaLight is luminance (nits) over its face: its flux is intensity × area × π
    case "RectAreaLight": return (raea * LUMENS * EXPOSURE) / (Math.PI * area);
    default: return raea * AMBIENT * EXPOSURE * Math.PI;
  }
}

/** a three.js light's intensity as raea's (toThree's inverse) */
export function toRaea(kind: LightKind, three: number, area = 1): number {
  return three / toThree(kind, 1, area);
}

/** three's tone mapping exposure (a multiplier) as raea's ToneMapping exposure (stops) */
export const stopsOf = (multiplier: number) => (multiplier > 0 ? Math.log2(multiplier) : 0);

/** a light's props as raea's relative intensity: a Bevy light's own `lux` (directional), `lumens` (point, spot, rect) or
 *  `nits` (ambient) over the scale above, else its 3d.tsx `intensity` (docs/control.feature FC1f) */
export function relativeOf(kind: string, p: Record<string, unknown>): number {
  const [field, k] = kind === "DirectionalLight" ? ["lux", LUX] : kind === "AmbientLight" || kind === "HemisphereLight" ? ["nits", AMBIENT] : ["lumens", LUMENS];
  const own = p[field as string];
  if (typeof own === "number") return own / (k as number);
  return typeof p.intensity === "number" ? p.intensity : 1;
}
