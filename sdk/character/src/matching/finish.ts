// perform's lookups and passes: each frame's asks, a switch's speed, still clips smoothed, an action's feet, the gaze.
import type { Vec3 } from "raea/3d";
import { B, bodyScale, joint, lookAt, pose, qmul, qnorm, qrot, qyaw, setWorld, type Pose, type Quat, type Skeleton } from "raea/character";
import { type RigEffect } from "../effects";
import { MotionLibrary, Performance, PerformanceSpec, Step, TagCond, crouched, guide, labelCost, stateMix, stateQuery, stepTags, swingCost, wrap } from "./all";

type Guide = ReturnType<typeof guide>;

/** what each frame asks of the library, by the guide's step at it (the spec's where a step says none) */
export function stepLookups(lib: MotionLibrary, spec: PerformanceSpec, G: Guide, ws: number) {
  const skel = lib.skel;
  const gAt = (k: number) => G[Math.min(G.length - 1, Math.max(0, k))];
  // each step's state as the library's z-scores (the spec's where a step says none)
  const wants = spec.steps.map((st) => { const s = ("state" in st && st.state) || spec.state; return s ? stateQuery(lib, s, "walk" in st) : undefined; });
  const after = spec.state ? stateQuery(lib, spec.state, false) : undefined;
  const wantAt = (t: number) => { const i = gAt(t).step; return t < G.length && i !== undefined ? wants[i] : after; };
  // (and a run's walk steps kept off clips well under its pace: a walk played at a run's 2.2 m/s stood the body up in
  // the middle of a run, 08_01 between a hurdle and the box, Shane 10-22: "drift or lose calculation")
  const clipPace = lib.clips.map((r) => { let d = 0; for (let f = r.start + 1; f < r.end; f++) d += Math.hypot(lib.ground[f * 3] - lib.ground[f * 3 - 3], lib.ground[f * 3 + 1] - lib.ground[f * 3 - 2]); return r.end - r.start > 1 ? (d * lib.fps) / (r.end - r.start - 1) : 0; });
  const gait = (st: Step, p?: Float32Array) => {
    const v = "walk" in st ? (st.speed ?? 0) : 0;
    // (not a short run between obstacles: a step or two checked to come in on the right foot is how people run them)
    const len = "walk" in st ? st.walk.reduce((a, w, i) => a + (i ? Math.hypot(w[0] - st.walk[i - 1][0], w[1] - st.walk[i - 1][1]) : 0), 0) : 0;
    // (nor one starting from a stand or coming to one, a walk the way to speed up and slow down)
    // (the paces by the body's size against the child's they were set on)
    const zb = bodyScale(lib.skel);
    if (v < 1.8 * zb || len < 1.5 * zb || ("walk" in st && ((st.leave ?? v) < 1.5 * zb || (st.enter ?? v) < 1.5 * zb))) return p;
    const q = p ? Float32Array.from(p) : new Float32Array(lib.clips.length);
    clipPace.forEach((c, i) => { if (c < 0.7 * v) q[i] += 20; });
    return q;
  };
  const pens = spec.steps.map((st) => { const s = ("state" in st && st.state) || spec.state; return gait(st, s ? labelCost(lib, s, ws) : undefined); });
  const penAfter = spec.state ? labelCost(lib, spec.state, ws) : undefined;
  const penAt = (t: number) => { const i = gAt(t).step; return t < G.length && i !== undefined ? pens[i] : penAfter; };
  // a wait's frames kept out of a capture's crouch (a walk's too pulled the woman's runs about: 6 more flags on levels 4-5)
  const slow = spec.steps.map((st) => "wait" in st);
  const maskAt = (t: number) => { const i = gAt(t).step; return i !== undefined && slow[i] ? crouched(lib) : undefined; };
  // a step holding things kept to frames whose arm swing keeps each one clear of the thigh and the waist (by a
  // centimetre more than its half width; when no frame is, the clearance is let go)
  // (walks at a walking pace only: a run's arm swings wide, and a run's frames kept clear of a ball carried in one hand
  // left the kid's carry over a hurdle and the box little to run from, 4 skates and a float; a wait's arm hardly
  // swings, and the frames left to the kid's patient look at a ball slid both feet, 4 skates)
  const swings = spec.steps.map((st) => {
    const c = "walk" in st && (st.speed ?? 0) < 1.8 * bodyScale(skel) ? st.limbs?.clear : undefined;
    return c ? swingCost(lib, Object.fromEntries(Object.entries(c).map(([s, v]) => [s, v! + 0.01]))) : undefined;
  });
  const swingAt = (t: number) => { const i = gAt(t).step; return i !== undefined && t < G.length ? swings[i] : undefined; };
  // each step's tag conditions (a database library's): its own and the spec's, and the defaults of its kind
  const conds = spec.steps.map((st) => stepTags(lib, st, spec.tags));
  const condAt = (t: number) => { const i = gAt(t).step; return i !== undefined && t < G.length ? conds[i] : stepTags(lib, undefined, spec.tags); };
  // the upper body's own conditions (limbs.upper): the step's with the upper's tags added (none on a library without tags)
  const upperConds = spec.steps.map((st) => {
    if (!("walk" in st || "wait" in st) || !st.limbs?.upper || !lib.db) return undefined;
    const a = st.tags, b = st.limbs.upper.tags;
    return stepTags(lib, { ...st, tags: { want: [...(a?.want ?? []), ...(b?.want ?? [])], forbid: [...(a?.forbid ?? []), ...(b?.forbid ?? [])], cost: { ...a?.cost, ...b?.cost } } }, spec.tags) ?? ({ want: [0, 0], forbid: [0, 0] } as TagCond);
  });
  const upperAt = (t: number) => { const g2 = gAt(t), i = g2.step; return i !== undefined && t < G.length && !g2.act ? upperConds[i] : undefined; };
  const upperHands = (t: number) => { const i = gAt(t).step, st = i !== undefined ? spec.steps[i] : undefined; return lib.hands && st && ("walk" in st || "wait" in st) ? st.limbs?.upper?.hands : undefined; };
  // the effects' target: each step's state as a mix of styles, eased from step to step (a third of a second)
  const mixes = spec.steps.map((st) => { const s = ("state" in st && st.state) || spec.state; return s ? stateMix(s) ?? {} : {}; });
  const mixAfter = spec.state ? stateMix(spec.state) ?? {} : {};
  const mixAt = (t: number) => { const i = gAt(t).step; return t < G.length && i !== undefined ? mixes[i] : mixAfter; };
  return { wantAt, penAt, maskAt, swingAt, condAt, upperAt, upperHands, mixAt };
}

