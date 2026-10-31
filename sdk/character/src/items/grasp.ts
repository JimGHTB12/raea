// A thing's distance field, the hands kept out of it, and a grip's fingers closed on it.
import type { Vec3 } from "raea/3d";
import { add, closeOn, surface, handPose, handSurface, norm, pose, qrot, scale3, slerp, sub, type Finger, type HandShape, type Pose, type Side, type Skeleton } from "raea/character";
import { type Guide } from "../grasp";
import { shapeOf, type Handling, type Thing } from "../things";
import { fingers } from "./all";

/** A thing's distance field in its own frame (its bottom's middle at the origin): metres outside its surface, less
 * than 0 inside. A wine glass (a stem grip) is its foot, its stem up to the bowl and the bowl. */
export function thingSdf(it: Thing): (p: Vec3) => number {
  const [w, hh, d] = it.size, s = shapeOf(it);
  const box = (p: Vec3, c: Vec3, e: Vec3) => {
    const q = [0, 1, 2].map((k) => Math.abs(p[k] - c[k]) - e[k]);
    return Math.hypot(Math.max(q[0], 0), Math.max(q[1], 0), Math.max(q[2], 0)) + Math.min(Math.max(q[0], q[1], q[2]), 0);
  };
  // (a cylinder up y from y0 to y1)
  const cyl = (p: Vec3, r: number, y0: number, y1: number) => {
    const a = Math.hypot(p[0], p[2]) - r, b = Math.abs(p[1] - (y0 + y1) / 2) - (y1 - y0) / 2;
    return Math.hypot(Math.max(a, 0), Math.max(b, 0)) + Math.min(Math.max(a, b), 0);
  };
  if (s === "ball") return (p) => Math.hypot(p[0], p[1] - w / 2, p[2]) - w / 2;
  if (s === "pole") return (p) => cyl([p[0], p[2], hh / 2 - p[1]], w / 2, -d / 2, d / 2);
  if (s === "round") {
    const g = it.grips?.[0];
    if (g?.kind === "stem") {
      const top = 2 * g.at[1];
      return (p) => Math.min(cyl(p, 0.4 * w, 0, 0.006), cyl(p, 0.0045, 0, top), cyl(p, w / 2, top, hh));
    }
    return (p) => cyl(p, w / 2, 0, hh);
  }
  return (p) => box(p, [0, hh / 2, 0], [w / 2, hh / 2, d / 2]);
}

export const fieldNormal = (f: (p: Vec3) => number, p: Vec3): Vec3 => {
  const e = 5e-4;
  return norm([f(add(p, [e, 0, 0])) - f(sub(p, [e, 0, 0])), f(add(p, [0, e, 0])) - f(sub(p, [0, e, 0])), f(add(p, [0, 0, e])) - f(sub(p, [0, 0, e]))]);
};

/** How far a thing in hand moves to be out of the hands' skin: the palms and the thumbs' balls (the fingers close on
 * it after), the deepest of their surface points into it pushed back to its surface, a few times over. */
export function outOfHands(skel: Skeleton, p: Pose, hands: Side[], field: (q: Vec3) => number, fingers = false): Vec3 {
  const w = pose(skel, p);
  const pts = hands.flatMap((s) => handSurface(skel, w, s).filter((x) => x.part === "palm" || x.part === "thumb1").map((x) => x.at));
  // (and, for a grip all five close on, the fingers' first phalanges laid straight, where the palm goes on: a can held
  // from above sat in them, and no finger closing on it could get out of it; a wine glass's stem sits between them)
  if (fingers) for (const s of hands) pts.push(...surface(skel, pose(skel, handPose(skel, p, s, {})), /^(index|middle|ring|pinky)1$/, s).map((x) => x.at));
  let off: Vec3 = [0, 0, 0];
  for (let k = 0; k < 6; k++) {
    let deep = 0, at: Vec3 | undefined;
    for (const q of pts) { const d = -field(sub(q, off)); if (d > deep) { deep = d; at = sub(q, off); } }
    if (!at || deep < 5e-4) break;
    off = sub(off, scale3(fieldNormal(field, at), deep + 5e-4));
  }
  return off;
}

/** How far a thing held against the body moves to be out of it: its trunk, neck, head and thighs (the arms round it
 * may touch it), the deepest point pushed back to the surface, a few times over. */
export function outOfBody(skel: Skeleton, p: Pose, field: (q: Vec3) => number, ahead_ = true): Vec3 {
  const w = pose(skel, p), pts = surface(skel, w, /^(chest|waist|torso|hips|neck|head|thigh)/).map((x) => x.at);
  const inside = (o: Vec3) => pts.some((q) => field(sub(q, o)) < -5e-4);
  if (!inside([0, 0, 0])) return [0, 0, 0];
  if (!ahead_) return pushOut(pts, field);
  // straight out ahead of the body, as little as clears it (pushed from each point in turn, a thing between the chin
  // and the chest went up and back and never cleared either)
  const fw = qrot(w.rot[0], [0, 0, 1]), ahead = norm([fw[0], 0, fw[2]]);
  if (!inside(scale3(ahead, 0.6))) {
    let lo = 0, hi = 0.6;
    for (let k = 0; k < 10; k++) { const m = (lo + hi) / 2; if (inside(scale3(ahead, m))) lo = m; else hi = m; }
    return scale3(ahead, hi);
  }
  return pushOut(pts, field);
}

