// raea/course: a side-scrolling course along +x, laid out from what its captures clear, run, and checked
// strictly against its own geometry, frame by frame.
//
//   const course = layCourse(lib, kid, { name: "warm-up", segments: [
//     { run: 3 }, { clear: fitAction(kid, clips.get("c127_28")!, "hurdle") }, { run: 2 },
//     { prop: "plinth", thing: { at: [1.2, 0, 0], size: [0.3, 0.5, 0.3] } },
//     { pick: "ball" }, { run: 2 }, { throw: "ball", to: "bin" }, { run: 3 },
//   ], items: { ball: { at: [1.2, 0.5, 0], size: [0.09, 0.09, 0.09], shape: "ball", mass: 0.15 } } });
//   const perf = perform(lib, course.plan.spec, { trace: true });
//   const live = interact(kid, perf, course.plan);
//   const report = checkCourse(kid, course, perf, live);
//   console.log(courseReport(report));
//
// A run is fitted to the obstacle after it (it leaves at the clip's run-up speed), an obstacle is placed where its
// clip's moment over it lands (fitAction), and things picked up or thrown on the way go through planItems. Items and
// props are placed relative to where the course has got to when they are declared: `at[0]` metres ahead.
//
// The check flags, each with its frame and how much:
//   penetrate  a body part (capsules round the bones) inside an obstacle or prop, deeper than 1.5 cm
//   sink       a sole below what it stands on, deeper than 2 cm
//   fall       the body down a gap, below the floor
//   float      in the air through a run longer than a running stride's flight (0.2 s; the library's own runs are off
//              the floor 0.07-0.13 s by the same measure), or an action's flight that doesn't fall as things fall (its
//              hips' fall off by more than a third of gravity)
//   skate      a planted foot sliding, more than 5 cm over one contact (heel, ball or toes' tip on the floor), or one
//              scraping along it at swing speed for a tenth of a second
//   pop        the hips jumping along the floor, a change of speed in one frame no body makes (over 30 m/s²)
//   drift      off the lane, more than 0.25 m to either side
//   backward   back the way it came by more than 15 cm, by the feet (a course runs one way)
//   held       a thing in hand more than 5 cm from the palms holding it
//   throw      a thing thrown that doesn't end in its target, or come down within 30 cm of its point
//   reach      a hand's goal out of reach by more than 4 cm
//   lint       a joint changing speed more suddenly than any capture in the library, by a fifth past motionLint's limit
//              (not the capture's own moves, nor a pick-up's squat or a throw)
//   unfinished the course not run to its end
import type { Vec3 } from "raea/3d";
import { B, add, bodyScale, joint, palmAt, pose, qrot, type Pose, type Skeleton } from "raea/character";
import { interact, planItems, type Interaction, type Item, type ItemClip, type ItemPlan, type ItemTask, type PickManner, type ThrowStyle } from "./items";
import { motionLint } from "./lint";
import type { ActionFit, MotionLibrary, Performance } from "./matching";
import type { Thing } from "./things";

export type Segment =
  /** run this far along the course (metres), at `speed` (m/s, by default a run's) */
  | { run: number; speed?: number }
  /** an obstacle, cleared by the action clip fitted to it (fitAction) */
  | { clear: ActionFit }
  /** a prop (a plinth, a bin, a table) placed `thing.at[0]` metres ahead of here; it doesn't move the course on */
  | { prop: string; thing: Thing }
  /** stop at an item and take it (it was placed with `items`, relative to here), the knees bent `bend` of the way (0 a stoop) */
  | { pick: string; hand?: "L" | "R" | "both"; bend?: number; face?: number; clip?: ItemClip; manner?: PickManner; look?: ItemClip; from?: "above" | "side" }
  /** throw the item held into a prop or to a point (relative to here) */
  | { throw: string; to: string | Vec3; style?: ThrowStyle | "auto"; clip?: ItemClip }
  | { wait: number };

