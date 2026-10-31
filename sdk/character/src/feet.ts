// Feet for raea/matching: planting a performance's feet on the floor, and measuring how well they stay put.
//
// A foot stands on its heel (the ankle over it) from the moment it comes down, then rolls onto its ball as the heel
// lifts. While it stands, the point it stands on is locked where it came down and the leg is bent (two-bone IK) to keep
// it there, whatever the clip's own foot does: the body is pulled onto its route and blended between clips, and the
// clip's foot slides with that. When it lifts, what the lock moved it by fades out over the start of the swing, so the
// next step lands where the swing takes it. Where a locked foot would need the leg longer than the clip's own, the foot rolls
// onto its ball (the heel lifts) as a trailing leg does; the hips are never lowered (that buckled the knees).
//
// footDrift measures any performance: how far each standing foot's pivot slides (cm per contact, and its speed) and how
// fast the knees bend while the feet stand (a buckle), so planting can be judged by numbers rather than by eye.
import type { Vec3 } from "raea/3d";
import { bodyScale, joint, pose, qaxis, qinv, qmul, qnorm, qrot, twoBone, type Pose, type Quat, type Skeleton } from "raea/character";
import { qlog } from "./effects";

const SIDES = [["LeftUpLeg", "LeftLeg", "LeftFoot", "LeftToeBase"], ["RightUpLeg", "RightLeg", "RightFoot", "RightToeBase"]] as const;

interface FootFrame {
  hip: Vec3;
  ankle: Vec3;
  ball: Vec3;
  rot: Quat;
}

/** each frame's feet, posed */
function feetOf(skel: Skeleton, p: Pose): FootFrame[] {
  const w = pose(skel, p);
  return SIDES.map((names) => {
    const [h, , a, b] = names.map((n) => joint(skel, n));
    return { hip: w.pos[h], ankle: w.pos[a], ball: w.pos[b], rot: w.rot[a] };
  });
}

/** the floor heights of a standing ankle and ball (the skeleton's at rest, standing on the floor), and the body's size
 * (`z`, bodyScale: the centimetres and speeds here were set on a child) */
function rests(skel: Skeleton): { ankle: number; ball: number; leg: number[]; z: number } {
  const J = skel.joints;
  const leg = SIDES.map((names) => {
    const [h, k, a] = names.map((n) => J[joint(skel, n)].head);
    return Math.hypot(k[0] - h[0], k[1] - h[1], k[2] - h[2]) + Math.hypot(a[0] - k[0], a[1] - k[1], a[2] - k[2]);
  });
  return { ankle: J[joint(skel, "LeftFoot")].head[1], ball: J[joint(skel, "LeftToeBase")].head[1], leg, z: bodyScale(skel) };
}

/** Which frames each foot stands: low (ankle within 3 cm of its standing height, or the ball within 1.2 cm: a child's swing
 * clears the floor by little more) and the clip says it stands (or, without the clip's word, it is nearly still); gaps of two frames filled, stands shorter than four frames dropped. */
