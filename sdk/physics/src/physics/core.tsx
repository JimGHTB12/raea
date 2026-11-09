// Seeded randomness, beats, rotations and parts.
import { type ReactNode } from "react";
import { hasNative, native } from "@raea/app/native";
import { type Vec3 } from "raea/3d";

export type Quat = [number, number, number, number];

export function node(kind: string, props: Record<string, unknown>, children?: ReactNode) {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(props)) if (v !== undefined && k !== "children" && typeof v !== "function") out[k] = v;
  return (
    <div data-3d={kind} data-props={JSON.stringify(out)}>
      {children}
    </div>
  );
}

// ---- seeded randomness and beats (pure) -------------------------------------------------------------------------

/** 0..1 from integers, the same on every run */
export function hash01(...xs: number[]): number {
  let h = 0x811c9dc5;
  for (const x of xs) {
    h ^= Math.floor(x) | 0;
    h = Math.imul(h, 0x01000193);
    h ^= h >>> 15;
    h = Math.imul(h, 0x2c1b3c6d);
    h ^= h >>> 12;
  }
  return ((h >>> 0) % 1000003) / 1000003;
}

/** The last of `times` at or before t: its index (-1 before the first) and how long ago it was, seconds. */
export function lastBeat(t: number, times: readonly number[]): { index: number; since: number; time: number } {
  let i = -1;
  for (let k = 0; k < times.length && times[k] <= t + 1e-9; k++) i = k;
  return { index: i, since: i < 0 ? Infinity : t - times[i], time: i < 0 ? -Infinity : times[i] };
}

/** A position that jumps to a new seeded spot on each beat in `times` (within `radius`, metres, on the axes `axes`
 *  scales) and holds it, back home `hold` seconds after the beat (Infinity: until the next). */
export function teleport(t: number, times: readonly number[], o: { radius: number; seed?: number; hold?: number; axes?: Vec3 }): Vec3 {
  const b = lastBeat(t, times);
  if (b.index < 0 || b.since > (o.hold ?? Infinity)) return [0, 0, 0];
  const s = o.seed ?? 0;
  const ax = o.axes ?? [1, 0.3, 1];
  return [0, 1, 2].map((a) => (hash01(s, b.index, a) * 2 - 1) * o.radius * ax[a]) as Vec3;
}

/** Positions snapped to a grid (`step` metres): a physics engine's position quantised. */
export function snapPose(p: Vec3, step: number): Vec3 {
  return step > 0 ? (p.map((x) => Math.round(x / step) * step) as Vec3) : p;
}

// ---- rotations ------------------------------------------------------------------------------------------------

/** The product a × b of two quaternions [x, y, z, w], `b` applied first, as Bevy's Quat * Quat. */
export function quatMul(a: Quat, b: Quat): Quat {
  const [ax, ay, az, aw] = a;
  const [bx, by, bz, bw] = b;
  return [aw * bx + ax * bw + ay * bz - az * by, aw * by - ax * bz + ay * bw + az * bx, aw * bz + ax * by - ay * bx + az * bw, aw * bw - ax * bx - ay * by - az * bz];
}

/** The quaternion [x, y, z, w] turning `angle` radians about `axis`, as Bevy's Quat::from_axis_angle. */
export function quatAxis(axis: Vec3, angle: number): Quat {
  const l = Math.hypot(...axis) || 1;
  const s = Math.sin(angle / 2) / l;
  return [axis[0] * s, axis[1] * s, axis[2] * s, Math.cos(angle / 2)];
}

function rotate(q: Quat, v: Vec3): Vec3 {
  const [x, y, z, w] = q;
  const ix = w * v[0] + y * v[2] - z * v[1];
  const iy = w * v[1] + z * v[0] - x * v[2];
  const iz = w * v[2] + x * v[1] - y * v[0];
  const iw = -x * v[0] - y * v[1] - z * v[2];
  return [ix * w + iw * -x + iy * -z - iz * -y, iy * w + iw * -y + iz * -x - ix * -z, iz * w + iw * -z + ix * -y - iy * -x];
}

/** A rigid motion of a part about a point (`about`, its rest centre): turn by q there, then move by `move`. As a
 *  <Motion>/<Part> position and quaternion. */
export function about(about: Vec3, q: Quat, move: Vec3 = [0, 0, 0]): { position: Vec3; quaternion: Quat } {
  const r = rotate(q, about);
  return { position: [about[0] - r[0] + move[0], about[1] - r[1] + move[1], about[2] - r[2] + move[2]], quaternion: q };
}

// ---- parts ----------------------------------------------------------------------------------------------------

export interface PartInfo {
  /** the part's own .glb in the asset store */
  file: string;
  /** the middle of its bounds, metres, where it stands in the source file */
  centre: Vec3;
  /** its bounds' low corner */
  lo: Vec3;
  /** its bounds' high corner */
  hi: Vec3;
  /** how many triangles it has */
  tris: number;
}

export interface PartMaterial {
  /** the magenta-and-black checker of a texture that didn't load */
  missing?: boolean;
  /** a colour over its own */
  color?: string;
  /** the colour it glows with */
  emissive?: string;
  /** 0..1, see-through to solid */
  opacity?: number;
  /** 0..1, smooth to matte */
  roughness?: number;
  /** drawn in its flat colour, without light or shade */
  unlit?: boolean;
}

