// A grip's lock: once a hand has closed on a thing, where it holds it and how its fingers lie are fixed for as long
// as it is held, so nothing is fitted again frame by frame (Shane, 11-01: "a lot of the hand placement on an object
// does tend to recompute constantly and makes it look like the hands are glitching, especially with bigger objects").
//
// Which leads is the thing's weight and size (handling): a light thing in one hand rides the hand, turned and moved
// with it however the arm swings; a thing in two hands, or one heavy for the body, leads, and the arms reach to their
// locked places on it (the thing joins the arms' IK as their goal).
import type { Vec3 } from "raea/3d";
import { add, cross, dot, handAlong, handPose, handSurface, norm, palmAt, palmFacing, pinchPoint, pose, qbasis, qinv, qmul, qnorm, qrot, reach, scale3, sub, type HandGoal, type HandShape, type Pose, type Quat, type Side, type Skeleton } from "raea/character";
import type { Handling } from "../things";

type Posed = ReturnType<typeof pose>;

/** one hand's hold in the thing's frame: the palm's middle, the way it faces and the fingers point, and their shape */
export interface HandLock { at: Vec3; facing: Vec3; along: Vec3; shape: HandShape }

export interface Lock {
  /** "hand": the thing rides the hand; "thing": the hands reach to it */
  lead: "hand" | "thing";
  hands: Partial<Record<Side, HandLock>>;
  /** the thing's push out of the body, eased over the frames (it jumped frame to frame when taken raw) */
  off: Vec3;
  /** how far the thing still has to slide into the hand (a pinch's pads), let go over the frames */
  slide?: Vec3;
}

/** the grips taken between the thumb's pad and the fingers' (pinchPoint): a pinched thing sits there */
export const PINCHED = new Set(["pinch", "tripod"]);

/**
 * A pinch's lock, as soon as the thing is taken: the thing (its middle `size[1] / 2` up from `at`) placed where the
 * captured pinch's pads meet in hand `s`, sliding there from where it is over the frames after (`slide`). A reach
 * lands within a few centimetres, and the pads closed on air beside a pencil and carried it under the fist (11-01).
 */
export function pinchLock(skel: Skeleton, w: Posed, at: Vec3, rot: Quat, size: Vec3, s: Side): Lock {
  const facing = norm(palmFacing(skel, w, s)), along = norm(handAlong(skel, w, s));
  const across = s === "R" ? cross(along, facing) : cross(facing, along), [pa, pp, pc] = pinchPoint(skel, s);
  const pads = add(palmAt(skel, w, s), add(add(scale3(along, pa), scale3(facing, pp)), scale3(across, pc)));
  const to = sub(pads, qrot(rot, [0, size[1] / 2, 0]));
  const l = lockOn(skel, w, to, rot, [s], {}, "hand");
  l.slide = sub(at, to);
  return l;
}

/** a thing ridden by its hand (`ridden`), still sliding into it by what's left of `l.slide`, eased a frame */
export function riddenSliding(skel: Skeleton, w: Posed, s: Side, l: Lock): { at: Vec3; rot: Quat } {
  const r = ridden(skel, w, s, l.hands[s]!);
  if (!l.slide) return r;
  l.slide = Math.hypot(...l.slide) < 1e-4 ? undefined : easeOff(l.slide, [0, 0, 0]);
  return l.slide ? { ...r, at: add(r.at, l.slide) } : r;
}

/** the lead for a grip: one light hand's thing goes with the hand, anything else leads the arms */
export const leadOf = (hands: Side[], how: Handling): Lock["lead"] => (hands.length === 1 && how.weight === "light" ? "hand" : "thing");

/** the hand's frame (fingers along, palm facing, and across), squared */
function handFrame(facing: Vec3, along: Vec3): Quat {
  const n = norm(facing), a = norm(sub(along, scale3(n, dot(along, n))));
  return qbasis(a, n, cross(a, n));
}

