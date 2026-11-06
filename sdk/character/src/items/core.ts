// Items' tasks and plans, and the moments and measures of a reach, a carry and a throw.
import type { Vec3 } from "raea/3d";
import { B, SIDES, joint, palmAt, pose, rest, type HandGoal, type Side, type Skeleton } from "raea/character";
import { type PerformanceSpec, type Step } from "../matching";
import { type Handling, type Thing } from "../things";
import { crouchDrop, crouchedAt, stoop } from "./all";

/** A thing to pick up (the Thing standard: its bottom's middle at `at`, `size` across, high and deep, turned `yaw`). */
export type Item = Thing;

export type ThrowStyle = "overhand" | "underhand" | "chest" | "toss" | "heave";

export type ItemTask =
  /** take the thing (with `hand`, or both hands if its handling takes two); with `face`, stood beside it facing that
   * yaw and reached across to (a thing at the side of the way taken in passing, not walked round to face). `manner`:
   * "quick" (the default) turns the trunk toward it as the hand goes out, back to the way on as it comes in; "patient"
   * looks about first (`look`, a capture of someone looking round, played where the body stands), takes it so, and
   * turns it over in the hand, looking at it, before going on. `from`: "above" takes a thing that fits in the hand from
   * over it, the palm on its top (a patient pick's default: raised to be looked at, the forearm turns the palm up under
   * it), "side" as its handling has it (a quick pick's default). `tag`: the guide to take it by (grasp.ts: "handle",
   * "stem", "body"...), else the one its grip calls for */
  | { pick: string; hand?: Side | "both"; bend?: number; face?: number; clip?: ItemClip; manner?: PickManner; look?: ItemClip; from?: "above" | "side"; tag?: string }
  /** put the thing held down with its bottom's middle at `at`, or on the thing `on` (`spot` metres from its top's
   * middle), turned `yaw` (by default as it is held) */
  | { place: string; at?: Vec3; on?: string; spot?: [number, number]; yaw?: number; bend?: number }
  /** let go of the thing held where it is: it falls */
  | { drop: string }
  /** throw the thing held to a point (where it lands) or into a thing (onto its top), in a style (by default the one
   * its handling and the distance call for) */
  | { throw: string; to: Vec3 | string; style?: ThrowStyle | "auto"; clip?: ItemClip }
  /** walk on (holding whatever is held) */
  | { walk: [number, number][]; speed?: number; enter?: number; leave?: number }
  /** a performance step as it is (an action clip over an obstacle), ending at `at` facing `face` */
  | { step: Step; at: [number, number]; face: number }
  /** stand (turned to `face`, a yaw, first) */
  | { wait: number; face?: number };

/** A capture a pick or a throw plays (with `clip`): an action clip in the library, from its frame `from` to `to`. The
 * body is the capture's (its mirror image's for the other hand); the arm alone is brought onto the thing (a pick's,
 * as the capture's hand comes lowest) or the thing rides the palm (a throw's, let go where the hand is fastest). */
export type ItemClip = string | { name: string; from?: number; to?: number };

/** how a thing is taken: quickly, the trunk turned to it, or patiently, looked about for and looked over */
export type PickManner = "quick" | "patient";

/** One of the plan's moments with a thing: which step it is, what happens, to what and how it is held. */
export interface Beat {
  step: number;
  kind: "pick" | "place" | "drop" | "throw" | "examine";
  /** a pick's: quick (the trunk turned to it) or patient (looked over in the hand after) */
  manner?: PickManner;
  item: string;
  hands: Side[];
  how: Handling;
  /** the thing's turn in the grip's frame (whose z is the body's way while held): a box squared to the body, a
   * handle turned to the hand */
  turn: number;
  /** how much slower every moment is than for a light thing */
  slow: number;
  /** where it goes, for a place */
  to?: { at: Vec3; yaw: number };
  throw?: { style: ThrowStyle; target: Vec3 };
  /** too heavy: strained at and let be */
  strain?: boolean;
  /** how a low reach is made: 0 bent over from the hips, straight-legged; 1 squatted, the back nearly upright */
  bend: number;
  /** the guide's tag the pick asked for (grasp.ts) */
  tag?: string;
  /** played from a capture: its key frame (the grip, the release) and its length, frames from its first */
  clip?: { key: number; len: number };
}

