// The body: its parts from the skeleton, the sim, and ragdoll().
import { add, blend, captured, cross, dot, joint, len, norm, pose as forward, qbasis, qfromTo, qinv, qmul, qnorm, qrot, scale3, sub, surface, type Pose, type Skeleton } from "../character";
import { Air, Body, Box, ID, Joint, Q, RagdollOptions, Skin, Sphere, V3, World, angles, clamp, frameFrom, irot, qlog, relative, skinAt, step, swingPose } from "./all";

// ---------------------------------------------------------------- the body

/** the parts: name, lead joint, share of the body's mass (de Leva's segments; the hand with the forearm) */
const PARTS: [string, string, number][] = [
  ["pelvis", "Hips", 0.112], ["abdomen", "LowerBack", 0.163], ["chest", "Spine1", 0.16], ["head", "Neck", 0.069],
  ["upperarm.L", "LeftArm", 0.027], ["lowerarm.L", "LeftForeArm", 0.022], ["upperarm.R", "RightArm", 0.027], ["lowerarm.R", "RightForeArm", 0.022],
  ["thigh.L", "LeftUpLeg", 0.142], ["shin.L", "LeftLeg", 0.043], ["foot.L", "LeftFoot", 0.014],
  ["thigh.R", "RightUpLeg", 0.142], ["shin.R", "RightLeg", 0.043], ["foot.R", "RightFoot", 0.014],
];

/** each joint's limits (radians), heros-journey's: swing about x (flexion), about z (abduction, + outwards on the left),
 * twist about y; "world" joints measured in the body's frame, "limb" joints with y up the limb to its parent. The
 * reference (no turn) is the body standing, arms hanging at its sides. */
type Lim = { x: [number, number]; z: [number, number]; y: [number, number]; k: number; frame: "world" | "limb" };

const LIMITS: Record<string, Lim> = {
  abdomen: { x: [-0.3, 0.6], z: [-0.3, 0.3], y: [-0.3, 0.3], k: 3.5, frame: "world" },
  chest: { x: [-0.25, 0.45], z: [-0.25, 0.25], y: [-0.35, 0.35], k: 3, frame: "world" },
  head: { x: [-0.6, 0.7], z: [-0.5, 0.5], y: [-0.9, 0.9], k: 0.9, frame: "world" },
  upperarm: { x: [-2.6, 0.9], z: [-0.4, 2.2], y: [-1.2, 1.2], k: 0.35, frame: "limb" },
  lowerarm: { x: [-2.5, 0.02], z: [-0.02, 0.02], y: [-0.6, 0.6], k: 0.2, frame: "limb" },
  thigh: { x: [-2.0, 0.5], z: [-0.35, 0.9], y: [-0.6, 0.6], k: 1.6, frame: "limb" },
  shin: { x: [-0.03, 2.4], z: [-0.02, 0.02], y: [-0.1, 0.1], k: 1.2, frame: "limb" },
  foot: { x: [-0.5, 0.8], z: [-0.35, 0.35], y: [-0.3, 0.3], k: 0.6, frame: "world" },
};

/** where the muscles' tone pulls each joint: the body lying slack, elbows and knees a little bent */
const RELAXED: Record<string, Q> = { lowerarm: swingPose(-0.35, 0, 0), shin: swingPose(0.25, 0, 0), thigh: swingPose(-0.1, 0, 0) };

/** N m per radian for a joint whose k is 1, on a 60 kg body (heros-journey's), and its most torque */
const STIFFNESS = 260, STRENGTH = 180;

/** each joint's damping, as a share of critical at heros-journey's full muscle: limp, but not a jelly */
const DAMPING = 0.35;

const INERTIA_FLOOR = 0.2;

/** the hands hang this far out from straight down in the reference (radians) */
const HANG = 0.15;

/** the skin samples a part touches with, and their radius (m) */
const SKIN_SAMPLES = 48, SKIN_R = 0.012;

/** how far off the skin the samples touch (m): between samples the skin is a few millimetres nearer the floor */
const SKIN_GAP = 0.004;

const LEG_PAIRS: [string, string][] = [["thigh.L", "thigh.R"], ["shin.L", "shin.R"], ["foot.L", "foot.R"], ["foot.L", "shin.R"], ["foot.R", "shin.L"]];

