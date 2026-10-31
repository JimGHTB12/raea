// Plants outside.
import { useCurrentFrame, useVideoConfig } from "@raea/app/clock";
import { Model, type Vec3 } from "raea/3d";
import { Footprint, Room } from "./all";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";
import { Cuboid, Mesh3d } from "raea/mesh";

// ---------------------------------------------------------------------------------------------------- outside

/** A kind of planting round the house: `count` of `src` between `band[0]` and `band[1]` metres out from the walls, no
 * two closer than `spacing`, each turned at random and scaled within `scale`; it sways `sway` radians in the wind. */
export interface Planting {
  src: string;
  count: number;
  band: [number, number];
  spacing?: number;
  scale?: [number, number];
  sway?: number;
}

/** A planted thing: where, its turn and scale, and its sway (radians, its phase and period in seconds). */
export interface Plant {
  src: string;
  at: Vec3;
  yaw: number;
  scale: number;
  sway: { amp: number; phase: number; period: number };
}

/** A seeded stream of numbers in [0, 1) (mulberry32): the same garden every frame, every render, every play. */
export function seeded(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** The plantings laid round the room, outside its walls (and clear of `avoid`: a bay, a patio), each kind in its band:
 * a point is drawn in the room's box grown by the band's far edge and kept when it is past the near edge and `spacing`
 * from everything already planted. Deterministic for a seed. */
export function plant(room: Room, plantings: Planting[], seed = 1, avoid: Footprint[] = []): Plant[] {
  const rnd = seeded(seed);
  const out: Plant[] = [];
  const { width: w, depth: d } = room;
  const outside = (x: number, z: number) => Math.hypot(Math.max(0, -x, x - w), Math.max(0, -z, z - d));
  for (const p of plantings) {
    const [near, far] = p.band, sp = p.spacing ?? 1;
    let placed = 0;
    for (let tries = 0; placed < p.count && tries < p.count * 400; tries++) {
      const x = -far + rnd() * (w + 2 * far), z = -far + rnd() * (d + 2 * far);
      const o = outside(x, z);
      if (o < near || o > far) continue;
      if (avoid.some((f) => x > f.min[0] - sp && x < f.max[0] + sp && z > f.min[1] - sp && z < f.max[1] + sp)) continue;
      if (out.some((q) => Math.hypot(q.at[0] - x, q.at[2] - z) < Math.max(sp, q.src === p.src ? sp : sp / 2))) continue;
      const [s0, s1] = p.scale ?? [1, 1];
      out.push({ src: p.src, at: [x, 0, z], yaw: rnd() * Math.PI * 2, scale: s0 + rnd() * (s1 - s0), sway: { amp: p.sway ?? 0, phase: rnd() * Math.PI * 2, period: 3 + rnd() * 2 } });
      placed++;
    }
  }
  return out;
}

/** A plant's lean in the wind at `t` seconds: a slow swing about x and z, out of step with its neighbours. */
export function swayAt(p: Plant, t: number): Vec3 {
  const w = (2 * Math.PI * t) / p.sway.period + p.sway.phase;
  return [p.sway.amp * Math.sin(w), p.yaw, p.sway.amp * 0.6 * Math.sin(0.73 * w + 1.1)];
}

/** Outside the room: the ground to the horizon (`size` across, just under the room's floor) and what is planted on it,
 * swaying with the clock (a render's frame, a game's time). */
export function Outside({ plants, ground = "#6d7f45", size = 120, centre = [0, 0] }: { plants: Plant[]; ground?: string; size?: number; centre?: [number, number] }) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = frame / fps;
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(centre[0], -0.03, centre[1])} name="ground"><Cuboid half_size={[size / 2, 0.01, size / 2]} /><StandardMaterial base_color={ground} perceptual_roughness={0.95} /></Mesh3d>
      {plants.map((p, i) => (
        // foliage meets foliage: a hedge's leaves in a tree's are planted so, not clipped
        <Model key={i} name={`plant-${i}`} src={p.src} position={p.at} rotation={p.sway.amp ? swayAt(p, t) : [0, p.yaw, 0]} scale={p.scale} allowOverlap />
      ))}
    </>
  );
}
