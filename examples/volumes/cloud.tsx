// A cumulus, one choice of raea/3d's <Volume> (docs/examples.feature EX6): copy it and make it yours.
import { Volume, type Vec3, type VolumeProps } from "raea/3d";

/** small seeded random numbers, the same for the same seed */
function rng(seed: number) {
  let s = (Math.floor(seed) * 2654435761 + 1013904223) >>> 0 || 1;
  return () => {
    s ^= s << 13;
    s >>>= 0;
    s ^= s >>> 17;
    s ^= s << 5;
    s >>>= 0;
    return s / 4294967296;
  };
}

export interface CloudProps extends Omit<VolumeProps, "size" | "sources"> {
  /** the box, [width, height, depth] ([16, 8, 8]) */
  size?: Vec3;
  /** with no `puffs`, how many seeded ones it starts from (9): a cumulus, a flat base and a heaped top */
  count?: number;
}

/** A cumulus: puffs of vapour at the start that billow slowly in a drifting turbulence and ride the wind; under a paint
 * look two or three tones of white and pale blue, shadowed by itself. */
export function Cloud({ size = [16, 8, 8], count = 9, puffs, ...rest }: CloudProps) {
  const [w, h, d] = size;
  if (puffs === undefined) {
    const r = rng((rest.seed ?? 0) + 7);
    if (!(Number.isInteger(count) && count >= 1 && count <= 64)) throw new RangeError(`Cloud: count must be a whole number from 1 to 64, got ${count}`);
    puffs = [];
    for (let i = 0; i < count; i++) {
      const u = count === 1 ? 0.5 : i / (count - 1);
      const x = (u - 0.5) * w * 0.62 + (r() - 0.5) * w * 0.08;
      const mid = 1 - Math.abs(u - 0.5) * 1.6;
      const rad = Math.min(h, d) * (0.16 + 0.2 * mid + 0.06 * r());
      puffs.push([x, h * 0.18 + rad * (0.55 + 0.5 * r() * mid), (r() - 0.5) * d * 0.25, rad]);
    }
  }
  return (
    <Volume
      {...rest}
      size={size}
      puffs={puffs}
      colors={rest.colors ?? ["#ffffff", "#dfe8f2", "#b8c8dc"]}
      buoyancy={rest.buoyancy ?? 0}
      weight={rest.weight ?? 0}
      cooling={rest.cooling ?? 0}
      dissipation={rest.dissipation ?? 0.004}
      fire={rest.fire ?? false}
      turbulence={rest.turbulence ?? 0.6}
      turbulenceSize={rest.turbulenceSize ?? Math.max(1, Math.min(w, h, d) * 0.4)}
      vorticity={rest.vorticity ?? 2}
      threshold={rest.threshold ?? 0.18}
      erodeSize={rest.erodeSize ?? Math.max(0.6, h * 0.12)}
      absorb={rest.absorb ?? 0.45}
    />
  );
}
