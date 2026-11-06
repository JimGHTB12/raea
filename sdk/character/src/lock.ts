// Contact-locked feet (the standard game method: Holden's "contact locking" in his motion-matching notes, after
// Bollo's inertialization talk): each foot is held where it touched down for as long as the library says it stands,
// and let go into its swing through a critically damped spring, so the offset the lock built up fades without a jump in
// place or speed. The leg reaches its foot by analytic two-bone IK (soft near full stretch); the foot keeps the clip's
// own turn, so the heel lifts and the ball pivots as the capture's did.
//
// The lock is an offset on the clip's ankle, never a place of its own: at touchdown the offset is whatever the last
// release left (so landing never jumps), while standing it grows by exactly what the clip's pivot slides (so the pivot
// stays put), and when the pivot moves from the heel to the ball the new lock is taken where the ball already is.
import type { Vec3 } from "raea/3d";
import { joint, pose, qinv, qmul, qnorm, qrot, qyaw, setWorld, twoBone, type Pose, type Quat, type Skeleton } from "raea/character";
import { LEGS, feetOf, kneePole, rests, standing } from "./feet";
import { qlog } from "./effects";

/** What lockFeet did per frame and foot, for checks and the debug overlay: whether it stood, on which pivot, the
 * locked spot (null in a swing), the ankle's target and where the solved ankle reached. */
export interface LockLog {
  stand: boolean[][];
  pivot: ("heel" | "ball" | null)[][];
  lock: ([number, number] | null)[][];
  target: Vec3[][];
  reached: Vec3[][];
  clip: Vec3[][];
}

export interface LockOptions {
  /** the release spring's half-life, as a share of the swing it fades over (and never over `maxHalf` seconds) */
  half?: number;
  maxHalf?: number;
  /** a heel sole more than this far over the ball's (m, on a 1.75 m body) has lifted: the ball is the pivot */
  heelUp?: number;
  log?: LockLog;
  /** a sole on the floor slower than this (m/s) stands, whatever the clip says */
  slow?: number;
  /** frames over which a landing foot slows onto its spot (3) */
  land?: number;
  /** passes of the legs' jerk easing (3) */
  ease?: number;
  /** how far behind the ankle the heel's point is, as a share of the ankle-to-ball run (0.36) */
  heelBack?: number;
  /** how far (body-height share, 0.03) a lock may get from where the clip has its foot before it lets go */
  radius?: number;
  /** the most of its length a standing leg is let reach before the hips come down (0.97; 1 never lowers them) */
  fold?: number;
}

/** a critically damped spring's offset and speed `t` seconds after release from x0 moving at v0 (half-life h) */
function decay(x0: number, v0: number, h: number, t: number): number {
  const y = (2 * Math.LN2) / Math.max(1e-4, h), j = v0 + x0 * y;
  return Math.exp(-y * t) * (x0 + j * t);
}

/** Each foot locked through its contacts and released on a spring; the legs solved to reach. `hint` is the clip's own
 * contacts per frame (see plantFeet). */
