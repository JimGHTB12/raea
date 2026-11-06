// Hands from captured poses (Shane, 11-01: "the fingers are definitely fucked up ... can we find some assets that
// exist for grips"): 35 hand poses made on a MakeHuman hand by DigitalN8m4r3 for Godot XR Tools (CC0), the grips,
// pinches and cups among them, read out of its Blender file by projects/psx/handposes.py into hands.json. Each
// finger bone is [its direction, its own x axis] in the hand's frame [along, palm, across] (across toward the thumb,
// palm out of the palm), so a right hand's pose is a left hand's too.
//
// A body's finger takes the capture's turn from the capture's rest as the body's arm takes a clip's (retarget.ts):
// laid first along the capture's bone at rest (F), then turned as the capture's bone turned from its rest (D), with
// the hand wherever it is.
import { type Vec3 } from "@raea/render/3d";
import { B, FINGERS, Finger, I, Pose, Quat, Side, Skeleton, add, cross, dot, fingerJoints, handRest, joint, norm, palmAt, pose, qbasis, qfromTo, qinv, qmul, qrot, rest, scale3, slerp, sub, withFingers } from "./all";
import data from "./hands.json" with { type: "json" };

/** a captured hand: each finger's three bones, [direction, x axis] in the hand's frame */
export type CapturedHand = Record<Finger, [Vec3, Vec3][]>;

const REST = data.rest as unknown as CapturedHand;
/** the captured poses by name: "Grip", "Grip Shaft", "Cup", "Pinch Tight", "Pinch Large", "Hold", "Rounded", ... */
export const HAND_POSES = data.poses as unknown as Record<string, CapturedHand>;

/** a hand-frame vector in the world, the frame given by its three axes */
const inFrame = (c: Vec3, f: { along: Vec3; palm: Vec3; across: Vec3 }): Vec3 => [
  c[0] * f.along[0] + c[1] * f.palm[0] + c[2] * f.across[0],
  c[0] * f.along[1] + c[1] * f.palm[1] + c[2] * f.across[1],
  c[0] * f.along[2] + c[1] * f.palm[2] + c[2] * f.across[2],
];
/** a bone's frame from its direction and x axis (squared), as a turn */
const boneFrame = (d: Vec3, x: Vec3): Quat => {
  const a = norm(d), z = norm(cross(a, x));
  return qbasis(a, cross(z, a), z);
};

const restTurns = new WeakMap<Skeleton, Partial<Record<Side, Map<string, Quat[][]>>>>();

/** each finger bone's world turn from rest, for a hand at rest, to take the captured pose `c` */
function turnsAtRest(skel: Skeleton, s: Side, c: CapturedHand): Quat[][] {
  const r = handRest(skel, s), F = fingerJoints(skel, s), J = skel.joints;
  return FINGERS.map((f, k) => F[k].map((i, ph) => {
    const [rd, rx] = REST[f][ph], [pd, px] = c[f][ph];
    const was = boneFrame(inFrame(rd, r), inFrame(rx, r)), now = boneFrame(inFrame(pd, r), inFrame(px, r));
    // F: the body's bone along the capture's at rest; D: the capture's bone's turn from its rest
    const fix = qfromTo(sub(J[i].tail, J[i].head), inFrame(rd, r));
    return qmul(qmul(now, qinv(was)), fix);
  }));
}

function cached(skel: Skeleton, s: Side, name: string, c: CapturedHand): Quat[][] {
  let m = restTurns.get(skel);
  if (!m) restTurns.set(skel, (m = {}));
  const bySide = (m[s] ??= new Map());
  let t = bySide.get(name);
  if (!t) bySide.set(name, (t = turnsAtRest(skel, s, c)));
  return t;
}

/** A hand in a captured pose (a name in HAND_POSES, or one given), `t` of the way from the pose's own fingers
 * (1: all of it). The fingers follow the hand wherever the pose has it. For a skeleton without fingers, the pose as
 * it was. */