export function standing(skel: Skeleton, feet: FootFrame[][], hint?: [number, number][], fps = 30, roots?: Vec3[]): boolean[][] {
  const r = rests(skel), n = feet.length, z = r.z;
  const out: boolean[][] = [[], []];
  for (let s = 0; s < 2; s++) {
    for (let t = 0; t < n; t++) {
      const f = feet[t][s], a = feet[Math.max(0, t - 1)][s], b = feet[Math.min(n - 1, t + 1)][s];
      const span = Math.max(1, Math.min(n - 1, t + 1) - Math.max(0, t - 1));
      const heelLow = f.ankle[1] < r.ankle + 0.03 * z, ballLow = f.ball[1] < r.ball + 0.012 * z;
      const v = (heelLow ? Math.hypot(b.ankle[0] - a.ankle[0], b.ankle[2] - a.ankle[2]) : Math.hypot(b.ball[0] - a.ball[0], b.ball[2] - a.ball[2])) * fps / span;
      // (where an action clip says the foot stands, 3, a few centimetres up is still standing: planting puts it on the
      // floor; and 2 is a foot stood on something above the floor, a box: standing wherever it is)
      const hinted = hint ? hint[t][s] > 0 : false, nearLow = Math.min(f.ankle[1] - r.ankle, f.ball[1] - r.ball) < 0.05 * z;
      // (and at a run, 1.6 m/s on, a foot the clip stands up to 4 cm up: a running capture's stance skims the floor)
      const ra = roots?.[Math.max(0, t - 1)], rb = roots?.[Math.min(n - 1, t + 1)];
      const running = ra && rb ? (Math.hypot(rb[0] - ra[0], rb[2] - ra[2]) * fps) / span > 1.6 * z : false;
      const acted = hint ? (hint[t][s] === 3 && nearLow) || hint[t][s] === 2 || (hint[t][s] === 1 && running && Math.min(f.ankle[1] - r.ankle, f.ball[1] - r.ball) < 0.04 * z) : false;
      out[s].push((heelLow || ballLow || acted) && (hint ? hinted : v < 0.3 * z));
    }
    const c = out[s];
    // a stand held on past the clip's word while the ball is still down (up to 4 frames): the toes push off before
    // they lift, and let go with them on the floor the foot slides off as its swing starts
    // (down on what it stood on: the lowest the ball was in the stand, where that is a box's top)
    for (let t = n - 1; t > 0; t--) if (c[t - 1] && !c[t]) {
      let b0 = t - 1, lowBall = Infinity;
      while (b0 >= 0 && c[b0]) lowBall = Math.min(lowBall, feet[b0--][s].ball[1]);
      const base = lowBall > r.ball + 0.06 * z ? lowBall : r.ball;
      for (let k = t; k < Math.min(n, t + 4) && feet[k][s].ball[1] < base + 0.015 * z; k++) c[k] = true;
    }
    // and begun early where the foot comes in low, the ball within 1.5 cm and slower than 1.2 m/s (up to 4 frames): a
    // capture's foot can land skimming on, and left swinging there it slides along the floor into its stand
    for (let t = 1; t < n; t++) if (c[t] && !c[t - 1]) {
      for (let k = t - 1; k >= Math.max(0, t - 4) && !c[k]; k--) {
        const f = feet[k][s], g = feet[Math.min(n - 1, k + 1)][s];
        if (f.ball[1] >= r.ball + 0.015 * z || Math.hypot(g.ball[0] - f.ball[0], g.ball[2] - f.ball[2]) * fps >= 1.2 * z) break;
        c[k] = true;
      }
    }
    // gaps of two frames filled, and up to eight where the ball stays down all through (within 2.5 cm): a capture's
    // foot pivoting on its ball between heel and toe-off, let go, slid 8 cm back across the floor)
    for (let t = 1; t < n - 1; t++) if (!c[t]) {
      let e = t, low = true;
      while (e < n && !c[e]) { low &&= feet[e][s].ball[1] < r.ball + 0.025 * z; e++; }
      if ((e - t <= 2 || (e - t <= 8 && low)) && t > 0 && c[t - 1] && e < n) for (let k = t; k < e; k++) c[k] = true;
      t = e;
    }
    for (let t = 0; t < n; t++) if (c[t]) { let e = t; while (e < n && c[e]) e++; if (e - t < 4) for (let k = t; k < e; k++) c[k] = false; t = e; }
  }
  return out;
}

/** The feet planted: see the head of the file. `hint` is the clip's own contacts per frame (each foot: 0 up, 1 down, and
 * from an action clip, measured from its own floor, 3 down and 2 stood on something above the floor). */
/** What planting did, frame by frame, for checks (motionLint): each foot's state and its events, the hips' drop (m)
 * and each foot's heel lift (radians). */
export interface PlantLog {
  state: ("swing" | "heel" | "ball")[][];
  events: { t: number; foot: 0 | 1; what: "lock" | "roll" | "release" | "restep" }[];
  drop: number[];
  lift: number[][];
}

