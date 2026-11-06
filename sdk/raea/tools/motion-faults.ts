// Locomotion faults (docs/locomotion.md's taxonomy), measured on delivered poses only, so any pipeline is judged alike.
// Each measure that has no fixed bar is also taken on the library's own locomotion clips (`reference`): a case is
// read against what real capture does, not a guessed number.
//
//   skate   C3  foot horizontal speed x (2 - 2^(h/H)) under H = 2.5 cm (Holden et al.'s skating), cm a frame, mean
//   float   C4  frames whose slower foot (the standing one) is over 3 cm up, %
//   sink    C5  frames with a sole over 1 cm under the floor, %
//   reach   B1  frames with a hip-to-ankle span over 99.5% of the leg's length (a leg pulled straight), %
//   drift   R1  the hips from the guide's point, cm: mean and most
//   crab    R2  frames moving over 1 m/s with the hips facing over 25 degrees off the way they go, %
//   spin    R3  frames turning the hips over 90 degrees a second on two standing feet, %
//   overrun R4  at a stop, how far the hips came back along the way they were going, cm
//   thrash  T2  switches a second, and returns to a clip left under half a second before
//   flip    T3  switches where a standing foot of the clip before is a swinging foot of the clip after
//   stall   T5  frames in runs of 6 or more where the body moves over 0.5 m/s and no limb joint turns 0.2 degrees, %
//   armPhase U2 the larger of the two sides' correlation of the upper arm's swing with its own thigh's (natural < 0)
//   armBack U3  how far behind the trunk the upper arms swing, degrees (99th percentile, while moving)
//   head    U4  the head's pitch and roll range over a second, degrees (median over the moving seconds)
//   trunk   U5  the chest's pitch range over a second, degrees (median)
//   jitter  K2  the body joints' mean change of turn speed, degrees a frame squared
import { B, SIDES, joint, pose, qrot, type Pose, type Skeleton } from "raea/character";
import { clipFrame } from "raea/clip";
import type { MotionLibrary, Performance } from "raea/matching";

type World = ReturnType<typeof pose>;
type V = number[];
const DEG = 180 / Math.PI;
const sub = (a: V, b: V) => a.map((x, i) => x - b[i]);
const len = (a: V) => Math.hypot(...a);
const dot = (a: V, b: V) => a.reduce((s, x, i) => s + x * b[i], 0);
const pct = (xs: number[], p: number) => { if (!xs.length) return 0; const s = [...xs].sort((a, b) => a - b); return s[Math.min(s.length - 1, Math.floor(p * s.length))]; };
const median = (xs: number[]) => pct(xs, 0.5);
const mean = (xs: number[]) => (xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : 0);
function corr(a: number[], b: number[]) {
  const ma = mean(a), mb = mean(b);
  let s = 0, sa = 0, sb = 0;
  for (let i = 0; i < a.length; i++) { s += (a[i] - ma) * (b[i] - mb); sa += (a[i] - ma) ** 2; sb += (b[i] - mb) ** 2; }
  return sa > 0 && sb > 0 ? s / Math.sqrt(sa * sb) : 0;
}

/** the bits of a skeleton the measures use */
function rig(skel: Skeleton) {
  const J = skel.joints, j = (n: string) => joint(skel, n), y = (n: string) => J[j(n)].head[1];
  const soles = SIDES.flatMap((s) => [B.foot(s), B.ball(s)].map((n) => ({ s, i: j(n), drop: [0, -y(n), 0] })));
  const legLen = (s: (typeof SIDES)[number]) => len(sub(J[j(B.thigh(s))].head, J[j(B.calf(s))].head)) + len(sub(J[j(B.calf(s))].head, J[j(B.foot(s))].head));
  const limbs = SIDES.flatMap((s) => [B.thigh(s), B.calf(s), B.upperarm(s), B.lowerarm(s)].map(j));
  const body = J.map((x, i) => i).filter((i) => !/_0[123]_/.test(J[i].name));
  return { j, soles, legLen, limbs, body };
}