export interface ItemPlan {
  spec: PerformanceSpec;
  items: Record<string, Item>;
  /** what the items rest on and land on (and walks go round) */
  things: Thing[];
  beats: Beat[];
  /** the steps that play a capture of looking round (before a patient pick): no thing in hand, and no obstacle */
  looks: number[];
  fps: number;
}

// the moments of a reach, seconds into the wait (a light thing's; a heavy one's are stretched): looking, the hand on
// its way, holding it, brought in
export const T = { look: 0.35, reach: [0.15, 0.85] as [number, number], grip: 0.85, lift: [0.95, 1.6] as [number, number], pick: 1.75 };

// the belly's and chest's front, ahead of the body's middle, and a little give (of the height: Anny's kid, man and
// woman all 0.08 at the belly, 0.077-0.086 at the chest); a thing held against the body rests there, never inside
export const FRONT = 0.092;

// the body's segments' masses, a share of the whole each, at the middle of its two joints (de Leva 1996): the trunk,
// the head, and each side's upper arm, forearm, hand, thigh, shin and foot
export const SEGMENTS: [string, string, number][] = [[B.pelvis, B.chest, 0.497], [B.neck, B.head, 0.081],
  ...SIDES.flatMap((s) => [[B.upperarm(s), B.lowerarm(s), 0.028], [B.lowerarm(s), B.hand(s), 0.016], [B.hand(s), B.hand(s), 0.006], [B.thigh(s), B.calf(s), 0.1], [B.calf(s), B.foot(s), 0.0465], [B.foot(s), B.ball(s), 0.0145]] as [string, string, number][])];

// a patient pick, after it: raised before the chest and turned over in the hand, looked at, then lowered to the hold
// (seconds)
export const E = { look: 2.2 };

// a quick pick turns the trunk toward the thing as far as this share of the way round to it (and no more than 0.7)
export const QUICK_TWIST = 0.6;

export const P = { lower: [0.1, 0.8] as [number, number], release: 0.85, back: [0.95, 1.5] as [number, number], place: 1.6 };

export const D = { release: 0.2, back: [0.25, 0.8] as [number, number], drop: 0.9 };

// a strain at a thing too heavy: the reach, the pull (twice), the hands off
export const S = { pull: [0.95, 2.4] as [number, number], off: [2.5, 3.0] as [number, number], strain: 3.1 };

// a throw's moments as shares of its length: the wind-up, the release, the follow-through, back to the walk
export const W = { wind: 0.4, release: 0.58, follow: 0.82 };

export const THROW_LEN: Record<ThrowStyle, number> = { overhand: 1.3, underhand: 1.2, chest: 1.1, toss: 1.4, heave: 1.8 };

// the launch angle (radians over level) and the most an adult throws a light thing that way, m/s
export const LAUNCH: Record<ThrowStyle, { angle: number; most: number }> = {
  overhand: { angle: 0.6, most: 14 }, underhand: { angle: 0.7, most: 7 }, chest: { angle: 0.35, most: 7 }, toss: { angle: 0.8, most: 6 }, heave: { angle: 0.7, most: 4.5 },
};

export const G = 9.81;

// the hands' turn through a throw, in the body's frame (sd: + for the left hand): the way the palm faces, the fingers
export const THROW_HAND: Record<ThrowStyle, (sd: number) => { facing: Vec3; point: Vec3 }> = {
  overhand: () => ({ facing: [0, 0.2, 1], point: [0, 1, 0.1] }),
  underhand: (sd) => ({ facing: [-0.2 * sd, 0.75, 0.6], point: [0, -0.5, 0.85] }),
  chest: (sd) => ({ facing: [-0.6 * sd, 0, 0.8], point: [0, 1, 0.1] }),
  toss: (sd) => ({ facing: [-0.6 * sd, 0.8, 0], point: [0, -0.2, 1] }),
  heave: (sd) => ({ facing: [-0.7 * sd, 0.7, 0], point: [0, -0.2, 1] }),
};