export interface CourseSpec {
  name: string;
  segments: Segment[];
  /** the items to pick up, each `at` relative to where the course has got to at its pick (metres ahead, up, across) */
  items?: Record<string, Item>;
  /** the run's speed between obstacles (m/s) */
  speed?: number;
  /** start from standing and stop at the end (by default the body runs in and runs on out, as a side-scroller's does:
   * starting and stopping are the walks' to check, bench/character/lint.ts) */
  stand?: boolean;
}

/** a solid the body must not go through */
export interface Solid {
  name: string;
  kind: "bar" | "post" | "box" | "block" | "prop";
  min: Vec3;
  max: Vec3;
  /** the obstacle it belongs to (an index into `obstacles`) */
  obstacle?: number;
}

export interface PlacedObstacle extends ActionFit {
  /** the anchor's place along the course, and the plan step that clears it */
  x: number;
  step: number;
}

export interface Course {
  spec: CourseSpec;
  plan: ItemPlan;
  obstacles: PlacedObstacle[];
  solids: Solid[];
  /** stretches of the course with no floor (x from, to) */
  gaps: [number, number][];
  props: Record<string, Thing>;
  /** items where they start, in the scene */
  items: Record<string, Item>;
  /** throws: which item to which prop or point */
  throws: { item: string; to: string | Vec3; step: number }[];
  end: number;
}

const EAST = Math.PI / 2;
/** the lane's half width for the 1.26 m child (obstacles span it; a hurdle's bar a little less), and for any other
 * body by its size (bodyScale): a man's shoulders took the duck bar's posts */
const LANE = 0.5;

