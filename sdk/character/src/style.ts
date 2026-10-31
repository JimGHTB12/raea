// Style vectors for raea: how a stretch of motion moves, as about twenty measurements a person can read (how
// fast, how far the arms swing, how the trunk leans, how the head hangs, how even the two sides are), so a motion
// library can be sorted and searched by state (tired, proud, limping) as well as by where the body is going.
//
//   const v = styleOf(kid, clip, 0, 30);            // the clip's first second
//   v.speed, v.trunkPitch, v.asymStance, ...
//   const table = styleTable(kid, clips, { labels }); // every clip, a window a second long every half second
//   nearestStyles(table, { trunkPitch: 25, headPitch: 30, speed: 0.6 }); // the windows that move most like that
//
// Every measurement is in the body's own ground frame (x its left, z ahead), in metres, degrees, seconds or ratios,
// and is the same for a clip and its mirror image except the signed ones (trunkRoll, asym*), which flip.
import type { Vec3 } from "raea/3d";
import { captured, joint, pose, qrot, sub, type Posed, type Skeleton } from "raea/character";
import { framePose, type Clip } from "./clip";

/** the measurements, in order, with what each one is */
export const STYLE = {
  speed: "how fast the hips travel over the ground, m/s",
  turn: "how fast the body turns, degrees a second",
  cadence: "steps a second",
  stride: "metres a step",
  stepWidth: "the feet's mean distance apart sideways, m",
  armSwingL: "how far the left hand swings fore and aft, m",
  armSwingR: "how far the right hand swings fore and aft, m",
  handHeight: "the hands' mean height over the hips, m (negative: below)",
  handSpread: "the hands' mean distance apart sideways, m",
  elbowFlex: "how bent the elbows are, degrees (0 straight)",
  shoulderRaise: "the shoulders' height over rest against the neck, m (positive: hunched up)",
  trunkPitch: "the trunk's lean forward, degrees (negative: back)",
  trunkRoll: "the trunk's lean to the body's left, degrees",
  headPitch: "the face's tilt down, degrees (negative: up)",
  kneeBend: "how bent the knees are on average, degrees",
  kneeLift: "how high the knees come over their rest height, m",
  crouch: "the hips' height as a share of their rest height",
  bounce: "how far the hips rise and fall, m",
  sway: "how far the hips sway sideways over the feet, m",
  asymSwing: "the arms' swing, left against right: (L - R) / (L + R)",
  asymStance: "time on each foot, left against right: (L - R) / (L + R)",
  energy: "how fast the joints turn, mean degrees a second",
  jerk: "how jerky the hands are, log10 m/s^3",
} as const;
export type StyleKey = keyof typeof STYLE;
export const STYLE_KEYS = Object.keys(STYLE) as StyleKey[];
export type StyleVector = Record<StyleKey, number>;

