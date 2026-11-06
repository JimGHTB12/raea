// The performance's spec, and the guide it becomes frame by frame.
import type { Vec3 } from "raea/3d";
import { bodyScale, qinv, qmul, qnorm, qrot, qyaw, type Pose } from "raea/character";
import { framePose, type Clip } from "../clip";
import { type PlantLog } from "../feet";
import { type LockLog } from "../lock";
import { qexp, qlog, type RigEffect } from "../effects";
import { MotionLibrary, MotionState, TagSpec, labelOf, rounded } from "./all";

// ---------------------------------------------------------------- the performance

export type Step =
  /** walk the route (points on the floor, x z), then turn to face `face` (a point, or a yaw) */
  | { walk: [number, number][]; face?: Vec3 | number; speed?: number; state?: MotionState; enter?: number; leave?: number; tags?: TagSpec; limbs?: Limbs }
  /** play an action clip (the library's `actions`) from frame `from` to `to`, placed so its hips at frame `anchor`
   * (the moment over the obstacle) are over `at` and its way (first frame to last) is `face`; the walk before it should `leave` at the
   * clip's run-up speed and the walk after `enter` at its run-out speed; or, with `yaw`, turned so its hips face `yaw` at
   * its first frame (a clip that stays where it is, whose first and last frames give it no way: a throw, a pick-up;
   * stood up, as they are there, not stooped, when the hips' heading swings). A name
   * ending in ~ plays the clip's mirror image (the other hand) */
  | { act: string; from?: number; to?: number; anchor: number; at: [number, number]; face: number; yaw?: number }
  /** stand, turned to `face`, for `wait` seconds (where it stopped, if that is within `slack` metres of the spot) */
  | { wait: number; face?: Vec3 | number; state?: MotionState; slack?: number; tags?: TagSpec; limbs?: Limbs }
  /** sit on a seat for `for` seconds, then stand up: walk to seatApproach(lib, seat) first */
  | { sit: Seat; for: number };

/** Per-limb control (docs/clips.feature), on a database library: the upper body (the spine up, the arms and the
 * head) plays its own match while the legs, hips and way play the step's. It is searched with the legs' own query (so
 * its stride is theirs) among frames with the upper's tags as well as the step's (a carry's arms and trunk from carry
 * captures while the legs walk from walks), kept unless something is clearly better, and blended through its own
 * switches as the body's are. On a library without tags the step plays whole. */
export interface Limbs {
  /** `hands`: where a hand should be in the body's frame (x to its left, y up from the floor, z ahead, m), matched on a
   * library with hands (a database's): a suitcase's hand low at the side, not a box's before the chest */
  upper?: { tags?: TagSpec; hands?: Partial<Record<"L" | "R", Vec3>> };
  /** the room a thing held in a hand needs from the thigh on its side and the waist as the arm swings (its half size
   * and a margin, m): frames whose swing comes closer (swingClearance) aren't matched, on any library */
  clear?: Partial<Record<"L" | "R", number>>;
}

/** A seat: its top under the middle of the hips, and the way a sitter faces (a yaw, or a point: the desk). */
export interface Seat {
  at: Vec3;
  facing: number | Vec3;
}

export const seatYaw = (s: Seat) => (typeof s.facing === "number" ? s.facing : Math.atan2(s.facing[0] - s.at[0], s.facing[2] - s.at[2]));

/** Where to stand, and face, to sit on a seat: the spot the library's sitting down starts from, before the seat. */
export function seatApproach(lib: MotionLibrary, seat: Seat): { at: [number, number]; face: number } {
  if (!lib.sit) throw new Error("seatApproach: the library has no sitting (motionLibrary's sit option)");
  const y = seatYaw(seat);
  const d = qrot(qyaw(y), [lib.sit.stand[0], 0, lib.sit.stand[1]]);
  return { at: [seat.at[0] + d[0], seat.at[2] + d[2]], face: y };
}

/** An action under way at a frame of the guide: which, its own frame, and where. */
export interface Act {
  j: number;
  seat?: Seat;
  /** its stretches' lengths in frames: down, sat, up */
  len: [number, number, number];
  /** an action clip's play: its first library frame and its anchor's, and where the anchor goes */
  play?: { from: number; anchor: number; at: [number, number]; face: number; rot: number; still?: boolean };
}