export function lockFeet(skel: Skeleton, raw: Pose[], hint?: [number, number][], fps = 30, o: LockOptions = {}): Pose[] {
  const n = raw.length;
  if (!n) return raw;
  const r = rests(skel), z = r.z, dt = 1 / fps;
  const half = o.half ?? 0.25, maxHalf = o.maxHalf ?? 0.12, heelUp = (o.heelUp ?? 0.01) * z;
  const radius = (o.radius ?? Number(process.env.RAEA_LOCK_RADIUS ?? 0.03)) * z;
  const feet = raw.map((p) => feetOf(skel, p));
  // the soles: the floor points under the heel (the ankle) and the ball, turned with the foot, and their heights
  const J = skel.joints, ai = LEGS.map((l) => joint(skel, l[2])), bi = LEGS.map((l) => joint(skel, l[3]));
  // (the heel's point is its back, where a foot strikes and rolls: 0.36 of the ankle-to-ball run behind the ankle, as
  // the body's foot surface has it; under the ankle, the locked point swung 3 cm forward as the foot rolled flat, and
  // the knee kinked 5 degrees at every landing)
  const back = [0, 1].map((s) => { const a = J[ai[s]].head, b = J[bi[s]].head, k = (o.heelBack ?? Number(process.env.RAEA_HEEL_BACK ?? 0.36)); return [-(b[0] - a[0]) * k, -a[1], -(b[2] - a[2]) * k] as Vec3; });
  const soles = raw.map((p) => { const w = pose(skel, p); return [0, 1].map((s) => [ai[s], bi[s]].map((i, k) => { const v = qrot(w.rot[i], k ? [0, -J[i].head[1], 0] : back[s]); return [w.pos[i][0] + v[0], w.pos[i][1] + v[1], w.pos[i][2] + v[2]] as Vec3; })); });
  const low = (t: number, s: number) => Math.min(soles[t][s][0][1], soles[t][s][1][1]);
  const speed = (t: number, s: number, k: number) => { const a = soles[Math.max(0, t - 1)][s][k], b = soles[Math.min(n - 1, t + 1)][s][k]; return (Math.hypot(b[0] - a[0], b[2] - a[2]) * fps) / (t > 0 && t < n - 1 ? 2 : 1); };
  // standing: where the clip says so, and wherever a sole is on the floor (under 2 cm) and slow (0.6 m/s): a capture's
  // foot lands before its contact says and skims after it; gaps of two frames filled, stands under three frames dropped
  const stand = standing(skel, feet, hint, fps, raw.map((p) => p.root)).map((c, s) => c.map((v, t) => v || (low(t, s) < 0.02 * z && Math.min(speed(t, s, 0), speed(t, s, 1)) < (o.slow ?? Number(process.env.RAEA_LOCK_SLOW ?? 0.6)) * z)));
  for (const c of stand) {
    for (let t = 1; t < n - 1; t++) if (!c[t] && c[t - 1]) { let e = t; while (e < n && !c[e]) e++; if (e < n && e - t <= 2) for (let k = t; k < e; k++) c[k] = true; t = e; }
    for (let t = 0; t < n; t++) if (c[t]) { let e = t; while (e < n && c[e]) e++; if (e - t < 3) for (let k = t; k < e; k++) c[k] = false; t = e; }
  }
  // each standing foot's turn (yaw) locked too, from touchdown until it is up on its ball (a foot flat on the floor
  // turned with the body swept its ball 3 cm round its heel), then held while it turns on the ball, and let go on the
  // spring; the soles turned with it about the ankle
  const yawOf = (t: number, s: number) => { const [h, b] = soles[t][s]; return Math.atan2(b[0] - h[0], b[2] - h[2]); };
  const wrapA = (a: number) => Math.atan2(Math.sin(a), Math.cos(a));
  const yo: number[][] = [[], []], onBallAt: boolean[][] = [[], []];
  for (let s = 0; s < 2; s++) {
    let lockYaw: number | null = null, onBall = false, rel: { at: number; x0: number; v0: number; h: number } | null = null;
    for (let t = 0; t < n; t++) {
      const [heel, ballP] = soles[t][s], prev = t ? yo[s][t - 1] : 0;
      if (stand[s][t]) {
        rel = null;
        onBall = (lockYaw !== null && onBall) || heel[1] > ballP[1] + heelUp;
        if (lockYaw === null) lockYaw = (t ? yawOf(t - 1, s) : yawOf(t, s)) + prev;
        if (onBall) lockYaw = yawOf(t, s) + prev; // (held: the clip turns it on its ball)
        yo[s].push(onBall ? prev : wrapA(lockYaw - yawOf(t, s)));
        onBallAt[s].push(onBall);
        continue;
      }
      onBallAt[s].push(false);
      if (lockYaw !== null) {
        const a = yo[s][t - 1], b = yo[s][Math.max(0, t - 2)];
        let e = t; while (e < n && !stand[s][e]) e++;
        rel = { at: t - 1, x0: a, v0: (a - b) / dt, h: Math.min(maxHalf, (e - t) * dt * half) };
        lockYaw = null; onBall = false;
      }
      yo[s].push(rel ? decay(rel.x0, rel.v0, rel.h, (t - rel.at) * dt) : 0);
    }
    for (let t = 0; t < n; t++) if (Math.abs(yo[s][t]) > 1e-6) {
      const an = feet[t][s].ankle, Q = qyaw(yo[s][t]);
      soles[t][s] = soles[t][s].map((p) => { const v = qrot(Q, [p[0] - an[0], p[1] - an[1], p[2] - an[2]]); return [an[0] + v[0], an[1] + v[1], an[2] + v[2]] as Vec3; });
    }
  }
  const off: Vec3[][] = [[], []];
  for (let s = 0; s < 2; s++) {
    let lock: [number, number] | null = null, onBall = false;
    let rel: { at: number; x0: Vec3; v0: Vec3; h: number } | null = null;
    for (let t = 0; t < n; t++) {
      const [heel, ballP] = soles[t][s], prev = t ? off[s][t - 1] : ([0, 0, 0] as Vec3);
      if (stand[s][t]) {
        rel = null;
        // the pivot: the heel until it is `heelUp` over the ball (the foot rolling forward), then the ball to the end
        const ball = onBallAt[s][t];
        const pivot = ball ? ballP : heel;
        if (!lock || ball !== onBall) {
          // touchdown, or the heel lifting: the lock taken where the (offset) pivot is now, so nothing moves this frame
          const was = t ? soles[t - 1][s][ball ? 1 : 0] : pivot;
          lock = t ? [was[0] + prev[0], was[2] + prev[2]] : [pivot[0], pivot[2]];
          onBall = ball;
        }
        if (o.log) { o.log.pivot[s][t] = ball ? "ball" : "heel"; o.log.lock[s][t] = [lock[0], lock[1]]; }
        // (the pivot on the floor; on the ball, the heel kept 1.6 cm up: a foot turning on its ball swept its heel round
        // across the floor)
        const o3: Vec3 = [lock[0] - pivot[0], ball ? Math.max(-pivot[1], 0.016 * z - heel[1]) : -pivot[1], lock[1] - pivot[2]];
        // let go once the lock is `radius` from where the clip has the foot (Holden's unlock radius): the body carried on
        // while a ball stayed locked, the leg stretched 5 to 14 cm short of it, and then snapped into the swing
        if (Math.hypot(o3[0], o3[2]) > radius && t > 0) {
          // (for a few frames, RAEA_LOCK_RELOCK (4; 0 for the rest of the stand), then it may lock again where it is once the clip's sole is slow (0.15 m/s): let
          // go for the whole stand, a pick-up's standing foot followed the clip's 3 cm wobble held up off the floor)
          const re = Number(process.env.RAEA_LOCK_RELOCK ?? 4);
          for (let e = t; e < n && stand[s][e]; e++) {
            let left = 0; while (e + left < n && stand[s][e + left]) left++;
            if (re > 0 && e >= t + re && left >= 5 && Math.min(speed(e, s, 0), speed(e, s, 1)) < 0.15 * z) break;
            stand[s][e] = false;
          }
        } else {
          off[s].push(o3);
          continue;
        }
      }
      if (o.log) { o.log.pivot[s][t] = null; o.log.lock[s][t] = null; }
      if (lock) {
        // let go: the spring starts from the offset and the speed it had over the stand's last frames
        const a = off[s][t - 1], b = off[s][Math.max(0, t - 2)];
        let e = t; while (e < n && !stand[s][e]) e++;
        rel = { at: t - 1, x0: a, v0: [0, 1, 2].map((k) => (a[k] - b[k]) / dt) as Vec3, h: Math.min(maxHalf, ((e - t) * dt * half)) };
        lock = null; onBall = false;
      }
      const u = rel ? (t - rel.at) * dt : 0;
      const d = rel ? ([0, 1, 2].map((k) => decay(rel!.x0[k], rel!.v0[k], rel!.h, u)) as Vec3) : ([0, 0, 0] as Vec3);
      // a swinging sole kept off the floor (1.6 cm): skimming it at swing speed is a skate
      d[1] = Math.max(d[1], 0.016 * z - low(t, s));
      off[s].push(d);
    }
    // the heights eased (a quarter, a half, a quarter, three times): a stand puts its pivot on the floor, a swing lifts
    // its sole clear, and the step between them is a jump
    for (let pass = 0; pass < Number(process.env.RAEA_LOCK_YPASS ?? 3); pass++) {
      const y = off[s].map((v) => v[1]);
      for (let t = 1; t < n - 1; t++) off[s][t][1] = 0.25 * y[t - 1] + 0.5 * y[t] + 0.25 * y[t + 1];
    }
    // each landing eased in: over the frames before touchdown the foot closes on the spot it locks at, arriving with no
    // speed (stopped dead in a frame, the knee kinked 4-6 degrees at every landing)
    const K = o.land ?? Number(process.env.RAEA_LOCK_LAND ?? 0);
    for (let t0 = 1; t0 < n; t0++) if (stand[s][t0] && !stand[s][t0 - 1]) {
      const A = feet[t0][s].ankle, d0 = off[s][t0];
      for (let k = 1; k <= K && t0 - k >= 0 && !stand[s][t0 - k]; k++) {
        const t = t0 - k, a = feet[t][s].ankle, d = off[s][t], u = 1 - k / (K + 1), w = u * u * (3 - 2 * u);
        for (const c of [0, 2]) d[c] -= w * (a[c] + d[c] - A[c] - d0[c]);
      }
    }
    // and after easing, a swinging sole still clear of the floor (easing pulled toe-offs back down to 1.2 cm, sliding)
    // (lifted smoothly: what each swinging frame needs, spread over the frames round it (the most of its neighbours',
    // faded over 3 frames, then averaged twice) and added; then the floor held. Held frame by frame with a bare max,
    // the lift came and went in a frame, the knee kinking with it: 9% of a short body's run frames)
    if (process.env.RAEA_LOCK_CLEAR !== "0") {
      const need = off[s].map((d, t) => (stand[s][t] ? 0 : Math.max(0, 0.016 * z - low(t, s) - d[1])));
      if (process.env.RAEA_LOCK_CLEAR_SMOOTH !== "0") {
        const Wd = 3, held = need.map((_, t) => { let m = 0; for (let k = -Wd; k <= Wd; k++) { const tt = t + k; if (tt < 0 || tt >= n || stand[s][tt] !== stand[s][t]) continue; m = Math.max(m, need[tt] * (1 - Math.abs(k) / (Wd + 1))); } return m; });
        for (let pass = 0; pass < 2; pass++) { const c = held.slice(); for (let t = 1; t < n - 1; t++) if (!stand[s][t]) held[t] = 0.25 * c[t - 1] + 0.5 * c[t] + 0.25 * c[t + 1]; }
        for (let t = 0; t < n; t++) if (!stand[s][t]) off[s][t][1] += Math.max(held[t], need[t]);
      } else for (let t = 0; t < n; t++) off[s][t][1] += need[t];
    }
  }
  // each leg solved to its ankle target (the clip's ankle plus the offset; never below the floor), knee toward the
  // clip's own
  if (o.log) o.log.stand = stand;
  // the hips lowered, smoothly, by what the legs need to reach their targets bent at least `bend` of their length
  // (a leg near straight turns its knee several degrees for a centimetre: every jog and run landing kinked the knee)
  const fold = o.fold ?? Number(process.env.RAEA_LOCK_FOLD ?? 0.97);
  const drop = new Array(n).fill(0);
  if (fold < 1) {
    for (let t = 0; t < n; t++) for (let s = 0; s < 2; s++) {
      const d = off[s][t], f = feet[t][s], tg = [f.ankle[0] + d[0], f.ankle[1] + d[1], f.ankle[2] + d[2]], h = f.hip;
      const dx = tg[0] - h[0], dz = tg[2] - h[2], L = fold * r.leg[s], dy = h[1] - tg[1];
      const need = L * L - dx * dx - dz * dz;
      drop[t] = Math.max(drop[t], need > 0 ? dy - Math.sqrt(need) : 0);
    }
    // eased over a third of a second either side (each frame the most of its neighbours', faded, then averaged)
    const W = Math.round(fps / 3), held = drop.map((_, t) => { let m = 0; for (let k = -W; k <= W; k++) { const v = drop[Math.max(0, Math.min(n - 1, t + k))]; m = Math.max(m, v * (1 - Math.abs(k) / (W + 1))); } return m; });
    for (let pass = 0; pass < 3; pass++) { const c = held.slice(); for (let t = 1; t < n - 1; t++) held[t] = 0.25 * c[t - 1] + 0.5 * c[t] + 0.25 * c[t + 1]; }
    for (let t = 0; t < n; t++) drop[t] = Math.min(held[t], 0.06 * z);
  }
  const out = raw.map((p0, t) => {
    const p = drop[t] > 1e-5 ? { ...p0, root: [p0.root[0], p0.root[1] - drop[t], p0.root[2]] as Vec3 } : p0;
    let q = p;
    for (let s = 0; s < 2; s++) {
      const d = off[s][t], f = feet[t][s];
      const tg: Vec3 = [f.ankle[0] + d[0], f.ankle[1] + d[1], f.ankle[2] + d[2]];
      if (o.log) { o.log.target[s][t] = tg; o.log.clip[s][t] = f.ankle; }
      if (Math.hypot(d[0], d[1], d[2]) < 1e-5 && Math.abs(yo[s][t]) < 1e-6) continue;
      if (Math.abs(yo[s][t]) > 1e-6) { const w = pose(skel, q); q = setWorld(skel, q, w, ai[s], qmul(qyaw(yo[s][t]), w.rot[ai[s]])); }
      const legs = [LEGS[s][0], LEGS[s][1], LEGS[s][2]];
      q = twoBone(skel, q, legs, tg, kneePole(skel, p, s), { keep: true });
    }
    return q;
  });
  // the legs' turns eased where the solve made them speed up or slow more suddenly than the clip's own (by 2 degrees a
  // frame, twice): a foot stopped at touchdown with the knee nearly straight swings the knee 5 degrees in a frame. Each
  // ease kept only where the standing sole stays within 2 mm.
  const passes = o.ease ?? Number(process.env.RAEA_LOCK_EASE ?? 3);
  for (let s = 0; s < 2; s++) {
    const js = [0, 1, 2].map((k) => joint(skel, LEGS[s][k]));
    const jerk = (P: Pose[], t: number, j: number) => {
      const v1 = qlog(qmul(qinv(P[t - 1].turns[j]), P[t].turns[j])), v2 = qlog(qmul(qinv(P[t].turns[j]), P[t + 1].turns[j]));
      return (Math.hypot(v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]) * 180) / Math.PI;
    };
    for (let pass = 0; pass < passes; pass++) {
      const was = out.map((q) => js.map((j) => q.turns[j]));
      for (let t = 1; t + 1 < n; t++) {
        if (!js.some((j) => jerk(out, t, j) > jerk(raw, t, j) + 2)) continue;
        const turns = out[t].turns.slice();
        js.forEach((j, i) => {
          const a = was[t - 1][i], b = was[t][i], c = was[t + 1][i], sg = (q: Quat) => (q[0] * b[0] + q[1] * b[1] + q[2] * b[2] + q[3] * b[3] < 0 ? -1 : 1);
          const sa = sg(a), sc = sg(c);
          turns[j] = qnorm([0, 1, 2, 3].map((k) => 0.25 * sa * a[k] + 0.5 * b[k] + 0.25 * sc * c[k]) as Quat);
        });
        const next = { ...out[t], turns };
        {
          // (standing, or a sole within 2.5 cm of the floor: eased swings at toe-off slid the foot along it)
          const A = pose(skel, out[t]), Bp = pose(skel, next);
          const sole = (w: typeof A, k: number) => { const v = qrot(w.rot[k], [0, -J[k].head[1], 0]); return [w.pos[k][0] + v[0], w.pos[k][1] + v[1], w.pos[k][2] + v[2]]; };
          const near = stand[s][t] || [ai[s], bi[s]].some((k) => Math.min(sole(A, k)[1], sole(Bp, k)[1]) < 0.025 * z);
          const moved = [ai[s], bi[s]].some((k) => { const a = sole(A, k), b = sole(Bp, k); return Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]) > 0.002 * z; });
          if (near && moved) continue;
        }
        out[t] = next;
      }
    }
  }
  if (o.log) out.forEach((q, t) => { const w = pose(skel, q); for (let s = 0; s < 2; s++) o.log!.reached[s][t] = w.pos[ai[s]]; });
  return out;
}

export const lockLog = (): LockLog => ({ stand: [[], []], pivot: [[], []], lock: [[], []], target: [[], []], reached: [[], []], clip: [[], []] });