/** Lay a course out along +x from where it starts (see the head of the file). */
export function layCourse(lib: MotionLibrary, skel: Skeleton, spec: CourseSpec): Course {
  const tasks: ItemTask[] = [], obstacles: PlacedObstacle[] = [], solids: Solid[] = [], gaps: [number, number][] = [];
  const props: Record<string, Thing> = {}, items: Record<string, Item> = {}, throws: Course["throws"] = [];
  const run = spec.speed ?? 2.2, zb = bodyScale(skel), lane = LANE * zb;
  let x = 0, v = spec.stand ? 0 : run, steps = 0;
  // the plan's steps: a pick or throw adds a walk to its spot and a wait, so count them as planItems makes them (a
  // throw turns first only if it has to: counted by planning, below)
  const segs = spec.segments;
  // place every item and prop first, relative to where the course is when each is declared
  {
    let cx = 0;
    for (const s of segs) {
      if ("run" in s) cx += s.run;
      else if ("clear" in s) cx += s.clear.dIn + s.clear.dOut;
      else if ("prop" in s) props[s.prop] = { ...s.thing, at: [cx + s.thing.at[0], s.thing.at[1], s.thing.at[2]] };
      else if ("pick" in s) {
        const it = spec.items?.[s.pick];
        if (!it) throw new Error(`layCourse: no item ${s.pick}`);
        items[s.pick] = { ...it, at: [cx + it.at[0], it.at[1], it.at[2]] };
        // the walk to it and back onto the lane moves the course on by about its distance ahead
        cx += it.at[0];
      }
    }
  }
  for (let i = 0; i < segs.length; i++) {
    // (a prop doesn't move the course on: what comes after it is next)
    const s = segs[i], next = segs.slice(i + 1).find((n) => !("prop" in n));
    if ("run" in s) {
      const leave = next && "clear" in next ? next.clear.vIn : next && ("pick" in next || "throw" in next || "wait" in next) ? 0 : !next ? (spec.stand ? 0 : run) : run;
      tasks.push({ walk: [[x + s.run, 0]], speed: s.speed ?? Math.max(1.8 * zb, run, leave, v), enter: v, leave });
      x += s.run;
      v = leave;
    } else if ("clear" in s) {
      const f = s.clear, ax = x + f.dIn;
      const k = obstacles.length;
      obstacles.push({ ...f, x: ax, step: -1 });
      const x0 = ax + f.from, x1 = ax + f.to, cx = (x0 + x1) / 2;
      const n = `${f.kind}${k}`;
      if (f.kind === "hurdle") {
        solids.push({ name: `${n}-bar`, kind: "bar", min: [cx - 0.03, f.top - 0.08, -0.85 * lane], max: [cx + 0.03, f.top, 0.85 * lane], obstacle: k });
        for (const z of [-0.9 * lane, 0.9 * lane]) solids.push({ name: `${n}-post${z > 0 ? "R" : "L"}`, kind: "post", min: [cx - 0.025, 0, z - 0.025], max: [cx + 0.025, f.top, z + 0.025], obstacle: k });
      } else if (f.kind === "duck") {
        solids.push({ name: `${n}-bar`, kind: "bar", min: [cx - 0.04, f.bottom, -lane], max: [cx + 0.04, f.top, lane], obstacle: k });
        for (const z of [-lane, lane]) solids.push({ name: `${n}-post${z > 0 ? "R" : "L"}`, kind: "post", min: [cx - 0.025, 0, z - 0.025], max: [cx + 0.025, f.top, z + 0.025], obstacle: k });
      } else if (f.kind === "gap") gaps.push([x0, x1]);
      else solids.push({ name: `${n}-box`, kind: f.kind === "box" ? "box" : "block", min: [x0, 0, -lane], max: [x1, f.top, lane], obstacle: k });
      tasks.push({ step: { act: f.clip, anchor: f.anchor, at: [ax, 0], face: EAST }, at: [ax + f.dOut, 0], face: EAST });
      x = ax + f.dOut;
      v = f.vOut;
    } else if ("prop" in s) continue;
    else if ("pick" in s) {
      // (turned back down the lane where it stands before going on: a walk started facing the thing steps into it)
      tasks.push({ pick: s.pick, hand: s.hand, bend: s.bend, face: s.face, clip: s.clip, manner: s.manner, look: s.look, from: s.from }, { wait: 0.2, face: EAST });
      x = items[s.pick].at[0];
      v = 0;
    } else if ("throw" in s) {
      const to = typeof s.to === "string" ? s.to : ([x + s.to[0], s.to[1], s.to[2]] as Vec3);
      tasks.push({ throw: s.throw, to, style: s.style, clip: s.clip });
      throws.push({ item: s.throw, to, step: -1 });
      v = 0;
    } else {
      tasks.push({ wait: s.wait });
      v = 0;
    }
  }
  for (const [n, t] of Object.entries(props)) solids.push({ name: n, kind: "prop", min: [t.at[0] - t.size[0] / 2, t.at[1], t.at[2] - t.size[2] / 2], max: [t.at[0] + t.size[0] / 2, t.at[1] + t.size[1], t.at[2] + t.size[2] / 2] });
  const plan = planItems(lib, { at: [0, 0, 0], facing: EAST }, items, tasks, { things: props, lead: 0.3 });
  // which plan step clears each obstacle, and which beat throws each throw
  let k = 0;
  // (not a pick's or a throw's capture)
  const beatSteps = new Set([...plan.beats.map((b) => b.step), ...plan.looks]);
  plan.spec.steps.forEach((st, i) => { if ("act" in st && !beatSteps.has(i)) obstacles[k++].step = i; });
  let t = 0;
  for (const b of plan.beats) if (b.kind === "throw") throws[t++].step = b.step;
  void steps;
  return { spec, plan, obstacles, solids, gaps, props, items, throws, end: x };
}

/** one thing the check found */
export interface Flag {
  kind: "penetrate" | "sink" | "fall" | "float" | "skate" | "pop" | "drift" | "backward" | "held" | "throw" | "reach" | "lint" | "unfinished";
  /** first frame, last frame and the worst one */
  frame: number;
  last: number;
  peak: number;
  /** how much at the worst (metres, m/s², fraction of gravity, degrees...), and what */
  amount: number;
  what: string;
}

export interface CourseCheck {
  name: string;
  frames: number;
  flags: Flag[];
  /** per obstacle: the body's least clearance from it (negative: into it) and the frames it was cleared over */
  obstacles: { clip: string; kind: string; clearance: number; frames: [number, number] }[];
  finished: boolean;
  pass: boolean;
  /** which flags on each frame, for an overlay */
  at(frame: number): Flag[];
}