const parts = new Map<string, PartInfo>();

/** glTF nodes cut out as their own file, standing where they stood (made once, kept in the asset store). */
export function part(src: string, nodes: string[], material?: PartMaterial): PartInfo {
  const key = JSON.stringify([src, nodes, material ?? null]);
  let p = parts.get(key);
  if (!p) {
    if (!hasNative()) throw new Error("part: cutting a glTF's nodes needs the raea engine (raea render)");
    p = native("physics.part", { src, nodes, ...(material ? { material } : {}) }) as PartInfo;
    parts.set(key, p);
  }
  return p;
}

export interface Glitch {
  /** triangles thrown out from the centre and turned (0 whole, 1 about the part's size away) */
  explode?: number;
  /** 0..1: that share of the vertices pulled to other vertices of the part (spikes across it) */
  scramble?: number;
  /** metres of per-triangle noise */
  jitter?: number;
  /** metres: the vertices snapped to a grid that size */
  snap?: number;
  /** metres, world: the trailing side dragged back along it (motion smear) */
  smear?: Vec3;
  /** which randomness (change it on a beat for a new pattern) */
  seed?: number;
}

export interface PartProps {
  /** the entity's name (the first node's by default) */
  name?: string;
  /** the glTF (staticFile("x.glb")) and the nodes that make the part */
  src: string;
  /** the glTF nodes that make the part */
  nodes: string[];
  /** a rigid motion from where it stands in the file (see `about`): position, and a quaternion [x, y, z, w] */
  position?: Vec3;
  /** its turn as a quaternion [x, y, z, w] (see `about`) */
  quaternion?: Quat;
  /** a uniform scale (1) */
  scale?: number;
  /** a material over the part's own (colour, glow, opacity) */
  material?: PartMaterial;
  /** how the part breaks up this frame: explode, scramble, jitter, snap, smear */
  glitch?: Glitch;
  /** it casts shadows */
  castShadow?: boolean;
}

/** A glTF's nodes drawn on their own, where they stood, moved by a rigid motion, glitched by the frame's values. */
export function Part(p: PartProps) {
  const info = part(p.src, p.nodes, p.material);
  const g = p.glitch;
  const glitch = g ? { explode: g.explode ?? 0, scramble: g.scramble ?? 0, jitter: g.jitter ?? 0, snap: g.snap ?? 0, smear: g.smear ?? [0, 0, 0], seed: g.seed ?? 0 } : undefined;
  return node("Group", { position: p.position ?? [0, 0, 0], quaternion: p.quaternion ?? [0, 0, 0, 1], scale: p.scale ?? 1 },
    node("Model", { src: info.file, name: p.name, time: 0, castShadow: p.castShadow, allowOverlap: true, ...(glitch ? { glitch } : {}) }));
}

export interface EchoesProps extends PartProps {
  /** how many echoes trail the part */
  count: number;
  /** metres between one echo and the next, or a function of the echo's number (1, 2, ...) giving its offset */
  step: Vec3 | ((k: number) => Vec3);
  /** the echoes' material (the part's by default): a flat glowing one reads as copies */
  echo?: PartMaterial;
}

/** Copies of a part trailing it: `count` echoes at offsets `step` apart (metres; or a function of the echo's number),
 *  in a flat glowing `material`, so a fork "duplicates" along a line. */
export function Echoes(p: EchoesProps) {
  const out: ReactNode[] = [];
  for (let k = 1; k <= p.count; k++) {
    const o = typeof p.step === "function" ? p.step(k) : (p.step.map((x) => x * k) as Vec3);
    const base = p.position ?? [0, 0, 0];
    out.push(<Part key={k} {...p} name={`${p.name ?? p.nodes[0]}-echo${k}`} material={p.echo ?? p.material} position={[base[0] + o[0], base[1] + o[1], base[2] + o[2]]} glitch={undefined} />);
  }
  return <>{out}</>;
}

export interface ZFightProps extends PartProps {
  /** radians the second copy is turned (~0.002): more is stronger stripes */
  amount: number;
  /** the second copy's material */
  other?: PartMaterial;
}

/** The part drawn twice in the same place, the second a hair turned, in another material: the two surfaces fight
 *  for the depth buffer and flicker in stripes (z-fighting), stronger with `amount` (radians, ~0.002). */
export function ZFight(p: ZFightProps) {
  const info = part(p.src, p.nodes);
  const q = quatMul(p.quaternion ?? [0, 0, 0, 1], quatAxis([0.3, 1, 0.2], p.amount));
  const m = about(info.centre, quatAxis([0.3, 1, 0.2], p.amount));
  const pos = p.position ?? [0, 0, 0];
  return (
    <>
      <Part {...p} />
      {p.amount > 0 && <Part {...p} name={`${p.name ?? p.nodes[0]}-fight`} material={p.other ?? { missing: true }} quaternion={q} position={[pos[0] + m.position[0], pos[1] + m.position[1], pos[2] + m.position[2]]} glitch={undefined} />}
    </>
  );
}