/** a sole point (the floor under a foot joint, turned as it turns) */
const solePos = (w: World, s: { i: number; drop: number[] }) => sub(w.pos[s.i], [0, 0, 0]).map((x, k) => x + (qrot(w.rot[s.i], s.drop as any) as V)[k]);
/** a joint's axis in the world */
const axis = (w: World, i: number, a: V) => qrot(w.rot[i], a as any) as V;
/** the hips' heading (radians) */
const heading = (w: World, hip: number) => { const f = axis(w, hip, [0, 0, 1]); return Math.atan2(f[0], f[2]); };
const wrap = (a: number) => Math.atan2(Math.sin(a), Math.cos(a));

/** per-frame turn speed of every joint, degrees a frame */
function turnSpeeds(W: World[]): number[][] {
  return W.map((w, f) => w.rot.map((q, i) => {
    if (!f) return 0;
    const p = W[f - 1].rot[i], d = Math.abs(q[0] * p[0] + q[1] * p[1] + q[2] * p[2] + q[3] * p[3]);
    return 2 * Math.acos(Math.min(1, d)) * DEG;
  }));
}

/** the measures any stretch of poses has (also taken on the library's clips) */
export function bodyMeasures(skel: Skeleton, W: World[], fps: number) {
  const R = rig(skel), hip = R.j(B.pelvis), chest = R.j(B.chest), head = R.j(B.head);
  const sp = turnSpeeds(W);
  const hipV = W.map((w, f) => { const a = W[Math.max(0, f - 2)], b = W[Math.min(W.length - 1, f + 2)], n = Math.min(W.length - 1, f + 2) - Math.max(0, f - 2); return n > 0 ? (Math.hypot(b.pos[hip][0] - a.pos[hip][0], b.pos[hip][2] - a.pos[hip][2]) * fps) / n : 0; });
  const moving = hipV.map((v) => v > 0.5);
  // U2, U3: each side's upper arm and thigh swing, as angles fore (+) and aft in the hips' sagittal plane
  const child = (n: string) => skel.joints.findIndex((x) => x.parent === R.j(n));
  const swingOf = (w: World, n: string) => { const i = R.j(n), c = child(n); const d = sub(w.pos[c], w.pos[i]), f = axis(w, hip, [0, 0, 1]); return Math.atan2(dot(d, f), -d[1]) * DEG; };
  let armPhase = -1, armBack: number[] = [];
  for (const s of SIDES) {
    const a: number[] = [], t: number[] = [];
    W.forEach((w, f) => { if (!moving[f]) return; a.push(swingOf(w, B.upperarm(s))); t.push(swingOf(w, B.thigh(s))); });
    if (a.length > 30) { armPhase = Math.max(armPhase, corr(a, t)); armBack.push(...a.map((x) => -x)); }
  }
  // U4, U5: head pitch and roll, chest pitch, as ranges over one-second windows while moving
  const pitch = (w: World, i: number) => Math.asin(Math.max(-1, Math.min(1, axis(w, i, [0, 0, 1])[1]))) * DEG;
  const roll = (w: World, i: number) => Math.asin(Math.max(-1, Math.min(1, axis(w, i, [1, 0, 0])[1]))) * DEG;
  const win = Math.round(fps), hp: number[] = [], hr: number[] = [], cp: number[] = [];
  for (let f = 0; f + win <= W.length; f += Math.max(1, win >> 1)) {
    if (moving.slice(f, f + win).filter(Boolean).length < win * 0.9) continue;
    const range = (g: (w: World) => number) => { const v = W.slice(f, f + win).map(g); return Math.max(...v) - Math.min(...v); };
    hp.push(range((w) => pitch(w, head))); hr.push(range((w) => roll(w, head))); cp.push(range((w) => pitch(w, chest)));
  }
  // B1: the gait's Froude number v^2/(g L) (L the hips' height at rest; people walk near 0.25, change to a run near
  // 0.5) and its cadence (steps a second, from the thighs' swing crossing each other), over the moving frames: a body
  // of another build than the capture's should keep the Froude number at its own speed, its cadence as 1/sqrt(L)
  const L = skel.joints[hip].head[1];
  const fwdOf = (w: World) => axis(w, hip, [0, 0, 1]);
  const gap = W.map((w) => { const f = fwdOf(w), a = w.pos[child(B.thigh("L"))], b = w.pos[child(B.thigh("R"))]; return dot(sub(a, b), f); });
  let steps = 0, movingN = 0, v2 = 0;
  for (let f = 1; f < W.length; f++) if (moving[f]) { movingN++; v2 += hipV[f] ** 2; if (Math.sign(gap[f]) !== Math.sign(gap[f - 1])) steps++; }
  const froude = movingN ? v2 / movingN / (9.81 * L) : 0, cadence = movingN ? (steps * fps) / movingN : 0;
  // K2: change of turn speed over the body's joints
  const jit: number[] = [];
  for (let f = 2; f < sp.length; f++) jit.push(mean(R.body.map((i) => Math.abs(sp[f][i] - sp[f - 1][i]))));
  return {
    armPhase, armBack: pct(armBack, 0.99), headPitch: median(hp), headRoll: median(hr), trunk: median(cp), jitter: mean(jit), froude, cadence,
    _sp: sp, _moving: moving, _hipV: hipV,
  };
}