export interface PerformanceSpec {
  /** where it starts standing (on the floor) and the yaw it faces */
  at: Vec3;
  facing: number;
  steps: Step[];
  /** how long the performance is, frames (the steps' time, then it stands) */
  frames?: number;
  /** the state it moves in (a step's own `state` overrides it): the library frames that move most like it are
   * chosen, as near as the route allows */
  state?: MotionState;
  /** tags every step's frames must have or must not (a database library's; a step's own add to these) */
  tags?: TagSpec;
}

export interface PerformOptions {
  /** how the guide pulls the body: true (Holden's velocity-limited adjustment and clamp, docs/locomotion.md; the
   * default) or the flat pull (false; RAEA_ADJUST=0) */
  adjust?: boolean | "1";
  /** passes of the five-frame smoothing over the matched joints (2 by default; 0 leaves them as matched) */
  smooth?: number;
  /** how the feet are held while they stand: "lock" (lockFeet, the default: contacts baked from the soles, heel then
   * ball locked, released on a spring) or "plant" (plantFeet, the older pinning; RAEA_FEET=plant) */
  feet?: "plant" | "lock";
  /** how often it searches the library, frames */
  every?: number;
  /** a switch's blend, seconds */
  blend?: number;
  /** the hips' and upper body's blend out of an action (its lean eased off; the legs keep `blend`), seconds */
  blendOut?: number;
  /** how hard the body is pulled onto the route each frame (0..1) */
  pull?: number;
  /** plant the feet while they stand (true: feet.ts) */
  pin?: boolean;
  /** the head turned ahead into turns (true) */
  gaze?: boolean;
  /** keep each stage's poses and what happened, for motionLint */
  trace?: boolean;
  /** with RAEA_LINT set, don't print the lint */
  quiet?: boolean;
  /** how much a state counts against the route when choosing frames (2) */
  stateWeight?: number;
  /** the rig effects by style label (`learnEffect`, or a library's effects.json): a state the library has no frames
   * for is laid over the frames it chooses */
  effects?: Record<string, RigEffect>;
}

const paces = new WeakMap<MotionLibrary, Map<string, number>>();

/** how fast a style's frames in the library travel while they move (over 0.15 m/s), m/s; 0 if it has none */
/** whether the body is going somewhere at library frame k (over a second about it, faster than 0.15 m/s) */
export function movingAt(lib: MotionLibrary, k: number, r: { start: number; end: number }): boolean {
  const a = Math.max(r.start, k - 15), b = Math.min(r.end - 1, k + 15);
  if (b <= a) return false;
  return (Math.hypot(lib.ground[b * 3] - lib.ground[a * 3], lib.ground[b * 3 + 1] - lib.ground[a * 3 + 1]) * lib.fps) / (b - a) > 0.15;
}

export function paceOf(lib: MotionLibrary, label: string): number {
  let m = paces.get(lib);
  if (!m) paces.set(lib, (m = new Map()));
  if (m.has(label)) return m.get(label)!;
  let sum = 0, n = 0;
  for (const r of lib.clips) {
    if (r.mirrored || labelOf(r.name) !== label) continue;
    for (let k = r.start + 15; k < r.end - 15; k++) {
      const v = (Math.hypot(lib.ground[(k + 15) * 3] - lib.ground[(k - 15) * 3], lib.ground[(k + 15) * 3 + 1] - lib.ground[(k - 15) * 3 + 1]) * lib.fps) / 30;
      if (v > 0.15) { sum += v; n++; }
    }
  }
  const v = n ? sum / n : 0;
  m.set(label, v);
  return v;
}

const ids = new WeakMap<object, number>();

/** the module's state its files assign (each field was a top-level let) */ export const S: { nextId: number } = { nextId: 1 };

/** an object's identity, for a cache key that doesn't serialise it */
export function idOf(x: object): number {
  let i = ids.get(x);
  if (!i) ids.set(x, (i = S.nextId++));
  return i;
}

