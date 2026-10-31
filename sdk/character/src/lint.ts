// A motion lint for raea/matching: every joint of a performance checked frame by frame for movement no capture
// in the library makes, each finding traced to the stage that made it and what happened about then.
//
//   const perf = perform(lib, spec, { trace: true });
//   const lint = motionLint(lib, perf);
//   console.log(lintReport(lint));
//   frame 212  LeftLeg   jerk 14.2°/f² (library's most 5.1)  made by: plant  about then: L lock @211, hips drop 9 mm/f
//
// What counts as sudden is the library's own: a joint's turn speed and its change of speed (both degrees a frame,
// about every axis) past the 99.5th percentile of any library frame's, scaled by the rate the performance plays at.
// A knee bent more than 15° past the blended clip's while its foot stands is a buckle (planting folding the leg). Each
// finding is measured again at every stage of the performance (the clip's frame as chosen, blended in, the feet
// planted, the elbows relaxed, the gaze), and the first stage where it shows is the one that made it.
import type { Quat } from "raea/character";
import { captured, joint, qinv, qmul, qrot, qyaw, type Pose, type Skeleton } from "raea/character";
import { framePose } from "./clip";
import { qlog } from "./effects";
import type { MotionLibrary, Performance } from "./matching";

export type Stage = "clip" | "blend" | "plant" | "elbows" | "gaze";
const STAGES: Stage[] = ["clip", "blend", "plant", "elbows", "gaze"];

export interface Finding {
  /** first and peak frames */
  frame: number;
  peak: number;
  frames: number;
  joint: string;
  kind: "speed" | "jerk" | "buckle";
  /** the peak's value (degrees a frame, a frame², or degrees past the clip's knee) and what the library allows */
  value: number;
  limit: number;
  /** the first stage that shows it */
  stage: Stage;
  /** what happened within three frames of it */
  causes: string[];
}

export interface Lint {
  findings: Finding[];
  frames: number;
}

/** a joint's turn in its parent's frame, the hips' without their heading */
function localTurn(p: Pose, i: number): Quat {
  const q = p.turns[i];
  if (i !== 0) return q;
  const f = qrot(q, [0, 0, 1]);
  return qmul(qyaw(-Math.atan2(f[0], f[2])), q);
}
/** a joint's turn from one frame to the next, as a rotation vector (radians) */
const step = (a: Pose, b: Pose, i: number) => qlog(qmul(qinv(localTurn(a, i)), localTurn(b, i)));
const DEG = 180 / Math.PI;
const watched = (name: string) => !/Finger|Thumb/.test(name);

const refs = new WeakMap<MotionLibrary, { speed: number[]; jerk: number[] }>();
/** each joint's 99.5th percentile turn speed and change of speed over the library's frames (degrees a frame) */
function libraryLimits(lib: MotionLibrary): { speed: number[]; jerk: number[] } {
  const hit = refs.get(lib);
  if (hit) return hit;
  const J = captured(lib.skel), sp: number[][] = J ? Array.from({ length: J }, () => []) : [], jk: number[][] = Array.from({ length: J }, () => []);
  for (const r of lib.clips) {
    if (r.mirrored) continue;
    let a = framePose(r.clip, 0), b = framePose(r.clip, 1);
    for (let f = 2; f < r.clip.frames; f++) {
      const c = framePose(r.clip, f);
      for (let i = 0; i < J; i++) {
        const v1 = step(a, b, i), v2 = step(b, c, i);
        sp[i].push(Math.hypot(...v2) * DEG);
        jk[i].push(Math.hypot(v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]) * DEG);
      }
      a = b; b = c;
    }
  }
  const p995 = (x: number[]) => { if (!x.length) return 0; const s = x.slice().sort((m, n) => m - n); return s[Math.min(s.length - 1, Math.floor(s.length * 0.995))]; };
  const out = { speed: sp.map(p995), jerk: jk.map(p995) };
  refs.set(lib, out);
  return out;
}

/** a knee's bend, degrees: the shin's turn at the knee (at rest the leg is straight) */
function kneeBend(skel: Skeleton, p: Pose, side: 0 | 1): number {
  return Math.hypot(...qlog(localTurn(p, joint(skel, side ? "RightLeg" : "LeftLeg")))) * DEG;
}