/** the body as capsules: bones (joint to joint) with a radius, as a share of the body's height (a child's build) */
const CAPSULES: [string, string, number][] = [
  [B.pelvis, B.chest, 0.075], [B.chest, B.neck, 0.07], [B.neck, B.head, 0.035],
  [B.thigh("L"), B.calf("L"), 0.045], [B.calf("L"), B.foot("L"), 0.034], [B.foot("L"), B.ball("L"), 0.014],
  [B.thigh("R"), B.calf("R"), 0.045], [B.calf("R"), B.foot("R"), 0.034], [B.foot("R"), B.ball("R"), 0.014],
  [B.upperarm("L"), B.lowerarm("L"), 0.028], [B.lowerarm("L"), B.hand("L"), 0.024],
  [B.upperarm("R"), B.lowerarm("R"), 0.028], [B.lowerarm("R"), B.hand("R"), 0.024],
];
/** the head as a ball over the Head joint (its middle a share of the height above, its radius) */
const HEAD = { up: 0.06, r: 0.07 };
/** the toes past the ball (share of the height) */
const TOE = 0.06;

/** how far a point is outside a box (negative inside: how deep) */
function boxDistance(p: Vec3, b: { min: Vec3; max: Vec3 }): number {
  const o = [0, 1, 2].map((k) => Math.max(b.min[k] - p[k], p[k] - b.max[k]));
  const out = Math.hypot(Math.max(0, o[0]), Math.max(0, o[1]), Math.max(0, o[2]));
  return out > 0 ? out : Math.max(o[0], o[1], o[2]);
}