/** the lock of the hands on a thing at `at`, turned `rot`, as the pose `w` has them and their fingers `shapes` */
export function lockOn(skel: Skeleton, w: Posed, at: Vec3, rot: Quat, hands: Side[], shapes: Partial<Record<Side, HandShape>>, lead: Lock["lead"]): Lock {
  const inv = qinv(rot), out: Lock = { lead, hands: {}, off: [0, 0, 0] };
  for (const s of hands) {
    out.hands[s] = {
      at: qrot(inv, sub(palmAt(skel, w, s), at)),
      facing: qrot(inv, palmFacing(skel, w, s)),
      along: qrot(inv, handAlong(skel, w, s)),
      shape: shapes[s] ?? {},
    };
  }
  return out;
}

/** a hand's goal on the thing where it is now: its locked place, moved and turned with it */
export function lockedGoal(skel: Skeleton, l: HandLock, at: Vec3, rot: Quat, elbow?: Vec3): HandGoal {
  const facing = qrot(rot, l.facing);
  // (reach rests a flat hand's palm half its thickness short of the goal along the way it faces)
  return { at: add(add(at, qrot(rot, l.at)), scale3(norm(facing), 0.009 * skel.height)), facing, point: qrot(rot, l.along), elbow };
}

/** where a thing ridden by hand `s` is, the hand as the pose `w` has it: its locked frame in the hand carried over */
export function ridden(skel: Skeleton, w: Posed, s: Side, l: HandLock): { at: Vec3; rot: Quat } {
  const now = handFrame(palmFacing(skel, w, s), handAlong(skel, w, s)), was = handFrame(l.facing, l.along);
  const rot = qnorm(qmul(now, qinv(was)));
  return { at: sub(palmAt(skel, w, s), qrot(rot, l.at)), rot };
}

/** the arms reached to their locked places on a thing that leads, and how far the palms still miss them (the
 * thing is moved that far after, so it stays in the hands: a reach lands within a centimetre) */
export function reachLocked(skel: Skeleton, p: Pose, lock: Lock, at: Vec3, rot: Quat, elbows: Partial<Record<Side, Vec3 | undefined>>): { p: Pose; miss: Vec3 } {
  const sides = Object.keys(lock.hands) as Side[];
  for (const s of sides) p = reach(skel, p, s, lockedGoal(skel, lock.hands[s]!, at, rot, elbows[s]));
  const w = pose(skel, p);
  let miss: Vec3 = [0, 0, 0];
  for (const s of sides) miss = add(miss, scale3(sub(palmAt(skel, w, s), add(at, qrot(rot, lock.hands[s]!.at))), 1 / sides.length));
  return { p, miss };
}

/** the push out of the body eased: taken at once when it grows, let back over a few frames when it shrinks */
export function easeOff(prev: Vec3, raw: Vec3): Vec3 {
  const lp = Math.hypot(...prev), lr = Math.hypot(...raw);
  return lr >= lp ? raw : [prev[0] + (raw[0] - prev[0]) * 0.2, prev[1] + (raw[1] - prev[1]) * 0.2, prev[2] + (raw[2] - prev[2]) * 0.2];
}

/** whether a hand's fingers have stopped closing: no joint's curl moved more than `tol` (radians) since the frame
 * before (locked while still closing, a pinch froze open on a pencil and never held it) */
export function settled(now?: HandShape, was?: HandShape, tol = 0.01): boolean {
  if (!now || !was) return false;
  for (const k of ["thumb", "index", "middle", "ring", "pinky"] as const) {
    const a = now[k] ?? [0, 0, 0], b = was[k] ?? [0, 0, 0];
    if (a.some((x, i) => Math.abs(x - b[i]) > tol)) return false;
  }
  return Math.abs((now.spread ?? 0) - (was.spread ?? 0)) <= tol && Math.abs((now.thumbOpp ?? 0) - (was.thumbOpp ?? 0)) <= tol;
}

/** how far the hands' skin is into a thing (its distance field `field`): a grip locks only clear of it */
export function depthIn(skel: Skeleton, p: Pose, hands: Side[], field: (q: Vec3) => number): number {
  const w = pose(skel, p);
  let deep = 0;
  for (const s of hands) for (const x of handSurface(skel, w, s)) deep = Math.max(deep, -field(x.at));
  return deep;
}


