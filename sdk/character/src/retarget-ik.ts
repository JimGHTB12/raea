// The body solved onto a capture frame by frame (retarget's `ik`; Shane, 11-01: "the whole human rig breaks with
// these ... lets do the ik based one"). The rotation transfer (retarget.ts: each joint turned as its source joint
// turned from rest) is only right where the two skeletons stand alike at rest and twist their bones alike, and every
// source broke it its own way (a T-pose, an A-pose, ACCAD's trunk lying along x). Here the capture is taken by where
// its joints are, which no rest pose changes, and the body is solved to it within what a person can do:
//
//   the trunk, neck and head point as the source's segments point (pelvis to the chest, the chest to the neck, the
//     neck to the head, the head to its crown), each turned no more than it needs (its twist the transfer's);
//   an elbow and a knee are hinges: the forearm and the shin bend about the joint's own axis, never back past
//     straight, and that bend's plane is what turns the upper arm and the thigh about their length (the transfer's
//     twist only while the limb is near straight and the plane says nothing);
//   the forearm's turn about its length (the hand's pronation), the wrist, the ankle and the head keep the
//     transfer's, held inside a person's range;
//   the limbs' bones point where the source's do, at the body's own lengths (a hand goes where the source's goes,
//     scaled to the body).
//
// Inputs are world turns from rest (the body's rest frames are the world's) and the source's joint places this frame,
// already faced and in the body's frame (A applied); out come the world turns solved.
import type { Vec3 } from "raea/3d";
import { B, SIDES, cross, dot, joint, len, norm, pose, qaxis, qfromTo, qinv, qmul, qrot, slerp, sub, surface, type Quat, type Side, type Skeleton } from "raea/character";

const DEG = Math.PI / 180;
/** a person's range, degrees: the forearm's turn about its length from rest, the wrist's bend and its turn, the ankle's
 * bend and turn, the head's and neck's turn, the upper arm's and thigh's turn about their length, the shin's */
export const IK_LIMITS = { pronate: 95, wrist: 75, wristTwist: 35, ankle: 50, ankleTwist: 25, neckTwist: 60, armTwist: 95, legTwist: 55, shinTwist: 12 };

/** the twist of turn `q` about unit `axis` (radians), and the turn with it taken out (its swing) */
function swingTwist(q: Quat, axis: Vec3): { swing: Quat; twist: number } {
  const p = dot([q[0], q[1], q[2]], axis);
  let tw: Quat = [axis[0] * p, axis[1] * p, axis[2] * p, q[3]];
  const l = Math.hypot(...tw);
  tw = l < 1e-9 ? [0, 0, 0, 1] : [tw[0] / l, tw[1] / l, tw[2] / l, tw[3] / l];
  const twist = 2 * Math.atan2(dot([tw[0], tw[1], tw[2]], axis), tw[3]);
  return { swing: qmul(q, qinv(tw)), twist: ((twist + 3 * Math.PI) % (2 * Math.PI)) - Math.PI };
}
const clamp = (x: number, lim: number) => Math.max(-lim * DEG, Math.min(lim * DEG, x));
/** `q` with its twist about `axis` held to `lim` degrees and its swing to `swingLim` (if given) */
function held(q: Quat, axis: Vec3, lim: number, swingLim?: number): Quat {
  const { swing, twist } = swingTwist(q, axis);
  let s = swing;
  if (swingLim !== undefined) {
    const a = 2 * Math.acos(Math.min(1, Math.abs(s[3])));
    if (a > swingLim * DEG) s = slerp([0, 0, 0, 1], s, (swingLim * DEG) / a);
  }
  return qmul(s, qaxis(axis, clamp(twist, lim)));
}
/** G turned the least that points its rest bone `rest` along `to` */
const aimed = (G: Quat, rest: Vec3, to: Vec3): Quat => (len(to) < 1e-9 ? G : qmul(qfromTo(qrot(G, rest), to), G));
const smooth = (a: number, b: number, x: number) => { const t = Math.max(0, Math.min(1, (x - a) / (b - a))); return t * t * (3 - 2 * t); };

