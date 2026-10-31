// Actions: a clip's floor, levelled, and fitted onto an obstacle.
import type { Vec3 } from "raea/3d";
import { bodyScale, captured, joint, pose, type Posed, type Skeleton } from "raea/character";
import { framePose, type Clip } from "../clip";

// ---------------------------------------------------------------- actions

/** an obstacle a clip clears: a bar to jump, a box to run over, a bar to duck under, a block to dive-roll over, a gap to leap */
export type ObstacleKind = "hurdle" | "box" | "duck" | "dive" | "gap";

/** What an action clip needs of a course to clear an obstacle of its kind: its run-up and run-out, its moment over
 * (or under) the obstacle (the act step's `anchor`), and the obstacle's shape as the body clears it, along the way the
 * clip travels. */
export interface ActionFit {
  kind: ObstacleKind;
  clip: string;
  anchor: number;
  /** the hips' speed into and out of the clip, m/s */
  vIn: number;
  vOut: number;
  /** how far the hips travel before the anchor and after it, along the way */
  dIn: number;
  dOut: number;
  /** the obstacle along the way (from the anchor, metres) and its bottom and top */
  from: number;
  to: number;
  bottom: number;
  top: number;
}

/** A clip's floor and each frame's contacts measured from it: captures stand a few centimetres above or below their
 * floor once on another body (its still feet's soles, the lower fifth of them, so a stretch stood on a box doesn't
 * count; a foot stands within 5 cm of it, moving under 0.4 m/s). perform stands an action clip on the floor by it, and
 * fitAction measures what a clip clears from it. */
export function clipFloor(skel: Skeleton, c: Clip): { floor: number; contact: [number, number][] } {
  const hit = floors.get(c);
  if (hit && hit.skel === skel) return hit.g;
  const F = ["LeftFoot", "RightFoot", "LeftToeBase", "RightToeBase"].map((n) => joint(skel, n));
  // (centimetres and speeds by the body's size, bodyScale: a capture on a bigger body travels further in the same time)
  const rest = F.map((i) => skel.joints[i].head[1]), z = bodyScale(skel);
  const soles = Array.from({ length: c.frames }, (_, f) => { const w = pose(skel, framePose(c, f)); return F.map((i, q) => [w.pos[i][0], w.pos[i][1] - rest[q], w.pos[i][2]] as Vec3); });
  const lowAt = (f: number, s: number) => (soles[f][s][1] < soles[f][s + 2][1] ? soles[f][s] : soles[f][s + 2]);
  const speedAt = (f: number, s: number) => { const a = Math.max(0, f - 1), b = Math.min(c.frames - 1, f + 1), p = lowAt(a, s), q = lowAt(b, s); return Math.hypot(q[0] - p[0], q[2] - p[2]) * c.fps / Math.max(1, b - a); };
  const still: number[] = [];
  for (let f = 0; f < c.frames; f++) for (const s of [0, 1]) if (speedAt(f, s) < 0.4 * z) still.push(lowAt(f, s)[1]);
  still.sort((m, n2) => m - n2);
  const floor = still.length ? Math.max(-0.03 * z, Math.min(0.06 * z, still[Math.floor(still.length * 0.2)])) : 0;
  // (a foot also stands, as 2, on something above the floor: still, and at its lowest about then, on a box)
  const localLow = (f: number, s: number) => { let m = Infinity; for (let g = Math.max(0, f - 8); g <= Math.min(c.frames - 1, f + 8); g++) m = Math.min(m, lowAt(g, s)[1]); return m; };
  const contact = Array.from({ length: c.frames }, (_, f) => [0, 1].map((s) => {
    const y = lowAt(f, s)[1], still = speedAt(f, s) < 0.4 * z;
    // (and one on the floor, within 1.5 cm, sliding: a capture's feet land from a roll still going, and a foot on
    // the floor that the clip slides is standing, planting puts it down)
    if (!still) return y - floor < 0.015 * z && speedAt(f, s) < 1.2 * z ? 1 : 0;
    if (y - floor < 0.05 * z) return 1;
    return localLow(f, s) - floor > 0.1 * z && y - localLow(f, s) < 0.03 * z ? 2 : 0;
  })) as [number, number][];
  const g = { floor, contact };
  floors.set(c, { skel, g });
  return g;
}

const floors = new WeakMap<Clip, { skel: Skeleton; g: { floor: number; contact: [number, number][] } }>();

/** A clip on a level floor: a capture's floor is often off level by a few centimetres over a run's length (one end of the
 * volume calibrated higher), so a stance at that end hovers or sinks however the clip is stood. The floor is fitted, as
 * a line along the way the hips go, through the soles of the feet standing still (those well over the line, on a box,
 * dropped, twice over), and taken off the root's height: the clip's floor at 0 all along. Unchanged with too few stances
 * to fit, and only the mean taken off when it barely moves. */