const ARM_AGAINST = ["pelvis", "abdomen", "chest", "head", "thigh.L", "thigh.R", "shin.L", "shin.R"];

/** asleep: every part under these speeds (m/s at its centre, rad/s) for this long (s) */
const SLEEP_V = 0.06, SLEEP_W = 0.6, SLEEP_FOR = 0.4;

interface Part { name: string; lead: number; mass: number; c: V3; inertia: V3; hull: Sphere[] }

/** farthest-point sampling: k of the points, spread over them (deterministic: from the first) */
function spread<T extends { p: V3 }>(pts: T[], k: number): T[] {
  if (pts.length <= k) return pts;
  const out = [pts[0]], d = pts.map((x) => len(sub(x.p, pts[0].p)));
  while (out.length < k) {
    let bi = 0;
    for (let i = 1; i < pts.length; i++) if (d[i] > d[bi]) bi = i;
    out.push(pts[bi]);
    for (let i = 0; i < pts.length; i++) d[i] = Math.min(d[i], len(sub(pts[i].p, pts[bi].p)));
  }
  return out;
}

/** a capsule fitted to points (their main axis, the middle 80% of their length, the median distance from it) as
 * spheres along it */
function capsule(pts: V3[]): { spheres: Sphere[]; c: V3; cov: number[][] } {
  const n = pts.length, cov = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
  if (!n) return { spheres: [], c: [0, 0, 0], cov };
  const c = scale3(pts.reduce(add, [0, 0, 0] as V3), 1 / n);
  for (const p of pts) { const d = sub(p, c); for (let a = 0; a < 3; a++) for (let b = 0; b < 3; b++) cov[a][b] += (d[a] * d[b]) / n; }
  let ax: V3 = [0.31, 0.83, 0.47];
  for (let k = 0; k < 40; k++) ax = norm([dot(cov[0] as V3, ax), dot(cov[1] as V3, ax), dot(cov[2] as V3, ax)]);
  const t = pts.map((p) => dot(sub(p, c), ax)).sort((a, b) => a - b);
  const rs = pts.map((p) => { const d = sub(p, c); return len(sub(d, scale3(ax, dot(d, ax)))); }).sort((a, b) => a - b);
  const r = Math.max(0.01, 0.85 * rs[Math.floor(n / 2)]);
  const lo = t[Math.floor(0.1 * (n - 1))] + r, hi = t[Math.ceil(0.9 * (n - 1))] - r;
  const spheres: Sphere[] = [];
  if (hi <= lo) spheres.push({ c, r });
  else { const m = Math.max(2, Math.ceil((hi - lo) / r) + 1); for (let i = 0; i < m; i++) spheres.push({ c: add(c, scale3(ax, lo + ((hi - lo) * i) / (m - 1))), r }); }
  return { spheres, c, cov };
}

