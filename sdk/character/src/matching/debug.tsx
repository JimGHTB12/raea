// What motion matching and foot locking did, drawn over the scene (Bevy gizmos), so slips, misses and the route can be
// seen where they happen rather than read off a table:
//   white   the guide's route on the floor; yellow, the hips' own trail over the last three seconds
//   soles   each foot's heel and ball over the last second and a half: green held by its lock, blue in the air, red on
//           the floor and moving without a lock (a skate), orange held but still moving more than 2 mm a frame (a slip)
//   lock    a green cross where each standing foot is locked (on its pivot: heel, then ball)
//   ik      a cyan cross at each ankle's target, joined to where the leg reached it in red when it missed by over 1 cm
// The performance has to be made with { trace: true, feet: "lock" } for the lock and the IK; without it the route and
// the trails still draw.
//
//   const perf = perform(lib, spec, { trace: true, feet: "lock" });
//   <MotionDebug skeleton={turbo} perf={perf} frame={f} />
import { Gizmo, type Vec3 } from "@raea/render/3d";
import { B, SIDES, joint, pose, qrot, type Skeleton } from "../character/all";
import type { Performance } from "./all";

const GREEN = "#36d67a", BLUE = "#5b8cff", RED = "#ff3b3b", ORANGE = "#ffa62b", CYAN = "#3be8ff", WHITE = "#f4f4f4", YELLOW = "#ffd84a";
const LIFT = 0.004;

interface Soles { at: Vec3[][][]; root: Vec3[] }
const cache = new WeakMap<Performance, Soles>();

/** every frame's sole points (each side's heel and ball, on the floor under the joint as the joint turns) and hips */
function solesOf(skel: Skeleton, perf: Performance): Soles {
  let c = cache.get(perf);
  if (c) return c;
  const J = skel.joints, ids = SIDES.map((s) => [joint(skel, B.foot(s)), joint(skel, B.ball(s))]);
  const at: Vec3[][][] = [], root: Vec3[] = [];
  for (let f = 0; f < perf.frames; f++) {
    const p = perf.pose(f), w = pose(skel, p);
    root.push([p.root[0], LIFT, p.root[2]]);
    at.push(ids.map((pair) => pair.map((i) => { const v = qrot(w.rot[i], [0, -J[i].head[1], 0]); return [w.pos[i][0] + v[0], w.pos[i][1] + v[1], w.pos[i][2] + v[2]] as Vec3; })));
  }
  c = { at, root };
  cache.set(perf, c);
  return c;
}

const cross = (p: Vec3, r: number): [Vec3, Vec3][] => [[[p[0] - r, p[1], p[2]], [p[0] + r, p[1], p[2]]], [[p[0], p[1], p[2] - r], [p[0], p[1], p[2] + r]], [[p[0], p[1] - r, p[2]], [p[0], p[1] + r, p[2]]]];

export interface MotionDebugProps {
  skeleton: Skeleton;
  perf: Performance;
  frame: number;
  /** how many frames of the sole trails to draw (45) and of the hips' trail (90) */
  trail?: number;
  hips?: number;
  /** draw over the body and floor (true) or depth-tested */
  front?: boolean;
}

export function MotionDebug({ skeleton, perf, frame, trail = 45, hips = 90, front = true }: MotionDebugProps) {
  const f = Math.max(0, Math.min(perf.frames - 1, Math.round(frame)));
  const S = solesOf(skeleton, perf), L = perf.trace?.lock, bias = front ? -1 : 0;
  const route = perf.guide.map((g) => [g.x, LIFT, g.z] as Vec3);
  const lines: [Vec3, Vec3][] = [], colors: string[] = [];
  const seg = (a: Vec3, b: Vec3, c: string) => { lines.push([a, b]); colors.push(c, c); };
  // the soles' trails, coloured by what each frame was
  for (let s = 0; s < 2; s++) for (let k = 0; k < 2; k++) for (let t = Math.max(1, f - trail); t <= f; t++) {
    const a = S.at[t - 1][s][k], b = S.at[t][s][k], held = L?.stand[s][t] ?? false, down = b[1] < 0.015;
    const move = Math.hypot(b[0] - a[0], b[2] - a[2]);
    const c = held ? (move > 0.002 && down ? ORANGE : GREEN) : down && move > 0.002 ? RED : BLUE;
    seg([a[0], Math.max(LIFT, a[1]), a[2]], [b[0], Math.max(LIFT, b[1]), b[2]], c);
  }
  if (L) for (let s = 0; s < 2; s++) {
    const lk = L.lock[s][f];
    if (lk) for (const [a, b] of cross([lk[0], LIFT, lk[1]], 0.05)) seg(a, b, GREEN);
    const tg = L.target[s][f], re = L.reached[s][f];
    if (tg && re) {
      for (const [a, b] of cross(tg, 0.025)) seg(a, b, CYAN);
      if (Math.hypot(tg[0] - re[0], tg[1] - re[1], tg[2] - re[2]) > 0.01) seg(tg, re, RED);
    }
  }
  return (
    <>
      <Gizmo name="debug-route" strip={route} color={WHITE} width={1.5} depthBias={bias} />
      <Gizmo name="debug-hips" strip={S.root.slice(Math.max(0, f - hips), f + 1)} color={YELLOW} width={2} depthBias={bias} />
      {lines.length > 0 && <Gizmo name="debug-feet" lines={lines} color={colors} width={3} depthBias={bias} />}
    </>
  );
}