export interface IkRig {
  /** body joint index -> the source joint it follows (-1 none) */
  src: number[];
  /** a joint's rest bone (to the next joint down its chain, or its own tail) */
  restDir: (i: number) => Vec3;
  /** the source's crown: the head joint's end site, in its own frame (or none) */
  crown?: Vec3;
}

/**
 * The world turns `G` (from rest, the transfer's) solved onto the source's joint places `P` (faced, in the body's
 * frame) and world turns `R` (raw, for the crown's end site), as the header says.
 */
export function solveFrame(skel: Skeleton, rig: IkRig, G: Quat[], P: Vec3[], R: Quat[], A: Quat): Quat[] {
  const out = G.slice(), J = skel.joints, j = (n: string) => joint(skel, n), s = (n: string) => rig.src[j(n)];
  const dirOf = (a: string, b: string): Vec3 | undefined => (s(a) >= 0 && s(b) >= 0 ? sub(P[s(b)], P[s(a)]) : undefined);
  // the pelvis tipped onto the source's hips (up along the hips to the spine), its heading the transfer's
  // (up to the first spine joint that sits clear of the hips: CMU's lower back starts at its hips)
  const pv = j(B.pelvis);
  const above = [...B.spine, B.neck].find((n) => { const d = dirOf(B.pelvis, n); return d && len(d) > 0.03; }), up = above && dirOf(B.pelvis, above);
  // (only a tip: a pelvis the transfer has far off, rolling or flipping, keeps the transfer's)
  if (above && up) {
    const r0 = sub(J[j(above)].head, J[pv].head), off = Math.acos(Math.max(-1, Math.min(1, dot(norm(qrot(out[pv], r0)), norm(up)))));
    const w = 1 - smooth(TIP - 10 * DEG, TIP, off);
    if (w > 0) out[pv] = slerp(out[pv], aimed(out[pv], r0, up), w);
  }
  // the trunk, neck and head along the source's segments
  const trunk: [string, string][] = [[B.spine[0], B.spine[1]], [B.spine[1], B.spine[2]], [B.spine[2], B.neck], [B.neck, B.head]];
  for (const [a, b] of trunk) { const d = dirOf(a, b); if (d && len(d) > 0.01) out[j(a)] = aimed(out[j(a)], sub(J[j(b)].head, J[j(a)].head), d); }
  const h = j(B.head);
  if (rig.crown && s(B.head) >= 0) {
    const crown = qrot(A, qrot(R[s(B.head)], rig.crown));
    // (the head's own bone runs up to its crown)
    out[h] = aimed(out[h], sub(J[h].tail, J[h].head), crown);
  }
  // the head and neck turned about their length no further than a person turns them, against the chest and neck
  for (const [n, p] of [[B.neck, B.chest], [B.head, B.neck]] as const) {
    const i = j(n), local = qmul(qinv(out[j(p)]), out[i]);
    out[i] = qmul(out[j(p)], held(local, norm(rig.restDir(i)), IK_LIMITS.neckTwist));
  }
  for (const side of SIDES) {
    limb(skel, rig, out, P, side, B.upperarm(side), B.lowerarm(side), B.hand(side), [0, 0, 1], { twist: IK_LIMITS.armTwist, mid: IK_LIMITS.pronate, end: IK_LIMITS.wristTwist, endSwing: IK_LIMITS.wrist });
    limb(skel, rig, out, P, side, B.thigh(side), B.calf(side), B.foot(side), [0, 0, -1], { twist: IK_LIMITS.legTwist, mid: IK_LIMITS.shinTwist, end: IK_LIMITS.ankleTwist, endSwing: IK_LIMITS.ankle });
    // the foot along the source's foot (the ankle to the ball)
    const f = j(B.foot(side)), d = dirOf(B.foot(side), B.ball(side));
    if (d) out[f] = aimed(out[f], rig.restDir(f), d);
  }
  return out;
}

