// A smoke plume, one choice of raea/3d's <Volume> (docs/examples.feature EX6): copy it and make it yours.
import { Volume, type Vec3, type VolumeProps } from "raea/3d";

export interface SmokeProps extends Omit<VolumeProps, "size" | "sources" | "puffs"> {
  /** the box it lives in, [width, height, depth] in metres ([6, 12, 6]): the plume is cut where it leaves */
  size?: Vec3;
  /** the burner: where in the box (metres from the foot; [0, 0.4, 0]), its radius (1 m), how much smoke and heat it
   * puts out a second at its centre (12 and 8), and the speed it pushes up with (3 m/s) */
  source?: Vec3;
  radius?: number;
  rate?: number;
  heat?: number;
  speed?: number;
  /** when the burner burns, seconds ([0, Infinity]) */
  from?: number;
  until?: number;
}

/** A smoke plume: hot smoke rising from a burner, leaning in the wind, its curls kept curling; with fire where it's
 * hottest (`fire={false}` for none). Under a paint look it's two or three flat tones lit by the sun, shadowed by
 * itself, with torn edges (ref: an illustrated burning field). */
export function Smoke({ size, source, radius, rate, heat, speed, from, until, ...rest }: SmokeProps) {
  return (
    <Volume
      {...rest}
      size={size ?? [6, 12, 6]}
      sources={[{ at: source ?? [0, 0.4, 0], radius: radius ?? 1, rate: rate ?? 12, heat: heat ?? 8, speed: [0, speed ?? 3, 0], from, until }]}
      colors={rest.colors ?? ["#77777b", "#4a4a4e", "#232326"]}
      windPull={rest.windPull ?? 1.2}
      turbulence={rest.turbulence ?? 1.5}
      turbulenceSize={rest.turbulenceSize ?? 1.5}
      erode={rest.erode ?? 0.15}
      erodeSize={rest.erodeSize ?? 0.9}
    />
  );
}
