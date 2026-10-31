// The solve: positional and angular constraints, skin contacts, wind, a step.
import { add, cross, dot, len, qinv, qmul, qnorm, qrot, scale3, sub } from "../character";
import { Q, V3, clamp, irot, qexp, qlog, swingTwist } from "./all";

// ---------------------------------------------------------------- the solver (heros-journey's physics.ts)

export interface Sphere { c: V3; r: number }

export interface Body { x: V3; q: Q; v: V3; w: V3; invMass: number; invI: V3; hull: Sphere[] }

/** a sample of the skin: on the parts `b` (the first its main one) by weights `w` (summing to 1), at `c` from each's
 * centre in its frame; a sphere of radius r */
export interface Skin { b: number[]; w: number[]; c: V3[]; r: number }

interface Range { lo: number; hi: number }

export interface Joint { a: number; b: number; pa: V3; pb: V3; fa: Q; fb: Q; x: Range; y: Range; z: Range; target: Q; stiffness: number; maxTorque: number; damping: number }

export interface Box { c: V3; h: V3; r: number }

/** the air: the drag's factor and each part's cross-section (m^2), the gusts' strength, their seed, and the time */
export interface Air { c: number; area: number[]; buffet: number; seed: number; t: number }

export interface World { bodies: Body[]; joints: Joint[]; boxes: Box[]; pairs: [number, number][]; skin: Skin[]; air?: Air }

const SUBSTEPS = 20;

const MU_STATIC = 0.9;

const MU_DYNAMIC = 0.7;

const GRAVITY: V3 = [0, -9.81, 0];

function invIWorld(b: Body, v: V3): V3 {
  const l = irot(b.q, v);
  return qrot(b.q, [l[0] * b.invI[0], l[1] * b.invI[1], l[2] * b.invI[2]]);
}

function genW(b: Body | null, r: V3, n: V3): number {
  if (!b) return 0;
  const rn = cross(r, n);
  return b.invMass + dot(rn, invIWorld(b, rn));
}

function rotate(b: Body, dw: V3) {
  const d = qmul([dw[0], dw[1], dw[2], 0], b.q);
  b.q = qnorm([b.q[0] + 0.5 * d[0], b.q[1] + 0.5 * d[1], b.q[2] + 0.5 * d[2], b.q[3] + 0.5 * d[3]]);
}

function positional(b1: Body | null, b2: Body | null, corr: V3, r1: V3, r2: V3, h: number, compliance = 0, limit = Infinity): number {
  const c = len(corr);
  if (c < 1e-12) return 0;
  const n = scale3(corr, 1 / c);
  const w = genW(b1, r1, n) + genW(b2, r2, n), a = compliance / (h * h);
  if (w + a === 0) return 0;
  const dl = Math.min(c / (w + a), limit), p = scale3(n, dl);
  if (b1) { b1.x = add(b1.x, scale3(p, b1.invMass)); rotate(b1, invIWorld(b1, cross(r1, p))); }
  if (b2) { b2.x = sub(b2.x, scale3(p, b2.invMass)); rotate(b2, scale3(invIWorld(b2, cross(r2, p)), -1)); }
  return dl;
}

function angular(b1: Body, b2: Body, corr: V3, h: number, compliance = 0, limit = Infinity) {
  const th = len(corr);
  if (th < 1e-12) return;
  const n = scale3(corr, 1 / th);
  const w = dot(n, invIWorld(b1, n)) + dot(n, invIWorld(b2, n)), a = compliance / (h * h);
  const dl = Math.min(th / (w + a), limit), p = scale3(n, dl);
  rotate(b1, invIWorld(b1, p));
  rotate(b2, scale3(invIWorld(b2, p), -1));
}

export const relative = (w: World, j: Joint): Q => qmul(qinv(qmul(w.bodies[j.a].q, j.fa)), qmul(w.bodies[j.b].q, j.fb));

/** a relative pose from swing angles about x and z and a twist about y */
export const swingPose = (x: number, z: number, twist: number): Q => qmul(qexp([x, 0, z]), [0, Math.sin(twist / 2), 0, Math.cos(twist / 2)]);

export function angles(r: Q) {
  const { swing, twist } = swingTwist(r, [0, 1, 0]);
  const s = qlog(swing), t = qlog(twist);
  return { x: s[0], z: s[2], twist: t[1] };
}

function towards(w: World, j: Joint, want: Q, h: number, compliance = 0, limit = Infinity) {
  const A = w.bodies[j.a], B = w.bodies[j.b];
  const e = qlog(qmul(qmul(qmul(A.q, j.fa), want), qinv(qmul(B.q, j.fb))));
  angular(B, A, e, h, compliance, limit);
}