/** the faults of one delivered performance */
export function faults(skel: Skeleton, lib: MotionLibrary, perf: Performance, W: World[]) {
  const fps = lib.fps, R = rig(skel), hip = R.j(B.pelvis), N = W.length;
  const m = bodyMeasures(skel, W, fps), sp = m._sp;
  // C3 to C5 and R3: the soles
  const H = 0.025;
  let skate = 0, skN = 0, float = 0, sink = 0, spin = 0;
  const down = (w: World, p: World | undefined, s: (typeof R.soles)[number]) => { const a = solePos(w, s), b = p ? solePos(p, s) : a; return { h: a[1], v: Math.hypot(a[0] - b[0], a[2] - b[2]) }; };
  for (let f = 1; f < N; f++) {
    const st = R.soles.map((s) => down(W[f], W[f - 1], s));
    for (const x of st) { if (x.h < H) skate += x.v * (2 - 2 ** (Math.max(0, x.h) / H)); skN++; }
    if (st.some((x) => x.h < -0.01)) sink++;
    // the standing foot: each side's lower sole, and the slower side's
    const side = SIDES.map((_, k) => ({ h: Math.min(st[k * 2].h, st[k * 2 + 1].h), v: Math.min(st[k * 2].v, st[k * 2 + 1].v) }));
    const stand = side[0].v <= side[1].v ? side[0] : side[1];
    if (stand.v < 0.005 && stand.h > 0.03 && !perf.guide[f]?.act) float++;
    if (side.every((x) => x.h < 0.015 && x.v < 0.005) && Math.abs(wrap(heading(W[f], hip) - heading(W[f - 1], hip))) * fps * DEG > 90) spin++;
  }
  // B1: a leg's span against its length
  let reach = 0;
  for (const w of W) if (SIDES.some((s) => len(sub(w.pos[R.j(B.foot(s))], w.pos[R.j(B.thigh(s))])) > 0.995 * R.legLen(s))) reach++;
  // R1, R2: the hips against the guide, and against the way they go
  const G = perf.guide, dr: number[] = [];
  let crab = 0;
  for (let f = 0; f < N; f++) {
    const g = G[Math.min(G.length - 1, f)];
    if (!g.act) dr.push(Math.hypot(W[f].pos[hip][0] - g.x, W[f].pos[hip][2] - g.z));
    if (m._hipV[f] > 1 && f >= 2 && f + 2 < N) {
      const a = W[f - 2].pos[hip], b = W[f + 2].pos[hip], go = Math.atan2(b[0] - a[0], b[2] - a[2]);
      if (Math.abs(wrap(heading(W[f], hip) - go)) * DEG > 25) crab++;
    }
  }
  // R4: at each stop (the guide moving, then not), the hips' most travel back along the way they came, over 1.5 s
  let overrun = 0;
  for (let f = 1; f < Math.min(N, G.length); f++) {
    if (!(G[f - 1].moving && !G[f].moving) || G[f].act) continue;
    const a = W[Math.max(0, f - 6)].pos[hip], b = W[f].pos[hip], d = [b[0] - a[0], b[2] - a[2]], n = Math.hypot(d[0], d[1]) || 1;
    let far = -Infinity, end = 0;
    for (let k = f; k < Math.min(N, f + Math.round(1.5 * fps)); k++) { const p = W[k].pos[hip], s = ((p[0] - b[0]) * d[0] + (p[2] - b[2]) * d[1]) / n; far = Math.max(far, s); end = s; }
    overrun = Math.max(overrun, far - end);
  }
  // T2, T3: switches
  const P = perf.played, at: number[] = [];
  for (let f = 1; f < P.length; f++) if (P[f].clip !== P[f - 1].clip || Math.abs(P[f].f - P[f - 1].f) > 2) at.push(f);
  const left = new Map<string, number>();
  let returns = 0, flips = 0;
  const byName = new Map(lib.clips.map((c) => [c.name + (c.mirrored ? "~" : ""), c]));
  const contactOf = (p: { clip: string; f: number }) => { const c = byName.get(p.clip); if (!c) return null; const k = c.start + p.f; return k < c.end ? [lib.contact[k * 2], lib.contact[k * 2 + 1]] : null; };
  for (const s of at) {
    const was = P[s - 1], now = P[s];
    if (was.clip !== now.clip) { const l = left.get(now.clip); if (l !== undefined && s - l < fps / 2) returns++; left.set(was.clip, s); }
    const a = contactOf(was), b = contactOf(now);
    if (a && b && ((a[0] === 1 && b[0] === 0) || (a[1] === 1 && b[1] === 0))) flips++;
  }
  // T5: limbs frozen while the body moves
  let stall = 0, run = 0;
  for (let f = 1; f < N; f++) {
    const frozen = m._hipV[f] > 0.5 && R.limbs.every((i) => sp[f][i] < 0.2);
    run = frozen ? run + 1 : 0;
    if (run === 6) stall += 6; else if (run > 6) stall++;
  }
  const P100 = (n: number) => (100 * n) / Math.max(1, N);
  return {
    skate: (100 * skate) / Math.max(1, skN), float: P100(float), sink: P100(sink), reach: P100(reach),
    drift: mean(dr) * 100, driftMax: Math.max(0, ...dr) * 100, crab: P100(crab), spin: P100(spin), overrun: overrun * 100,
    switchesPerS: (at.length * fps) / Math.max(1, N), returns, flips, stall: P100(stall),
    armPhase: m.armPhase, armBack: m.armBack, headPitch: m.headPitch, headRoll: m.headRoll, trunk: m.trunk, jitter: m.jitter, froude: m.froude, cadence: m.cadence,
  };
}

/** the same body measures over the library's own locomotion clips (unmirrored): what real capture does */
export function reference(skel: Skeleton, lib: MotionLibrary, names: string[]) {
  const rows = lib.clips.filter((c) => !c.mirrored && names.includes(c.name)).map((c) => {
    const nj = c.clip.turns.length / (c.clip.frames * 4);
    const W = Array.from({ length: c.clip.frames }, (_, k) => pose(skel, clipFrame(c.clip, k, nj) as Pose));
    return bodyMeasures(skel, W, lib.fps);
  });
  const keys = ["armPhase", "armBack", "headPitch", "headRoll", "trunk", "jitter", "froude", "cadence"] as const;
  return Object.fromEntries(keys.map((k) => [k, median(rows.map((r) => r[k]).filter((x) => Number.isFinite(x) && x !== -1))])) as Record<(typeof keys)[number], number>;
}