export function plantFeet(skel: Skeleton, raw: Pose[], hint?: [number, number][], fps = 30, log?: PlantLog): Pose[] {
  const n = raw.length;
  if (!n) return raw;
  const r = rests(skel), z = r.z;
  let feet = raw.map((p) => feetOf(skel, p));
  const stand = standing(skel, feet, hint, fps, raw.map((p) => p.root));
  // the knee's bend (degrees) a leg needs to span `d` from hip to ankle
  const bones = SIDES.map((nm) => { const [h, k, a] = nm.map((x) => skel.joints[joint(skel, x)].head); return [Math.hypot(k[0] - h[0], k[1] - h[1], k[2] - h[2]), Math.hypot(a[0] - k[0], a[1] - k[1], a[2] - k[2])]; });
  const bendFor = (s: number, d: number) => { const [la, lb] = bones[s], c = (la * la + lb * lb - d * d) / (2 * la * lb); return 180 - (Math.acos(Math.max(-1, Math.min(1, c))) * 180) / Math.PI; };
  // each foot's ankle target where it is held (null where it swings free), and the offset it carries into a swing
  const target: (Vec3 | null)[][] = [[], []], stepping: boolean[][] = [[], []];
  const sm = (x: number) => { const c = Math.min(1, Math.max(0, x)); return c * c * (3 - 2 * c); };
  for (let s = 0; s < 2; s++) {
    let carry: [number, number] = [0, 0], carryAt = -1, carryLen = 1;
    let lock: [number, number] | null = null, onBall = false, minAnkle = Infinity, minBall = Infinity, ballAt = 0;
    let restep: { from: Vec3; at: number; len: number } | null = null;
    for (let t = 0; t < n; t++) {
      const f = feet[t][s];
      // the offset left from the last stand, fading over the first part of the swing
      let off: [number, number] = [0, 0];
      if (carryAt >= 0) {
        const u = Math.min(1, (t - carryAt) / carryLen), k = 1 - u * u * (3 - 2 * u);
        off = [carry[0] * k, carry[1] * k];
      }
      if (!stand[s][t]) {
        if (lock) {
          // lifting off: carry what the lock moved the foot by into the swing (over 60% of it, 10 frames at most)
          const prev = target[s][t - 1]!, was = feet[t - 1][s];
          carry = [prev[0] - was.ankle[0], prev[2] - was.ankle[2]];
          let e = t; while (e < n && !stand[s][e]) e++;
          // over 60% of the swing (10 frames at most), longer for a big offset (no faster than 1.5 cm a frame, up to the
          // whole swing: the first step from standing carries 19 cm, and made up fast the thigh whipped round 19° a frame)
          carryAt = t - 1; carryLen = Math.max(2, Math.min(Math.max(10, e - t), Math.max(Math.round((e - t) * 0.6), Math.ceil(Math.hypot(carry[0], carry[1]) / (0.015 * z)))));
          const u = 1 / carryLen, k = 1 - u * u * (3 - 2 * u);
          off = [carry[0] * k, carry[1] * k];
          log?.events.push({ t, foot: s as 0 | 1, what: "release" });
          lock = null; onBall = false; restep = null;
        }
        // swinging, still held to the clip's foot (and what it carries): the leg is solved every frame, so the IK starting
        // or stopping never snaps a knee, and a swing under lowered hips keeps its height
        target[s].push([f.ankle[0] + off[0], f.ankle[1], f.ankle[2] + off[1]]);
        if (log) log.state[s][t] = "swing";
        continue;
      }
      // standing: the heel's spot where the foot came down (where it shows, the carried offset with it), then the
      // ball's once the heel lifts, taken from where the held foot's ball is at that moment
      const ballOff: [number, number] = [f.ball[0] - f.ankle[0], f.ball[2] - f.ankle[2]];
      if (!lock) { lock = [f.ankle[0] + off[0], f.ankle[2] + off[1]]; onBall = false; carryAt = -1; minAnkle = Infinity; minBall = Infinity; log?.events.push({ t, foot: s as 0 | 1, what: "lock" }); }
      // (or the heel 1.2 cm up from its lowest in this stand: a foot standing a little high lifts its heel before it is 3
      // cm up, and held at the ankle meanwhile its toes swept back 4 cm across the floor)
      // (stood on a box's top, 6 cm up and more, measured from where the foot is on it: from the floor, its heel was
      // always up and its ball never down, so it never rolled and slid off the top on its heel)
      minAnkle = Math.min(minAnkle, f.ankle[1]); minBall = Math.min(minBall, f.ball[1]);
      const onTop = minAnkle > r.ankle + 0.06 * z, ballBase = onTop ? minBall : r.ball;
      // (and a heel still under the floor hasn't lifted: a foot laid on the floor in a roll, its ankle 7 cm under its
      // standing height, rolled onto its ball as the ankle came up and was held at the floor, skidding round the ball)
      const heelUp = ((!onTop && f.ankle[1] > r.ankle + 0.03 * z) || (f.ankle[1] > minAnkle + 0.012 * z && f.ankle[1] > r.ankle - 0.01 * z)) && f.ball[1] < ballBase + 0.03 * z;
      if (!onBall && heelUp) {
        const prev: Vec3 = target[s][t - 1] ?? [lock[0], 0, lock[1]], pb = feet[Math.max(0, t - 1)][s];
        lock = [prev[0] + (pb.ball[0] - pb.ankle[0]), prev[2] + (pb.ball[2] - pb.ankle[2])];
        onBall = true;
        log?.events.push({ t, foot: s as 0 | 1, what: "roll" });
      }
      // (on the ball, the ball on the floor: a capture's foot rolling up out of a dive-roll had its toes 3 cm up, pivoting
      // on nothing; eased down over eight frames and back up over the stand's last three, so neither the roll nor the
      // lift jerks the ankle)
      let left = t; while (left < n && stand[s][left]) left++;
      if (onBall) ballAt++; else ballAt = 0;
      const ballDown = onBall ? Math.max(0, f.ball[1] - ballBase) * Math.min(sm(ballAt / 8), sm((left - t) / 3)) : 0;
      let tg: Vec3 = onBall ? [lock[0] - ballOff[0], f.ankle[1] - ballDown, lock[1] - ballOff[1]] : [lock[0], f.ankle[1], lock[1]];
      // a foot left too far from where the clip has it (8% of the height: the body stopping or turning over it) takes a
      // quick step there, lifted a little, rather than the leg contorting to reach it; then it stands again
      // (or one the knee would have to bend 8° more for than the clip bends it: near straight a leg folds a lot for a
      // little, 2 cm shorter folded a knee 34°, a buckle as the body stops or turns over a planted foot)
      const hip = f.hip, legTo = (a: Vec3) => Math.hypot(a[0] - hip[0], a[1] - hip[1], a[2] - hip[2]);
      // (not in a stand's last few frames: the foot is about to go anyway, and a step back across the floor at toe-off is
      // a slide)
      if (!restep && left - t > 6 && (Math.hypot(tg[0] - f.ankle[0], tg[2] - f.ankle[2]) > 0.08 * skel.height || bendFor(s, legTo(tg)) - bendFor(s, legTo(f.ankle)) > 8)) {
        // (with the other foot standing all the while: stepped as the other swings, the body is in the air)
        const d = Math.hypot(tg[0] - f.ankle[0], tg[2] - f.ankle[2]), len = Math.max(12, Math.min(20, Math.round(d / (0.008 * z))));
        let other = true;
        for (let k = t; k < Math.min(n, t + len) && other; k++) other = stand[1 - s][k];
        if (other) {
          restep = { from: tg, at: t, len };
          log?.events.push({ t, foot: s as 0 | 1, what: "restep" });
        }
      }
      if (restep) {
        // (a step: up first, across while up, down last; slid across low it is a skate)
        // (lifted 3.5% of the height over its first 30%, across over the middle 40%, down over the last 30%)
        const u = Math.min(1, (t - restep.at) / restep.len);
        const k = sm((u - 0.3) / 0.4), up = Math.min(sm(u / 0.3), sm((1 - u) / 0.3)), fr = restep.from;
        tg = [fr[0] + (f.ankle[0] - fr[0]) * k, f.ankle[1] + 0.035 * skel.height * up, fr[2] + (f.ankle[2] - fr[2]) * k];
        if (u >= 1) { lock = [f.ankle[0], f.ankle[2]]; onBall = false; restep = null; log?.events.push({ t, foot: s as 0 | 1, what: "lock" }); }
      }
      target[s].push(tg);
      stepping[s][t] = !!restep;
      if (log) log.state[s][t] = restep ? "swing" : onBall ? "ball" : "heel";
    }
  }
  // a standing foot put on the floor: captures (and other bodies' legs on them) stand a centimetre or a few above it, so
  // each stand is lowered by its lowest sole's height over the stand (once on the floor, a heel lifting still lifts),
  // faded over ten frames into the swings either side
  for (let s = 0; s < 2; s++) {
    const dy = new Array(n).fill(0), at: [number, number, number][] = [];
    for (let t = 0; t < n; t++) {
      if (!stand[s][t]) continue;
      let e = t, low = Infinity, acted = false;
      while (e < n && stand[s][e]) { const f = feet[e][s]; low = Math.min(low, f.ankle[1] - r.ankle, f.ball[1] - r.ball); acted ||= (hint?.[e][s] ?? 0) >= 2; e++; }
      // (a stand more than 6 cm up is on something, left where it is; a stride's stands are left as the library has them,
      // a standing still's, half a second on, put down too)
      const d = low > 0.06 * z || (!acted && e - t < 15) ? 0 : Math.max(0, low);
      for (let k = t; k < e; k++) dy[k] = d;
      at.push([t, e - 1, d]);
      t = e;
    }
    for (let t = 0; t < n; t++) {
      if (stand[s][t]) continue;
      let v = 0;
      for (const [a0, b0, d] of at) { const gap = t < a0 ? a0 - t : t - b0; v = Math.max(v, d * Math.max(0, 1 - gap / 10)); }
      dy[t] = v;
    }
    for (let t = 0; t < n; t++) if (dy[t] && target[s][t]) target[s][t] = [target[s][t]![0], target[s][t]![1] - dy[t], target[s][t]![2]];
  }
  // a swinging foot clears the floor: one skimming it under 1.5 cm while moving on at swing speed (1.2 m/s and up, the
  // most at 2) is lifted to it, the lift smoothed over the frames either side so the swing doesn't hitch
  for (let s = 0; s < 2; s++) {
    // (and no foot, standing or swinging, below the floor: a capture's landing jolts its foot through it)
    const need = new Array(n).fill(0);
    for (let t = 0; t < n; t++) {
      const tg = target[s][t], a = target[s][Math.max(0, t - 1)], b = target[s][Math.min(n - 1, t + 1)];
      if (!tg || !a || !b) continue;
      const f = feet[t][s], dy = tg[1] - f.ankle[1], low = Math.min(f.ankle[1] - r.ankle, f.ball[1] - r.ball) + dy;
      const v = (Math.hypot(b[0] - a[0], b[2] - a[2]) * fps) / 2, w = stand[s][t] ? 0 : Math.min(1, Math.max(0, (v - 1.2 * z) / (0.8 * z)));
      need[t] = Math.max(Math.max(0, 0.015 * z - low) * w, -low);
    }
    for (let t = 0; t < n; t++) {
      if (!target[s][t]) continue;
      let up = 0;
      for (let k = -2; k <= 2; k++) { const m = need[Math.max(0, Math.min(n - 1, t + k))]; up = Math.max(up, m * (1 - Math.abs(k) / 3)); }
      if (up > 0) target[s][t] = [target[s][t]![0], target[s][t]![1] + up, target[s][t]![2]];
    }
  }
  // no hips lowered: lowering them folded a leg near straight at heel strike (2 cm folded it 15-20°, a buckle the lint
  // found at every lock); a foot out of reach rolls onto its ball instead, below
  // a heel lift let go with the foot, faded over the start of its swing like the offset
  const rollGo: ({ ax: Vec3; th: number; d: Vec3; at: number; len: number } | null)[] = [null, null];
  const lastRoll: ({ ax: Vec3; th: number; d: Vec3 } | null)[] = [null, null];
  // a standing foot out of the leg's reach (past the clip's own leg, or 99.5% of it) rolls up onto its ball (the heel
  // lifts, the ball stays put), as a trailing leg does at the end of a long step: the heel's turn (about the ball's
  // across axis, level and square to the foot) that brings the ankle into reach, 0.8 at most
  const rollFor = (s: number, q: Pose, tg: Vec3): { th: number; ax: Vec3; lifted: (th: number) => Vec3; out: boolean } => {
    const [hi, , ai, bi] = SIDES[s].map((nm) => joint(skel, nm));
    const w = pose(skel, q), hip = w.pos[hi], own = Math.hypot(...[0, 1, 2].map((i) => w.pos[ai][i] - hip[i]) as [number, number, number]);
    const L = Math.min(0.9995 * r.leg[s], Math.max(0.995 * r.leg[s], own));
    const reachOf = (a: Vec3) => Math.hypot(a[0] - hip[0], a[1] - hip[1], a[2] - hip[2]);
    const ball: Vec3 = [tg[0] + (w.pos[bi][0] - w.pos[ai][0]), w.pos[bi][1], tg[2] + (w.pos[bi][2] - w.pos[ai][2])];
    const along: Vec3 = [tg[0] - ball[0], tg[1] - ball[1], tg[2] - ball[2]];
    const ax = norm3([along[2], 0, -along[0]]);
    const lifted = (th: number): Vec3 => { const v = rotAbout(ax, th, along); return [ball[0] + v[0], ball[1] + v[1], ball[2] + v[2]]; };
    if (reachOf(tg) <= L) return { th: 0, ax, lifted, out: false };
    const up = lifted(0.1)[1] > tg[1] ? 1 : -1;
    let lo = 0, hi2 = 0.8;
    if (reachOf(lifted(up * hi2)) <= L) for (let k = 0; k < 10; k++) { const m = (lo + hi2) / 2; if (reachOf(lifted(up * m)) > L) lo = m; else hi2 = m; }
    return { th: up * hi2, ax, lifted, out: true };
  };
  const out = raw.map((p, t) => {
    if (log) { log.drop[t] = 0; log.lift[0][t] = 0; log.lift[1][t] = 0; }
    let q: Pose = p;
    for (let s = 0; s < 2; s++) {
      let tg = target[s][t];
      if (!tg) continue;
      const legs = [SIDES[s][0], SIDES[s][1], SIDES[s][2]];
      const [hi, , ai, bi] = SIDES[s].map((nm) => joint(skel, nm));
      const roll = (ax: Vec3, th: number) => {
        const w2 = pose(skel, q), shin = skel.joints[ai].parent;
        const turns = q.turns.slice();
        turns[ai] = qmul(qinv(w2.rot[shin]), qmul(qaxis(ax, th), w2.rot[ai]));
        q = { ...q, turns };
      };
      if (stand[s][t]) {
        rollGo[s] = null;
        lastRoll[s] = null;
        const rf = rollFor(s, q, tg);
        if (rf.out) {
          const { ax, lifted, th } = rf;
          if (log) log.lift[s][t] = Math.abs(th);
          const was = tg;
          tg = lifted(th);
          q = twoBone(skel, q, legs, tg, undefined, { keep: true });
          roll(ax, th);
          // the ankle's move with it too, so letting go doesn't drop the heel back in a frame
          lastRoll[s] = { ax, th, d: [tg[0] - was[0], tg[1] - was[1], tg[2] - was[2]] };
          continue;
        }
      } else if (lastRoll[s]) {
        // let go rolled: the roll fades over the first part of the swing (40% of it, 8 frames at most)
        let e = t; while (e < n && !stand[s][e]) e++;
        rollGo[s] = { ...lastRoll[s]!, at: t - 1, len: Math.max(2, Math.min(8, Math.round((e - t) * 0.4))) };
        lastRoll[s] = null;
      }
      const g = rollGo[s];
      const k = !stand[s][t] && g ? (() => { const u = Math.min(1, (t - g.at) / g.len); return 1 - u * u * (3 - 2 * u); })() : 0;
      if (g && k > 1e-3) tg = [tg[0] + g.d[0] * k, tg[1] + g.d[1] * k, tg[2] + g.d[2] * k];
      const base = q, own = pose(skel, p).pos[ai], clipBend = kneeOf(skel, p, s);
      q = twoBone(skel, base, legs, tg, undefined, { keep: true });
      // never a buckle: a knee bent more than 12° past the clip's own lets the foot give toward the clip's, just enough
      // (never down: the clip's foot can be through the floor)
      // (not in a quick step: a foot lifted to step folds the knee, as it should)
      if (!stepping[s][t] && kneeOf(skel, q, s) - clipBend > 12) {
        let lo = 0, hi3 = 1;
        for (let it = 0; it < 8; it++) {
          const m = (lo + hi3) / 2, t2: Vec3 = [tg[0] + (own[0] - tg[0]) * m, tg[1] + Math.max(0, own[1] - tg[1]) * m, tg[2] + (own[2] - tg[2]) * m];
          if (kneeOf(skel, twoBone(skel, base, legs, t2, undefined, { keep: true }), s) - clipBend > 12) lo = m; else hi3 = m;
        }
        tg = [tg[0] + (own[0] - tg[0]) * hi3, tg[1] + Math.max(0, own[1] - tg[1]) * hi3, tg[2] + (own[2] - tg[2]) * hi3];
        q = twoBone(skel, base, legs, tg, undefined, { keep: true });
      }
      if (g && k > 1e-3) roll(g.ax, g.th * k);
      else if (g && !stand[s][t]) rollGo[s] = null;
    }
    return q;
  });
  // a leg's turns smoothed where planting made them change speed more suddenly than the clip's, by 2°/f² (twice, a
  // quarter, a half, a quarter): a lock, a roll or the knee guard giving way turned a knee or hip in a frame
  for (let s = 0; s < 2; s++) {
    const js = [SIDES[s][0], SIDES[s][1], SIDES[s][2]].map((nm) => joint(skel, nm)), ai = js[2], bi = joint(skel, SIDES[s][3]);
    const jerk = (P: Pose[], t: number, j: number) => {
      const v1 = qlog(qmul(qinv(P[t - 1].turns[j]), P[t].turns[j])), v2 = qlog(qmul(qinv(P[t].turns[j]), P[t + 1].turns[j]));
      return (Math.hypot(v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]) * 180) / Math.PI;
    };
    for (let pass = 0; pass < 2; pass++) {
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
        // (a foot standing or near the floor kept within 2 mm of where it was: 5 mm, summed over a stand, slid it)
        { const A = pose(skel, out[t]).pos, B = pose(skel, next).pos, lim = stand[s][t] || A[ai][1] < r.ankle + 0.03 * r.z || A[bi][1] < r.ball + 0.03 * r.z ? 0.002 * r.z : 1; if ([ai, bi].some((k) => Math.hypot(A[k][0] - B[k][0], A[k][1] - B[k][1], A[k][2] - B[k][2]) > lim)) continue; }
        out[t] = next;
      }
    }
  }
  feet = [];
  return out;
}