const D = 180 / Math.PI;
const wrap = (a: number) => Math.atan2(Math.sin(a), Math.cos(a));
const angleAt = (a: Vec3, b: Vec3, c: Vec3) => {
  const u = sub(a, b), v = sub(c, b);
  const d = (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / (Math.hypot(...u) * Math.hypot(...v) || 1);
  return Math.acos(Math.max(-1, Math.min(1, d))) * D;
};

/** The style of a clip's frames f0 to f1 (exclusive). */
export function styleOf(skel: Skeleton, c: Clip, f0: number, f1: number): StyleVector {
  const J = (n: string) => joint(skel, n);
  const [hips, neck, head, arm, fore, hand, up, leg, foot, toe] = [J("Hips"), J("Neck"), J("Head"), ["LeftArm", "RightArm"].map(J), ["LeftForeArm", "RightForeArm"].map(J),
    ["LeftHand", "RightHand"].map(J), ["LeftUpLeg", "RightUpLeg"].map(J), ["LeftLeg", "RightLeg"].map(J), ["LeftFoot", "RightFoot"].map(J), ["LeftToeBase", "RightToeBase"].map(J)];
  const R = skel.joints.map((j) => j.head);
  const restHips = R[hips][1], restKnee = R[leg[0]][1], restShoulder = (R[arm[0]][1] + R[arm[1]][1]) / 2 - R[neck][1];
  const ankleRest = R[foot[0]][1], toeRest = R[toe[0]][1];
  f0 = Math.max(0, f0); f1 = Math.min(c.frames, f1);
  const n = f1 - f0, dt = 1 / c.fps, T = Math.max(1, n - 1) * dt;
  const W: Posed[] = [], P = [];
  for (let f = f0; f < f1; f++) { const p = framePose(c, f); P.push(p); W.push(pose(skel, p)); }
  // each frame's ground frame: the hips over the floor, facing the way the hips face
  const yaw = W.map((w) => { const v = qrot(w.rot[hips], [0, 0, 1]); return Math.atan2(v[0], v[2]); });
  const body = (k: number, p: Vec3): Vec3 => {
    const o = W[k].pos[hips], d = [p[0] - o[0], p[1], p[2] - o[2]], s = Math.sin(-yaw[k]), co = Math.cos(-yaw[k]);
    return [co * d[0] + s * d[2], d[1], -s * d[0] + co * d[2]];
  };
  const bodyDir = (k: number, d: Vec3): Vec3 => { const s = Math.sin(-yaw[k]), co = Math.cos(-yaw[k]); return [co * d[0] + s * d[2], d[1], -s * d[0] + co * d[2]]; };
  const mean = (f: (k: number) => number) => { let s = 0; for (let k = 0; k < n; k++) s += f(k); return s / n; };
  const range = (f: (k: number) => number) => { let a = Infinity, b = -Infinity; for (let k = 0; k < n; k++) { const v = f(k); a = Math.min(a, v); b = Math.max(b, v); } return b - a; };
  // contact: a foot stands while its ankle or its ball is low and still (as the matcher says)
  const stand = [0, 1].map((s) => W.map((w, k) => {
    const a = Math.max(0, k - 1), b = Math.min(n - 1, k + 1), sp = (i: number) => Math.hypot(...sub(W[b].pos[i], W[a].pos[i])) / (Math.max(1, b - a) * dt);
    return (w.pos[foot[s]][1] < ankleRest + 0.03 && sp(foot[s]) < 0.25) || (w.pos[toe[s]][1] < toeRest + 0.025 && sp(toe[s]) < 0.25);
  }));
  // a step each time the feet pass each other (fore and aft, in the body's frame), more than 4 cm apart either side:
  // contacts flicker on a sliding foot, the feet's order doesn't
  let steps = 0, side = 0;
  for (let k = 0; k < n; k++) {
    const d = body(k, W[k].pos[foot[0]])[2] - body(k, W[k].pos[foot[1]])[2];
    const sd = d > 0.04 ? 1 : d < -0.04 ? -1 : 0;
    if (sd && side && sd !== side) steps++;
    if (sd) side = sd;
  }
  const travel = Math.hypot(W[n - 1].pos[hips][0] - W[0].pos[hips][0], W[n - 1].pos[hips][2] - W[0].pos[hips][2]);
  let turned = 0;
  for (let k = 1; k < n; k++) turned += Math.abs(wrap(yaw[k] - yaw[k - 1]));
  const speed = travel / T, cadence = steps / T;
  const swing = [0, 1].map((s) => range((k) => body(k, W[k].pos[hand[s]])[2]));
  const onFoot = [0, 1].map((s) => stand[s].filter(Boolean).length);
  // the joints' turning: each joint's local turn, frame to frame
  let spin = 0;
  for (let k = 1; k < n; k++) for (let i = 1; i < captured(skel); i++) {
    const a = P[k - 1].turns[i], b = P[k].turns[i];
    spin += 2 * Math.acos(Math.min(1, Math.abs(a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]))) * D / dt;
  }
  let jerk = 0, jn = 0;
  for (let k = 3; k < n; k++) for (const s of [0, 1]) {
    const p = (j: number) => W[j].pos[hand[s]];
    const d = [0, 1, 2].map((a) => (p(k)[a] - 3 * p(k - 1)[a] + 3 * p(k - 2)[a] - p(k - 3)[a]) / dt ** 3);
    jerk += Math.hypot(...d); jn++;
  }
  const trunk = (k: number) => bodyDir(k, sub(W[k].pos[neck], W[k].pos[hips]));
  const face = (k: number) => bodyDir(k, qrot(W[k].rot[head], [0, 0, 1]));
  const ankles = (k: number) => [0, 1].map((s) => body(k, W[k].pos[foot[s]]));
  return {
    speed,
    turn: (turned / T) * D,
    cadence,
    stride: cadence > 0 ? speed / cadence : 0,
    stepWidth: mean((k) => { const a = ankles(k); return Math.abs(a[0][0] - a[1][0]); }),
    armSwingL: swing[0],
    armSwingR: swing[1],
    handHeight: mean((k) => (W[k].pos[hand[0]][1] + W[k].pos[hand[1]][1]) / 2 - W[k].pos[hips][1]),
    handSpread: mean((k) => Math.abs(body(k, W[k].pos[hand[0]])[0] - body(k, W[k].pos[hand[1]])[0])),
    elbowFlex: mean((k) => [0, 1].reduce((a, s) => a + (180 - angleAt(W[k].pos[arm[s]], W[k].pos[fore[s]], W[k].pos[hand[s]])) / 2, 0)),
    shoulderRaise: mean((k) => (W[k].pos[arm[0]][1] + W[k].pos[arm[1]][1]) / 2 - W[k].pos[neck][1]) - restShoulder,
    trunkPitch: mean((k) => { const t = trunk(k); return Math.atan2(t[2], t[1]) * D; }),
    trunkRoll: mean((k) => { const t = trunk(k); return Math.atan2(t[0], t[1]) * D; }),
    headPitch: mean((k) => { const f = face(k); return Math.atan2(-f[1], Math.hypot(f[0], f[2])) * D; }),
    kneeBend: mean((k) => [0, 1].reduce((a, s) => a + (180 - angleAt(W[k].pos[up[s]], W[k].pos[leg[s]], W[k].pos[foot[s]])) / 2, 0)),
    kneeLift: Math.max(...[0, 1].map((s) => Math.max(...W.map((w) => w.pos[leg[s]][1])) - restKnee)),
    crouch: mean((k) => W[k].pos[hips][1]) / restHips,
    bounce: range((k) => W[k].pos[hips][1]),
    sway: range((k) => { const a = ankles(k); return -(a[0][0] + a[1][0]) / 2; }),
    asymSwing: (swing[0] - swing[1]) / (swing[0] + swing[1] || 1),
    asymStance: (onFoot[0] - onFoot[1]) / (onFoot[0] + onFoot[1] || 1),
    energy: n > 1 ? spin / ((n - 1) * (captured(skel) - 1)) : 0,
    jerk: Math.log10(jerk / Math.max(1, jn) + 1e-6),
  };
}

