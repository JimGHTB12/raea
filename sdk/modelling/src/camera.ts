// The camera.
import { Solid } from "@raea/render/solid";
import { Built, DEG, Step, V3, add3, len3, lerp, lerp3, meshOf, mul3, sub3 } from "./all";

// ---------------------------------------------------------------- the camera

interface Cam {
  target: V3;
  dist: number;
  yaw: number;
  pitch: number;
}

const camSims = new WeakMap<Built, Map<string, { dt: number; frames: Cam[] }>>();

/** what a step frames: its parts' bounds */
export function focusOf(b: Built, s: Step | undefined): { c: V3; r: number } {
  const all = () => {
    let min: V3 = [Infinity, Infinity, Infinity], max: V3 = [-Infinity, -Infinity, -Infinity];
    for (const { solid } of b.objects.values()) {
      const m = meshOf(solid);
      min = [Math.min(min[0], m.min[0]), Math.min(min[1], m.min[1]), Math.min(min[2], m.min[2])];
      max = [Math.max(max[0], m.max[0]), Math.max(max[1], m.max[1]), Math.max(max[2], m.max[2])];
    }
    for (const x of b.bones) for (const p of [x.head, x.tail]) for (let k = 0; k < 3; k++) { min[k] = Math.min(min[k], p[k]); max[k] = Math.max(max[k], p[k]); }
    if (!Number.isFinite(min[0])) return { c: [0, 1, 0] as V3, r: 2 };
    return { c: mul3(add3(min, max), 0.5), r: len3(sub3(max, min)) / 2 };
  };
  if (!s) return all();
  const of = (...ss: (Solid | null | undefined)[]) => {
    let min: V3 = [Infinity, Infinity, Infinity], max: V3 = [-Infinity, -Infinity, -Infinity];
    for (const x of ss) {
      if (!x) continue;
      const m = meshOf(x);
      min = [Math.min(min[0], m.min[0]), Math.min(min[1], m.min[1]), Math.min(min[2], m.min[2])];
      max = [Math.max(max[0], m.max[0]), Math.max(max[1], m.max[1]), Math.max(max[2], m.max[2])];
    }
    return { c: mul3(add3(min, max), 0.5), r: Math.max(0.25, len3(sub3(max, min)) / 2) };
  };
  switch (s.kind) {
    case "add": return of(s.after);
    case "transform": return of(s.before, s.after);
    case "boolean": return of(s.before, s.tool);
    case "mirror": return of(s.after);
    case "extrude": case "revolve": case "sculpt": case "smooth": return of(s.after);
    case "array": return of(s.after);
    default: return all();
  }
}

const YAWS = [-28, 22, -12, 34, -36, 14, -20, 30, -6, 26];

function camSim(b: Built, aspect: number, fov: number): { dt: number; frames: Cam[] } {
  const key = `${aspect.toFixed(3)}:${fov}`;
  let byKey = camSims.get(b);
  if (!byKey) camSims.set(b, (byKey = new Map()));
  const hit = byKey.get(key);
  if (hit) return hit;
  const vf = fov * DEG, hf = 2 * Math.atan(Math.tan(vf / 2) * aspect);
  const fit = Math.min(vf, hf);
  // each step's goal
  const goals = b.steps.map((s, i) => {
    const f = focusOf(b, s);
    const v = s.view ?? {};
    const whole = ["bone", "weights", "pose", "finish", "animate", "wait"].includes(s.kind);
    return {
      start: s.start,
      cam: {
        target: f.c,
        dist: (f.r / Math.sin(fit / 2)) * (whole ? 1.05 : 1.25) / (v.zoom ?? 1),
        yaw: (v.yaw ?? YAWS[i % YAWS.length]) * DEG,
        pitch: (v.pitch ?? (whole ? 10 : 18 + (i % 3) * 5)) * DEG,
      } as Cam,
    };
  });
  const dt = 1 / 48;
  const end = b.duration + 8;
  const frames: Cam[] = [];
  const first = goals[0]?.cam ?? { target: [0, 1, 0], dist: 6, yaw: 0, pitch: 0.3 };
  let x = [...first.target, first.dist, first.yaw, first.pitch];
  let v = [0, 0, 0, 0, 0, 0];
  // an underdamped spring: it overshoots a touch and settles, like a hand on a mouse
  const w = 2 * Math.PI * 0.55, z = 0.72;
  let gi = 0;
  for (let i = 0; i * dt <= end; i++) {
    const t = i * dt;
    while (gi + 1 < goals.length && goals[gi + 1].start <= t + 0.15) gi++;
    const g = goals[gi]?.cam ?? first;
    const goal = [...g.target, g.dist, g.yaw, g.pitch];
    for (let k = 0; k < 6; k++) {
      const a = w * w * (goal[k] - x[k]) - 2 * z * w * v[k];
      v[k] += a * dt;
      x[k] += v[k] * dt;
    }
    frames.push({ target: [x[0], x[1], x[2]], dist: x[3], yaw: x[4], pitch: x[5] });
  }
  const sim = { dt, frames };
  byKey.set(key, sim);
  return sim;
}

export function camAt(b: Built, at: number, aspect: number, fov: number): { eye: V3; target: V3; yaw: number; pitch: number; dist: number } {
  const sim = camSim(b, aspect, fov);
  const f = Math.max(0, at) / sim.dt;
  const i = Math.min(sim.frames.length - 2, Math.floor(f));
  const u = Math.min(1, f - i);
  const A = sim.frames[i], B = sim.frames[i + 1];
  // a slow drift on top, and a breath of hand-held wobble
  const drift = at * 1.6 * DEG + Math.sin(at * 0.9) * 0.6 * DEG;
  const yaw = lerp(A.yaw, B.yaw, u) + drift;
  const pitch = lerp(A.pitch, B.pitch, u) + Math.sin(at * 0.63 + 1) * 0.4 * DEG;
  const dist = lerp(A.dist, B.dist, u);
  const target = lerp3(A.target, B.target, u);
  const eye = add3(target, [dist * Math.cos(pitch) * Math.sin(yaw), dist * Math.sin(pitch), dist * Math.cos(pitch) * Math.cos(yaw)]);
  return { eye, target, yaw, pitch, dist };
}