/** the parts as they are at the pose `p` (fixing the in-between joints' turns), in each part's lead joint's frame */
function buildParts(skel: Skeleton, p: Pose) {
  const J = skel.joints, nc = captured(skel);
  const lead = PARTS.map(([, j]) => joint(skel, j));
  // each joint's part: its nearest lead up the chain
  const partOf = J.map((_, i) => { let k = i; while (k >= 0 && !lead.includes(k)) k = J[k].parent; return lead.indexOf(k); });
  const w = forward(skel, p);
  const local = (pi: number, x: V3): V3 => irot(w.rot[lead[pi]], sub(x, w.pos[lead[pi]]));
  // the skin's samples, by the part their skin joint rides; a skeleton without them gets rings round its bones
  const samples: { p: V3; n: V3 }[][] = PARTS.map(() => []);
  // the contact samples: where (world, at the pose), their normal, and their parts' weights
  const touch: { p: V3; n: V3; parts: [number, number][] }[] = [];
  if (skel.surface && skel.skin) {
    // each sample to the part whose bones it is nearest (by its skin joint, the pelvis had four: the hips' skin rides
    // the thighs and the lower back)
    const bones: { pi: number; a: V3; b: V3 }[] = [];
    for (let i = 0; i < nc; i++) if (partOf[i] >= 0) bones.push({ pi: partOf[i], a: w.pos[i], b: add(w.pos[i], qrot(w.rot[i], sub(J[i].tail, J[i].head))) });
    const toSeg = (x: V3, a: V3, b: V3) => { const ab = sub(b, a), l2 = dot(ab, ab), t = l2 > 1e-12 ? clamp(dot(sub(x, a), ab) / l2, 0, 1) : 0; return len(sub(x, add(a, scale3(ab, t)))); };
    const S = skel.surface, sk = skel.skin;
    surface(skel, w).forEach((s, i) => {
      let best = -1, bd = Infinity;
      for (const bn of bones) { const d = toSeg(s.at, bn.a, bn.b); if (d < bd) { bd = d; best = bn.pi; } }
      samples[best].push({ p: local(best, s.at), n: irot(w.rot[lead[best]], s.normal) });
      // its weights by part (the skin joints' drives'), the least dropped, the main part first
      const by = new Map<number, number>();
      S[i].j.forEach((j, k) => { const pi = partOf[sk[j].drive]; if (pi >= 0) by.set(pi, (by.get(pi) ?? 0) + S[i].w[k]); });
      const tot = [...by.values()].reduce((a, b) => a + b, 0);
      const parts = [...by].filter(([, x]) => x > 0.05 * tot).sort((a, b) => b[1] - a[1]);
      const kept = parts.reduce((a, [, x]) => a + x, 0);
      touch.push({ p: s.at, n: s.normal, parts: parts.map(([pi, x]) => [pi, x / kept]) });
    });
  } else {
    const R = 0.035 * skel.height;
    for (let i = 0; i < nc; i++) {
      const pi = partOf[i], jt = J[i];
      if (pi < 0 || len(sub(jt.tail, jt.head)) < 1e-6) continue;
      const a = local(pi, w.pos[i]), b = local(pi, add(w.pos[i], qrot(w.rot[i], sub(jt.tail, jt.head))));
      const ax = norm(sub(b, a)), u = norm(cross(ax, Math.abs(ax[1]) < 0.9 ? [0, 1, 0] : [1, 0, 0])), v = cross(ax, u);
      for (let s = 0; s <= 2; s++) for (let k = 0; k < 8; k++) {
        const th = (k * Math.PI) / 4, n = add(scale3(u, Math.cos(th)), scale3(v, Math.sin(th)));
        samples[pi].push({ p: add(add(a, scale3(sub(b, a), s / 2)), scale3(n, R)), n });
        const wp = add(w.pos[lead[pi]], qrot(w.rot[lead[pi]], samples[pi][samples[pi].length - 1].p));
        touch.push({ p: wp, n: qrot(w.rot[lead[pi]], n), parts: [[pi, 1]] });
      }
    }
  }
  const mass = 20 * skel.height * skel.height;
  const parts: Part[] = PARTS.map(([name, , share], pi) => {
    const m = mass * share, cap = capsule(samples[pi].map((s) => s.p));
    // the moments of the samples' spread about the part's own axes (a shell, near enough)
    const cv = cap.cov, I = [cv[1][1] + cv[2][2], cv[0][0] + cv[2][2], cv[0][0] + cv[1][1]].map((x) => m * x);
    const top = Math.max(...I);
    return {
      name, lead: lead[pi], mass: m, c: cap.c,
      inertia: I.map((x) => Math.max(x, INERTIA_FLOOR * top, m * (0.03 * skel.height) ** 2)) as V3,
      hull: cap.spheres.map((s) => ({ c: sub(s.c, cap.c), r: s.r })),
    };
  });
  // the contact samples, spread over each main part, as Skins: each part's offset to it from its centre, in its frame
  const skin: Skin[] = [];
  PARTS.forEach((_, pi) => {
    for (const t of spread(touch.filter((t) => t.parts[0][0] === pi), SKIN_SAMPLES)) {
      const c = add(t.p, scale3(t.n, SKIN_GAP - SKIN_R));
      skin.push({
        b: t.parts.map(([b]) => b), w: t.parts.map(([, x]) => x), r: SKIN_R,
        c: t.parts.map(([b]) => sub(irot(w.rot[lead[b]], sub(c, w.pos[lead[b]])), parts[b].c)),
      });
    }
  });
  return { parts, skin };
}

/** each part's place (its centre) and turn at a pose */
function placeParts(skel: Skeleton, parts: Part[], p: Pose): { x: V3; q: Q }[] {
  const w = forward(skel, p);
  return parts.map((pt) => ({ x: add(w.pos[pt.lead], qrot(w.rot[pt.lead], pt.c)), q: w.rot[pt.lead] }));
}

