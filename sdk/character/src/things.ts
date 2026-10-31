// Things for raea/character: one standard for anything a body walks round, reaches for, puts something on,
// picks up, carries or throws. A thing is a shape in its own frame (its bottom's middle at the origin, x across, y up,
// z its front, metres), placed at `at` and turned `yaw` about up, with a mass. Everything a body needs of it is derived
// from those, unless the thing says otherwise: the footprint walks keep clear of, the surfaces things go on, where to
// stand to use it, and how it is handled (the grip for its shape and size, the lift and carry for its weight).
//
//   const table: Thing = { at: [1.6, 0, 0.2], size: [0.7, 0.5, 0.5], mass: 12 };
//   const mug: Thing = { at: placeOn(table), size: [0.08, 0.1, 0.08], shape: "round", mass: 0.3, grips: [{ at: [0.06, 0.05, 0] }] };
//   footprint(table, 0.25)            the floor it covers, grown by a clearance (x, z points)
//   standAt(table, mug.at, { from })  where to stand, facing what, to reach the mug on it
//   handling(mug, skel)               "handle", one hand, light: a quick reach and a carry out to the side
import type { Vec3 } from "raea/3d";
import type { Obstacle } from "./matching";
import type { Guide } from "./grasp";

export type Shape = "box" | "round" | "ball" | "pole";
export type Face = "front" | "back" | "left" | "right";

/** a place a hand holds a thing, in its frame (a mug's handle, a bag's straps) */
export interface Grip {
  at: Vec3;
  /** the way the fingers close round it (along a handle's bar), in its frame */
  along?: Vec3;
  /** a handle (by default), or a stem (a wine glass's: held between the fingers, the bowl over the hand) */
  kind?: "handle" | "stem";
}

/** a flat place things go on: its height in the thing's frame and its rectangle (x, z) */
export interface Surface {
  y: number;
  min: [number, number];
  max: [number, number];
}

export interface Thing {
  /** its bottom's middle */
  at: Vec3;
  /** across (x; a round thing's or a ball's diameter), high, deep (z; a pole's length lies along z) */
  size: Vec3;
  yaw?: number;
  shape?: Shape;
  /** kilograms (by default a box of its size at 150 kg/m³: furniture, a parcel; a ball is hollow, 60) */
  mass?: number;
  /** where things go on it (its top, by default) */
  surfaces?: Surface[];
  /** the sides a body may stand at to use it (every side, by default) */
  use?: Face[];
  /** whether walks go round it (a thing standing on the floor does; a cup on a table doesn't) */
  blocks?: boolean;
  /** solid down to the floor (a plinth, a chest, a counter's kickboard flush with its front): no room under its edge,
   * so a body bending its knees to reach onto it stands further back */
  solid?: boolean;
  /** handles (by default it is held by its sides, round its middle or from under it) */
  grips?: Grip[];
  /** how hands take it (grasp.ts, docs/grasp-guides.feature): cylinders, pinches, spheres and slabs in its frame,
   * tagged; in place of the ones its shape, size and grips give it (autoGuides) */
  guides?: Guide[];
}

const add = (a: Vec3, b: Vec3): Vec3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const rotY = (yaw: number, v: Vec3): Vec3 => { const c = Math.cos(yaw), s = Math.sin(yaw); return [c * v[0] + s * v[2], v[1], -s * v[0] + c * v[2]]; };

/** a point in the thing's frame, in the scene */
export function toWorld(t: Thing, v: Vec3): Vec3 {
  return add(t.at, rotY(t.yaw ?? 0, v));
}
/** a point in the scene, in the thing's frame */
export function toLocal(t: Thing, p: Vec3): Vec3 {
  return rotY(-(t.yaw ?? 0), [p[0] - t.at[0], p[1] - t.at[1], p[2] - t.at[2]]);
}

export const shapeOf = (t: Thing): Shape => t.shape ?? "box";
export const blocks = (t: Thing): boolean => t.blocks ?? t.at[1] < 0.02;
/** kilograms, said or guessed from its size */
export function massOf(t: Thing): number {
  if (t.mass !== undefined) return t.mass;
  const [w, h, d] = t.size, s = shapeOf(t);
  const vol = s === "ball" ? (Math.PI / 6) * w * w * w : s === "round" ? (Math.PI / 4) * w * w * h : w * h * d;
  return vol * (s === "ball" ? 60 : 150);
}