/**
 * One limb solved: the upper bone `up` along the source's, its twist from the hinge's plane; the lower bone `mid` bent
 * about the hinge only (toward `bend` at rest: forward for an elbow, back for a knee), never past straight, turned
 * about its length as the transfer had it within `lim.mid`; the end (`end`: the hand, the foot) kept as it was against
 * the lower bone, within its range.
 */
function limb(skel: Skeleton, rig: IkRig, G: Quat[], P: Vec3[], _side: Side, up: string, mid: string, end: string, bend: Vec3, lim: { twist: number; mid: number; end: number; endSwing: number }) {
  const u = joint(skel, up), m = joint(skel, mid), e = joint(skel, end), su = rig.src[u], sm = rig.src[m], se = rig.src[e];
  if (su < 0 || sm < 0 || se < 0) return;
  const ru = norm(rig.restDir(u)), rm = norm(rig.restDir(m)), re = norm(rig.restDir(e));
  // the hinge at rest: the axis that bends the lower bone toward `bend` (forward for an elbow, back for a knee)
  const hinge0 = norm(cross(ru, bend));
  const du = norm(sub(P[sm], P[su])), dm = norm(sub(P[se], P[sm]));
  const theta = Math.acos(Math.max(-1, Math.min(1, dot(du, dm))));
  // the end against the lower bone as the transfer had it (kept through the lower bone's new turn)
  const endLocal = qmul(qinv(G[m]), G[e]);
  // the upper bone along the source's, its twist the transfer's
  let Gu = aimed(G[u], ru, du);
  // the hinge's plane: the twist about the upper bone that lays the hinge across the bend (a bent limb says it)
  const want = norm(cross(du, dm)), have = qrot(Gu, hinge0);
  const w = smooth(8 * DEG, 25 * DEG, theta);
  if (w > 0 && len(cross(du, dm)) > 1e-6) {
    const a = Math.atan2(dot(cross(have, want), du), dot(have, want));
    // (a bend the hinge can't make without the bone turned past a person's range is a limb hyperextended: held straight)
    const tw = swingTwist(qmul(qinv(G[joint(skel, J0(skel, up))]), qmul(qaxis(du, a), Gu)), ru).twist;
    if (Math.abs(tw) <= lim.twist * DEG + 0.35) Gu = qmul(qaxis(du, a * w), Gu);
  }
  // the upper bone's twist against its parent held to a person's range
  const pu = joint(skel, J0(skel, up)), lu = qmul(qinv(G[pu]), Gu);
  Gu = qmul(G[pu], held(lu, ru, lim.twist));
  // the lower bone: the upper's frame bent about the hinge as far as the source bends (none if it bends the wrong way)
  const hingeNow = qrot(Gu, hinge0);
  const signed = Math.atan2(dot(cross(du, dm), hingeNow), dot(du, dm));
  const bentBy = Math.max(0, signed);
  // (the body's rest frames are the world's, and its rest elbow is bent (turbo's by 44°): straight on the upper bone
  // is the upper's turn with the rest bend taken out)
  let Gm = qmul(qaxis(hingeNow, bentBy), qmul(Gu, qfromTo(rm, ru)));
  // its turn about its own length as the transfer had it, within range
  const tw = swingTwist(qmul(qinv(Gm), G[m]), rm).twist;
  Gm = qmul(Gm, qaxis(rm, clamp(tw, lim.mid)));
  // (and laid exactly along the source's lower bone where the hinge alone falls short: a bend off the hinge plane, or a
  // nearly straight limb whose plane is noise, up to a few degrees past straight as real elbows and knees go)
  if (signed > -BACK || theta < LOOSE) Gm = aimed(Gm, rm, dm);
  G[u] = Gu; G[m] = Gm;
  G[e] = qmul(Gm, held(endLocal, re, lim.end, lim.endSwing));
}

