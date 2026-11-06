// A BVH retargeted onto a character (docs/skeleton.feature SK3, SK5, SK6): the psx trials' method (projects/psx,
// trials/T4-body/retarget.py), measured there on CMU 02_01 and 100STYLE's Neutral and Old walks.
//
// The character's skeleton is MakeHuman's game_engine rig in its own A-pose, every joint's rest frame the world's; a
// BVH's rest is its actor's T-pose, its frames the world's too. A source map (sdk/raea/tools/maps/<source>.json) names
// the source joint each body joint follows. Each frame, a joint's turn in the body's frame is
//
//     G = A · S · A⁻¹ · F
//
// S the source joint's world turn from its rest, A the facing (a turn about up only: a lean in it tipped the clip
// and sank a walker 17 cm), and F the rest fix: each limb bone laid along its source bone at rest (the T-pose's arms
// onto the A-pose's), the hand along the forearm (no source hand has a length), the forearm and hand twisted so the
// thumb goes where the source's does (palms down for a source without a thumb). The trunk, neck and head keep their
// own build and take the capture's turns from rest. The pelvis travels as the source's hips do, scaled by the hips'
// height over the feet.
import { B, SIDES, captured, joint, pose, qaxis, qfromTo, qinv, qmul, qnorm, qrot, slerp, sub, type Quat, type Skeleton } from "raea/character";
import type { Vec3 } from "raea/3d";
import { clipFrame, type Bvh, type Clip } from "./clip";
import { clearOfTrunk, solveFrame, type IkRig } from "./retarget-ik";

/** retarget solved onto the capture's joint places (retarget-ik.ts) */
export const retargetIK = (skel: Skeleton, b: Bvh, o: RetargetOptions): Clip => retarget(skel, b, { ...o, ik: true });

/** The clavicle's share of the arm's elevation off rest. 0: the capture's own clavicles, turned from the body's rest,
 * already carry the shoulder's rhythm (0.3 was set while clavicles took their source's slope, and sank them 6° more) */
export const SHRUG = 0;
/** how far it turns, degrees: down, up */
const SHRUG_LIMIT: [number, number] = [-20, 25];
/** both arms raised past this off hanging (degrees) is a T-pose: such frames at a clip's ends are left out */
const T_POSE = 70;