/** Check a run of a course strictly (see the head of the file). */
export function checkCourse(skel: Skeleton, course: Course, perf: Performance, live?: Interaction, lib?: MotionLibrary): CourseCheck {
  const h = skel.height, n = live ? live.frames : perf.frames, fps = course.plan.fps, g = 9.81;
  // (what counts as a foot down, by the body's size (bodyScale): a capture retargeted onto a man keeps its timing and
  // scales its travel, so his stance skims higher and his feet come down faster; and a slide and a pop judged by it,
  // as an eye judges them, a foot's slide against the foot: a sink or a part through a thing stays in centimetres)
  const z = bodyScale(skel);
  const poseAt = (f: number): Pose => (live ? live.pose(f) : perf.pose(f));
  const J = (s: string) => joint(skel, s);
  const caps = CAPSULES.map(([a, b, r]) => ({ a: J(a), b: J(b), r: r * h, name: `${a}-${b}` }));
  const head = J(B.head);
  const feet = (["L", "R"] as const).map((s) => {
    const a = J(B.foot(s)), t = J(B.ball(s));
    // (the toes' tip, past the ball the skeleton ends at: a foot pushing off stands on it, its ball a few cm up)
    const jt = skel.joints[t], d = [0, 1, 2].map((k) => jt.tail[k] - jt.head[k]), dl = Math.hypot(...d) || 1;
    const tip = d.map((v) => (v / dl) * TOE * h) as Vec3;
    return { s, a, t, tip, ah: skel.joints[a].head[1], th: jt.head[1], tiph: jt.head[1] + tip[1] };
  });
  const footJoints = new Set(feet.flatMap((ft) => [ft.a, ft.t]));
  const raw: Flag[] = [];
  const flag = (kind: Flag["kind"], f: number, amount: number, what: string) => raw.push({ kind, frame: f, last: f, peak: f, amount, what });
  // what stands under a point: the floor (none over a gap), or the top of a box or prop it is over
  const supportAt = (p: Vec3): number => {
    let y = course.gaps.some(([a, b]) => p[0] > a && p[0] < b) && Math.abs(p[2]) < 1.2 ? -Infinity : 0;
    for (const s of course.solids) {
      if (s.kind === "post" || s.kind === "bar") continue;
      if (p[0] >= s.min[0] - 0.02 && p[0] <= s.max[0] + 0.02 && p[2] >= s.min[2] - 0.02 && p[2] <= s.max[2] + 0.02 && p[1] > s.max[1] - 0.06) y = Math.max(y, s.max[1]);
    }
    return y;
  };
  const stepAt = (f: number) => perf.step(Math.min(f, perf.frames - 1));
  const W: ReturnType<typeof pose>[] = [];
  for (let f = 0; f < n; f++) W.push(pose(skel, poseAt(f)));
  const clear = course.obstacles.map(() => Infinity);
  const clearFrames = course.obstacles.map(() => [Infinity, -Infinity] as [number, number]);
  let maxX = -Infinity;
  const air: boolean[] = [];
  const contact: { at: Vec3; ball: Vec3; tip: Vec3; from: number; travel: number }[] = [];
  const lastLow: ([Vec3, Vec3] | null)[] = [null, null], scrape = [0, 0];
  for (let f = 0; f < n; f++) {
    const w = W[f], root = poseAt(f).root, st = stepAt(f);
    // penetration: each capsule against each solid (five points along the bone)
    for (const c of caps) {
      const A = w.pos[c.a], B = w.pos[c.b];
      for (let si = 0; si < course.solids.length; si++) {
        const s = course.solids[si];
        let d = Infinity;
        for (let k = 0; k <= 4; k++) { const u = k / 4; d = Math.min(d, boxDistance([A[0] + (B[0] - A[0]) * u, A[1] + (B[1] - A[1]) * u, A[2] + (B[2] - A[2]) * u], s) - c.r); }
        if (s.obstacle !== undefined && d < clear[s.obstacle]) clear[s.obstacle] = d;
        if (d < -0.015) flag("penetrate", f, -d, `${c.name} into ${s.name}`);
      }
    }
    {
      const H = w.pos[head], p: Vec3 = [H[0], H[1] + HEAD.up * h, H[2]];
      for (const s of course.solids) {
        const d = boxDistance(p, s) - HEAD.r * h;
        if (s.obstacle !== undefined && d < clear[s.obstacle]) clear[s.obstacle] = d;
        if (d < -0.015) flag("penetrate", f, -d, `head into ${s.name}`);
      }
    }
    for (const [i, o] of course.obstacles.entries()) if (st?.index === o.step) { clearFrames[i][0] = Math.min(clearFrames[i][0], f); clearFrames[i][1] = Math.max(clearFrames[i][1], f); }
    // the feet on what they stand on
    let lowest = Infinity, onGround = false;
    for (const ft of feet) {
      const A = w.pos[ft.a], T = w.pos[ft.t];
      const P = add(T, qrot(w.rot[ft.t], ft.tip)), soleA = A[1] - ft.ah, soleT = T[1] - ft.th, soleP = P[1] - ft.tiph;
      const pt: Vec3 = soleT < soleA ? [T[0], soleT, T[2]] : [A[0], soleA, A[2]];
      const sup = supportAt(pt);
      lowest = Math.min(lowest, pt[1]);
      if (sup === -Infinity) { contact[ft.s === "L" ? 0 : 1] = undefined!; continue; }
      // (the tip says whether the foot touches, not how deep: the skin's toes bend where the skeleton's don't)
      const above = pt[1] - sup, touch = Math.min(above, soleP - sup);
      if (above < -0.02) flag("sink", f, -above, `${ft.s} sole ${(-above * 100).toFixed(1)} cm into what it stands on`);
      const k = ft.s === "L" ? 0 : 1;
      // (a foot comes to stand slowly: one skimming low at running speed is swinging)
      // (the same point a frame before: heel against heel, ball against ball, as the lower of the two changes over)
      const prev = lastLow[k], ballLow = soleT < soleA, was = prev ? (ballLow ? prev[1] : prev[0]) : null;
      const speed = was ? Math.hypot((ballLow ? T : A)[0] - was[0], (ballLow ? T : A)[2] - was[2]) * fps : 0;
      lastLow[k] = [A, T];
      // (a foot at swing speed is swinging, however low it skims: a toe clears the floor by a centimetre or two in a
      // stride; one scraping along it, within a centimetre, for a tenth of a second is skating)
      scrape[k] = above < 0.01 * z && speed > 1.2 * z ? scrape[k] + 1 : 0;
      if (scrape[k] === 3) flag("skate", f, speed, `${ft.s} foot scraping along the floor at ${speed.toFixed(1)} m/s`);
      if (touch < 0.025 * z && speed < 1.2 * z && (contact[k] || speed < 1.0 * z)) {
        onGround = true;
        const c = contact[k];
        // (a foot rolls heel to ball to toes, so it slides only where its heel, ball and toes all do)
        if (c) {
          // (frame by frame, the least moved of the heel and ball that bear: whichever the foot stands on stays put)
          const bear = [[A, c.at, soleA], [T, c.ball, soleT]].filter(([, , y]) => (y as number) - sup < 0.025 * z);
          const step = bear.length ? Math.min(...bear.map(([q, q0]) => Math.hypot((q as Vec3)[0] - (q0 as Vec3)[0], (q as Vec3)[2] - (q0 as Vec3)[2]))) : 0;
          c.travel += step;
          c.at = A; c.ball = T; c.tip = P;
          if (c.travel > 0.05 * z && f - c.from >= 2) flag("skate", f, c.travel, `${ft.s} foot slid ${(c.travel * 100).toFixed(1)} cm while planted`);
        } else contact[k] = { at: A, ball: T, tip: P, from: f, travel: 0 };
      } else contact[k] = undefined!;
    }
    // (or borne by another part on the floor or a box: a dive-roll's hands, then its back and shoulders)
    // (the feet's own joints judged above, as feet)
    if (!onGround) for (const c of caps) {
      for (const j of [c.a, c.b]) {
        if (footJoints.has(j)) continue;
        const P = w.pos[j], sup = supportAt(P);
        if (sup > -Infinity && P[1] - c.r - sup < 0.03 * z) onGround = true;
      }
      if (onGround) break;
    }
    air.push(!onGround);
    // down a gap
    if (course.gaps.some(([a, b]) => root[0] > a - 0.1 && root[0] < b + 0.1) && lowest < -0.05) flag("fall", f, -lowest, "below the floor over a gap");
    // the lane and the way
    // (except going to a thing to pick it up, there and back: those walks face it at their end)
    // (and a pick's or a throw's capture, which reaches to the side)
    const sp = st ? course.plan.spec.steps[st.index] : undefined, aside = !!sp && ("wait" in sp || ("walk" in sp && sp.face !== undefined) || ("act" in sp && sp.yaw !== undefined));
    if (Math.abs(root[2]) > 0.25 * z && !aside) flag("drift", f, Math.abs(root[2]), `${Math.abs(root[2]).toFixed(2)} m off the lane`);
    // (by where the feet are: hips sitting back into a squat or a throw's wind-up go nowhere)
    const fx = (w.pos[feet[0].a][0] + w.pos[feet[1].a][0]) / 2;
    maxX = Math.max(maxX, fx);
    // (not a throw's or a pick's capture, whose step back into the wind-up is the throw's; the furthest on is taken from
    // where it ends up)
    const captured = !!sp && "act" in sp && sp.yaw !== undefined;
    if (captured) maxX = fx;
    else if (maxX - fx > 0.15) flag("backward", f, maxX - fx, `${(maxX - fx).toFixed(2)} m back the way it came`);
    // the hips' change of speed along the floor
    if (f >= 2) {
      const r0 = poseAt(f - 2).root, r1 = poseAt(f - 1).root;
      const a = Math.hypot(root[0] - 2 * r1[0] + r0[0], root[2] - 2 * r1[2] + r0[2]) * fps * fps;
      if (a > 30 * z) flag("pop", f, a, `hips jumped (${a.toFixed(0)} m/s² along the floor)`);
    }
    // things in hand
    if (live) {
      for (const id of Object.keys(course.items)) {
        if (!live.held(id, f)) continue;
        const it = live.item(id, f), sz = course.items[id].size, mid: Vec3 = [it.at[0], it.at[1] + sz[1] / 2, it.at[2]];
        const reach = Math.max(sz[0], sz[1], sz[2]) / 2 + 0.05;
        const d = Math.min(...(["L", "R"] as const).map((s) => Math.hypot(...([0, 1, 2].map((k2) => palmAt(skel, w, s)[k2] - mid[k2]) as [number, number, number]))));
        if (d > reach) flag("held", f, d - reach + 0.05, `${id} ${(d * 100).toFixed(0)} cm from the nearest palm`);
      }
      const m = live.miss(f);
      if (m > 0.04) flag("reach", f, m, `a hand ${(m * 100).toFixed(1)} cm short of its goal`);
    }
  }
  // flights: in the air through a walk longer than a stride's flight, or through an action not falling as things fall
  for (let f = 0; f < n; f++) {
    if (!air[f]) continue;
    let e = f;
    while (e + 1 < n && air[e + 1]) e++;
    const len = e - f + 1, kinds = new Set<string>();
    for (let k = f; k <= e; k++) kinds.add(stepAt(k)?.kind ?? "end");
    const overGap = course.gaps.some(([a, b]) => { const x0 = poseAt(f).root[0], x1 = poseAt(e).root[0]; return x1 > a && x0 < b; });
    if (!kinds.has("act") && len > 0.2 * fps && !overGap) flag("float", f, len / fps, `in the air ${(len / fps).toFixed(2)} s through a ${[...kinds].join("/")}`);
    else if (len >= 8) {
      // the hips' mean fall over the middle of the flight (its ends blur with take-off and landing)
      let acc = 0, c = 0;
      for (let k = f + 2; k <= e - 2; k++) { acc += (poseAt(k + 1).root[1] - 2 * poseAt(k).root[1] + poseAt(k - 1).root[1]) * fps * fps; c++; }
      if (c >= 3) {
        const off = Math.abs(acc / c + g) / g;
        if (off > 0.35) flag("float", f, off, `a ${(len / fps).toFixed(2)} s flight falling at ${(-acc / c).toFixed(1)} m/s² (gravity 9.8)`);
      }
    }
    f = e;
  }
  // throws: where the thing ended
  if (live) for (const t of course.throws) {
    const it = live.item(t.item, n - 1).at;
    if (typeof t.to === "string") {
      const p = course.props[t.to];
      const inside = Math.abs(it[0] - p.at[0]) < p.size[0] / 2 + 0.02 && Math.abs(it[2] - p.at[2]) < p.size[2] / 2 + 0.02 && it[1] <= p.at[1] + p.size[1] + 0.02;
      if (!inside) flag("throw", n - 1, Math.hypot(it[0] - p.at[0], it[2] - p.at[2]), `${t.item} ended ${Math.hypot(it[0] - p.at[0], it[2] - p.at[2]).toFixed(2)} m from the ${t.to}'s middle, not in it`);
    } else {
      // (where it first comes down: a ball rolls on from there)
      let k = 0;
      while (k < n - 1 && (live.held(t.item, k) || !live.item(t.item, k).at.some((v, i) => i !== 1 && v !== live.item(t.item, 0).at[i]))) k++;
      while (k < n - 1 && live.item(t.item, k).at[1] > 0.005) k++;
      const at = live.item(t.item, k).at, d = Math.hypot(at[0] - t.to[0], at[2] - t.to[2]);
      if (d > 0.3) flag("throw", k, d, `${t.item} came down ${d.toFixed(2)} m from where it was thrown to`);
    }
  }
  // the library's own limits on every joint's turn
  if (lib && perf.trace) {
    const tr = perf.trace, poses = Array.from({ length: n }, (_, f) => poseAt(f));
    const l = motionLint(lib, { ...perf, frames: n, trace: { ...tr, stages: { ...tr.stages, gaze: poses } } });
    // (past 1.2 times the library's own limit, which has its margin already: just over it doesn't show; not what the
    // capture does itself, the clip stage, a landing's jolt being a landing's; and not through a thing picked up, put
    // down or thrown, whose squat and throw are quicker than any walk and are checked as held, reach and throw)
    const beatSteps = new Set(course.plan.beats.map((b) => b.step));
    const inBeat = (f: number) => { const st = stepAt(f); return !!st && beatSteps.has(st.index); };
    for (const fd of l.findings) if (fd.kind !== "speed" && fd.stage !== "clip" && !inBeat(fd.peak) && fd.value / fd.limit > 1.2) raw.push({ kind: "lint", frame: fd.frame, last: fd.frame + fd.frames - 1, peak: fd.peak, amount: fd.value / fd.limit, what: `${fd.joint} ${fd.kind} ${(fd.value / fd.limit).toFixed(1)}x the library's (${fd.stage})` });
  }
  const endRoot = poseAt(n - 1).root;
  const finished = endRoot[0] >= course.end - 0.3;
  if (!finished) flag("unfinished", n - 1, course.end - endRoot[0], `stopped ${(course.end - endRoot[0]).toFixed(2)} m short of the end`);
  // runs of the same kind and thing within three frames as one flag at its worst
  raw.sort((a, b) => a.kind.localeCompare(b.kind) || a.what.split(" ")[0].localeCompare(b.what.split(" ")[0]) || a.frame - b.frame);
  const flags: Flag[] = [];
  const subject = (fl: Flag) => fl.kind + ":" + fl.what.replace(/[-\d.]+ ?(cm|m|s|x|m\/s²)?/g, "#").slice(0, 40);
  for (const fl of raw) {
    const prev = flags[flags.length - 1];
    if (prev && subject(prev) === subject(fl) && fl.frame - prev.last <= 3) {
      prev.last = Math.max(prev.last, fl.last);
      if (fl.amount > prev.amount) { prev.amount = fl.amount; prev.peak = fl.peak; prev.what = fl.what; }
    } else flags.push({ ...fl });
  }
  flags.sort((a, b) => a.frame - b.frame);
  const byFrame = new Map<number, Flag[]>();
  for (const fl of flags) for (let f = fl.frame; f <= fl.last; f++) { if (!byFrame.has(f)) byFrame.set(f, []); byFrame.get(f)!.push(fl); }
  return {
    name: course.spec.name, frames: n, flags,
    obstacles: course.obstacles.map((o, i) => ({ clip: o.clip, kind: o.kind, clearance: clear[i], frames: clearFrames[i] })),
    finished, pass: flags.length === 0,
    at: (f) => byFrame.get(f) ?? [],
  };
}