export function levelClip(skel: Skeleton, c: Clip): Clip {
  const F = ["LeftFoot", "RightFoot", "LeftToeBase", "RightToeBase"].map((n) => joint(skel, n)), rest = F.map((i) => skel.joints[i].head[1]), z = bodyScale(skel);
  const W = Array.from({ length: c.frames }, (_, f) => pose(skel, framePose(c, f)));
  const a = W[0].pos[0], b = W[c.frames - 1].pos[0], L = Math.hypot(b[0] - a[0], b[2] - a[2]);
  const along = (f: number) => (L > 0.5 ? ((W[f].pos[0][0] - a[0]) * (b[0] - a[0]) + (W[f].pos[0][2] - a[2]) * (b[2] - a[2])) / L : 0);
  const pts: [number, number][] = [];
  for (let f = 0; f < c.frames; f++) for (const s of [0, 1]) {
    const lowAt = (g: number) => { const p = W[g].pos[F[s]][1] - rest[s] < W[g].pos[F[s + 2]][1] - rest[s + 2] ? F[s] : F[s + 2]; return W[g].pos[p]; };
    const p0 = lowAt(Math.max(0, f - 1)), p1 = lowAt(Math.min(c.frames - 1, f + 1));
    if ((Math.hypot(p1[0] - p0[0], p1[2] - p0[2]) * c.fps) / 2 > 0.4 * z) continue;
    pts.push([along(f), Math.min(W[f].pos[F[s]][1] - rest[s], W[f].pos[F[s + 2]][1] - rest[s + 2])]);
  }
  if (pts.length < 8) return c;
  let k0 = 0, k1 = 0, use = pts;
  for (let it = 0; it < 3; it++) {
    const n = use.length, mx = use.reduce((m, p) => m + p[0], 0) / n, my = use.reduce((m, p) => m + p[1], 0) / n;
    const sxx = use.reduce((m, p) => m + (p[0] - mx) ** 2, 0), sxy = use.reduce((m, p) => m + (p[0] - mx) * (p[1] - my), 0);
    k1 = L > 0.5 && sxx > 0.05 * n ? sxy / sxx : 0;
    k0 = my - k1 * mx;
    const next = pts.filter((p) => p[1] - (k0 + k1 * p[0]) < 0.04 * z);
    if (next.length < 8) break;
    use = next;
  }
  // (no more than 8 cm over its length: a steeper fit is a wrong one; and the line at the lower fifth of the stances,
  // as clipFloor stands it, not their middle)
  if (Math.abs(k1) * L > 0.08 * z) k1 = 0;
  const res = use.map((p) => p[1] - k1 * p[0]).sort((x, y) => x - y);
  k0 = res[Math.floor(res.length * 0.2)];
  const root = new Float32Array(c.root);
  for (let f = 0; f < c.frames; f++) root[f * 3 + 1] -= k0 + k1 * along(f);
  return { ...c, root };
}

/** How an action clip clears an obstacle of a kind (its longest flight, its stretch up on a box, its lowest head, its
 * highest whole body), so a course can be laid out from what its captures clear. */
