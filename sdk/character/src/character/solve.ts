// The body's surface, reaching, planting, looking, sitting, and solving goals.
import { type Vec3 } from "@raea/render/3d";
import { FINGERS, Finger, I, LIMB, Pose, Posed, Side, Skeleton, add, cross, fingerJoints, flatHand, held, joint, len, norm, palmAt, pose, qaxis, qfromTo, qmul, qrot, qyaw, scale3, setWorld, skinJoints, slerp, sub, twoBone, withFingers } from "./all";

export interface FingerSegment { joint: number; head: Vec3; tail: Vec3; radius: number }

/** Each finger's bones posed (thumb first), phalanx 1 to 3: from its joint to the next's (the last to its tip), and
 * its thickness, so a grip can test a finger against a thing's surface (a capsule a bone long, `radius` round). */
export function fingerSegments(skel: Skeleton, w: Posed, s: Side): { finger: Finger; segments: FingerSegment[] }[] {
  const J = skel.joints;
  w = withFingers(skel, w);
  return fingerJoints(skel, s).map((ch, k) => ({
    finger: FINGERS[k],
    segments: ch.map((i, ph) => ({
      joint: i, head: w.pos[i], radius: J[i].radius ?? 0.008,
      tail: ph < 2 ? w.pos[ch[ph + 1]] : add(w.pos[i], qrot(w.rot[i], sub(J[i].tail, J[i].head))),
    })),
  }));
}

export interface SurfacePoint { part: string; side: Side | ""; joint: string; at: Vec3; normal: Vec3 }

/** The skin's surface samples posed (character.py's `surface`): each a vertex of the skinned mesh, where the skin puts it
 * (the glb's own linear blend of its skin joints) and its normal. `parts` picks them by part ("palm", "thumb1",
 * "index2", "chest", "waist", "foreArm"...) or by skin joint's name (a RegExp tests both); `side` by side. For hugging a
 * thing to the chest without it going in, keeping fingertips on a thing's surface. */
export function surface(skel: Skeleton, w: Posed, parts?: RegExp | string[], side?: Side): SurfacePoint[] {
  if (!skel.surface || !skel.skin) return [];
  const sk = skel.skin, sj = skinJoints(skel, w);
  const want = (part: string, name: string) => !parts || (parts instanceof RegExp ? parts.test(part) || parts.test(name) : parts.includes(part) || parts.includes(name));
  const out: SurfacePoint[] = [];
  for (const x of skel.surface) {
    if ((side && x.side !== side) || !want(x.part, sk[x.j[0]].name)) continue;
    const v = add(sk[x.j[0]].head, x.o);
    let at: Vec3 = [0, 0, 0], n: Vec3 = [0, 0, 0], tw = 0;
    x.j.forEach((j, k) => {
      at = add(at, scale3(add(sj[j].pos, qrot(sj[j].rot, sub(v, sk[j].head))), x.w[k]));
      n = add(n, scale3(qrot(sj[j].rot, x.n), x.w[k]));
      tw += x.w[k];
    });
    out.push({ part: x.part, side: x.side, joint: sk[x.j[0]].name, at: scale3(at, 1 / tw), normal: norm(n) });
  }
  return out;
}

/** A hand's surface samples posed: the palm (and the back of the hand) and each finger's and the thumb's phalanges. */
export const handSurface = (skel: Skeleton, w: Posed, s: Side): SurfacePoint[] => surface(skel, w, /^(palm|thumb|index|middle|ring|pinky)\d?$/, s);

export interface HandGoal {
  /** the point the palm's middle goes to (the body's frame unless the Character turns it); a flat hand rests on it, its
   * palm's skin on the point rather than the middle of the hand */
  at: Vec3;
  /** the hand flat: its fingers this way, the palm down (or `facing`) */
  point?: Vec3;
  facing?: Vec3;
  /** which way the elbow points (down and out by default) */
  elbow?: Vec3;
}

/** The hand put on a point: flat first if it points, then the arm bent to bring the palm there. */
export function reach(skel: Skeleton, p: Pose, s: Side, g: HandGoal): Pose {
  if (g.point) p = flatHand(skel, p, s, g.point, g.facing);
  // a flat hand's middle half its thickness off the surface it rests on (about 1% of the body's height)
  if (g.point) g = { ...g, at: sub(g.at, scale3(norm(g.facing ?? [0, -1, 0]), 0.009 * skel.height)) };
  const hi = joint(skel, s === "L" ? "LeftHand" : "RightHand");
  // the wrist where the palm lands at the at, with the hand's turn as it will be (twoBone keeps it)
  let w = pose(skel, p);
  const off = sub(palmAt(skel, w, s), w.pos[hi]);
  // down and out turned with the hips (in the scene's frame a pose's root turns the body; a pole fixed to the scene
  // swung the elbow round as the body turned to a shelf)
  const ahead = qrot(w.rot[0], [0, 0, 1]);
  const elbow = g.elbow ?? qrot(qyaw(Math.atan2(ahead[0], ahead[2])), [s === "L" ? 0.4 : -0.4, -1, -0.3]);
  for (let k = 0; k < 3; k++) {
    p = twoBone(skel, p, LIMB.arm(s), sub(g.at, off), elbow);
    w = pose(skel, p);
  }
  return p;
}

