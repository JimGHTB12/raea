// steer: how perform steers the body. The pace a walk in a state goes at, the library frame a performance starts from,
// the pull that keeps the body on the guide; and the parts of the body a switch's fade treats apart, and each joint's offset in it.
import { qexp, type RigEffect } from "../effects";
import type { Vec3 } from "raea/3d";
import type { Quat, Skeleton } from "raea/character";
import { FUTURE, MotionLibrary, NF, Performance, PerformanceSpec, paceOf, stateMix, wrap } from "./all";

/** the spec with each walk in a state at the state's pace (an old man's is a third of neutral's), unless the step says
 * its own */
export function pacedSpec(lib: MotionLibrary, spec: PerformanceSpec, effects?: Record<string, RigEffect>): PerformanceSpec {
  return effects ? { ...spec, steps: spec.steps.map((st) => {
    const s = ("state" in st && st.state) || spec.state, mix = s ? stateMix(s) : null;
    if (!("walk" in st) || st.speed !== undefined || !mix) return st;
    // the heaviest style the library has frames of sets the pace (its walks' own); the others scale it by their
    // effects' (a library with none of them: its Neutral's pace, or the default walk's, scaled)
    const own = Object.entries(mix).filter(([l]) => paceOf(lib, l) > 0).sort((x, y) => y[1] - x[1])[0];
    let k = own ? paceOf(lib, own[0]) : paceOf(lib, "Neutral") || 1;
    for (const [l, w] of Object.entries(mix)) if (!own || l !== own[0]) k *= (effects[l]?.speed ?? 1) ** w;
    return { ...st, speed: Math.max(0.2, k) };
  }) } : spec;
}

/** where a performance starts: the frame nearest standing still facing ahead (`nearest0` finds it from the features),
 * or a walk entered at speed mid-stride, the frame whose way on is the guide's (`query` asks it) */
export function startFrame(lib: MotionLibrary, spec: PerformanceSpec, nearest0: (s: Float32Array) => number, query: (k: number, t: number) => Float32Array): number {
    const s = new Float32Array(NF);
    for (let d = 0; d < 15; d++) s[d] = lib.mean[d];
    for (let d = 6; d < 15; d++) s[d] = 0;
    let c = 15;
    for (const _ of FUTURE) { s[c++] = 0; s[c++] = 0; }
    for (const _ of FUTURE) { s[c++] = 0; s[c++] = 1; }
    const k0 = nearest0(s);
    // (a walk entered at speed starts mid-stride: the frame whose way on is the guide's, its pose any)
    const first = spec.steps[0];
    if (!(first && "walk" in first && (first.enter ?? 0) > 0.5)) return k0;
    const g = query(k0, 0), F = lib.features;
    let best = k0, bd = Infinity;
    for (let k2 = 0; k2 < lib.size; k2++) {
      if (lib.jump[k2] !== 1) continue;
      let dd = 0;
      for (let d = 15; d < NF; d++) dd += (F[k2 * NF + d] - (g[d] - lib.mean[d]) * lib.scale[d]) ** 2;
      if (dd < bd) { bd = dd; best = k2; }
    }
    return best;
}

/** how the guide pulls the body, and how hard */
export interface Pull { adjust: boolean; aPos: number; aRot: number; adjRatio: number; clampDist: number; pull: number }

/** the body's place (x, z, heading) pulled toward the guide's frame `g`, against where it shows (`w`: u of the way on
 * to the next library frame; at a rate other than 1 the ground frame moves in whole library frames, and pulling that
 * against the guide shakes the body); `from` its place before this frame's library steps */
export function pullToGuide(at: { x: number; z: number; yaw: number }, from: { x: number; z: number; yaw: number }, g: Performance["guide"][number], w: { dx: number; dz: number }, P: Pull) {
  let { x, z, yaw } = at;
  const { adjust, aPos, aRot, adjRatio, clampDist, pull } = P, px0 = from.x, pz0 = from.z, pyaw0 = from.yaw;
  // (a wait with slack pulled only for how far it is past it: a stop at a thing taken in passing overran its spot
  // 8 cm, was pulled back over standing feet, and at the start after them the feet were 10 cm from the clip's)
  let ex = g.x - x - w.dx, ez = g.z - z - w.dz;
  if (g.slack) { const e = Math.hypot(ex, ez), k2 = e > g.slack ? 1 - g.slack / e : 0; ex *= k2; ez *= k2; }
  if (adjust) {
    // Holden's velocity-limited adjustment (docs/locomotion.md 1): a damper toward the guide, but never more in a
    // frame than a share of what the clip itself moves the body (still, nothing pulls; a plant-turn turns only as
    // the clip turns), then clamped to a most distance from it
    const moved = Math.hypot(x - px0, z - pz0), turned = Math.abs(wrap(yaw - pyaw0));
    let ax = ex * aPos, az = ez * aPos;
    const al = Math.hypot(ax, az), cap = adjRatio * moved;
    if (al > cap) { ax *= cap / al; az *= cap / al; }
    x += ax; z += az;
    let ay = wrap(g.yaw - yaw) * aRot;
    const capY = adjRatio * turned;
    if (Math.abs(ay) > capY) ay = Math.sign(ay) * capY;
    yaw += ay;
    const rx = g.x - x - w.dx, rz = g.z - z - w.dz, rl = Math.hypot(rx, rz), lim = Math.max(clampDist, g.slack ?? 0);
    if (rl > lim) { x += rx * (1 - lim / rl); z += rz * (1 - lim / rl); }
  } else {
    x += ex * pull;
    z += ez * pull;
    yaw += wrap(g.yaw - yaw) * pull;
  }
  return { x, z, yaw };
}

/** the skeleton's parts as a switch fades them: the thighs, whether each joint is above the legs, and the trunk (the hips,
 * the spine, the neck and the head, the arms' clavicles not) */
export function fadeParts(skel: Skeleton) {
  // (the legs: the pelvis's children that lead down to a thigh)
  const legOf = (i: number): boolean => i > 0 && (/^thigh_/.test(skel.joints[i].name) || skel.joints.some((j, c) => j.parent === i && legOf(c)));
  const thighs = skel.joints.flatMap((j, i) => (j.parent === 0 && legOf(i) ? [i] : []));
  const upper = skel.joints.map((_, i) => { for (let a = i; a > 0; a = skel.joints[a].parent) if (thighs.includes(a)) return false; return true; });
  // (the trunk: the hips, the spine, the neck and the head, the arms' clavicles not)
  const trunk = skel.joints.map((j, i) => i === 0 || /^(spine|neck|head)/.test(j.name));
  return { thighs, upper, trunk };
}

/** joint i's blend offset at a Hermite fade's weights (the offset's, its velocity's) */
export const offAt = (O: { x: Vec3[]; v: Vec3[] }, i: number, [a, b]: [number, number]): Quat =>
  qexp([O.x[i][0] * a + O.v[i][0] * b, O.x[i][1] * a + O.v[i][1] * b, O.x[i][2] * a + O.v[i][2] * b]);