export function fitAction(skel: Skeleton, c: Clip, kind: ObstacleKind): ActionFit {
  const n = c.frames, W: Posed[] = Array.from({ length: n }, (_, f) => pose(skel, framePose(c, f)));
  // (lengths and speeds by the body's size, bodyScale: set on the child, a man's head ducked into the bar)
  const J = (s: string) => joint(skel, s), z = bodyScale(skel);
  const feet = ["LeftFoot", "RightFoot", "LeftToeBase", "RightToeBase"].map(J), head = J("Head");
  const hip = (f: number) => W[f].pos[0];
  const a = hip(0), b = hip(n - 1), L = Math.hypot(b[0] - a[0], b[2] - a[2]), dir = [(b[0] - a[0]) / L, (b[2] - a[2]) / L];
  const along = (p: number[]) => (p[0] - a[0]) * dir[0] + (p[2] - a[2]) * dir[1];
  // (the capture's joints: the fingers' own reach past them, and the course was laid from these)
  const nc = captured(skel), low = (f: number) => Math.min(...W[f].pos.slice(0, nc).map((p) => p[1]));
  const lowFoot = (f: number) => Math.min(...feet.map((i) => W[f].pos[i][1]));
  const speed = (f0: number, f1: number) => Math.hypot(hip(f1)[0] - hip(f0)[0], hip(f1)[2] - hip(f0)[2]) * c.fps / (f1 - f0);
  const still = (f: number, i: number) => { const p = Math.max(0, f - 1), q = Math.min(n - 1, f + 1); return Math.hypot(...[0, 1, 2].map((k) => W[q].pos[i][k] - W[p].pos[i][k])) * c.fps / (q - p) < 0.5 * z; };
  const planted = (f: number) => [0, 1].some((s) => still(f, feet[s]) || still(f, feet[s + 2]));
  const mid = (f: number) => f > n * 0.2 && f < n * 0.8;
  let anchor = 0, from = 0, to = 0, bottom = 0, top = 0;
  const argBest = (score: (f: number) => number) => { let best = -1, bs = -Infinity; for (let f = 0; f < n; f++) if (mid(f) && score(f) > bs) { bs = score(f); best = f; } return best; };
  if (kind === "hurdle" || kind === "gap") {
    // the longest flight: neither foot planted
    let best: [number, number] = [0, 0];
    for (let f = 1; f < n; f++) {
      if (planted(f) || !planted(f - 1)) continue;
      let g = f;
      while (g < n && !planted(g)) g++;
      if (g - f > best[1] - best[0]) best = [f, g];
      f = g;
    }
    anchor = best[0];
    for (let f = best[0]; f < best[1]; f++) if (hip(f)[1] > hip(anchor)[1]) anchor = f;
    if (kind === "hurdle") {
      from = -0.03 * z; to = 0.03 * z; bottom = 0; top = Math.max(0.15 * z, low(anchor) - 0.06 * z);
    } else {
      // the gap between where the toes leave and where they land, a hand short of each
      const s = along(hip(anchor));
      // the planted foot's toes as it leaves, and the landing foot's heel as it lands
      const leaving = [0, 1].filter((k) => still(best[0] - 1, feet[k]) || still(best[0] - 1, feet[k + 2]));
      const landing = [0, 1].filter((k) => still(best[1], feet[k]) || still(best[1], feet[k + 2]));
      // (past the ball by the toes, 6% of the height, and 5 cm more: the run-in arrives a few cm off the clip's)
      from = Math.max(...(leaving.length ? leaving : [0, 1]).map((k) => along(W[best[0] - 1].pos[feet[k + 2]]))) + 0.06 * skel.height + 0.05 * z - s;
      to = Math.min(...(landing.length ? landing : [0, 1]).map((k) => along(W[best[1]].pos[feet[k]]))) - 0.08 * z - s;
      bottom = -0.6 * z; top = 0;
    }
  } else if (kind === "box") {
    // the stretch both feet are up off the floor: on the box
    const up: number[] = [];
    for (let f = 0; f < n; f++) if (lowFoot(f) > 0.15 * z) up.push(f);
    const f0 = up[0], f1 = up[up.length - 1];
    anchor = Math.round((f0 + f1) / 2);
    const s = along(hip(anchor));
    const tops = up.filter((f) => planted(f)).map(lowFoot).sort((x, y) => x - y);
    // (its lower fifth: the feet stand on it, none of them in it)
    top = (tops[Math.floor(tops.length * 0.2)] ?? 0.3 * z) - 0.02 * z;
    // from the first foot stood on it to the last, a foot's length beyond each (not the feet swinging over its ends)
    const on: number[] = [];
    for (let f = Math.max(0, f0 - 3); f <= Math.min(n - 1, f1 + 3); f++) for (const i of feet) if (still(f, i) && W[f].pos[i][1] > top - 0.05 * z) on.push(along(W[f].pos[i]));
    from = (on.length ? Math.min(...on) : Math.min(...feet.map((i) => along(W[f0].pos[i])))) - 0.08 * z - s;
    to = (on.length ? Math.max(...on) : Math.max(...feet.map((i) => along(W[f1].pos[i])))) + 0.08 * z - s;
    bottom = 0;
  } else if (kind === "duck") {
    anchor = argBest((f) => -W[f].pos[head][1]);
    let hi = 0;
    for (let f = anchor - 4; f <= anchor + 4; f++) hi = Math.max(hi, W[Math.max(0, Math.min(n - 1, f))].pos[head][1]);
    bottom = hi + 0.17 * z; top = bottom + 0.06 * z; from = -0.04 * z; to = 0.04 * z;
  } else {
    // the dive: the whole body highest off the floor
    anchor = argBest(low);
    bottom = 0; top = Math.max(0.15 * z, low(anchor) - 0.08 * z);
    // as long as nothing comes down within a few cm of its top over it (the hands planted before it, the feet after)
    const s = along(hip(anchor));
    from = -0.2 * z; to = 0.2 * z;
    for (let f = 0; f < n; f++) for (const p of W[f].pos.slice(0, nc)) {
      if (p[1] > top + 0.05 * z) continue;
      const d = along(p) - s;
      if (d < 0 && d > from) from = d;
      if (d >= 0 && d < to) to = d;
    }
    from = Math.min(from + 0.02 * z, -0.04 * z); to = Math.max(to - 0.02 * z, 0.04 * z);
  }
  const sA = along(hip(anchor));
  // (heights from the clip's floor, which perform stands it on)
  const fl = clipFloor(skel, c).floor, gapK = kind === "gap";
  return { kind, clip: c.name, anchor, vIn: speed(0, 8), vOut: speed(n - 9, n - 1), dIn: sA, dOut: L - sA, from, to, bottom: gapK ? bottom : bottom - fl, top: gapK ? top : top - fl };
}
