// A contrail, one choice of raea/3d's <Volume> (docs/examples.feature EX6): copy it and make it yours.
import { Volume, type Vec3, type VolumeProps, type VolumeSource } from "raea/3d";

export interface ContrailProps extends Omit<VolumeProps, "position" | "size" | "sources" | "puffs"> {
  /** where the plane is at `start` and at `end` (seconds), world metres */
  from: Vec3;
  to: Vec3;
  start?: number;
  end?: number;
  /** the trail's width when new (0.35 m) and how much it spreads (turbulence) as it ages */
  width?: number;
  /** how many engines (1) and how far apart (metres) */
  engines?: number;
  spacing?: number;
}

/** A contrail: a thin trail of vapour left by an engine moving `from` -> `to`, spreading and breaking up as it ages. Put
 * a small plane at its head (contrailHead gives where it is). */
export function Contrail({ from, to, start = 0, end = 10, width = 0.35, engines = 1, spacing, ...rest }: ContrailProps) {
  if (!(end > start)) throw new RangeError(`Contrail: end must be after start, got ${start} -> ${end}`);
  if (!(width > 0)) throw new RangeError(`Contrail: width must be above 0, got ${width}`);
  if (!(Number.isInteger(engines) && engines >= 1 && engines <= 4)) throw new RangeError(`Contrail: engines must be 1 to 4, got ${engines}`);
  // the box around the path, its foot under the middle; the sources in the box's metres
  const pad = width * 6 + 1;
  const lo = [0, 1, 2].map((k) => Math.min(from[k], to[k]) - pad);
  const hi = [0, 1, 2].map((k) => Math.max(from[k], to[k]) + pad);
  const size: Vec3 = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
  const foot: Vec3 = [(lo[0] + hi[0]) / 2, lo[1], (lo[2] + hi[2]) / 2];
  const local = (p: Vec3): Vec3 => [p[0] - foot[0], p[1] - foot[1], p[2] - foot[2]];
  const apart = spacing ?? width * 3;
  const dir = [to[0] - from[0], to[2] - from[2]];
  const len = Math.hypot(dir[0], dir[1]) || 1;
  const side = [-dir[1] / len, dir[0] / len];
  const sources: VolumeSource[] = [];
  for (let i = 0; i < engines; i++) {
    const o = (i - (engines - 1) / 2) * apart;
    const off = (p: Vec3): Vec3 => [p[0] + side[0] * o, p[1], p[2] + side[1] * o];
    sources.push({ at: local(off(from)), to: local(off(to)), from: start, until: end, radius: width, rate: 9, heat: 0, speed: [0, 0, 0] });
  }
  return (
    <Volume
      {...rest}
      position={foot}
      size={size}
      sources={sources}
      resolution={rest.resolution ?? Math.min(160, Math.max(64, Math.round(Math.max(...size) / (width * 0.7))))}
      colors={rest.colors ?? ["#ffffff", "#e6edf5", "#c4d2e2"]}
      buoyancy={rest.buoyancy ?? 0}
      weight={rest.weight ?? 0}
      cooling={rest.cooling ?? 0}
      dissipation={rest.dissipation ?? 0.05}
      fire={rest.fire ?? false}
      turbulence={rest.turbulence ?? 0.25}
      turbulenceSize={rest.turbulenceSize ?? Math.max(0.5, width * 4)}
      vorticity={rest.vorticity ?? 1.5}
      threshold={rest.threshold ?? 0.1}
      erodeSize={rest.erodeSize ?? Math.max(0.3, width * 1.5)}
      absorb={rest.absorb ?? 0.3}
    />
  );
}

/** where a contrail's engine is at `time`: for the plane at its head */
export function contrailHead(from: Vec3, to: Vec3, time: number, start = 0, end = 10): Vec3 {
  const u = Math.max(0, Math.min(1, (time - start) / (end - start)));
  return [from[0] + (to[0] - from[0]) * u, from[1] + (to[1] - from[1]) * u, from[2] + (to[2] - from[2]) * u];
}