export interface FootGoal {
  /** the floor's point under the ankle */
  at: Vec3;
  /** the sole laid flat on the floor (true by default) pointing the toes this way (where they point now by default) */
  flat?: boolean;
  toes?: Vec3;
}

/** A foot planted: the leg bent so the ankle stands over `at` at its height at rest, the sole flat. */
export function plant(skel: Skeleton, p: Pose, s: Side, g: FootGoal): Pose {
  const fi = joint(skel, s === "L" ? "LeftFoot" : "RightFoot");
  const ti = joint(skel, s === "L" ? "LeftToeBase" : "RightToeBase");
  const J = skel.joints;
  const ankle = J[fi].head[1];
  if (g.flat !== false) {
    const w = pose(skel, p);
    const restDir = sub(J[ti].head, J[fi].head);
    const now = qrot(w.rot[fi], restDir);
    const toes = g.toes ?? now;
    // the rest foot's own pitch kept (its heel is higher than its ball), its heading turned to the toes'
    const headingRest = Math.atan2(restDir[0], restDir[2]), heading = Math.atan2(toes[0], toes[2]);
    p = setWorld(skel, p, w, fi, qmul(qyaw(heading - headingRest), I));
  }
  return twoBone(skel, p, LIMB.leg(s), add(g.at, [0, ankle, 0]), undefined);
}

/** The head (with the neck taking a share) turned so the eyes look at `target`. */
export function lookAt(skel: Skeleton, p: Pose, target: Vec3, share: [number, number, number] = [0.25, 0.25, 0.5]): Pose {
  const chain = ["Neck", "Neck1", "Head"].map((n) => joint(skel, n));
  const eyesRest = skel.landmarks.eyes;
  const head = chain[2];
  for (let k = 0; k < 4; k++) {
    const w0 = pose(skel, p);
    const eyes = add(w0.pos[head], qrot(w0.rot[head], sub(eyesRest, skel.joints[head].head)));
    const fwd = qrot(w0.rot[head], [0, 0, 1]);
    const want = norm(sub(target, eyes));
    // the turn taking the face's forward to the target, split up the neck (yaw about the body's up, then pitch)
    const turn = qfromTo(fwd, want);
    // each joint's share of the turn, about the same axis in the body's frame, so the shares add up to the whole
    for (let c = 0; c < 3; c++) {
      const w = pose(skel, p);
      p = setWorld(skel, p, w, chain[c], qmul(slerp(I, turn, share[c]), w.rot[chain[c]]));
    }
  }
  return p;
}

export interface SeatGoal {
  /** the seat's top, under the middle of the hips */
  at: Vec3;
  /** where the feet stand on the floor (each under its ankle), by default a thigh's length ahead */
  feet?: { L?: Vec3; R?: Vec3 };
  /** the floor's height (0) */
  floor?: number;
  /** how far forward the back leans, radians (0.15) */
  lean?: number;
}

/** Sat on a seat: the hips on it, the thighs level ahead, the shins down to the feet on the floor, the back up (a
 * little forward). In the body's frame, facing +z. */
export function sit(skel: Skeleton, p: Pose, g: SeatGoal): Pose {
  const J = skel.joints;
  const hips = J[0].head;
  // the hip joints sit a little above the seat (the buttocks under them): their height over the lowest of the body at
  // the hips, from the rest's leg (about a fifth of the thigh)
  const over = skel.landmarks.sitDrop ?? 0.3 * len(sub(J[joint(skel, "LeftLeg")].head, J[joint(skel, "LeftUpLeg")].head));
  p = { ...p, root: [g.at[0], g.at[1] + over + (hips[1] - J[joint(skel, "LeftUpLeg")].head[1]), g.at[2]] };
  const lean = g.lean ?? 0.15;
  p = held(skel, {
    LeftUpLeg: [0.08, -0.05, 1], RightUpLeg: [-0.08, -0.05, 1],
    LowerBack: [0, 1, Math.tan(lean)], Spine: [0, 1, Math.tan(lean)], Spine1: [0, 1, Math.tan(lean)],
  }, p);
  const floor = g.floor ?? 0;
  for (const s of ["L", "R"] as Side[]) {
    const w = pose(skel, p);
    const knee = w.pos[joint(skel, s === "L" ? "LeftLeg" : "RightLeg")];
    const at = g.feet?.[s] ?? [knee[0], floor, knee[2] + 0.05];
    p = plant(skel, p, s, { at, toes: [0, 0, 1] });
  }
  return p;
}

