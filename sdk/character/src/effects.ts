// Rig effects for raea/character: a state (sad, proud, old, limping on the left) laid over any motion as the
// differences a style's capture makes from the same actor's neutral walk, so a body moves in states nobody captured
// on it, and in mixes of them.
//
//   const old = learnEffect(kid, neutralWalk, oldWalk);           // once, from 100STYLE's Old_FW against Neutral_FW
//   const sadLimp = mixEffects([[sad, 0.6], [limpLeft, 0.8]]);
//   const p2 = applyEffect(kid, p, sadLimp, { base: clipMeans(kid, clip) });
//
// For each joint an effect holds:
//   posture    the turn its mean moves by (the style's mean turn against neutral's), a rotation vector
//   amplitude  how far it moves about its mean, against neutral, along each of the mean's own axes (a gain)
// and for the whole body:
//   rate       how fast it steps, against neutral (the cadence's ratio): a performance plays its frames that much faster
// The hips' height follows from the legs: the body is lowered or raised so its lowest foot stays where it was (a
// bent-kneed style crouches, a bouncy one bounces, and no foot leaves the floor or sinks through it).
// Left and right joints are learned apart, so a limp or a dragged leg is the effect's asymmetry. Effects mix in the
// log: postures add (weighted), gains and rates multiply (as powers), so half of one and half of another is a body
// between them, and a weight of 0 is the motion as it was.
import type { Vec3 } from "raea/3d";
import { captured, joint, pose, qinv, qmul, qnorm, qrot, qyaw, type Pose, type Quat, type Skeleton } from "raea/character";
import { framePose, type Clip } from "./clip";
import { styleMean } from "./style";

export interface RigEffect {
  /** per joint: the mean's turn (a rotation vector, radians) and the gains about the mean (x, y, z of the mean's frame) */
  posture: Vec3[];
  amplitude: Vec3[];
  rate: number;
  /** how fast it travels, against neutral (a walk's speed's ratio): a performance's walks go at that much of their
   * speed unless a step says its own */
  speed?: number;
}

// rotation vectors and quaternions
export function qlog(q: Quat): Vec3 {
  const s = q[3] < 0 ? -1 : 1, x = q[0] * s, y = q[1] * s, z = q[2] * s, w = Math.min(1, q[3] * s);
  const n = Math.hypot(x, y, z);
  if (n < 1e-9) return [2 * x, 2 * y, 2 * z];
  const a = (2 * Math.atan2(n, w)) / n;
  return [x * a, y * a, z * a];
}
export function qexp(v: Vec3): Quat {
  const a = Math.hypot(v[0], v[1], v[2]);
  if (a < 1e-9) return qnorm([v[0] / 2, v[1] / 2, v[2] / 2, 1]);
  const s = Math.sin(a / 2) / a;
  return [v[0] * s, v[1] * s, v[2] * s, Math.cos(a / 2)];
}

const feetOf = new WeakMap<Skeleton, number[]>();
const FEET = (skel: Skeleton) => {
  let f = feetOf.get(skel);
  if (!f) feetOf.set(skel, (f = ["LeftFoot", "RightFoot", "LeftToeBase", "RightToeBase"].map((n) => joint(skel, n))));
  return f;
};

/** the hips' turn without its heading: the body's own pitch and roll */
function hipsLocal(q: Quat): { yaw: number; q: Quat } {
  const f = qrot(q, [0, 0, 1]), yaw = Math.atan2(f[0], f[2]);
  return { yaw, q: qnorm(qmul(qyaw(-yaw), q)) };
}

/** The mean turn of each joint over a clip's frames (the hips' without their heading), and the hips' mean height. */
export interface ClipMeans {
  turns: Quat[];
  height: number;
}
const meansCache = new WeakMap<Clip, ClipMeans>();
export function clipMeans(skel: Skeleton, c: Clip, f0 = 0, f1 = c.frames): ClipMeans {
  const whole = f0 === 0 && f1 === c.frames;
  if (whole && meansCache.has(c)) return meansCache.get(c)!;
  const nj = captured(skel), turns: Quat[] = [];
  const frames: Pose[] = [];
  for (let f = f0; f < f1; f++) frames.push(framePose(c, f));
  for (let i = 0; i < nj; i++) {
    const qs = frames.map((p) => (i === 0 ? hipsLocal(p.turns[0]).q : p.turns[i]));
    // the mean on the sphere: from the first, a few steps of the mean rotation vector about the estimate
    let m = qs[0];
    for (let it = 0; it < 4; it++) {
      const s: Vec3 = [0, 0, 0];
      for (const q of qs) { const v = qlog(qmul(qinv(m), q)); s[0] += v[0]; s[1] += v[1]; s[2] += v[2]; }
      m = qnorm(qmul(m, qexp([s[0] / qs.length, s[1] / qs.length, s[2] / qs.length])));
    }
    turns.push(m);
  }
  const height = frames.reduce((a, p) => a + p.root[1], 0) / frames.length;
  const out = { turns, height };
  if (whole) meansCache.set(c, out);
  return out;
}