export interface Performance {
  frames: number;
  /** the frame's pose in the scene (draw the Character at the origin) */
  pose(frame: number): Pose;
  /** the route the body was sent along: per frame the place it was meant to be and its yaw */
  guide: { x: number; z: number; yaw: number; moving: boolean; act?: Act; step?: number; speed?: number; slack?: number }[];
  /** which library clip each frame played, for checks */
  played: { clip: string; f: number }[];
  /** with limbs.upper: the clip the upper body played each frame (null where it played the body's) */
  upper?: ({ clip: string; f: number } | null)[];
  /** what an action is doing at a frame: sitting down, sat or standing up, and how far through that stretch (0..1);
   * null while walking or standing (to ease goals in once sat: hands to the desk, eyes to the screen) */
  phase(frame: number): { action: "sit"; stage: "down" | "sat" | "up"; u: number; since: number; left: number } | null;
  /** which of the spec's steps a frame is in, its kind, and the seconds since it began and until it ends; null once the
   * steps are over (to ease goals in while the body waits: a hand to a handle at the fridge) */
  step(frame: number): { index: number; kind: "walk" | "wait" | "sit" | "act"; since: number; left: number } | null;
  /** with `trace`: the pose at each stage (the clip's frame as chosen, blended in, the feet planted, the elbows relaxed,
   * the gaze), the frames a switch began, the rate (library frames a frame) and what planting did, for motionLint */
  trace?: { stages: Record<"clip" | "blend" | "plant" | "elbows" | "gaze", Pose[]>; switches: number[]; rate: number[]; plant?: PlantLog; lock?: LockLog };
}

export const wrap = (a: number) => Math.atan2(Math.sin(a), Math.cos(a));