export const smooth = (u: number) => { const x = Math.min(1, Math.max(0, u)); return x * x * (3 - 2 * x); };

export const span = (s: number, [a, b]: [number, number]) => smooth((s - a) / (b - a));

export const mix = (a: Vec3, b: Vec3, u: number): Vec3 => [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u, a[2] + (b[2] - a[2]) * u];

export const lerp = (a: number, b: number, u: number) => a + (b - a) * u;

/** how far a body bending `bend` of the way down at the knees leans over at most, radians (a stoop all the way over,
 * a squat upright enough to balance) */
export const leanFor = (bend: number) => lerp(1.4, 0.7, bend);

/** How far ahead of the hips a hand reaches a point at height `y` easily, bent the way `bend` says: measured on the
 * body standing at rest with the crouch interact uses, so the plan never asks a squat to reach past its arms (it would
 * fold flat over its knees to get there). */
export function reachAhead(skel: Skeleton, y: number, bend: number): number {
  const p0 = rest(skel), x = skel.joints[joint(skel, B.upperarm("R"))].head[0], h = skel.height;
  const ok = (d: number) => {
    const goal: HandGoal = { at: [x, y, d] }, { drop, stoop } = crouchDrop(skel, p0, { R: goal }, leanFor(bend), bend);
    const pa = palmAt(skel, pose(skel, crouchedAt(skel, p0, { R: goal }, drop, leanFor(bend), stoop).pose), "R");
    return Math.hypot(pa[0] - x, pa[1] - y, pa[2] - d) < 0.025;
  };
  // the furthest reached (close in can fail too: a hand down between the knees)
  let lo = 0;
  for (let k = 12; k >= 1 && !lo; k--) if (ok((0.05 * k) * h)) lo = 0.05 * k * h;
  if (!lo) return 0.1 * h;
  let hi = lo + 0.05 * h;
  for (let k = 0; k < 5; k++) { const m = (lo + hi) / 2; if (ok(m)) lo = m; else hi = m; }
  return lo;
}

export const wrap = (a: number) => Math.atan2(Math.sin(a), Math.cos(a));

/** The style a thing is thrown in when none is said: one hand overhand far, underhand near; a heavy thing heaved; a
 * box or a big ball from the chest far, tossed from low near. */
export function autoStyle(how: Handling, dist: number): ThrowStyle {
  if (how.hands === 1) return dist > 3 ? "overhand" : "underhand";
  if (how.weight === "heavy") return "heave";
  return dist > 2 ? "chest" : "toss";
}

// interact's per-frame goals, and its hands on things
export type Goals = { hands: Partial<Record<Side, HandGoal>>; look?: Vec3; weight: number; lookWeight: number; fingers: Partial<Record<Side, number>>; twist: number; pitch: number; roll: number; back: number; abduct: Partial<Record<Side, number>>; dip: number; lean: number; squat: boolean; bend: number; grasps: Grasp[] };
// a hand on a thing: which, how it is gripped, how far the hand is on it and how far the fingers have closed
// (`guide`: one hand placed on the thing's guide, which then rides that hand rigidly)
export type Grasp = { id: string; hands: Side[]; grip: Handling["grip"]; w: number; close: number; guide?: boolean };
// a thing in hand rides on the hands where they really are once the body is solved (`w` of the way: eased in as it
// is lifted, out as it is put down), so it never parts from a hand that falls short of its goal
export type Attach = { id: string; hands: Side[]; w: number };

// a hand's size for keeping clear of the furniture (out to its side, up and down)
export const HAND: [number, number] = [0.035, 0.03];
export const halfOf = (it: Thing): [number, number] => [Math.max(it.size[0], it.size[2]) / 2 + 0.02, it.size[1] / 2 + 0.02];
