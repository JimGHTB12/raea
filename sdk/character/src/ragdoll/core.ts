// Ragdoll options, vectors and rotations.
import type { Vec3 } from "@raea/render/3d";
import { cross, dot, len, norm, qinv, qmul, qnorm, qrot, scale3, sub, type Quat } from "../character";

export interface RagdollOptions {
  /** frames a second of the motion (30) */
  fps?: number;
  /** boxes the body falls on besides the ground, in the body's frame (a course's solids) */
  solids?: { min: Vec3; max: Vec3 }[];
  /** m/s added to every part at `from`: a trip forward */
  push?: Vec3;
  /** the muscles' tone, from none (0) to the heros-journey body's walking muscles (1); 0.04 by default, limp */
  stiffness?: number;
  /** the ground's height (0); null for none */
  ground?: number | null;
  /** seconds after which the body sleeps however it lies (8) */
  maxSeconds?: number;
  /** air: each part dragged back by the air as the square of its speed, by its size (its hull's cross-section), so
   * the body falling free comes up to this speed (m/s) and no faster, and its light, wide parts (hands, forearms)
   * trail the heavy ones. None by default (a vacuum, as before) */
  terminal?: number;
  /** gusts: a smooth random push across the fall (x and z) on each part, by its size, m/s^2 for the body's average
   * part; each part its own, so they turn it over as well as shove it. Seeded by `seed` (0). None by default */
  buffet?: number;
  seed?: number;
}

// ---------------------------------------------------------------- vectors and quaternions (the rest from character)

export type V3 = Vec3;

export type Q = Quat;

export const ID: Q = [0, 0, 0, 1];

export const clamp = (x: number, lo: number, hi: number) => (x < lo ? lo : x > hi ? hi : x);

export const irot = (q: Q, v: V3): V3 => qrot(qinv(q), v);

export function qlog(q: Q): V3 {
  const s = q[3] < 0 ? -1 : 1, x = q[0] * s, y = q[1] * s, z = q[2] * s, w = q[3] * s;
  const sn = Math.sqrt(x * x + y * y + z * z);
  if (sn < 1e-9) return [2 * x, 2 * y, 2 * z];
  const a = 2 * Math.atan2(sn, w);
  return [(x / sn) * a, (y / sn) * a, (z / sn) * a];
}

export function qexp(v: V3): Q {
  const a = len(v);
  if (a < 1e-9) return qnorm([v[0] / 2, v[1] / 2, v[2] / 2, 1]);
  const s = Math.sin(a / 2) / a;
  return [v[0] * s, v[1] * s, v[2] * s, Math.cos(a / 2)];
}

export function swingTwist(q: Q, axis: V3): { swing: Q; twist: Q } {
  const p = q[0] * axis[0] + q[1] * axis[1] + q[2] * axis[2];
  let twist: Q = qnorm([axis[0] * p, axis[1] * p, axis[2] * p, q[3]]);
  if (Math.abs(p) < 1e-12 && Math.abs(q[3]) < 1e-12) twist = ID;
  return { swing: qmul(q, qinv(twist)), twist };
}

/** a frame from its y axis and a forward hint: x = y cross z */
export function frameFrom(y: V3, fwd: V3 = [0, 0, 1]): Q {
  const yy = norm(y), z = norm(sub(fwd, scale3(yy, dot(fwd, yy)))), x = cross(yy, z);
  const t = x[0] + yy[1] + z[2];
  let q: Q;
  if (t > 0) { const s = Math.sqrt(t + 1) * 2; q = [(yy[2] - z[1]) / s, (z[0] - x[2]) / s, (x[1] - yy[0]) / s, s / 4]; }
  else if (x[0] > yy[1] && x[0] > z[2]) { const s = Math.sqrt(1 + x[0] - yy[1] - z[2]) * 2; q = [s / 4, (yy[0] + x[1]) / s, (z[0] + x[2]) / s, (yy[2] - z[1]) / s]; }
  else if (yy[1] > z[2]) { const s = Math.sqrt(1 + yy[1] - x[0] - z[2]) * 2; q = [(yy[0] + x[1]) / s, s / 4, (z[1] + yy[2]) / s, (z[0] - x[2]) / s]; }
  else { const s = Math.sqrt(1 + z[2] - x[0] - yy[1]) * 2; q = [(z[0] + x[2]) / s, (z[1] + yy[2]) / s, s / 4, (x[1] - yy[0]) / s]; }
  return qnorm(q);
}