function solveJoint(w: World, j: Joint, h: number) {
  const A = w.bodies[j.a], B = w.bodies[j.b];
  if (j.stiffness > 0 && j.maxTorque > 0) towards(w, j, j.target, h, 1 / j.stiffness, j.maxTorque * h * h);
  const r = angles(relative(w, j));
  const cx = clamp(r.x, j.x.lo, j.x.hi), cz = clamp(r.z, j.z.lo, j.z.hi), ct = clamp(r.twist, j.y.lo, j.y.hi);
  if (cx !== r.x || cz !== r.z || ct !== r.twist) towards(w, j, swingPose(cx, cz, ct), h);
  const ra = qrot(A.q, j.pa), rb = qrot(B.q, j.pb);
  positional(A, B, sub(add(B.x, rb), add(A.x, ra)), ra, rb, h);
}

/** a sphere against an axis-aligned box: the normal out of the box and the depth, or null */
function sphereBox(s: V3, r: number, box: Box): { n: V3; d: number } | null {
  const p = sub(s, box.c), h = box.h;
  const d: V3 = [p[0] - clamp(p[0], -h[0], h[0]), p[1] - clamp(p[1], -h[1], h[1]), p[2] - clamp(p[2], -h[2], h[2])];
  const dist = len(d);
  if (dist > 1e-9) return dist >= r ? null : { n: scale3(d, 1 / dist), d: r - dist };
  let best = 0, depth = Infinity, sign = 1;
  for (let k = 0; k < 3; k++) {
    const dk = h[k] - Math.abs(p[k]);
    if (dk < depth) { depth = dk; best = k; sign = p[k] < 0 ? -1 : 1; }
  }
  const n: V3 = [0, 0, 0];
  n[best] = sign;
  return { n, d: depth + r };
}

interface Contact { sk: Skin; n: V3; off: V3; lambda: number }

/** where a skin sample is: the blend of where each of its parts puts it (the skin's own linear blend) */
export function skinAt(w: World, sk: Skin, xs?: V3[], qs?: Q[]): V3 {
  let p: V3 = [0, 0, 0];
  sk.b.forEach((bi, k) => { const x = xs ? xs[bi] : w.bodies[bi].x, q = qs ? qs[bi] : w.bodies[bi].q; p = add(p, scale3(add(x, qrot(q, sk.c[k])), sk.w[k])); });
  return p;
}

/** the sample (moved by `off`) from each of its parts' centres */
const skinArms = (w: World, sk: Skin, off: V3): V3[] => sk.b.map((bi, k) => add(qrot(w.bodies[bi].q, sk.c[k]), off));

/** the sample's generalised inverse mass along n: each part's, by its weight squared */
function skinW(w: World, sk: Skin, arms: V3[], n: V3): number {
  let W = 0;
  sk.b.forEach((bi, k) => { W += sk.w[k] * sk.w[k] * genW(w.bodies[bi], arms[k], n); });
  return W;
}

/** a correction (or with `vel`, an impulse) of `lambda` along n at the sample, shared by its parts by their weights,
 * which moves the blended point by lambda times skinW */
function skinMove(w: World, sk: Skin, arms: V3[], n: V3, lambda: number, vel: boolean) {
  sk.b.forEach((bi, k) => {
    const B = w.bodies[bi], p = scale3(n, lambda * sk.w[k]), dw = invIWorld(B, cross(arms[k], p));
    if (vel) { B.v = add(B.v, scale3(p, B.invMass)); B.w = add(B.w, dw); }
    else { B.x = add(B.x, scale3(p, B.invMass)); rotate(B, dw); }
  });
}

/** a gust on part i at the air's time: a smooth noise across the fall (x and z), a few slow sines, scaled by the
 * part's size against the average part's */
function gust(air: Air, i: number): V3 {
  const r = (k: number) => { const v = Math.sin((air.seed * 7.13 + i * 3.71 + k * 1.37) * 12.9898) * 43758.5453; return v - Math.floor(v); };
  const ax = (k: number) => {
    let v = 0;
    for (let m = 0; m < 3; m++) v += Math.sin(2 * Math.PI * (0.35 + 1.1 * r(k * 5 + m)) * air.t + 6.283 * r(k * 5 + m + 2.5)) / (m + 1);
    return v / 1.83;
  };
  const mean = air.area.reduce((x, y) => x + y, 0) / air.area.length;
  const k = air.area[i] / mean;
  return [k * ax(0), 0, k * ax(1)];
}