// the hips' speed along the floor carried through each switch and eased out over 0.4 s, as the pose is blended: the
// new frame's own speed straight away jolted the hips (a run switched into a slower capture slowed 1 m/s in a frame,
// 30 m/s²)
export function carrySpeed(raw: Pose[], switches: number[], fps: number) {
  const L = Math.round(0.4 * fps);
  // (an action's first frame is placed on at the speed come in with, so its jump is a frame later)
  const jumpAt = (t: number) => {
    const r0 = raw[t - 2].root, r1 = raw[t - 1].root, r2 = raw[t].root;
    return [(r1[0] - r0[0]) - (r2[0] - r1[0]), (r1[2] - r0[2]) - (r2[2] - r1[2])];
  };
  for (const ts of switches) {
    if (ts < 2 || ts + 1 >= raw.length) continue;
    const a = jumpAt(ts), b = jumpAt(ts + 1), t0 = Math.hypot(b[0], b[1]) > Math.hypot(a[0], a[1]) ? ts + 1 : ts, dv = t0 > ts ? b : a;
    // (none left by the last frame: a performance still ends where it was sent)
    const len = Math.min(L, raw.length - 1 - t0);
    if (len < 2) continue;
    for (let s = 0; s <= len; s++) {
      // (the offset the jump was a frame, going on at it a frame, out to none still: a cubic between)
      const v = s / len, h = 2 * v ** 3 - 3 * v * v + 1 + (v ** 3 - 2 * v * v + v) * len, r = raw[t0 + s].root;
      raw[t0 + s] = { ...raw[t0 + s], root: [r[0] + dv[0] * h, r[1], r[2] + dv[1] * h] };
    }
  }
}

// a clip that stays where it is (a throw, a pick-up) smoothed (twice, a quarter, a half, a quarter: its hips' way and
// every joint's turn): its capture's one-frame jolts showed as hitches (a pick-up's hips 7 mm out of line for a
// frame, its back with them), where an obstacle's are take-offs and landings, kept (the legs too: the hips smoothed
// over legs as they were slid the planted feet)
export function smoothStill(raw: Pose[], G: Guide) {
  for (let pass = 0; pass < 2; pass++) {
    const was = raw.slice();
    for (let t = 1; t + 1 < raw.length; t++) {
      const P = G[t]?.act?.play;
      if (!P?.still || G[t - 1]?.act?.play !== P || G[t + 1]?.act?.play !== P) continue;
      const [a, b, c] = [was[t - 1], was[t], was[t + 1]];
      const turns = b.turns.map((q, i) => {
        const qa = a.turns[i], qc = c.turns[i], sg = (r: Quat) => (r[0] * q[0] + r[1] * q[1] + r[2] * q[2] + r[3] * q[3] < 0 ? -1 : 1), sa = sg(qa), sc = sg(qc);
        return qnorm([0, 1, 2, 3].map((k) => 0.25 * sa * qa[k] + 0.5 * q[k] + 0.25 * sc * qc[k]) as Quat);
      });
      raw[t] = { root: [0.25 * a.root[0] + 0.5 * b.root[0] + 0.25 * c.root[0], 0.25 * a.root[1] + 0.5 * b.root[1] + 0.25 * c.root[1], 0.25 * a.root[2] + 0.5 * b.root[2] + 0.25 * c.root[2]], turns };
    }
  }
}