/** How far a thing moves to be out of the points `pts`: the deepest pushed back to its surface, a few times over. */
function pushOut(pts: Vec3[], field: (q: Vec3) => number): Vec3 {
  let off: Vec3 = [0, 0, 0];
  for (let k = 0; k < 8; k++) {
    let deep = 0, at: Vec3 | undefined;
    for (const q of pts) { const d = -field(sub(q, off)); if (d > deep) { deep = d; at = sub(q, off); } }
    if (!at || deep < 5e-4) break;
    off = sub(off, scale3(fieldNormal(field, at), deep + 5e-4));
  }
  return off;
}

// the grips a guide places the hand for (the rest keep their own placement below: a pinch, a pencil's tripod and a
// glass's stem as they are, a mug's handle (its bar isn't in the thing's distance field), the two-hand holds' search)
export const GUIDED = new Set<string>(["wrap", "top"]);

/** a guide moved into another frame: its points by `pt`, its directions by `dir` */
export function turnGuide(g: Guide, pt: (v: Vec3) => Vec3, dir: (v: Vec3) => Vec3): Guide {
  switch (g.kind) {
    case "cylinder": return { ...g, at: pt(g.at), axis: dir(g.axis) };
    case "sphere": return { ...g, at: pt(g.at) };
    case "pinch": return { ...g, at: pt(g.at), across: dir(g.across) };
    case "slab": return { ...g, at: pt(g.at), normal: dir(g.normal), up: dir(g.up ?? [0, 1, 0]) };
  }
}

// each grip's fingers: which close on the thing (the rest held as `rest` says), how far they may close (a fist's
// curl is about 1.5 at the knuckle), the fan and the thumb's opposition. A ball in the palm is held by all five,
// spread round it; a pencil by the thumb, index and middle, the ring and pinky tucked; a wine glass's stem between
// the thumb and the first two fingers, the others curled under the foot; a box's edge by the four fingers curled
// under it, the thumb along its side; a big thing by spread fingers lying on it
export const GRASP: Record<Handling["grip"], { on: Finger[]; rest?: Partial<Record<Finger, [number, number, number]>>; max: number; spread: number; opp: number }> = {
  palm: { on: ["thumb", "index", "middle", "ring", "pinky"], max: 1.4, spread: 0.35, opp: 0.7 },
  top: { on: ["thumb", "index", "middle", "ring", "pinky"], max: 1.6, spread: 0.3, opp: 0.5 },
  wrap: { on: ["thumb", "index", "middle", "ring", "pinky"], max: 1.6, spread: 0, opp: 0.8 },
  handle: { on: ["thumb", "index", "middle", "ring", "pinky"], max: 1.6, spread: 0, opp: 0.6 },
  pole: { on: ["thumb", "index", "middle", "ring", "pinky"], max: 1.6, spread: 0, opp: 0.8 },
  pinch: { on: ["thumb", "index"], rest: { middle: [0.6, 0.7, 0.4], ring: [0.8, 0.9, 0.5], pinky: [0.9, 1, 0.6] }, max: 1.3, spread: 0.1, opp: 0.9 },
  tripod: { on: ["thumb", "index", "middle"], rest: { ring: [1.3, 1.4, 0.9], pinky: [1.4, 1.5, 1] }, max: 1.3, spread: 0.15, opp: 0.9 },
  stem: { on: ["thumb", "index", "middle"], rest: { ring: [1.2, 1.3, 0.9], pinky: [1.3, 1.4, 1] }, max: 1.5, spread: -0.05, opp: 0.8 },
  edges: { on: ["index", "middle", "ring", "pinky"], rest: { thumb: [0.15, 0.1, 0.05] }, max: 1.5, spread: 0, opp: 0 },
  sides: { on: ["thumb", "index", "middle", "ring", "pinky"], max: 0.6, spread: 0.1, opp: 0.2 },
  cradle: { on: ["thumb", "index", "middle", "ring", "pinky"], max: 0.7, spread: 0.3, opp: 0.15 },
  hug: { on: ["thumb", "index", "middle", "ring", "pinky"], max: 0.9, spread: 0.2, opp: 0.15 },
};

/**
 * A hand's fingers closed on a thing (its distance field in the scene, `field`): closeOn curls each the grip closes
 * until its skin rests on the thing, the thumb on the fingers too, no further than the grip closes them by `close`;
 * laid over the pose `w` of the way (the capture's own fingers the rest). `hand.shape` is the hand's shape on it the
 * frame before, which its fingers move on from (closeOn), and is set to this frame's.
 */
export function graspThing(skel: Skeleton, p: Pose, s: Side, field: (q: Vec3) => number, grip: Handling["grip"], close: number, w: number, hand?: { shape?: HandShape }): Pose {
  const shape = closeOn(skel, p, s, field, GRASP[grip], close, hand?.shape);
  if (hand) hand.shape = shape;
  const done = handPose(skel, p, s, shape);
  if (w >= 1) return done;
  return { ...done, turns: done.turns.map((q, i) => (i < p.turns.length ? slerp(p.turns[i], q, w) : slerp([0, 0, 0, 1], q, w))) };
}