interface Sim { states: { x: V3; q: Q }[][]; restAt: number; from: number; fps: number; world: World; parts: Part[]; base: Pose; frames: Pose[]; still: number; asleep: boolean; perFrame: number; maxFrames: number }

function makeSim(skel: Skeleton, motion: (f: number) => Pose, from: number, o: RagdollOptions): Sim {
  const fps = o.fps ?? 30, base = motion(from);
  const { parts, skin } = buildParts(skel, base);
  const J = skel.joints, leadPart = new Map(parts.map((p, i) => [p.lead, i]));
  const parentPart = (i: number) => { let k = J[parts[i].lead].parent; while (k >= 0 && !leadPart.has(k)) k = J[k].parent; return k < 0 ? -1 : leadPart.get(k)!; };
  // the momentum: each part's speed and spin over the motion's last frame before `from`
  const now = placeParts(skel, parts, base), before = placeParts(skel, parts, motion(from - 1)), push = o.push ?? [0, 0, 0];
  const bodies: Body[] = parts.map((pt, i) => {
    const dq = qmul(now[i].q, qinv(before[i].q));
    return {
      x: now[i].x, q: now[i].q,
      v: add(scale3(sub(now[i].x, before[i].x), fps), push),
      w: scale3(qlog(dq), fps),
      invMass: 1 / pt.mass,
      invI: pt.inertia.map((x) => 1 / x) as V3,
      hull: pt.hull,
    };
  });
  // the reference each joint's limits are measured from: the body standing, arms hanging (each lead's turn in the
  // body's frame)
  // (the forearm's the upper arm's: the arm straight)
  const ref = parts.map((pt) => {
    const kind = pt.name.split(".")[0];
    if (kind !== "upperarm" && kind !== "lowerarm") return ID;
    const side = pt.name.endsWith(".L") ? "L" : "R", j = J[joint(skel, side === "L" ? "LeftArm" : "RightArm")];
    return qfromTo(sub(j.tail, j.head), [(side === "L" ? 1 : -1) * Math.sin(HANG), -Math.cos(HANG), 0]);
  });
  // an elbow's hinge, in the arm's own frame at rest: the way the motion bends it (the capture's elbow is a hinge, but
  // not about the frame's axes: on CMU's rig it is tilted 31 degrees off), else CMU's
  const hinge = (pt: Part): V3 => {
    const side = pt.name.endsWith(".L") ? 1 : -1;
    let m: V3 = [0, 0, 0];
    for (let f = Math.max(0, from - 60); f <= from; f++) m = add(m, qlog(motion(f).turns[pt.lead] ?? ID));
    return len(m) / (Math.min(from, 60) + 1) > 0.25 ? norm(m) : norm([-0.019, -0.858 * side, 0.513 * side]);
  };
  const tone = o.stiffness ?? 0.04, strength = parts.reduce((s, p) => s + p.mass, 0) / 60;
  const w0 = forward(skel, base);
  const joints: Joint[] = [];
  parts.forEach((pt, b) => {
    const a = parentPart(b);
    if (a < 0) return;
    const kind = pt.name.split(".")[0], lim = LIMITS[kind], right = pt.name.endsWith(".R");
    const anchor = w0.pos[pt.lead];
    let F: Q = ID;
    if (kind === "lowerarm") {
      // x against the elbow's hinge (flexion is - about x), y up the forearm, in the arm's frame at rest
      const j = J[pt.lead], y = norm(sub(j.head, j.tail)), hx = scale3(hinge(pt), -1);
      const x = norm(sub(hx, scale3(y, dot(hx, y))));
      F = qmul(ref[b], qbasis(x, y, cross(x, y)));
    } else if (lim.frame === "limb") {
      const j = J[pt.lead];
      F = frameFrom(qrot(ref[b], sub(j.head, j.tail)));
    }
    const z: [number, number] = right ? [-lim.z[1], -lim.z[0]] : lim.z;
    const K = STIFFNESS * lim.k * strength, top = Math.max(...pt.inertia);
    joints.push({
      a, b,
      pa: irot(bodies[a].q, sub(anchor, bodies[a].x)),
      pb: irot(bodies[b].q, sub(anchor, bodies[b].x)),
      fa: qnorm(qmul(qinv(ref[a]), F)),
      fb: qnorm(qmul(qinv(ref[b]), F)),
      x: { lo: lim.x[0], hi: lim.x[1] }, y: { lo: lim.y[0], hi: lim.y[1] }, z: { lo: z[0], hi: z[1] },
      target: RELAXED[kind] ?? ID,
      stiffness: K * tone,
      maxTorque: STRENGTH * lim.k * strength * Math.sqrt(tone),
      damping: 2 * DAMPING * Math.sqrt(K * top),
    });
  });
  const idx = (n: string) => parts.findIndex((p) => p.name === n);
  const pairs: [number, number][] = LEG_PAIRS.map(([p, q]) => [idx(p), idx(q)]);
  for (const s of ["L", "R"]) for (const t of ARM_AGAINST) { pairs.push([idx(`lowerarm.${s}`), idx(t)]); if (t !== "chest") pairs.push([idx(`upperarm.${s}`), idx(t)]); }
  pairs.push([idx("lowerarm.L"), idx("lowerarm.R")]);
  // (a pair already in each other at `from`, the capsules being rough, is left out: pushed apart, it would kick)
  const overlapping = ([ia, ib]: [number, number]) => bodies[ia].hull.some((sa) => bodies[ib].hull.some((sb) =>
    len(sub(add(bodies[ia].x, qrot(bodies[ia].q, sa.c)), add(bodies[ib].x, qrot(bodies[ib].q, sb.c)))) < sa.r + sb.r));
  const boxes: Box[] = (o.solids ?? []).map((s) => {
    const h = scale3(sub(s.max, s.min), 0.5);
    return { c: scale3(add(s.min, s.max), 0.5), h, r: len(h) };
  });
  const g = o.ground === undefined ? 0 : o.ground;
  if (g !== null) {
    const c = bodies[0].x, H: V3 = [500, 1, 500];
    boxes.push({ c: [c[0], g - 1, c[2]], h: H, r: len(H) });
  }
  const perFrame = Math.max(1, Math.ceil(60 / fps));
  // the air: each part's cross-section from its hull (its spheres' discs, each counted where it isn't the last's), and
  // the drag's factor that holds the whole body's weight at the terminal speed
  let air: Air | undefined;
  if (o.terminal || o.buffet) {
    const area = parts.map((pt) => pt.hull.reduce((sum, sp, k) => {
      const prev = pt.hull[k - 1], over = prev ? Math.max(0, Math.min(1, (prev.r + sp.r - len(sub(sp.c, prev.c))) / (2 * sp.r))) : 0;
      return sum + Math.PI * sp.r * sp.r * (1 - 0.5 * over);
    }, 0));
    const weight = parts.reduce((m, p) => m + p.mass, 0) * 9.81, A = area.reduce((x, y) => x + y, 0);
    air = { c: o.terminal ? weight / (A * o.terminal * o.terminal) : 0, area, buffet: o.buffet ?? 0, seed: o.seed ?? 0, t: 0 };
  }
  return {
    states: [], restAt: -1, from, fps, parts, base, frames: [], still: 0, asleep: false, perFrame,
    maxFrames: Math.ceil((o.maxSeconds ?? 8) * fps),
    world: { bodies, joints, boxes, pairs: pairs.filter((p) => !overlapping(p)), skin, air },
  };
}