/** the route's guide: where the body is meant to be each frame, at walking pace with easing at starts and stops */
export function guide(spec: PerformanceSpec, fps: number, lib: MotionLibrary): Performance["guide"] {
  const out: Performance["guide"] = [];
  let x = spec.at[0], z = spec.at[2], yaw = spec.facing;
  const faceYaw = (f: Vec3 | number | undefined, from: [number, number]) => (f === undefined ? undefined : typeof f === "number" ? f : Math.atan2(f[0] - from[0], f[2] - from[1]));
  const turnTo = (want: number | undefined, frames: number, slack?: number) => {
    // a stand-still turn at a natural pace (a quarter turn in about two thirds of a second), then holding
    for (let f = 0; f < frames; f++) {
      if (want !== undefined) {
        const d = wrap(want - yaw);
        yaw += Math.sign(d) * Math.min(Math.abs(d), (2.4 / fps) * Math.min(1, Math.abs(d) / 0.5 + 0.2));
      }
      out.push({ x, z, yaw, moving: false, slack });
    }
  };
  const starts: number[] = [];
  for (const [si, st] of spec.steps.entries()) {
    starts.push(out.length);
    if ("wait" in st) {
      turnTo(faceYaw(st.face, [x, z]), Math.round(st.wait * fps), st.slack);
      continue;
    }
    if ("act" in st) {
      const mir = st.act.endsWith("~"), r = lib.clips.find((c) => c.name === (mir ? st.act.slice(0, -1) : st.act) && c.mirrored === mir);
      if (!r) throw new Error(`perform: no action clip ${st.act} (motionLibrary's actions option)`);
      const from = r.start + (st.from ?? 0), to = r.start + Math.min(r.end - r.start - 1, st.to ?? r.end - r.start - 1), anchor = r.start + st.anchor;
      // turned by the way the clip travels from its first frame to its last (its hips can face anywhere at the anchor:
      // mid-dive, rolling, they face back): the whole capture's, as fitAction measures it, however much of it plays
      const e = r.end - 1, way = Math.atan2(lib.ground[e * 3] - lib.ground[r.start * 3], lib.ground[e * 3 + 1] - lib.ground[r.start * 3 + 1]);
      const rot = st.yaw !== undefined ? st.yaw - lib.ground[from * 3 + 2] : st.face - way, ax = lib.ground[anchor * 3], az = lib.ground[anchor * 3 + 1];
      const play = { from, anchor, at: st.at, face: st.face, rot, still: st.yaw !== undefined };
      // (the walk in ends where the clip starts: its last frame and the clip's first were the same place, a frame stood
      // still in a run, and the body came to every action a frame ahead, 5-26 cm, eased back over its standing feet)
      { const k0 = from, d0 = qrot(qyaw(rot), [lib.ground[k0 * 3] - ax, 0, lib.ground[k0 * 3 + 1] - az]), lp = out[out.length - 1];
        // (along the way: the clip's first frame stands a few cm to the side of the lane)
        if (lp && lp.moving && !lp.act && Math.abs((lp.x - st.at[0] - d0[0]) * Math.sin(st.face) + (lp.z - st.at[1] - d0[2]) * Math.cos(st.face)) < 0.01) { out.pop(); starts[starts.length - 1]--; } }
      for (let j = 0; j <= to - from; j++) {
        const k = from + j, d = qrot(qyaw(rot), [lib.ground[k * 3] - ax, 0, lib.ground[k * 3 + 1] - az]);
        x = st.at[0] + d[0];
        z = st.at[1] + d[2];
        yaw = lib.ground[k * 3 + 2] + rot;
        out.push({ x, z, yaw, moving: true, act: { j, len: [to - from + 1, 0, 0], play } });
      }
      continue;
    }
    if ("sit" in st) {
      if (!lib.sit) throw new Error("perform: a sit step needs a library with sitting (motionLibrary's sit option)");
      const len: [number, number, number] = [lib.sit.down[1] - lib.sit.down[0], Math.max(1, Math.round(st.for * fps)), lib.sit.up[1] - lib.sit.up[0]];
      const a = seatApproach(lib, st.sit);
      for (let j = 0; j < len[0] + len[1] + len[2]; j++) out.push({ x: a.at[0], z: a.at[1], yaw: a.face, moving: false, act: { j, seat: st.sit, len } });
      // stood up again about where it sat down from
      x = a.at[0];
      z = a.at[1];
      yaw = a.face;
      continue;
    }
    // a walk's own corners (a route's are rounded already) rounded a little: no one walks to a corner and pivots
    // (from where the body is: a walk whose first point is elsewhere goes there first; taken as its start, the guide
    // jumped 3 m and the body was pulled across at 11 m/s with its feet down)
    const w0 = st.walk[0], far = !w0 || Math.hypot(w0[0] - x, w0[1] - z) > 0.05;
    const pts = rounded(far ? [[x, z] as [number, number], ...st.walk] : st.walk, [], 0);
    const segs: { a: [number, number]; b: [number, number]; l: number }[] = [];
    for (let k = 1; k < pts.length; k++) segs.push({ a: pts[k - 1], b: pts[k], l: Math.hypot(pts[k][0] - pts[k - 1][0], pts[k][1] - pts[k - 1][1]) });
    const total = segs.reduce((a, s) => a + s.l, 0);
    // m/s and m/s² (a child's walk at the library's walks' pace, a bigger body's by its size: its captures are scaled
    // so; a run speeds up harder)
    const v0 = st.speed ?? bodyScale(lib.skel), acc = 1.2 * Math.max(1, v0);
    // the speed along the way: up from `enter`, down to `leave` at the end (a stop, by default), and slowed for the bends
    // (no more than 1.2 m/s² across the way, as a walker leans into a bend: a 0.4 m arc at 0.7 m/s), found forward and
    // back over 2 cm steps; then the time at each step
    const ve = st.enter ?? 0, vl = st.leave ?? 0, ds = 0.02, ns = Math.max(1, Math.ceil(total / ds));
    const dirAt = (s: number) => { let r = s; for (const sg of segs) { if (r <= sg.l || sg === segs[segs.length - 1]) return Math.atan2(sg.b[0] - sg.a[0], sg.b[1] - sg.a[1]); r -= sg.l; } return 0; };
    const vs = new Float64Array(ns + 1);
    for (let i = 0; i <= ns; i++) {
      // the bend: the turn over 30 cm about here
      const s = (i / ns) * total, a = dirAt(Math.max(0, s - 0.15)), b = dirAt(Math.min(total, s + 0.15));
      const k = Math.abs(wrap(b - a)) / Math.max(0.05, Math.min(total, s + 0.15) - Math.max(0, s - 0.15));
      vs[i] = Math.min(v0, k > 1e-3 ? Math.sqrt((1.2 * Math.max(1, v0)) / k) : v0);
    }
    const stepD = total / ns;
    vs[0] = Math.min(vs[0], ve);
    for (let i = 1; i <= ns; i++) vs[i] = Math.min(vs[i], Math.sqrt(vs[i - 1] ** 2 + 2 * acc * stepD));
    vs[ns] = Math.min(vs[ns], vl);
    for (let i = ns - 1; i >= 0; i--) vs[i] = Math.min(vs[i], Math.sqrt(vs[i + 1] ** 2 + 2 * acc * stepD));
    const ts = new Float64Array(ns + 1);
    for (let i = 1; i <= ns; i++) ts[i] = ts[i - 1] + (2 * stepD) / Math.max(1e-3, vs[i] + vs[i - 1]);
    const T = ts[ns], vpk = Math.max(...vs);
    const sAt = (t: number) => {
      let lo = 0, hi = ns;
      while (hi - lo > 1) { const m = (lo + hi) >> 1; if (ts[m] <= t) lo = m; else hi = m; }
      const u = ts[hi] > ts[lo] ? (t - ts[lo]) / (ts[hi] - ts[lo]) : 0;
      return Math.min(total, (lo + Math.max(0, Math.min(1, u))) * stepD);
    };
    // the way it ends facing, eased into over the last 0.8 m so it arrives turned (the stand-still turn after is what is left)
    const endFace = faceYaw(st.face, pts[pts.length - 1]);
    const n = Math.max(1, Math.round(T * fps));
    const at = (s: number): { p: [number, number]; dir: number } => {
      let r = s;
      for (const sg of segs) {
        if (r <= sg.l || sg === segs[segs.length - 1]) {
          const u = sg.l > 0 ? Math.min(1, r / sg.l) : 0;
          return { p: [sg.a[0] + (sg.b[0] - sg.a[0]) * u, sg.a[1] + (sg.b[1] - sg.a[1]) * u], dir: Math.atan2(sg.b[0] - sg.a[0], sg.b[1] - sg.a[1]) };
        }
        r -= sg.l;
      }
      return { p: pts[pts.length - 1], dir: yaw };
    };
    // turn towards the way first if it is far round (a person turns before setting off)
    const first = at(0.01).dir;
    if (Math.abs(wrap(first - yaw)) > 1.2) turnTo(first, Math.round((Math.abs(wrap(first - yaw)) / 2.4) * fps) + 6);
    for (let f = 1; f <= n; f++) {
      const s = sAt((f / n) * T);
      const g = at(Math.min(total, s));
      // facing along the way, eased round the corners (the look a little ahead), and over the last 0.8 m toward the end's
      // facing, as far round as a walker turns while still stepping (about 50°)
      const ahead = at(Math.min(total, s + 0.35));
      let want = Math.atan2(ahead.p[0] - g.p[0], ahead.p[1] - g.p[1]);
      const left = total - s;
      if (endFace !== undefined && left < 0.8) {
        const u = 1 - left / 0.8, k = u * u * (3 - 2 * u), d = wrap(endFace - want);
        want += Math.sign(d) * Math.min(Math.abs(d), 0.9) * k;
      }
      if (Math.hypot(ahead.p[0] - g.p[0], ahead.p[1] - g.p[1]) > 0.05 || endFace !== undefined) yaw += wrap(want - yaw) * 0.25;
      x = g.p[0];
      z = g.p[1];
      out.push({ x, z, yaw, moving: vpk > 0, speed: v0 });
    }
    const fy = faceYaw(st.face, [x, z]);
    // (with the slack of a wait after it: the body stops where it can, then turns)
    const nx = spec.steps[si + 1];
    if (fy !== undefined) turnTo(fy, Math.round((Math.abs(wrap(fy - yaw)) / 2.4) * fps) + 15, nx && "wait" in nx ? nx.slack : undefined);
  }
  starts.push(out.length);
  for (let k = 0; k + 1 < starts.length; k++) for (let i = starts[k]; i < starts[k + 1]; i++) out[i].step = k;
  if (spec.frames !== undefined) while (out.length < spec.frames) out.push({ ...out[out.length - 1], moving: false, step: undefined });
  return out;
}

export const performances = new Map<string, Performance>();

/** A clip between frames f and f + 1 (u of the way), on a Catmull-Rom curve through the frames either side: a straight
 * line between frames bends at every frame, and played at a rate other than 1 the hands jerk at each bend. */
export function between(c: Clip, f: number, u: number): Pose {
  const P = [f - 1, f, f + 1, f + 2].map((g) => framePose(c, g));
  const cr = (a: number, b: number, c2: number, d: number) => b + 0.5 * u * (c2 - a + u * (2 * a - 5 * b + 4 * c2 - d + u * (3 * (b - c2) + d - a)));
  const root = [0, 1, 2].map((i) => cr(P[0].root[i], P[1].root[i], P[2].root[i], P[3].root[i])) as Vec3;
  const turns = P[1].turns.map((b, j) => {
    const v = P.map((p) => qlog(qmul(qinv(b), p.turns[j])));
    return qnorm(qmul(b, qexp([0, 1, 2].map((i) => cr(v[0][i], v[1][i], v[2][i], v[3][i])) as Vec3)));
  });
  return { root, turns };
}