/** each joint's spread about its mean along the mean's axes (rms of the rotation vector), and the hips' rise and fall */
function spread(skel: Skeleton, c: Clip, m: ClipMeans, f0: number, f1: number): { axes: Vec3[]; bounce: number } {
  const nj = captured(skel), ss = Array.from({ length: nj }, () => [0, 0, 0] as Vec3);
  let b = 0;
  for (let f = f0; f < f1; f++) {
    const p = framePose(c, f);
    for (let i = 0; i < nj; i++) {
      const q = i === 0 ? hipsLocal(p.turns[0]).q : p.turns[i];
      const v = qlog(qmul(qinv(m.turns[i]), q));
      for (let a = 0; a < 3; a++) ss[i][a] += v[a] * v[a];
    }
    b += (p.root[1] - m.height) ** 2;
  }
  const n = Math.max(1, f1 - f0);
  return { axes: ss.map((s) => s.map((x) => Math.sqrt(x / n)) as Vec3), bounce: Math.sqrt(b / n) };
}

/** A style's effect: how its clip (frames s0 to s1) differs from the same actor's neutral clip (n0 to n1). */
export function learnEffect(skel: Skeleton, neutral: Clip, styled: Clip, o: { n?: [number, number]; s?: [number, number] } = {}): RigEffect {
  const [n0, n1] = o.n ?? [0, neutral.frames], [s0, s1] = o.s ?? [0, styled.frames];
  const mn = clipMeans(skel, neutral, n0, n1), ms = clipMeans(skel, styled, s0, s1);
  const sn = spread(skel, neutral, mn, n0, n1), ss = spread(skel, styled, ms, s0, s1);
  const clamp = (x: number) => Math.min(4, Math.max(0.15, x));
  // a joint that hardly moves in neutral (under a degree) keeps its amplitude: a ratio of noise means nothing; and a
  // gain is held within half to double (a few steps' spread is noisy, and past that the noise is what gets amplified:
  // on held-out halves of 100STYLE's walks, wider gains made the walks less like the real ones, not more)
  const gain = (a: number, b: number) => (a < 0.017 ? 1 : Math.min(2, Math.max(0.5, b / a)));
  const vn = styleMean(skel, neutral, n0, n1), vs = styleMean(skel, styled, s0, s1), cn = vn.cadence, cs = vs.cadence;
  const e: RigEffect = {
    posture: mn.turns.map((q, i) => qlog(qmul(ms.turns[i], qinv(q)))),
    amplitude: sn.axes.map((a, i) => [gain(a[0], ss.axes[i][0]), gain(a[1], ss.axes[i][1]), gain(a[2], ss.axes[i][2])] as Vec3),
    rate: cn > 0.2 && cs > 0.2 ? clamp(cs / cn) : 1,
    speed: vn.speed > 0.1 ? clamp(vs.speed / vn.speed) : 1,
  };
  // each arm's swing, calibrated: the joints' gains don't come out as the hand's swing (a posture that rolls the trunk
  // or turns the arm changes how much of a joint's swing is the hand's, fore and aft), so neutral with the effect is
  // measured and each arm's gains scaled until its hand swings as the style's does (a limp's arms swing unevenly)
  const arms = [["LeftShoulder", "LeftArm", "LeftForeArm", "LeftHand"], ["RightShoulder", "RightArm", "RightForeArm", "RightHand"]].map((s) => s.map((n) => joint(skel, n)));
  // and likewise the hips' and thighs' gains for the feet's sway under the hips (which their gains overshoot)
  const legs = ["Hips", "LeftUpLeg", "RightUpLeg"].map((n) => joint(skel, n)).filter((i) => i >= 0);
  const want = [vs.armSwingL, vs.armSwingR], m1 = Math.min(n1, n0 + 30 * neutral.fps);
  for (let it = 0; it < 3; it++) {
    const got = styleMean(skel, mapped(skel, neutral, n0, m1, (p) => applyEffect(skel, p, e, { base: mn })));
    if (got.sway > 0.01 && vs.sway > 0.01) {
      const k = Math.min(1.5, Math.max(0.67, vs.sway / got.sway));
      for (const i of legs) e.amplitude[i] = e.amplitude[i].map((x) => Math.min(2.5, Math.max(0.4, x * k))) as Vec3;
    }
    [got.armSwingL, got.armSwingR].forEach((g, s) => {
      if (g < 0.02 || want[s] < 0.02) return;
      const k = Math.min(1.5, Math.max(0.67, want[s] / g));
      for (const i of arms[s]) e.amplitude[i] = e.amplitude[i].map((x) => Math.min(2.5, Math.max(0.4, x * k))) as Vec3;
    });
  }
  return e;
}