/** the pose the world's parts make: each lead's turn from its parent's, the joints between as they were at `from` */
function poseOf(skel: Skeleton, sim: Sim): Pose {
  const J = skel.joints, nc = captured(skel), { base, parts, world } = sim;
  const turns = base.turns.slice(), rot: Q[] = new Array(nc);
  const byLead = new Map(parts.map((p, i) => [p.lead, i]));
  for (let i = 0; i < nc; i++) {
    const par = J[i].parent, b = byLead.get(i);
    rot[i] = b !== undefined ? world.bodies[b].q : qmul(rot[par], base.turns[i] ?? ID);
    turns[i] = par < 0 ? rot[i] : qnorm(qmul(qinv(rot[par]), rot[i]));
  }
  const pel = world.bodies[0];
  return { root: sub(pel.x, qrot(pel.q, parts[0].c)), turns };
}

function advance(skel: Skeleton, sim: Sim, upTo: number) {
  const snap = () => sim.states.push(sim.world.bodies.map((b) => ({ x: b.x, q: b.q })));
  if (!sim.frames.length) { sim.frames.push(poseOf(skel, sim)); snap(); }
  while (sim.frames.length <= upTo) {
    if (sim.asleep) { sim.frames.push(sim.frames[sim.frames.length - 1]); sim.states.push(sim.states[sim.states.length - 1]); continue; }
    const dt = 1 / (sim.fps * sim.perFrame);
    for (let k = 0; k < sim.perFrame; k++) step(sim.world, dt);
    const calm = sim.world.bodies.every((b) => len(b.v) < SLEEP_V && len(b.w) < SLEEP_W);
    sim.still = calm ? sim.still + 1 : 0;
    sim.frames.push(poseOf(skel, sim));
    snap();
    if (sim.still >= SLEEP_FOR * sim.fps || sim.frames.length > sim.maxFrames) {
      sim.asleep = true;
      // (asleep where it lies: the frame it came to rest is the one every later frame is)
      sim.restAt = sim.frames.length - 1;
    }
  }
}