/** The floor it covers (x, z), grown `grow` metres all round: a box's four corners, a round thing's twelve. */
export function footprint(t: Thing, grow = 0): [number, number][] {
  const s = shapeOf(t);
  if (s === "round" || s === "ball") {
    const r = t.size[0] / 2 + grow;
    return Array.from({ length: 12 }, (_, k) => { const p = toWorld(t, [r * Math.cos((k * Math.PI) / 6), 0, r * Math.sin((k * Math.PI) / 6)]); return [p[0], p[2]] as [number, number]; });
  }
  const x = t.size[0] / 2 + grow, z = t.size[2] / 2 + grow;
  return ([[-x, -z], [x, -z], [x, z], [-x, z]] as [number, number][]).map(([a, b]) => { const p = toWorld(t, [a, 0, b]); return [p[0], p[2]] as [number, number]; });
}

/** What walks go round: the things that block, as route obstacles. */
export function thingObstacles(things: Thing[]): Obstacle[] {
  return things.filter(blocks).map((t) => {
    const poly = footprint(t);
    return { min: [Math.min(...poly.map((p) => p[0])), Math.min(...poly.map((p) => p[1]))], max: [Math.max(...poly.map((p) => p[0])), Math.max(...poly.map((p) => p[1]))], poly, top: t.at[1] + t.size[1] };
  });
}

/** Its surfaces: the ones it says, else its top (a ball and a pole have none). */
export function surfacesOf(t: Thing): Surface[] {
  if (t.surfaces) return t.surfaces;
  const s = shapeOf(t);
  if (s === "ball" || s === "pole") return [];
  return [{ y: t.size[1], min: [-t.size[0] / 2, -t.size[2] / 2], max: [t.size[0] / 2, t.size[2] / 2] }];
}

/** Where a thing put on `t` stands: on its surface `surface` (0, the first), `at` metres from the surface's middle
 * (x, z in t's frame), its bottom's middle. */
export function placeOn(t: Thing, o: { surface?: number; at?: [number, number] } = {}): Vec3 {
  const s = surfacesOf(t)[o.surface ?? 0];
  if (!s) throw new Error("placeOn: the thing has no surface to put things on");
  const c: [number, number] = [(s.min[0] + s.max[0]) / 2 + (o.at?.[0] ?? 0), (s.min[1] + s.max[1]) / 2 + (o.at?.[1] ?? 0)];
  return toWorld(t, [c[0], s.y, c[1]]);
}

/** The thing a point rests on: the one whose surface is under it, within 3 cm of its height. */
export function under(things: Thing[], p: Vec3): Thing | undefined {
  return things.find((t) => surfacesOf(t).some((s) => {
    const l = toLocal(t, p);
    return Math.abs(l[1] - s.y) < 0.03 && l[0] >= s.min[0] - 1e-3 && l[0] <= s.max[0] + 1e-3 && l[2] >= s.min[1] - 1e-3 && l[2] <= s.max[1] + 1e-3;
  }));
}

/** the nearest distance from a floor point to a convex polygon's inside (0 within it) */
export function polyDistance(p: [number, number], poly: [number, number][]): number {
  let inside = true, best = Infinity;
  // the winding, so inside is on the same side of every edge
  let area = 0;
  for (let k = 0; k < poly.length; k++) { const a = poly[k], b = poly[(k + 1) % poly.length]; area += a[0] * b[1] - b[0] * a[1]; }
  const sgn = Math.sign(area) || 1;
  for (let k = 0; k < poly.length; k++) {
    const a = poly[k], b = poly[(k + 1) % poly.length];
    const ex = b[0] - a[0], ez = b[1] - a[1], px = p[0] - a[0], pz = p[1] - a[1];
    if (sgn * (ex * pz - ez * px) < 0) inside = false;
    const u = Math.max(0, Math.min(1, (px * ex + pz * ez) / (ex * ex + ez * ez || 1)));
    best = Math.min(best, Math.hypot(px - ex * u, pz - ez * u));
  }
  return inside ? 0 : best;
}

/** the nearest distance from a floor point to an obstacle (its polygon, else its box) */
export function obstacleDistance(p: [number, number], b: Obstacle): number {
  if (b.poly) return polyDistance(p, b.poly);
  const dx = Math.max(b.min[0] - p[0], 0, p[0] - b.max[0]), dz = Math.max(b.min[1] - p[1], 0, p[1] - b.max[1]);
  return Math.hypot(dx, dz);
}

/** A body's size for standing and reaching, from its height: how far its front is from its middle, and the reach from
 * the shoulder that is still easy (an arm's 0.37 of the height, a little short of straight). */