/** frames f0 to f1 of a clip, each pose changed by `f` */
function mapped(skel: Skeleton, c: Clip, f0: number, f1: number, f: (p: Pose) => Pose): Clip {
  // (a clip's frames turn the capture's joints only: the fingers' own come after them)
  const nj = captured(skel), n = f1 - f0, root = new Float32Array(n * 3), turns = new Float32Array(n * nj * 4);
  for (let k = 0; k < n; k++) { const p = f(framePose(c, f0 + k)); root.set(p.root, k * 3); p.turns.slice(0, nj).forEach((q, i) => turns.set(q, (k * nj + i) * 4)); }
  return { name: c.name, fps: c.fps, frames: n, root, turns };
}

/** Effects mixed by weight: postures added, gains and rates multiplied as powers. */
export function mixEffects(parts: [RigEffect, number][]): RigEffect {
  const nj = parts[0][0].posture.length;
  const posture = Array.from({ length: nj }, () => [0, 0, 0] as Vec3), amplitude = Array.from({ length: nj }, () => [1, 1, 1] as Vec3);
  let rate = 1, speed = 1;
  for (const [e, w] of parts) {
    for (let i = 0; i < nj; i++) for (let a = 0; a < 3; a++) {
      posture[i][a] += e.posture[i][a] * w;
      amplitude[i][a] *= e.amplitude[i][a] ** w;
    }
    rate *= e.rate ** w;
    speed *= (e.speed ?? 1) ** w;
  }
  return { posture, amplitude, rate, speed };
}

/** A pose with an effect laid over it (`weight` of it, 1 by default): each joint moves about `base`'s mean (the
 * clip the pose comes from) as much more or less as the effect says, about a mean turned by its posture. */
export function applyEffect(skel: Skeleton, p: Pose, e: RigEffect, o: { base: ClipMeans; weight?: number }): Pose {
  const w = o.weight ?? 1;
  if (w === 0) return p;
  const turns = p.turns.map((q, i) => {
    let yaw = 0, local = q;
    if (i === 0) ({ yaw, q: local } = hipsLocal(q));
    const m = o.base.turns[i], v = qlog(qmul(qinv(m), local)), g = e.amplitude[i], d = e.posture[i];
    const moved = qmul(qmul(qexp([d[0] * w, d[1] * w, d[2] * w]), m), qexp([v[0] * g[0] ** w, v[1] * g[1] ** w, v[2] * g[2] ** w]));
    return qnorm(i === 0 ? qmul(qyaw(yaw), moved) : moved);
  });
  // grounded: the lowest of the feet where it was
  const low = (q: Pose) => { const w2 = pose(skel, q); return Math.min(...FEET(skel).map((i) => w2.pos[i][1])); };
  const out: Pose = { root: p.root, turns };
  return { root: [p.root[0], p.root[1] + low(p) - low(out), p.root[2]], turns };
}

const SWAP = (n: string) => n.startsWith("Left") ? "Right" + n.slice(4) : n.startsWith("Right") ? "Left" + n.slice(5) : /^L[A-Z]/.test(n) ? "R" + n.slice(1) : /^R[A-Z]/.test(n) ? "L" + n.slice(1) : n;

/** An effect seen in a mirror: left and right swapped (a limp on the left one on the right), for mirrored motion. */
export function mirrorEffect(skel: Skeleton, e: RigEffect): RigEffect {
  const other = skel.joints.slice(0, e.posture.length).map((j) => joint(skel, SWAP(j.name)));
  return {
    posture: other.map((o) => { const v = e.posture[o]; return [v[0], -v[1], -v[2]] as Vec3; }),
    amplitude: other.map((o) => [...e.amplitude[o]] as Vec3),
    rate: e.rate,
    speed: e.speed,
  };
}

/** States by name, as mixes of 100STYLE's styles: a style's own name works too (Old, LimpLeft, Zombie, ...). These
 * are the nearest captured styles, not measurements of the state itself: "tired" is a guess at a mix. */
export const STATES: Record<string, Record<string, number>> = {
  sad: { Depressed: 1 },
  happy: { Elated: 1 },
  angry: { Angry: 1 },
  proud: { Proud: 1 },
  old: { Old: 1 },
  tired: { Depressed: 0.5, Old: 0.35 },
  drunk: { Drunk: 1 },
  stiff: { Stiff: 1 },
  // crouched: CMU subject 136's crouched walk (100STYLE's Crouched is only bent: its hips drop 4%, a real crouch's 13-16%)
  crouched: { CrouchWalk: 1 },
  scared: { ShieldedLeft: 0.5, ShieldedRight: 0.5, CrouchWalk: 0.3 },
  "limp-left": { LimpLeft: 1 },
  "limp-right": { LimpRight: 1 },
  "drag-left-leg": { DragLeftLeg: 1 },
  "drag-right-leg": { DragRightLeg: 1 },
  "walking-stick": { WalkingStickRight: 1 },
};