/** A fall: the pose at a frame, and what the simulation knows about it. */
export interface Fall {
  (f: number): Pose;
  /** each joint between parts at frame f (>= from): its swing about x and z and twist about y in its frame, radians,
   * and its limits */
  joints(f: number): { name: string; x: number; z: number; twist: number; lim: { x: [number, number]; z: [number, number]; y: [number, number] } }[];
  /** the lowest point of the skin's contact samples at frame f (>= from), metres */
  lowest(f: number): number;
  /** the frame the body came to rest (it is that pose from then on), stepping as far as it takes */
  restsAt(): number;
}

const sims = new WeakMap<Skeleton, WeakMap<(f: number) => Pose, Map<string, Sim>>>();

/** The body going limp at frame `from` of `motion` and falling under gravity onto the ground (y = 0) and `solids`, all
 * in the body's frame (the frame the Pose's root is in). Before `from`, the motion's own pose. Deterministic, stepped
 * once from `from` and memoised (by skeleton, motion function and options), so any frame can be asked first. */
export function ragdoll(skel: Skeleton, motion: (f: number) => Pose, from: number, o: RagdollOptions = {}): Fall {
  let bySkel = sims.get(skel);
  if (!bySkel) sims.set(skel, (bySkel = new WeakMap()));
  let byMotion = bySkel.get(motion);
  if (!byMotion) bySkel.set(motion, (byMotion = new Map()));
  const key = JSON.stringify([from, o.fps, o.solids, o.push, o.stiffness, o.ground, o.maxSeconds, o.terminal, o.buffet, o.seed]);
  let sim: Sim | undefined;
  const get = () => {
    sim ??= byMotion!.get(key);
    if (!sim) byMotion!.set(key, (sim = makeSim(skel, motion, from, o)));
    return sim;
  };
  const at = (f: number) => { const s = get(), i = Math.max(0, Math.round(f - from)); advance(skel, s, i); return { s, st: s.states[i] }; };
  const fall = ((f: number) => {
    if (f < from) return motion(f);
    const s = get(), t = f - from, i = Math.floor(t);
    advance(skel, s, i + 1);
    return t === i ? s.frames[i] : blend(s.frames[i], s.frames[i + 1], t - i);
  }) as Fall;
  fall.joints = (f) => {
    const { s, st } = at(f), w: World = { ...s.world, bodies: s.world.bodies.map((b, i) => ({ ...b, x: st[i].x, q: st[i].q })) };
    return s.world.joints.map((j) => ({ name: s.parts[j.b].name, ...angles(relative(w, j)), lim: { x: [j.x.lo, j.x.hi], z: [j.z.lo, j.z.hi], y: [j.y.lo, j.y.hi] } }));
  };
  fall.lowest = (f) => {
    const { s, st } = at(f);
    let m = Infinity;
    for (const sk of s.world.skin) m = Math.min(m, skinAt(s.world, sk, st.map((x) => x.x), st.map((x) => x.q))[1] - sk.r);
    return m;
  };
  fall.restsAt = () => { const s = get(); while (!s.asleep) advance(skel, s, s.frames.length); return from + s.restAt; };
  return fall;
}