const bodyOf = (height: number) => ({ front: 0.09 * height, reach: 0.33 * height });

/**
 * Where to stand, facing which way, to use `t` at the point `target` (on it, or the thing itself): on one of its
 * usable sides, square to that side and across from the target, close enough that the hand reaches it and as near
 * as it can be to where the body comes `from`, clear of `others` (and inside `bounds`). For one hand it steps aside so
 * that hand's shoulder is square to the target (a reach straight ahead, not across the body).
 */
export function standAt(t: Thing, target: Vec3, o: { from: [number, number]; height: number; others?: Obstacle[]; bounds?: Obstacle; hand?: "L" | "R"; gap?: number; comfy?: number; behind?: number; elbows?: number }): { at: [number, number]; face: number } {
  const b = bodyOf(o.height), local = toLocal(t, target);
  const s = shapeOf(t), sides: Face[] = t.use ?? ["front", "back", "left", "right"];
  const hx = t.size[0] / 2, hz = t.size[2] / 2;
  const cands: { at: [number, number]; face: number; cost: number }[] = [];
  const dirs: { n: [number, number]; along: number; edge: number }[] = [];
  if (s === "round" || s === "ball") {
    // round: anywhere round it, every 30°
    for (let k = 0; k < 12; k++) { const a = (k * Math.PI) / 6; dirs.push({ n: [Math.sin(a), Math.cos(a)], along: 0, edge: hx }); }
  } else {
    for (const f of sides) {
      // the side's outward normal in t's frame, and the target across from it (kept off the corners)
      const n: [number, number] = f === "front" ? [0, 1] : f === "back" ? [0, -1] : f === "left" ? [1, 0] : [-1, 0];
      const halfLen = n[0] ? hz : hx, edge = n[0] ? hx : hz;
      const along = Math.max(-halfLen + 0.05, Math.min(halfLen - 0.05, n[0] ? local[2] : local[0]));
      dirs.push({ n, along, edge });
    }
  }
  // (the body's front a hand's width off the edge by default: close enough to use it, never pressed to it)
  const gap = o.gap ?? 0.06 * o.height;
  for (const d of dirs) {
    // the body's front just off the edge, then as close to the target as the reach needs: a target deep in the surface
    // brings the body no closer (it leans), one far out lets it stand back
    // (and, with `comfy`, back far enough that the target is that far from the body's middle: an easy arm's length)
    const tn = d.n[0] * local[0] + d.n[1] * local[2];
    const out = Math.max(d.edge + b.front + gap, o.comfy !== undefined ? tn + o.comfy : -Infinity);
    // the stand point in t's frame: out along the normal, across from the target along the side
    const round = s === "round" || s === "ball";
    const lp: Vec3 = round ? [d.n[0] * out, 0, d.n[1] * out] : d.n[0] ? [d.n[0] * out, 0, d.along] : [d.along, 0, d.n[1] * out];
    const w = toWorld(t, lp);
    let at: [number, number] = [w[0], w[2]];
    const face = Math.atan2(target[0] - at[0], target[2] - at[1]);
    if (o.hand) {
      // step aside: the body's left, facing `face`, is (cos, -sin)
      const sd = o.hand === "L" ? -1 : 1, k = 0.09 * o.height * sd;
      at = [at[0] + Math.cos(face) * k, at[1] - Math.sin(face) * k];
    }
    const reachD = Math.hypot(target[0] - at[0], target[2] - at[1]) - 0.05 * o.height;
    const tooFar = Math.max(0, reachD - b.reach);
    // (and, with `behind`, clear that far back too: the room a squat's hips sit back into)
    const back: [number, number] = [at[0] - Math.sin(face) * (o.behind ?? 0), at[1] - Math.cos(face) * (o.behind ?? 0)];
    // (and, with `elbows`, clear that far to either side across the body on the way to the target: the elbows of two
    // hands round it)
    const across = (u: number, sd: number): [number, number] => [at[0] + (target[0] - at[0]) * u + Math.cos(face) * sd * (o.elbows ?? 0), at[1] + (target[2] - at[1]) * u - Math.sin(face) * sd * (o.elbows ?? 0)];
    const elbowsHit = (ob: Obstacle) => [0.3, 0.6, 0.9].some((u) => obstacleDistance(across(u, 1), ob) < 0.02 || obstacleDistance(across(u, -1), ob) < 0.02);
    const crowded = (o.others ?? []).some((ob) => obstacleDistance(at, ob) < b.front + 0.05 || (o.behind ? obstacleDistance(back, ob) < 0.05 : false));
    // (the elbows a cost, not a bar: a thing put down in a tight spot is still put down, from the clearest side)
    // (a thing lying lower than the knees is reached over)
    const tight = o.elbows ? (o.others ?? []).filter((ob) => (ob.top ?? Infinity) > 0.25 * o.height && elbowsHit(ob)).length : 0;
    const outside = o.bounds ? at[0] < o.bounds.min[0] + b.front || at[0] > o.bounds.max[0] - b.front || at[1] < o.bounds.min[1] + b.front || at[1] > o.bounds.max[1] - b.front : false;
    if (crowded || outside) continue;
    cands.push({ at, face, cost: Math.hypot(at[0] - o.from[0], at[1] - o.from[1]) + 20 * tooFar + 3 * tight });
  }
  if (!cands.length) throw new Error("standAt: nowhere clear to stand at the thing");
  cands.sort((a, c) => a.cost - c.cost);
  return { at: cands[0].at, face: cands[0].face };
}