export function capturedHand(skel: Skeleton, p: Pose, s: Side, which: string | CapturedHand, t = 1): Pose {
  const F = fingerJoints(skel, s);
  if (!F.length) return p;
  const c = typeof which === "string" ? HAND_POSES[which] : which;
  if (!c) throw new Error(`capturedHand: no hand pose ${which} (there are ${Object.keys(HAND_POSES).join(", ")})`);
  const G = typeof which === "string" ? cached(skel, s, which, c) : turnsAtRest(skel, s, c);
  const turns = p.turns.slice();
  while (turns.length < skel.joints.length) turns.push(I);
  const hand = pose(skel, p).rot[joint(skel, B.hand(s))];
  F.forEach((chain, k) => {
    let parent = hand;
    chain.forEach((i, ph) => {
      const world = qmul(hand, G[k][ph]);
      const local = qmul(qinv(parent), world);
      turns[i] = t >= 1 ? local : slerp(turns[i] ?? I, local, t);
      parent = qmul(parent, turns[i]);
    });
  });
  return { ...p, turns };
}

const locals = new WeakMap<Skeleton, Partial<Record<Side, Map<string, Quat[][]>>>>();

/** each finger joint's own turn (against the bone it hangs from) in the captured pose `name`: the same wherever the
 * hand is, so a path between poses is turns to slerp */
export function capturedLocal(skel: Skeleton, s: Side, name: string): Quat[][] {
  let m = locals.get(skel);
  if (!m) locals.set(skel, (m = {}));
  const bySide = (m[s] ??= new Map());
  let L = bySide.get(name);
  if (!L) {
    const c = HAND_POSES[name];
    if (!c) throw new Error(`capturedLocal: no hand pose ${name}`);
    const G = cached(skel, s, name, c);
    bySide.set(name, (L = G.map((g) => g.map((q, ph) => (ph === 0 ? q : qmul(qinv(g[ph - 1]), q))))));
  }
  return L;
}

/** finger `k`'s joint `ph` at `u` along `path` (captured poses, open to closed: 0 the first, 1 the second, ...) */
export function alongPath(skel: Skeleton, s: Side, path: readonly string[], k: number, ph: number, u: number): Quat {
  const v = Math.max(0, Math.min(path.length - 1, u)), i = Math.min(path.length - 2, Math.floor(v));
  if (path.length === 1) return capturedLocal(skel, s, path[0])[k][ph];
  return slerp(capturedLocal(skel, s, path[i])[k][ph], capturedLocal(skel, s, path[i + 1])[k][ph], v - i);
}

/** A hand along a captured path: each finger's joints where `at` puts them along it (a finger left out at the path's
 * start), from finger `from` on (1: the thumb left as it is). The fingers' joint turns only; the hand as the pose has
 * it. */
export function pathHand(skel: Skeleton, p: Pose, s: Side, path: readonly string[], at: Partial<Record<Finger, [number, number, number]>>, from = 0): Pose {
  const F = fingerJoints(skel, s);
  if (!F.length) return p;
  const turns = p.turns.slice();
  while (turns.length < skel.joints.length) turns.push(I);
  F.forEach((chain, k) => k >= from && chain.forEach((i, ph) => { turns[i] = alongPath(skel, s, path, k, ph, at[FINGERS[k]]?.[ph] ?? 0); }));
  return { ...p, turns };
}

/** where a captured pinch's pads meet (between the thumb's tip and the index's), from the palm's middle (palmAt), in
 * the hand's frame [along, palm, across]: where a pinched thing sits in the hand */
export function pinchPoint(skel: Skeleton, s: Side, name = "Pinch Tight"): Vec3 {
  const p = capturedHand(skel, rest(skel), s, name), w = withFingers(skel, pose(skel, p)), r = handRest(skel, s), J = skel.joints;
  const tip = (n: string) => { const i = joint(skel, n); return add(w.pos[i], qrot(w.rot[i], sub(J[i].tail, J[i].head))); };
  const mid = scale3(add(tip(B.finger("thumb", 3, s)), tip(B.finger("index", 3, s))), 0.5), d = sub(mid, palmAt(skel, w, s));
  return [dot(d, r.along), dot(d, r.palm), dot(d, r.across)];
}