/** Lint a performance made with `trace: true` (see the head of the file). */
export function motionLint(lib: MotionLibrary, perf: Performance, o: { margin?: number } = {}): Lint {
  const tr = perf.trace;
  if (!tr) throw new Error("motionLint: perform with { trace: true }");
  const skel = lib.skel, lim = libraryLimits(lib), m = o.margin ?? 1.25, n = perf.frames;
  const names = skel.joints.slice(0, captured(skel)).map((j) => j.name);
  const sideOf = (name: string): 0 | 1 | -1 => (/^(Left|L[A-Z])/.test(name) ? 0 : /^(Right|R[A-Z])/.test(name) ? 1 : -1);
  const metric = (st: Stage, t: number, i: number, kind: "speed" | "jerk") => {
    const P = tr.stages[st];
    if (t < 2 || t >= P.length) return 0;
    const v2 = step(P[t - 1], P[t], i);
    if (kind === "speed") return Math.hypot(...v2) * DEG;
    const v1 = step(P[t - 2], P[t - 1], i);
    return Math.hypot(v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]) * DEG;
  };
  const limitAt = (t: number, i: number, kind: "speed" | "jerk") => {
    const r = Math.max(1, tr.rate[t] ?? 1);
    return kind === "speed" ? Math.max(2, lim.speed[i] * r * m) : Math.max(1.5, lim.jerk[i] * r * r * m * 1.2);
  };
  const causes = (t: number, name: string): string[] => {
    const out: string[] = [], side = sideOf(name), leg = /UpLeg|Leg$|Foot|Toe|Hips/.test(name);
    for (const s of tr.switches) if (Math.abs(s - t) <= 3) out.push(`clip switch @${s}`);
    if (tr.plant) {
      for (const e of tr.plant.events) if (Math.abs(e.t - t) <= 3 && leg && (side < 0 || side === e.foot)) out.push(`${e.foot ? "R" : "L"} ${e.what} @${e.t}`);
      const d = tr.plant.drop, dd = Math.abs((d[t] ?? 0) - (d[t - 1] ?? 0));
      if (dd > 0.004) out.push(`hips drop ${Math.round(dd * 1000)} mm/f (${Math.round((d[t] ?? 0) * 1000)} mm)`);
      for (const f of [0, 1] as const) {
        const l = tr.plant.lift[f], dl = Math.abs((l[t] ?? 0) - (l[t - 1] ?? 0));
        if (dl > 0.05 && leg && (side < 0 || side === f)) out.push(`${f ? "R" : "L"} heel lift ${Math.round(dl * DEG)}°/f`);
      }
    }
    const r0 = tr.rate[t - 1] ?? 1, r1 = tr.rate[t] ?? 1;
    if (Math.abs(r1 - r0) / r0 > 0.03) out.push(`rate ${r0.toFixed(2)}→${r1.toFixed(2)}`);
    return out;
  };
  const raw: Finding[] = [];
  for (let i = 0; i < names.length; i++) {
    if (!watched(names[i])) continue;
    for (const kind of ["speed", "jerk"] as const) {
      for (let t = 2; t < n; t++) {
        const lmt = limitAt(t, i, kind), v = metric("gaze", t, i, kind);
        if (v <= lmt) continue;
        const stage = STAGES.find((st) => metric(st, t, i, kind) > lmt) ?? "gaze";
        raw.push({ frame: t, peak: t, frames: 1, joint: names[i], kind, value: v, limit: lmt, stage, causes: causes(t, names[i]) });
      }
    }
  }
  // buckles: a standing knee folded well past the clip's own
  if (tr.plant) for (const s of [0, 1] as const) {
    const ki = joint(skel, s ? "RightLeg" : "LeftLeg");
    for (let t = 0; t < n; t++) {
      if (tr.plant.state[s][t] === "swing") continue;
      const over = kneeBend(skel, tr.stages.gaze[t], s) - kneeBend(skel, tr.stages.blend[t], s);
      if (over > 15) raw.push({ frame: t, peak: t, frames: 1, joint: names[ki], kind: "buckle", value: over, limit: 15, stage: "plant", causes: causes(t, names[ki]) });
    }
  }
  // runs of the same joint, kind and stage within two frames, as one finding at its peak
  raw.sort((a, b) => a.joint.localeCompare(b.joint) || a.kind.localeCompare(b.kind) || a.frame - b.frame);
  const findings: Finding[] = [];
  for (const f of raw) {
    const last = findings[findings.length - 1];
    if (last && last.joint === f.joint && last.kind === f.kind && last.stage === f.stage && f.frame - (last.frame + last.frames - 1) <= 2) {
      last.frames = f.frame - last.frame + 1;
      if (f.value / f.limit > last.value / last.limit) { last.value = f.value; last.limit = f.limit; last.peak = f.frame; }
      for (const c of f.causes) if (!last.causes.includes(c)) last.causes.push(c);
    } else findings.push({ ...f, causes: [...f.causes] });
  }
  findings.sort((a, b) => a.frame - b.frame);
  return { findings, frames: n };
}

/** A lint as lines: a count by stage and kind, then each finding. */
export function lintReport(l: Lint, o: { max?: number } = {}): string {
  if (!l.findings.length) return `motion lint: clean over ${l.frames} frames`;
  const by = new Map<string, number>();
  for (const f of l.findings) by.set(`${f.stage}/${f.kind}`, (by.get(`${f.stage}/${f.kind}`) ?? 0) + 1);
  const head = `motion lint: ${l.findings.length} findings over ${l.frames} frames (${[...by].map(([k, v]) => `${k} ${v}`).join(", ")})`;
  const unit = (k: Finding["kind"]) => (k === "speed" ? "°/f" : k === "jerk" ? "°/f²" : "° past the clip");
  const lines = l.findings.slice(0, o.max ?? 60).map((f) =>
    `  frame ${String(f.peak).padStart(4)}${f.frames > 1 ? ` (${f.frames}f)` : ""}  ${f.joint.padEnd(14)} ${f.kind} ${f.value.toFixed(1)}${unit(f.kind)} (limit ${f.limit.toFixed(1)})  made by: ${f.stage}${f.causes.length ? `  about then: ${f.causes.join(", ")}` : ""}`);
  return [head, ...lines, ...(l.findings.length > (o.max ?? 60) ? [`  ... ${l.findings.length - (o.max ?? 60)} more`] : [])].join("\n");
}