// an action clip's standing feet held by the body: where the capture slides a foot it says is down (a roll landing
// still going, 15 cm), the whole body is moved back by the slide, the leg as the clip has it (held by the leg it
// folded 20° past the clip's), and the offset eased out once no foot slides (the slower foot's, both down)
export function holdActionFeet(skel: Skeleton, raw: Pose[], G: Guide, contacts: [number, number][]) {
  const lows = B.feet().map((nm) => joint(skel, nm));
  const lowOf = (t: number, s: number) => { const w = pose(skel, raw[t]); const a = w.pos[lows[s]], b = w.pos[lows[s + 2]]; return a[1] - skel.joints[lows[s]].head[1] < b[1] - skel.joints[lows[s + 2]].head[1] ? a : b; };
  let off: [number, number] = [0, 0], from: [number, number] = [0, 0], since = -1;
  const EASE = 10;
  let prev: Vec3[] | null = null;
  for (let t = 0; t < raw.length; t++) {
    const now = [lowOf(t, 0), lowOf(t, 1)];
    let slide: [number, number] | null = null;
    // (landing only, past the obstacle: a slide at the take-off held would move the body into what it clears; and
    // none in a clip that stays where it is: a throw's front foot pivots on its ball, and held, jolted the hips)
    const A = G[t]?.act, landing = !!A?.play && !A.play.still && A.play === G[t - 1]?.act?.play && A.play.from + A.j > A.play.anchor;
    if (prev && landing) for (const s of [0, 1]) {
      if (contacts[t][s] !== 3 || contacts[t - 1][s] !== 3) continue;
      const d: [number, number] = [now[s][0] - prev[s][0], now[s][2] - prev[s][2]];
      if (!slide || Math.hypot(d[0], d[1]) < Math.hypot(slide[0], slide[1])) slide = d;
    }
    prev = now;
    if (slide && Math.hypot(slide[0], slide[1]) > 0.004 && Math.hypot(slide[0], slide[1]) < 0.04) { off = [off[0] - slide[0], off[1] - slide[1]]; since = -1; }
    else if (off[0] || off[1]) {
      if (since < 0) { since = 0; from = off; }
      since++;
      const k = since >= EASE ? 0 : 1 - smooth(since / EASE);
      off = [from[0] * k, from[1] * k];
    }
    if (off[0] || off[1]) { const r = raw[t].root; raw[t] = { ...raw[t], root: [r[0] + off[0], r[1], r[2] + off[1]] }; }
  }
}

/** the sit a frame is in, and the step */
export function stages(G: Guide, spec: PerformanceSpec, fps: number): Pick<Performance, "phase" | "step"> {
  const phase: Performance["phase"] = (frame) => {
    const a = G[Math.min(G.length - 1, Math.max(0, Math.round(frame)))]?.act;
    // (a sit's only: an action clip's play, a pick-up or a throw, is an act with no seat)
    if (!a || !a.seat || Math.round(frame) >= G.length) return null;
    const [d, s] = a.len, j = a.j;
    const stage = j < d ? "down" : j < d + s ? "sat" : "up";
    const at = stage === "down" ? j : stage === "sat" ? j - d : j - d - s, n = a.len[stage === "down" ? 0 : stage === "sat" ? 1 : 2];
    return { action: "sit", stage, u: at / n, since: at / fps, left: (n - at) / fps };
  };
  const step: Performance["step"] = (frame) => {
    const f = Math.round(frame), k = G[f]?.step;
    if (k === undefined) return null;
    let a = f, b = f;
    while (a > 0 && G[a - 1].step === k) a--;
    while (b + 1 < G.length && G[b + 1].step === k) b++;
    const st = spec.steps[k];
    return { index: k, kind: "walk" in st ? "walk" : "wait" in st ? "wait" : "act" in st ? "act" : "sit", since: (f - a) / fps, left: (b + 1 - f) / fps };
  };
  return { phase, step };
}

export const smooth = (u: number) => {
  const t = Math.min(1, Math.max(0, u));
  return t * t * (3 - 2 * t);
};

/** The feet pinned: while a foot stands its ankle stays where it came down (the legs bent to it). It is let go over a
 * few frames as it lifts, or early if the body has drifted from it (a pin held too far straightens the leg and snaps
 * the knee). */
/** The head turned ahead into a turn: a walker looks round a corner, or to where they will face, about two thirds of a
 * second before the body gets there. The face turns by most of the way the guide will have turned by then (up to
 * about 45°), about up only, so the clip's own nods stay. */