export interface Drift {
  /** foot skate: the feet's speed across the floor weighted by how near the floor they are (1 on it, 0 from H up: 2.5 cm
   * for the heel, 1.5 for the ball), cm/s over all frames, both feet: the usual measure, near 0 for planted feet */
  skateCms: number;
  /** stands measured (heel within 1.5 cm of standing, or the ball within 1 cm, four frames or more), both feet */
  stands: number;
  /** how far the standing point slid over a stand, cm: the mean and the worst */
  slideCm: number;
  worstCm: number;
  /** how fast the knees bent while standing, degrees a frame: the 95th percentile and the most (a buckle) */
  kneeP95: number;
  kneeMax: number;
}

/** How well a performance's feet stay put while they stand (see the head of the file). */
export function footDrift(skel: Skeleton, poseAt: (f: number) => Pose, frames: number, fps = 30): Drift {
  const r = rests(skel);
  const feet: FootFrame[][] = [], knees: number[][] = [];
  for (let f = 0; f < frames; f++) {
    const p = poseAt(f), w = pose(skel, p);
    feet.push(feetOf(skel, p));
    knees.push(SIDES.map((names) => {
      const [h, k, a] = names.map((x) => w.pos[joint(skel, x)]);
      const u = [h[0] - k[0], h[1] - k[1], h[2] - k[2]], v = [a[0] - k[0], a[1] - k[1], a[2] - k[2]];
      return (Math.acos(Math.max(-1, Math.min(1, (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / Math.hypot(...u) / Math.hypot(...v)))) * 180) / Math.PI;
    }));
  }
  // the floor under the feet: how low each point stands (its 5th percentile: a body floating a centimetre over the
  // floor still stands, and one odd frame below it doesn't move the floor)
  const low = (a: number[]) => a.slice().sort((x, y) => x - y)[Math.floor(a.length * 0.05)] ?? 0;
  const floorA = low(feet.flatMap((ff) => ff.map((x) => x.ankle[1]))), floorB = low(feet.flatMap((ff) => ff.map((x) => x.ball[1])));
  let skate = 0;
  for (let t = 1; t < frames; t++) for (let s = 0; s < 2; s++) {
    const a = feet[t - 1][s], b = feet[t][s];
    for (const [pa, pb, fl, H] of [[a.ankle, b.ankle, floorA, 0.025], [a.ball, b.ball, floorB, 0.015]] as [Vec3, Vec3, number, number][]) {
      const h = (pa[1] + pb[1]) / 2 - fl, w = Math.max(0, Math.min(1, 2 - 2 ** (h / H)));
      skate += w * Math.hypot(pb[0] - pa[0], pb[2] - pa[2]) * fps * 100;
    }
  }
  skate /= Math.max(1, (frames - 1) * 2);
  const heelOn = (x: FootFrame) => x.ankle[1] - floorA < 0.015, ballOn = (x: FootFrame) => x.ball[1] - floorB < 0.01;
  // standing: low and slower than a swing (a child's swing skims the floor: low alone took the swing's start for a
  // slide; a foot skidding faster than 0.5 m/s would go uncounted, and shows in the skate)
  const sp = (s: number, t: number) => { const a = feet[Math.max(0, t - 1)][s], b = feet[Math.min(frames - 1, t + 1)][s]; return Math.hypot(b.ball[0] - a.ball[0], b.ball[2] - a.ball[2]) * fps / 2; };
  const st = [0, 1].map((s) => feet.map((ff, t) => (heelOn(ff[s]) || ballOn(ff[s])) && sp(s, t) < 0.5));
  for (const c of st) for (let t = 0; t < frames; t++) if (c[t]) { let e = t; while (e < frames && c[e]) e++; if (e - t < 4) for (let k = t; k < e; k++) c[k] = false; t = e; }
  const slides: number[] = [], kneeV: number[] = [];
  for (let s = 0; s < 2; s++) {
    let acc = 0, inStand = false;
    for (let t = 1; t < frames; t++) {
      if (st[s][t] && st[s][t - 1]) {
        // the pivot: the heel while it is down, else the ball
        const a = feet[t - 1][s], b = feet[t][s];
        const d = heelOn(a) && heelOn(b) ? Math.hypot(b.ankle[0] - a.ankle[0], b.ankle[2] - a.ankle[2]) : Math.hypot(b.ball[0] - a.ball[0], b.ball[2] - a.ball[2]);
        acc += d; inStand = true;
        kneeV.push(Math.abs(knees[t][s] - knees[t - 1][s]));
      } else if (inStand) { slides.push(acc * 100); acc = 0; inStand = false; }
    }
    if (inStand) slides.push(acc * 100);
  }
  const pct = (a: number[], q: number) => { if (!a.length) return 0; const b = a.slice().sort((x, y) => x - y); return b[Math.min(b.length - 1, Math.floor(b.length * q))]; };
  const mean = (a: number[]) => (a.length ? a.reduce((x, y) => x + y, 0) / a.length : 0);
  return { skateCms: skate, stands: slides.length, slideCm: mean(slides), worstCm: Math.max(0, ...slides), kneeP95: pct(kneeV, 0.95), kneeMax: Math.max(0, ...kneeV) };
}

const norm3 = (v: Vec3): Vec3 => { const l = Math.hypot(v[0], v[1], v[2]) || 1; return [v[0] / l, v[1] / l, v[2] / l]; };
const rotAbout = (axis: Vec3, th: number, v: Vec3): Vec3 => qrot(qaxis(axis, th), v);
/** a knee's bend, degrees: the shin's turn at the knee (at rest the leg is straight) */
const kneeOf = (skel: Skeleton, p: Pose, s: number) => { const q = p.turns[joint(skel, SIDES[s][1])], w = Math.min(1, Math.abs(q[3])); return (2 * Math.acos(w) * 180) / Math.PI; };