export interface Goals {
  hands?: Partial<Record<Side, HandGoal>>;
  feet?: Partial<Record<Side, FootGoal>>;
  look?: Vec3;
  sit?: SeatGoal;
  /** how much of the goals is met, 0..1 (1): ease them in and out (the hands to the desk once sat) */
  weight?: number;
  /** the gaze's own weight, under `weight` (1) */
  lookWeight?: number;
  /** false: the back stays as the pose has it, however far the hands must reach */
  lean?: boolean;
}

/** Between two poses: the root moved and every turn slerped `w` of the way. */
export function blend(a: Pose, b: Pose, w: number): Pose {
  if (w <= 0) return a;
  if (w >= 1) return b;
  // (over both poses' turns: one may turn the fingers' own joints and the other not)
  const n = Math.max(a.turns.length, b.turns.length);
  return { root: add(a.root, scale3(sub(b.root, a.root), w)), turns: Array.from({ length: n }, (_, i) => slerp(a.turns[i] ?? I, b.turns[i] ?? I, w)) };
}

/** The back bent toward the hands' goals (from the hips, the lower back taking least) until every palm is in easy
 * reach (`reach` of the arm's length, 0.85), up to `most` radians: someone at a desk leans in to the mouse rather than
 * locking an arm straight. */
export function leanToward(skel: Skeleton, p: Pose, hands: Partial<Record<Side, HandGoal>>, o: { reach?: number; most?: number } = {}): Pose {
  const goals = (Object.entries(hands) as [Side, HandGoal | undefined][]).filter((e): e is [Side, HandGoal] => !!e[1]);
  if (!goals.length) return p;
  const chain = ["LowerBack", "Spine", "Spine1"].map((n) => joint(skel, n)), share = [0.25, 0.35, 0.4];
  const J = skel.joints;
  const arm = (s: Side) => {
    const [a, b, c] = LIMB.arm(s).map((n) => joint(skel, n));
    return { a, l: len(sub(J[b].head, J[a].head)) + len(sub(J[c].head, J[b].head)) };
  };
  const easy = o.reach ?? 0.85;
  const w0 = pose(skel, p);
  // which way to lean: from between the shoulders toward the goals, level
  const mid = scale3(add(w0.pos[arm("L").a], w0.pos[arm("R").a]), 0.5);
  const toward = goals.reduce<Vec3>((acc, [, h]) => add(acc, sub(h.at, mid)), [0, 0, 0]);
  const axis = norm(cross([0, 1, 0], [toward[0], 0, toward[2]]));
  if (len(axis) < 0.5) return p;
  const leaned = (th: number) => {
    let q = p;
    for (let c = 0; c < 3; c++) {
      const w = pose(skel, q);
      q = setWorld(skel, q, w, chain[c], qmul(qaxis(axis, th * share[c]), w.rot[chain[c]]));
    }
    return q;
  };
  const far = (q: Pose) => {
    const w = pose(skel, q);
    return Math.max(...goals.map(([s, h]) => len(sub(h.at, w.pos[arm(s).a])) / arm(s).l));
  };
  if (far(p) <= easy) return p;
  // the least lean that brings them in (or the most it will lean)
  let lo = 0, hi = o.most ?? 0.9;
  if (far(leaned(hi)) > easy) return leaned(hi);
  for (let k = 0; k < 12; k++) {
    const m = (lo + hi) / 2;
    if (far(leaned(m)) > easy) lo = m;
    else hi = m;
  }
  return leaned(hi);
}

/** Every goal met on top of a pose, in order: the seat, the feet, the lean toward the hands, the hands, the gaze;
 * `weight` of the way. */
export function solve(skel: Skeleton, p: Pose, g: Goals): Pose {
  const from = p;
  if (g.sit) p = sit(skel, p, g.sit);
  for (const [s, f] of Object.entries(g.feet ?? {})) if (f) p = plant(skel, p, s as Side, f);
  if (g.hands && g.lean !== false) p = leanToward(skel, p, g.hands);
  for (const [s, h] of Object.entries(g.hands ?? {})) if (h) p = reach(skel, p, s as Side, h);
  if (g.look) p = blend(p, lookAt(skel, p, g.look), g.lookWeight ?? 1);
  return blend(from, p, g.weight ?? 1);
}