export function leadGaze(skel: Skeleton, poses: Pose[], G: Performance["guide"]): Pose[] {
  const head = joint(skel, B.head), neck = joint(skel, B.neck), eyes = skel.landmarks.eyes, lead = 20;
  // the lead each frame, then smoothed over a third of a second either side: read raw it jumped where one step ends
  // and the next begins, and the head jerked (the motion lint's finding at the end of every walk)
  const want = poses.map((_, f) => {
    const g = G[Math.min(G.length - 1, f)], ahead = G[Math.min(G.length - 1, f + lead)];
    if (!g || !ahead || g.act || ahead.act) return 0;
    const d = wrap(ahead.yaw - g.yaw);
    return Math.sign(d) * Math.min(Math.abs(d), 1) * 0.8;
  });
  const R = 10;
  const turns = want.map((_, f) => { let s = 0, sw = 0; for (let k = -R; k <= R; k++) { const w = 1 + Math.cos((Math.PI * k) / (R + 1)); s += w * want[Math.max(0, Math.min(want.length - 1, f + k))]; sw += w; } return s / sw; });
  return poses.map((p, f) => {
    const g = G[Math.min(G.length - 1, f)], turn = turns[f];
    if (g?.act || Math.abs(turn) < 0.002) return p;
    // the turn about the world's up, half at the neck and half at the head (RAEA_GAZE_ARC=1: the old look-at, whose
    // shortest arc from a face pitched down to the same pitch turned aside tilts the head: turn 90's head rolled 10.7
    // degrees over a second where the clips roll 5)
    if (process.env.RAEA_GAZE_ARC !== "1") {
      let q = p;
      for (const j of [neck, head]) q = setWorld(skel, q, pose(skel, q), j, qmul(qyaw(turn / 2), pose(skel, q).rot[j]));
      return q;
    }
    const w = pose(skel, p);
    const at = add3(w.pos[head], qrot(w.rot[head], [eyes[0] - skel.joints[head].head[0], eyes[1] - skel.joints[head].head[1], eyes[2] - skel.joints[head].head[2]]));
    const fwd = qrot(qyaw(turn), qrot(w.rot[head], [0, 0, 1]));
    return lookAt(skel, p, [at[0] + 2 * fwd[0], at[1] + 2 * fwd[1], at[2] + 2 * fwd[2]]);
  });
}

export const add3 = (a: Vec3, b: Vec3): Vec3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];

const strides = new Map<string, RigEffect>();

/** A walk's stride lengthened by `k` as a rig effect: the thighs' swing about the body's left-right axis (their
 * flexion) widened by k and the knees' by its square root, nothing else changed (rounded to 1% so it caches). */
export function strideEffect(skel: Skeleton, k: number): RigEffect {
  const kk = Math.round(k * 100) / 100, key = `${skel.name}:${kk}`;
  const hit = strides.get(key);
  if (hit) return hit;
  const e: RigEffect = {
    posture: skel.joints.map(() => [0, 0, 0] as Vec3),
    amplitude: skel.joints.map((j) => (/^thigh_/.test(j.name) ? [kk, 1, 1] : /^calf_/.test(j.name) ? [Math.sqrt(kk), 1, 1] : [1, 1, 1]) as Vec3),
    rate: 1,
  };
  strides.set(key, e);
  return e;
}

/** whether RAEA_LINT is set (every performance lints itself) */
export const lintEnv = () => { const g = globalThis as { process?: { env?: Record<string, string | undefined> } }; return !!g.process?.env?.RAEA_LINT; };

/** The matched motion's joints smoothed over time (the binomial mean of the five frames round each, twice): a switch's
 * blend and the library's frame steps left the toes and feet snapping 10 to 25 degrees in a frame, which the planting
 * that follows does not undo. The root's place and turn are the guide's and stay. */
export function smoothTurns(raw: Pose[], passes = 2) {
  const TAPS: [number, number][] = [[-2, 1], [-1, 4], [0, 6], [1, 4], [2, 1]];
  for (let pass = 0; pass < passes; pass++) {
    const was = raw.slice(), n = raw.length;
    for (let t = 0; t < n; t++) {
      const turns = was[t].turns.map((c, i) => {
        if (i === 0) return c;
        const m = [0, 0, 0, 0];
        for (const [d, w] of TAPS) {
          const q = was[Math.min(n - 1, Math.max(0, t + d))].turns[i], sg = q[0] * c[0] + q[1] * c[1] + q[2] * c[2] + q[3] * c[3] < 0 ? -w : w;
          for (let a = 0; a < 4; a++) m[a] += sg * q[a];
        }
        return qnorm(m as Quat);
      });
      raw[t] = { ...was[t], turns };
    }
  }
}