/** A clip's style over frames f0 to f1 as the mean of its seconds (every half second): a whole take's own, since one
 * long window's speed is only how far it ended from where it began. */
export function styleMean(skel: Skeleton, c: Clip, f0 = 0, f1 = c.frames): StyleVector {
  const w = Math.round(c.fps), h = Math.max(1, Math.round(c.fps / 2)), rows: StyleVector[] = [];
  for (let f = f0; f + w <= f1; f += h) rows.push(styleOf(skel, c, f, f + w));
  if (!rows.length) return styleOf(skel, c, f0, f1);
  const out = {} as StyleVector;
  for (const k of STYLE_KEYS) out[k] = rows.reduce((a, r) => a + r[k], 0) / rows.length;
  return out;
}

export interface StyleRow {
  clip: string;
  /** the window's first frame and its end */
  f0: number;
  f1: number;
  /** what the clip is: its style's name (100STYLE) or its description (CMU) */
  label: string;
  v: StyleVector;
}
export interface StyleTable {
  rows: StyleRow[];
  /** each measurement's mean and spread over the rows, for comparing them on one scale */
  mean: StyleVector;
  sd: StyleVector;
}

/** Every clip's style, a window `window` seconds long every `hop` seconds. */
export function styleTable(skel: Skeleton, clips: Map<string, Clip>, o: { window?: number; hop?: number; labels?: Record<string, string> } = {}): StyleTable {
  const rows: StyleRow[] = [];
  for (const [name, c] of clips) {
    const w = Math.round((o.window ?? 1) * c.fps), h = Math.max(1, Math.round((o.hop ?? 0.5) * c.fps));
    for (let f = 0; f + w <= Math.max(w, c.frames); f += h) rows.push({ clip: name, f0: f, f1: Math.min(c.frames, f + w), label: o.labels?.[name] ?? name, v: styleOf(skel, c, f, f + w) });
  }
  return withStats(rows);
}

export function withStats(rows: StyleRow[]): StyleTable {
  const mean = {} as StyleVector, sd = {} as StyleVector;
  for (const k of STYLE_KEYS) {
    const xs = rows.map((r) => r.v[k]), m = xs.reduce((a, b) => a + b, 0) / (xs.length || 1);
    mean[k] = m;
    sd[k] = Math.sqrt(xs.reduce((a, b) => a + (b - m) ** 2, 0) / (xs.length || 1)) || 1;
  }
  return { rows, mean, sd };
}

/** The rows that move most like `want` (any of the measurements, each on the table's own scale), nearest first. */
export function nearestStyles(t: StyleTable, want: Partial<StyleVector>, n = 10): (StyleRow & { d: number })[] {
  const keys = Object.keys(want) as StyleKey[];
  return t.rows
    .map((r) => ({ ...r, d: Math.sqrt(keys.reduce((a, k) => a + ((r.v[k] - want[k]!) / t.sd[k]) ** 2, 0)) }))
    .sort((a, b) => a.d - b.d)
    .slice(0, n);
}

/** Each label's mean style (its rows averaged). */
export function styleByLabel(t: StyleTable): Map<string, StyleVector> {
  const by = new Map<string, StyleRow[]>();
  for (const r of t.rows) by.set(r.label, [...(by.get(r.label) ?? []), r]);
  const out = new Map<string, StyleVector>();
  for (const [l, rs] of by) {
    const v = {} as StyleVector;
    for (const k of STYLE_KEYS) v[k] = rs.reduce((a, r) => a + r.v[k], 0) / rs.length;
    out.set(l, v);
  }
  return out;
}