/** How a body handles a thing: its grip, one hand or two, and how heavy it is to the body. */
export interface Handling {
  /** pinch: a small thing between finger and thumb; tripod: a thin thing (a pencil) between the thumb and the first
   * fingers; top: a thing that fits in the hand taken from above, the palm on its top and the fingers and thumb down
   * round its sides; wrap: a hand round a round thing; handle: by its handle; stem: a wine glass by its stem, the bowl over the
   * hand; palm: a ball in the palm; sides: two hands on opposite sides; edges: two hands on a box's lower edges, the
   * fingers under it; cradle: a big ball from under, the hands spread on its lower half, light at the sides, against
   * the belly; hug: the arms round it, the hands under its bottom, against the belly and chest; pole: two hands along
   * it */
  grip: "pinch" | "tripod" | "top" | "wrap" | "handle" | "stem" | "palm" | "sides" | "edges" | "cradle" | "hug" | "pole";
  hands: 1 | 2;
  /** its mass to the body's (a body's mass guessed from its height: 23 kg at 1.26 m, 70 at 1.75) */
  load: number;
  /** light (under 3% of the body: a cup, a book), medium (under 12%), heavy (under 35%), or too heavy to lift */
  weight: "light" | "medium" | "heavy" | "immovable";
}

/** a body's mass from its height (BMI about 15 for a child, rising to 22 for an adult) */
export const bodyMass = (height: number) => (height < 1.5 ? 14.5 : 22) * height * height;

/** The handling a thing's shape, size and weight call for, for a body `height` tall. */
export function handling(t: Thing, height: number): Handling {
  const load = massOf(t) / bodyMass(height);
  const weight = load < 0.03 ? "light" : load < 0.12 ? "medium" : load < 0.35 ? "heavy" : "immovable";
  const s = shapeOf(t), hand = 0.1 * height; // a hand spans about a tenth of the height
  const wide = Math.max(t.size[0], s === "box" ? t.size[2] : 0);
  // (a pencil, a brush, a spoon: thin, light and no longer than two hands, held as it is written with)
  if (s === "pole" && weight === "light" && t.size[0] < 0.15 * hand && t.size[2] < 2.5 * hand) return { grip: "tripod", hands: 1, load, weight };
  if (s === "pole") return { grip: "pole", hands: weight === "light" && t.size[2] < 0.5 * height ? 1 : 2, load, weight };
  if (t.grips?.length && weight !== "heavy") return { grip: t.grips[0].kind === "stem" ? "stem" : "handle", hands: 1, load, weight };
  // (a ball too big or heavy for a palm is held from under, whatever it weighs: hugged, its curve rolls off the arms)
  if (s === "ball") return wide <= 0.9 * hand && weight === "light" ? { grip: "palm", hands: 1, load, weight } : { grip: "cradle", hands: 2, load, weight };
  if (weight === "heavy" || wide > 0.32 * height) return { grip: "hug", hands: 2, load, weight };
  if (Math.max(...t.size) < 0.4 * hand) return { grip: "pinch", hands: 1, load, weight };
  if (s === "round" && t.size[0] <= 0.9 * hand && weight === "light") return { grip: "wrap", hands: 1, load, weight };
  if (wide <= 0.9 * hand && weight === "light") return { grip: "wrap", hands: 1, load, weight };
  // (a box by its lower edges, the fingers under it: flat on its sides it is squeezed, and slips)
  return { grip: s === "box" ? "edges" : "sides", hands: 2, load, weight };
}