/** A check as lines: pass or fail, the obstacles' clearances, then every flag. */
export function courseReport(c: CourseCheck, o: { max?: number } = {}): string {
  const by = new Map<string, number>();
  for (const f of c.flags) by.set(f.kind, (by.get(f.kind) ?? 0) + 1);
  const head = `${c.pass ? "PASS" : "FAIL"} ${c.name}: ${c.frames} frames${c.flags.length ? `, ${c.flags.length} flags (${[...by].map(([k, v]) => `${k} ${v}`).join(", ")})` : ""}`;
  const obs = c.obstacles.map((ob) => `  ${ob.kind.padEnd(6)} ${ob.clip.padEnd(8)} frames ${ob.frames[0]}-${ob.frames[1]}  clearance ${ob.clearance === Infinity ? "-" : `${(ob.clearance * 100).toFixed(1)} cm`}`);
  const lines = c.flags.slice(0, o.max ?? 40).map((f) => `  ${f.kind.padEnd(10)} frame ${String(f.peak).padStart(4)}${f.last > f.frame ? ` (${f.frame}-${f.last})` : ""}  ${f.what}`);
  return [head, ...obs, ...lines, ...(c.flags.length > (o.max ?? 40) ? [`  ... ${c.flags.length - (o.max ?? 40)} more`] : [])].join("\n");
}