const DEG = Math.PI / 180;
const len = (v: Vec3) => Math.hypot(v[0], v[1], v[2]);
const nrm = (v: Vec3): Vec3 => { const l = len(v) || 1; return [v[0] / l, v[1] / l, v[2] / l]; };
const dotv = (a: Vec3, b: Vec3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const crossv = (a: Vec3, b: Vec3): Vec3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const angle = (a: Vec3, b: Vec3) => Math.acos(Math.max(-1, Math.min(1, dotv(nrm(a), nrm(b)))));
/** v with its part along `axis` (a unit vector) taken out */
const across = (v: Vec3, axis: Vec3): Vec3 => nrm(sub(v, [axis[0] * dotv(v, axis), axis[1] * dotv(v, axis), axis[2] * dotv(v, axis)]));

/** A BVH frame's turns (from its rest, relative to the parent) and its root's place, in BVH units. */
function bvhFrame(b: Bvh, values: number[]): { root: Vec3; turns: Quat[] } {
  let c = 0;
  let root: Vec3 = [0, 0, 0];
  const turns = b.joints.map((j) => {
    let q: Quat = [0, 0, 0, 1];
    for (const ch of j.channels) {
      const v = values[c++];
      if (ch === "Xposition") root = [v, root[1], root[2]];
      else if (ch === "Yposition") root = [root[0], v, root[2]];
      else if (ch === "Zposition") root = [root[0], root[1], v];
      // the channels in their order, each about the frame the ones before left (R = R1·R2·R3)
      else q = qmul(q, qaxis(ch[0] === "X" ? [1, 0, 0] : ch[0] === "Y" ? [0, 1, 0] : [0, 0, 1], v * DEG));
    }
    return q;
  });
  return { root, turns };
}

/** a BVH frame's world turns and places (forward kinematics from its offsets) */
function bvhWorld(b: Bvh, f: { root: Vec3; turns: Quat[] }): { rot: Quat[]; pos: Vec3[] } {
  const rot: Quat[] = [], pos: Vec3[] = [];
  b.joints.forEach((j, i) => {
    if (j.parent < 0) { rot[i] = f.turns[i]; pos[i] = f.root; return; }
    const p = j.parent, o = qrot(rot[p], j.offset);
    rot[i] = qmul(rot[p], f.turns[i]);
    pos[i] = [pos[p][0] + o[0], pos[p][1] + o[1], pos[p][2] + o[2]];
  });
  return { rot, pos };
}

/** The body joint each limb bone aims at (its next joint down the limb), and the bone the hand carries on from. */
const AIM: Record<string, string> = Object.fromEntries(SIDES.flatMap((s) => [
  [B.clavicle(s), B.upperarm(s)], [B.upperarm(s), B.lowerarm(s)], [B.lowerarm(s), B.hand(s)],
  [B.thigh(s), B.calf(s)], [B.calf(s), B.foot(s)], [B.foot(s), B.ball(s)],
]));

export interface RetargetOptions {
  name: string;
  fps?: number;
  /** metres a BVH unit (ACCAD's cm 0.01, cgspeed's CMU 0.056444): with it the clip records its `scale`, the body's
   * hip height over the actor's, for Froude-scaled play; without, no scale */
  unit?: number;
  /** seconds of the BVH to take */
  from?: number;
  to?: number;
  /** body joint -> the BVH's joint (a source map); by the skeleton's own names when left out. A body joint left out
   * holds still against its parent. `thumb_01_l`/`_r` name the source's thumb, if it has one (its end site is where
   * the thumb points). */
  map?: Record<string, string>;
  /** the clavicles' share of the arm's elevation (SHRUG by default; 0 leaves the capture's clavicles) */
  shrug?: number;
  /** keep T-pose frames at the ends (left out by default) */
  keepTPose?: boolean;
  /** passes of the five-frame smoothing over each joint's turn (1 by default; 0 keeps the capture's every flicker) */
  smooth?: number;
  /** the neck's and head's mean turns to take out (a clip's `carriage`: one actor's neutral walk), so they move about
   * the body's own carriage as the capture moves about its; the clip's own mean when left out, none with false */
  carriage?: Record<string, Quat> | false;
  /** a frame of channel values the BVH's turns are measured from instead of its zero pose: for a source whose zero
   * pose isn't its actor standing (ACCAD's MotionBuilder exports lie the trunk along x at zero; their actors stand
   * in each one's A1_Stand), so its rest is that frame, turned to face +z */
  rest?: number[];
  /** solve the body onto the capture's joint places within a person's range (retarget-ik.ts) rather than transfer
   * each joint's turn; the neck and head then follow the capture's, not a carriage */
  ik?: boolean;
}

/** A BVH retargeted onto a skeleton, resampled to `fps`, from `from` to `to` seconds. The BVH's up is y and its actor
 * faces +z at rest (cgspeed's CMU files and 100STYLE's do); its travel is scaled by the hips' height. */
export function retarget(skel: Skeleton, b: Bvh, o: RetargetOptions): Clip {
  const fps = o.fps ?? 30, shrug = o.shrug ?? SHRUG;
  const J = skel.joints.slice(0, captured(skel)), nj = J.length;
  const srcOf = (name: string) => {
    const want = o.map ? o.map[name] : name;
    if (!want) return -1;
    const i = b.joints.findIndex((x) => x.name === want);
    if (i < 0 && o.map) throw new Error(`retarget ${o.name}: the map sends ${name} to ${want}, which the BVH hasn't`);
    return i;
  };
  const src = J.map((j) => srcOf(j.name));
  if (src[0] < 0) throw new Error(`retarget ${o.name}: the BVH has no joint for the root ${J[0].name}`);
  const need = [B.thigh("L"), B.thigh("R"), B.foot("L"), B.foot("R")];
  for (const n of need) if (src[joint(skel, n)] < 0) throw new Error(`retarget ${o.name}: the BVH has no joint for ${n}`);
  const S = (n: string) => src[joint(skel, n)];
  const T = (n: string) => J[joint(skel, n)].head;
  // the source at rest (its rest frames are the world's), or at its `rest` frame, every joint's turn there taken as
  // its rest (Rf: what each world turn is measured from)
  const zero = bvhWorld(b, { root: [0, 0, 0], turns: b.joints.map(() => [0, 0, 0, 1] as Quat) });
  let R0 = zero.pos, Rf: Quat[] | undefined;
  if (o.rest) {
    const w = bvhWorld(b, { ...bvhFrame(b, o.rest), root: [0, 0, 0] });
    const lt = sub(w.pos[S(B.thigh("L"))], w.pos[S(B.thigh("R"))]);
    // its left onto +x, so it faces +z
    const Y = qfromTo(nrm([lt[0], 0, lt[2]]), [1, 0, 0]);
    R0 = w.pos.map((p) => qrot(Y, p));
    Rf = w.rot.map((q) => qinv(qmul(Y, q)));
  }
  // the facing: a turn about up taking the source's left (its left thigh from its right) onto the body's
  const flatX = (v: Vec3) => nrm([v[0], 0, v[2]]);
  const A = qfromTo(flatX(sub(R0[S(B.thigh("L"))], R0[S(B.thigh("R"))])), flatX(sub(T(B.thigh("L")), T(B.thigh("R")))));
  const Ai = qinv(A);
  // the source scaled to the body by the hips' height over the feet at rest
  const feetS = [B.foot("L"), B.foot("R"), B.ball("L"), B.ball("R")].map(S).filter((i) => i >= 0);
  const hs = R0[src[0]][1] - Math.min(...feetS.map((i) => R0[i][1]));
  const ht = J[0].head[1] - Math.min(...B.feet().map((n) => T(n)[1]));
  const k = ht / hs;
  // the rest fix F per joint
  const fix: Quat[] = J.map(() => [0, 0, 0, 1] as Quat);
  const restDir = (i: number): Vec3 => {
    const c = J.findIndex((x) => x.parent === i && AIM[J[i].name] === x.name);
    return c >= 0 ? sub(J[c].head, J[i].head) : sub(J[i].tail, J[i].head);
  };
  J.forEach((j, i) => {
    const aim = AIM[j.name];
    // (not the clavicles: a capture's shoulder joints sit where its markers put them, ACCAD's sloped 34 degrees and CMU's
    // nearly level, so a clavicle laid along its source's took that marker set's slope; it moves from the body's own rest)
    if (aim !== undefined && src[i] >= 0 && !SIDES.some((s) => j.name === B.clavicle(s))) {
      const si = src[joint(skel, aim)];
      const sd = si >= 0 ? sub(R0[si], R0[src[i]]) : undefined;
      if (sd && len(sd) > 1e-6) fix[i] = qfromTo(restDir(i), qrot(A, sd));
    }
  });
  for (const s of SIDES) {
    const h = joint(skel, B.hand(s)), fa = joint(skel, B.lowerarm(s));
    // the hand along the forearm: its rest bone turned as the forearm's was, then onto the source forearm's line
    if (src[h] >= 0 && src[fa] >= 0) {
      const sd = qrot(A, sub(R0[src[h]], R0[src[fa]]));
      fix[h] = qfromTo(sub(J[h].tail, J[h].head), sd);
    }
    // the forearm and hand twisted about their own length so the thumb goes the source's way (forward if it has none)
    const thumbName = o.map?.[B.finger("thumb", 1, s)];
    const ti = thumbName ? b.joints.findIndex((x) => x.name === thumbName) : -1;
    const want: Vec3 = ti >= 0 && b.joints[ti].end ? qrot(A, sub(add3(R0[ti], b.joints[ti].end!), R0[src[h]])) : [0, 0, 1];
    const thumb0 = sub(skel.joints[joint(skel, B.finger("thumb", 1, s))].head, J[h].head);
    for (const i of [fa, h]) {
      if (src[i] < 0) continue;
      const axis = nrm(qrot(fix[i], restDir(i)));
      const have = across(qrot(fix[i], thumb0), axis), to = across(want, axis);
      const a = angle(have, to) * (dotv(crossv(have, to), axis) > 0 ? 1 : -1);
      fix[i] = qmul(qaxis(axis, a), fix[i]);
    }
  }
  // the shoulders' rest: each upper arm's elevation off hanging along the chest, in the body at rest
  const chest = joint(skel, B.chest);
  const chestUp = nrm(restDir(chest));
  const elevation = (arm: Vec3, up: Vec3) => angle(arm, [-up[0], -up[1], -up[2]]) / DEG;
  const restElev = Object.fromEntries(SIDES.map((s) => [s, elevation(restDir(joint(skel, B.upperarm(s))), chestUp)]));
  // T-pose frames at the ends (the source's own arms raised past T_POSE on both sides)
  const dur = (b.frames.length - 1) * b.frameTime;
  let t0 = Math.max(0, o.from ?? 0), t1 = Math.min(dur, o.to ?? dur);
  if (!o.keepTPose) {
    const raised = (t: number) => {
      const w = bvhWorld(b, bvhFrame(b, b.frames[Math.min(b.frames.length - 1, Math.round(t / b.frameTime))]));
      const up = sub(w.pos[S(B.head)] ?? w.pos[src[0]], w.pos[src[0]]);
      return SIDES.every((s) => {
        const a = S(B.upperarm(s)), f = S(B.lowerarm(s));
        return a >= 0 && f >= 0 && elevation(sub(w.pos[f], w.pos[a]), up) > T_POSE;
      });
    };
    // (a clip held in a T-pose throughout is a pose, not a calibration: kept)
    let a = t0, z = t1;
    while (a < z && raised(a)) a += 1 / fps;
    while (z > a && raised(z)) z -= 1 / fps;
    if (a < z) { t0 = a; t1 = z; }
  }
  const n = Math.max(1, Math.floor((t1 - t0) * fps) + 1);
  const root = new Float32Array(n * 3), turns = new Float32Array(n * nj * 4);
  const clav = SIDES.map((s) => [joint(skel, B.clavicle(s)), joint(skel, B.upperarm(s))]);
  const headS = S(B.head), rig: IkRig | undefined = o.ik ? { src, restDir, crown: headS >= 0 ? b.joints[headS].end : undefined } : undefined;
  for (let f = 0; f < n; f++) {
    const at = (t0 + f / fps) / b.frameTime;
    const a = Math.min(b.frames.length - 1, Math.floor(at)), c = Math.min(b.frames.length - 1, a + 1), u = at - a;
    const FA = bvhFrame(b, b.frames[a]), FB = bvhFrame(b, b.frames[c]);
    const fr = { root: lerp3(FA.root, FB.root, u), turns: FA.turns.map((q, i) => slerp(q, FB.turns[i], u)) };
    const ws = bvhWorld(b, fr), raw = ws.rot;
    if (Rf) ws.rot = ws.rot.map((q, i) => qmul(q, Rf![i]));
    const G: Quat[] = [];
    for (let i = 0; i < nj; i++) {
      const p = J[i].parent;
      G[i] = src[i] >= 0 ? qnorm(qmul(qmul(qmul(A, ws.rot[src[i]]), Ai), fix[i])) : p < 0 ? [0, 0, 0, 1] : G[p];
    }
    // the shoulder rhythm: each clavicle turned by `shrug` of its arm's elevation off rest (the upper arm keeps its
    // captured world turn, so the arm still points where the capture's does)
    if (shrug) SIDES.forEach((s, si) => {
      const [ci, ui] = clav[si];
      if (src[ci] < 0 || src[ui] < 0) return;
      const up = nrm(qrot(G[chest], restDir(chest)));
      const e = elevation(qrot(G[ui], restDir(ui)), up);
      const lift = Math.max(SHRUG_LIMIT[0], Math.min(SHRUG_LIMIT[1], shrug * (e - restElev[s])));
      const d = nrm(qrot(G[ci], restDir(ci)));
      G[ci] = qnorm(qmul(qaxis(nrm(crossv(up, d)), -lift * DEG), G[ci]));
    });
    if (rig) { const solved = solveFrame(skel, rig, G, ws.pos.map((x) => qrot(A, x)), raw, A); G.splice(0, G.length, ...(clearOfTrunk(skel, solved, nj))); }
    for (let i = 0; i < nj; i++) {
      const p = J[i].parent;
      turns.set(p < 0 ? G[i] : qmul(qinv(G[p]), G[i]), (f * nj + i) * 4);
    }
    const r = qrot(A, fr.root);
    root.set([r[0] * k, r[1] * k, r[2] * k], f * 3);
  }
  // the neck and head carried as the body carries them, moving about it as the capture moves about its mean (the
  // capture's own carriage onto a body of another build put the head 18 degrees off and read wrong). The mean is the
  // clip's own or `carriage` (one actor's neutral walk), so a style's own carriage (an old man's hanging head) stays
  const carriage: Record<string, Quat> = {};
  if (o.carriage !== false && !o.ik) for (const name of [B.neck, B.head]) {
    const i = joint(skel, name);
    let mq = o.carriage?.[name];
    if (!mq) {
      const m = [0, 0, 0, 0];
      for (let f = 0; f < n; f++) {
        const o4 = (f * nj + i) * 4, sg = turns[o4 + 3] < 0 ? -1 : 1;
        for (let a = 0; a < 4; a++) m[a] += sg * turns[o4 + a];
      }
      mq = qnorm(m as Quat);
    }
    carriage[name] = mq;
    const off = qinv(mq);
    for (let f = 0; f < n; f++) {
      const o4 = (f * nj + i) * 4;
      turns.set(qmul(off, Array.from(turns.subarray(o4, o4 + 4)) as Quat), o4);
    }
  }
  // a capture's frame-to-frame noise (toe markers flick, a solved hand jumps a frame) taken out: each joint's turn the
  // binomial mean of the five frames round it, `smooth` times over (1, a 30 fps clip's default: under a frame of lag)
  for (let pass = 0; pass < (o.smooth ?? 1); pass++) {
    const was = turns.slice();
    for (let f = 0; f < n; f++) for (let i = 0; i < nj; i++) {
      const c = (f * nj + i) * 4, m = [0, 0, 0, 0];
      for (const [d, w] of TAPS) {
        const o4 = (Math.min(n - 1, Math.max(0, f + d)) * nj + i) * 4;
        const sg = was[o4] * was[c] + was[o4 + 1] * was[c + 1] + was[o4 + 2] * was[c + 2] + was[o4 + 3] * was[c + 3] < 0 ? -w : w;
        for (let a = 0; a < 4; a++) m[a] += sg * was[o4 + a];
      }
      turns.set(qnorm(m as Quat), c);
    }
  }
  // feet on the floor: the clip lifted so its lowest ball, over the clip, is at the balls' height at rest (feet.ts
  // plants them frame by frame)
  const clip: Clip = { name: o.name, fps, frames: n, root, turns, carriage, ...(o.unit ? { scale: k / o.unit } : {}) };
  const balls = SIDES.map(B.ball).map((x) => joint(skel, x));
  let low = Infinity;
  for (let f = 0; f < n; f++) {
    const w = pose(skel, clipFrame(clip, f, nj));
    for (const t of balls) low = Math.min(low, w.pos[t][1]);
  }
  const lift = J[balls[0]].head[1] - low;
  for (let f = 0; f < n; f++) root[f * 3 + 1] += lift;
  return clip;
}

/** the binomial five taps (frame offset, weight) */
const TAPS: [number, number][] = [[-2, 1 / 16], [-1, 4 / 16], [0, 6 / 16], [1, 4 / 16], [2, 1 / 16]];
const add3 = (a: Vec3, b: Vec3): Vec3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const lerp3 = (a: Vec3, b: Vec3, u: number): Vec3 => [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u, a[2] + (b[2] - a[2]) * u];