/** a joint's parent's name */
function J0(skel: Skeleton, name: string): string {
  const j = skel.joints[joint(skel, name)];
  return skel.joints[j.parent].name;
}

const TRUNK = /^(hips|waist|torso|chest)$/, ARM = /^(palm|foreArm)$/;

/**
 * The arms out of the trunk: a body built wider than the source's (turbo's hips and waist against a slim actor's) has
 * a hanging hand or a forearm across the belly in it. Each arm in it is turned about its shoulder, the way the trunk's
 * skin faces where it goes in deepest, by as little as clears it (a few steps, 30° at most). World turns in and out.
 */
/** the trunk's surface points binned in CELL cubes: a point's neighbours are the 27 cells round it, so an arm point
 * more than CELL from the trunk (well clear, or deeper than any arm goes) skips the search */
const CELL = 0.08;
/** how far past straight an elbow or knee may follow its source */
const BACK = 6 * DEG;
/** the most the pelvis is tipped onto the source's */
const TIP = 35 * DEG;
/** how far an arm may sink into the trunk before it is pushed out (cloth and flesh give that much) */
const SKIN = 0.004;
/** the deepest an arm is taken to be in the trunk (deeper readings are points outside, beside the trunk's edge) */
const REACH = 0.05;
/** a bend this small follows the source whatever its plane: a near-straight limb's plane is noise */
const LOOSE = 20 * DEG;
function cells(pts: { at: Vec3; normal: Vec3 }[]) {
  const bins = new Map<string, { at: Vec3; normal: Vec3 }[]>(), key = (a: number, b: number, c: number) => `${a},${b},${c}`;
  for (const p of pts) { const k = key(...(p.at.map((x) => Math.floor(x / CELL)) as Vec3)); (bins.get(k) ?? bins.set(k, []).get(k)!).push(p); }
  return (at: Vec3) => {
    const [i, j, k] = at.map((x) => Math.floor(x / CELL)), out: { at: Vec3; normal: Vec3 }[] = [];
    for (let a = -1; a <= 1; a++) for (let b = -1; b <= 1; b++) for (let c = -1; c <= 1; c++) out.push(...(bins.get(key(i + a, j + b, k + c)) ?? []));
    return out;
  };
}

export function clearOfTrunk(skel: Skeleton, G: Quat[], nj: number): Quat[] {
  const J = skel.joints, out = G.slice();
  const posed = () => pose(skel, { root: [0, 0, 0], turns: out.map((q, i) => (J[i].parent < 0 ? q : qmul(qinv(out[J[i].parent]), q))).slice(0, nj) });
  for (const s of SIDES) {
    const chain = [B.upperarm(s), B.lowerarm(s), B.hand(s)].map((n) => joint(skel, n));
    let turned = 0;
    for (let k = 0; k < 8; k++) {
      const w = posed(), near = cells(surface(skel, w, TRUNK)), arm = surface(skel, w, ARM, s);
      let deep = 0, at: Vec3 | undefined, n: Vec3 | undefined;
      for (const a of arm) {
        let best = Infinity, d = 0, nn: Vec3 = [0, 0, 0];
        for (const x of near(a.at)) { const v = sub(a.at, x.at), dd = dot(v, v); if (dd < best) { best = dd; d = -dot(v, x.normal); nn = x.normal; } }
        // (a point truly in is no further from the skin than it is deep: a far nearest point is the trunk's other side)
        if (d > deep && best < REACH * REACH) { deep = d; at = a.at; n = nn; }
      }
      if (deep < (turned ? 0.004 : SKIN) || !at || !n) break;
      const sh = w.pos[chain[0]], arm0 = sub(at, sh), axis = cross(arm0, n);
      if (len(axis) < 1e-6) break;
      const step = Math.min(30 * DEG - turned, (deep + 0.004) / Math.max(0.1, len(arm0)));
      if (step <= 0) break;
      const q = qaxis(norm(axis), step);
      for (const i of chain) out[i] = qmul(q, out[i]);
      turned += step;
    }
  }
  return out;
}