/** One step of `dt` seconds, in place. */
export function step(w: World, dt: number) {
  const h = dt / SUBSTEPS, n = w.bodies.length, air = w.air;
  for (let s = 0; s < SUBSTEPS; s++) {
    const px: V3[] = [], pq: Q[] = [];
    if (air) air.t += h;
    for (let i = 0; i < n; i++) {
      const b = w.bodies[i];
      px.push(b.x);
      pq.push(b.q);
      b.v = add(b.v, scale3(GRAVITY, h));
      if (air) {
        // the air on the part: drag against its speed, by its cross-section; gusts across the fall
        const a = air.area[i], sp = len(b.v);
        b.v = add(b.v, scale3(b.v, -air.c * a * sp * b.invMass * h));
        if (air.buffet > 0) b.v = add(b.v, scale3(gust(air, i), air.buffet * h));
      }
      b.x = add(b.x, scale3(b.v, h));
      rotate(b, scale3(b.w, h));
    }
    for (const j of w.joints) solveJoint(w, j, h);
    // the limbs against each other and the trunk
    for (const [ia, ib] of w.pairs) {
      const A = w.bodies[ia], B = w.bodies[ib];
      for (const sa of A.hull) {
        const ca = add(A.x, qrot(A.q, sa.c));
        for (const sb of B.hull) {
          const cb = add(B.x, qrot(B.q, sb.c)), d = sub(ca, cb), dist = len(d), pen = sa.r + sb.r - dist;
          if (pen <= 0 || dist < 1e-9) continue;
          const nn = scale3(d, 1 / dist);
          positional(A, B, scale3(nn, pen), sub(sub(ca, scale3(nn, sa.r)), A.x), sub(add(cb, scale3(nn, sb.r)), B.x), h);
        }
      }
    }
    // the skin against the ground and boxes: normal corrections, then static friction against the load on the
    // sample's main part
    const contacts: Contact[] = [], sumL = new Array(n).fill(0);
    for (const sk of w.skin) {
      const c = skinAt(w, sk);
      for (const box of w.boxes) {
        const dc = sub(c, box.c), reach = box.r + sk.r;
        if (dot(dc, dc) > reach * reach) continue;
        const hit = sphereBox(c, sk.r, box);
        if (!hit) continue;
        const arms = skinArms(w, sk, scale3(hit.n, -sk.r));
        const lambda = hit.d / skinW(w, sk, arms, hit.n);
        skinMove(w, sk, arms, hit.n, lambda, false);
        contacts.push({ sk, n: hit.n, off: scale3(hit.n, -sk.r), lambda });
        sumL[sk.b[0]] += lambda;
      }
    }
    for (const c of contacts) {
      const now = skinAt(w, c.sk), dp = sub(now, skinAt(w, c.sk, px, pq));
      const dpt = sub(dp, scale3(c.n, dot(dp, c.n))), dl = len(dpt);
      if (dl < 1e-12) continue;
      const t = scale3(dpt, -1 / dl), arms = skinArms(w, c.sk, c.off), lt = dl / Math.max(skinW(w, c.sk, arms, t), 1e-12);
      if (lt < MU_STATIC * sumL[c.sk.b[0]]) skinMove(w, c.sk, arms, t, lt, false);
    }
    for (let i = 0; i < n; i++) {
      const b = w.bodies[i];
      b.v = scale3(sub(b.x, px[i]), 1 / h);
      const dq = qmul(b.q, qinv(pq[i])), sg = dq[3] >= 0 ? 2 / h : -2 / h;
      b.w = [dq[0] * sg, dq[1] * sg, dq[2] * sg];
    }
    // dynamic friction and no bounce
    for (const c of contacts) {
      const arms = skinArms(w, c.sk, c.off);
      let vc: V3 = [0, 0, 0];
      c.sk.b.forEach((bi, k) => { const B = w.bodies[bi]; vc = add(vc, scale3(add(B.v, cross(B.w, arms[k])), c.sk.w[k])); });
      const vn = dot(vc, c.n), vt = sub(vc, scale3(c.n, vn)), vtl = len(vt);
      let dv: V3 = [0, 0, 0];
      if (vtl > 1e-9) dv = scale3(vt, -Math.min((MU_DYNAMIC * c.lambda) / h, vtl) / vtl);
      if (vn < 0) dv = add(dv, scale3(c.n, -vn));
      const dvl = len(dv);
      if (dvl < 1e-12) continue;
      const nn = scale3(dv, 1 / dvl);
      skinMove(w, c.sk, arms, nn, dvl / skinW(w, c.sk, arms, nn), true);
    }
    // the joints' damping on the two parts' relative spin
    for (const j of w.joints) {
      const A = w.bodies[j.a], B = w.bodies[j.b], rel = sub(B.w, A.w), rl = len(rel);
      if (rl < 1e-9) continue;
      const nn = scale3(rel, 1 / rl), ws = dot(nn, invIWorld(A, nn)) + dot(nn, invIWorld(B, nn));
      const imp = Math.min(j.damping * h * rl, rl / ws);
      B.w = sub(B.w, invIWorld(B, scale3(nn, imp)));
      A.w = add(A.w, invIWorld(A, scale3(nn, imp)));
    }
  }
}
