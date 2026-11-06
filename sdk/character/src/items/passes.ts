// interact's passes over the frames, a way clear of things, and a throw's keys and flight.
import type { Vec3 } from "raea/3d";
import { B, add, qinv, joint, len, palmAt, pose, qnorm, qrot, qyaw, reach, rest, scale3, sub, type HandShape, type Pose, type Quat, type Side, type Skeleton } from "raea/character";
import { surfacesOf, toLocal, type Handling, type Thing } from "../things";
import { Beat, G, ItemPlan, LAUNCH, ThrowStyle, Track, W, graspThing, mix, smooth, thingSdf } from "./all";

// the hips' drop over the frames: the most any frame within a third of a second needs, eased over as long, so the
// knees start bending before a low reach and never jump between frames (a throw's dip is keyed smooth, and added)
export function eased(v: number[], R = 10) {
    const most = v.map((_, f) => Math.max(...v.slice(Math.max(0, f - R), f + R + 1)));
    return most.map((_, f) => {
      let sw = 0, x = 0;
      for (let k = -R; k <= R; k++) { const wk = 1 + Math.cos((Math.PI * k) / (R + 1)); sw += wk; x += wk * most[Math.max(0, Math.min(most.length - 1, f + k))]; }
      return x / sw;
    });
}

// an arm carrying a thing smoothed over the frames either side (twice, a quarter, a half, a quarter): solved frame by
// frame its elbow and wrist took up every turn of the body under it, a hurdle's take-off and landing 1.4 times as
// sudden as any capture's (only where it carries, all three frames, not at the edges; and never taking a hand on a
// goal more than a centimetre further from it: a throw's wind-up fell 4.5 cm short)
export function smoothCarries(skel: Skeleton, poses: Pose[], work: { attach: { w: number; hands: Side[] }[]; goals: { hands: Partial<Record<Side, { at: Vec3 }>> } }[]) {
  const carries = (f: number, s: Side) => work[f].attach.some((a) => a.w >= 1 && a.hands.includes(s));
  for (const s of ["L", "R"] as Side[]) {
    const js = B.arm(s).map((x) => joint(skel, x));
    for (let pass = 0; pass < 2; pass++) {
      const was = poses.map((q) => js.map((j) => q.turns[j]));
      for (let f = 1; f + 1 < poses.length; f++) {
        if (!carries(f - 1, s) || !carries(f, s) || !carries(f + 1, s)) continue;
        const turns = poses[f].turns.slice();
        js.forEach((j, i) => {
          const a = was[f - 1][i], b = was[f][i], c = was[f + 1][i], sg = (q: Quat) => (q[0] * b[0] + q[1] * b[1] + q[2] * b[2] + q[3] * b[3] < 0 ? -1 : 1);
          const sa = sg(a), sc = sg(c);
          turns[j] = qnorm([0, 1, 2, 3].map((k) => 0.25 * sa * a[k] + 0.5 * b[k] + 0.25 * sc * c[k]) as Quat);
        });
        const goal = work[f].goals.hands[s], missOf = (q: Pose) => (goal ? len(sub(palmAt(skel, pose(skel, q), s), goal.at)) : 0);
        const next = { ...poses[f], turns };
        if (!goal || missOf(next) <= missOf(poses[f]) + 0.01) poses[f] = next;
      }
    }
  }
}

/** each dodge held over the frames either side, then eased: a hand dodging for one frame jerked the arm */
export function easeDodges(skel: Skeleton, plan: ItemPlan, poses: Pose[], dodge: Record<Side, (Vec3 | undefined)[]>, carried: (string | undefined)[], tracks: Map<string, Track[]>,
  grasped: { id: string; hands: Side[]; grip: Handling["grip"]; close: number }[][], shapes: Record<Side, (HandShape | undefined)[]>) {
  for (const s of ["L", "R"] as Side[]) {
    const d = dodge[s], n = poses.length, R = 4;
    if (!d.some(Boolean)) continue;
    const held = Array.from({ length: n }, (_, f) => {
      let best: Vec3 = [0, 0, 0];
      for (let k = -R; k <= R; k++) { const v = d[f + k]; if (v && len(v) > len(best)) best = v; }
      return best;
    });
    for (let f = 0; f < n; f++) {
      let v: Vec3 = [0, 0, 0], wsum = 0;
      for (let k = -R; k <= R; k++) { const g = f + k; if (g < 0 || g >= n) continue; const wk = R + 1 - Math.abs(k); v = add(v, scale3(held[g], wk)); wsum += wk; }
      v = scale3(v, 1 / wsum);
      if (len(v) > 1e-4) {
        poses[f] = reach(skel, poses[f], s, { at: add(palmAt(skel, pose(skel, poses[f]), s), v) });
        // (a thing in that hand, held over the same frames, goes with it)
        const id = carried.slice(Math.max(0, f - R), f + R + 1).find(Boolean);
        if (id && tracks.get(id)![f].held) { const tr = tracks.get(id)!; tr[f] = { ...tr[f], at: add(tr[f].at, v) }; }
        // (the fingers closed again on what that hand holds, as the hand now meets it)
        for (const gr of (grasped[f] ?? []).filter((g) => g.hands.includes(s))) {
          const tr = tracks.get(gr.id)!, sdf = thingSdf(plan.items[gr.id]);
          const hand = { shape: shapes[s][f - 1] };
          poses[f] = graspThing(skel, poses[f], s, (q: Vec3) => sdf(qrot(qinv(tr[f].rot), sub(q, tr[f].at))), gr.grip, gr.close, 1, hand);
          shapes[s][f] = hand.shape;
        }
      }
    }
  }
}

/**
 * A way from `a` to `b` for something `half` metres across (out to the side, up and down) that keeps clear of the
 * furniture: straight (with a little `arc` up) when the straight line is clear, else up from `a` to the height that
 * clears every piece of furniture it would have gone through, across above them, and down onto `b`, its corners
 * rounded. Returns the point `u` of the way along it (0..1, by distance, so the speed along it is even).
 */
export function clearWay(a: Vec3, b: Vec3, things: Thing[], half: [number, number], arc = 0): (u: number) => Vec3 {
  const [rh, rv] = half;
  const tops = things.map((t) => Math.max(t.size[1], ...surfacesOf(t).map((sf) => sf.y)));
  const inside = (p: Vec3, i: number) => {
    const t = things[i], l = toLocal(t, p);
    return Math.abs(l[0]) < t.size[0] / 2 + rh && Math.abs(l[2]) < t.size[2] / 2 + rh && l[1] < tops[i] + rv - 1e-3 && l[1] > -rv;
  };
  const hit = new Set<number>();
  for (let k = 1; k < 24; k++) { const p = mix(a, b, k / 24); things.forEach((_, i) => { if (inside(p, i)) hit.add(i); }); }
  if (!hit.size) return (u) => add(mix(a, b, u), [0, arc * Math.sin(Math.PI * u), 0]);
  // over the top of everything in the way, with room for it (and as high as the ends, if they are higher)
  const yc = Math.max(...[...hit].map((i) => things[i].at[1] + tops[i] + rv + 0.02));
  const pts: Vec3[] = [a];
  if (a[1] < yc) pts.push([a[0], yc, a[2]]);
  if (b[1] < yc) pts.push([b[0], yc, b[2]]);
  pts.push(b);
  // the corners rounded (a quadratic curve across each, cut back at most 5 cm or half the shorter side)
  const dense: Vec3[] = [pts[0]];
  for (let k = 1; k < pts.length - 1; k++) {
    const p0 = pts[k - 1], p1 = pts[k], p2 = pts[k + 1];
    const c = Math.min(0.05, len(sub(p1, p0)) / 2, len(sub(p2, p1)) / 2);
    const e0 = add(p1, scale3(sub(p0, p1), c / (len(sub(p0, p1)) || 1))), e1 = add(p1, scale3(sub(p2, p1), c / (len(sub(p2, p1)) || 1)));
    for (let j = 0; j <= 8; j++) { const w = j / 8; dense.push(add(add(scale3(e0, (1 - w) * (1 - w)), scale3(p1, 2 * w * (1 - w))), scale3(e1, w * w))); }
  }
  dense.push(pts[pts.length - 1]);
  const cum = [0];
  for (let k = 1; k < dense.length; k++) cum.push(cum[k - 1] + len(sub(dense[k], dense[k - 1])));
  const L = cum[cum.length - 1] || 1;
  return (u) => {
    const d = Math.min(1, Math.max(0, u)) * L;
    let k = 1;
    while (k < cum.length - 1 && cum[k] < d) k++;
    const seg = cum[k] - cum[k - 1] || 1;
    return mix(dense[k - 1], dense[k], (d - cum[k - 1]) / seg);
  };
}

/** A throw's keys in the body's frame: the thing's middle, the trunk's twist (radians, + to the left) and pitch (+
 * forward) and the hips' dip, at the hold, the wind-up, the release and the follow-through. A one-hand throw winds up
 * on its hand's side; a heave winds to the right. */
interface ThrowKey { mid: Vec3; twist: number; pitch: number; dip: number }

export function throwKeys(style: ThrowStyle, b: Beat, held: Vec3, m: { h: number; chestY: number; shoulderY: number; hipsY: number }): ThrowKey[] {
  const { h, chestY: c, shoulderY: sh, hipsY: hp } = m, sd = b.hands.length === 1 ? (b.hands[0] === "L" ? 1 : -1) : -1;
  const k = (mid: Vec3, twist = 0, pitch = 0, dip = 0): ThrowKey => ({ mid, twist, pitch, dip });
  const rest = k(held, 0, -Math.min(0.18, 0.6 * b.how.load));
  switch (style) {
    // (each follow-through carries on the way the hand was going as it let go, toward the target, and slows there)
    case "overhand": return [rest, k([sd * 0.17 * h, sh + 0.06 * h, -0.1 * h], sd * 0.5, -0.1, 0.02 * h), k([sd * 0.08 * h, sh + 0.08 * h, 0.18 * h], -sd * 0.3, 0.1), k([-sd * 0.06 * h, c - 0.12 * h, 0.32 * h], -sd * 0.45, 0.25, 0.03 * h)];
    case "underhand": return [rest, k([sd * 0.13 * h, hp - 0.03 * h, -0.1 * h], sd * 0.25, 0.2, 0.09 * h), k([sd * 0.07 * h, c - 0.08 * h, 0.24 * h], -sd * 0.1, 0, 0.01 * h), k([sd * 0.05 * h, sh - 0.08 * h, 0.26 * h], -sd * 0.15, -0.05)];
    case "chest": return [rest, k([0, c - 0.04 * h, 0.11 * h], 0, -0.05, 0.03 * h), k([0, c - 0.02 * h, 0.3 * h], 0, 0.1), k([0, c + 0.01 * h, 0.36 * h], 0, 0.14)];
    case "toss": return [rest, k([0, hp - 0.07 * h, 0.15 * h], 0, 0.3, 0.13 * h), k([0, c - 0.02 * h, 0.26 * h], 0, -0.05), k([0, sh + 0.08 * h, 0.31 * h], 0, -0.1)];
    case "heave": return [rest, k([sd * 0.14 * h, hp - 0.03 * h, 0.08 * h], sd * 0.6, 0.3, 0.15 * h), k([-sd * 0.04 * h, c, 0.26 * h], -sd * 0.25, -0.05), k([-sd * 0.08 * h, sh + 0.02 * h, 0.33 * h], -sd * 0.4, 0)];
  }
}

/** Where a throw is at `u` of its length: eased to the wind-up, sped up through the release (the hand fastest as it
 * lets go), eased out through the follow-through, then the trunk and knees back as the arms go back to the walk. */
export function throwAt(keys: ThrowKey[], u: number): ThrowKey {
  // the wind-up eased from the hold; then Hermite curves through the release, the speed there set so the hand comes
  // into it like an ease-in and leaves at the same speed and way, bent toward the follow-through, slowing to a stop
  // (the follow-through's reach and the trunk's carry-on set by that speed, so the hand never overshoots and comes
  // back)
  const v = (k: ThrowKey) => [...k.mid, k.twist, k.pitch, k.dip];
  const [a, b, c, f] = keys.map(v), d1 = W.release - W.wind, d2 = W.follow - W.release;
  const n = (x: number[]) => { const l = Math.hypot(x[0], x[1], x[2]) || 1; return [x[0] / l, x[1] / l, x[2] / l]; };
  const into = n(c.map((x, i) => x - b[i])), out = n(f.map((x, i) => x - c[i])), dir = n(into.map((x, i) => x + out[i]));
  const reachIn = Math.hypot(c[0] - b[0], c[1] - b[1], c[2] - b[2]);
  // the release's velocity (per unit u): the ease-in's end speed, for the place along `dir`; for the trunk, along its way
  const vel = [...dir.map((x) => (x * 2 * reachIn) / d1), ...[3, 4, 5].map((i) => (2 * (c[i] - b[i])) / d1)];
  const end = c.map((x, i) => (i < 3 ? x + out[i] * ((vel.slice(0, 3).reduce((s2, y) => s2 + y * y, 0) ** 0.5) * d2) / 2.5 : x + (vel[i] * d2) / 2.5));
  const hermite = (p0: number[], m0: number[], p1: number[], m1: number[], w: number) => {
    const w2 = w * w, w3 = w2 * w, h00 = 2 * w3 - 3 * w2 + 1, h10 = w3 - 2 * w2 + w, h01 = -2 * w3 + 3 * w2, h11 = w3 - w2;
    return p0.map((x, i) => h00 * x + h10 * m0[i] + h01 * p1[i] + h11 * m1[i]);
  };
  const zero = [0, 0, 0, 0, 0, 0];
  const key = (x: number[]): ThrowKey => ({ mid: [x[0], x[1], x[2]], twist: x[3], pitch: x[4], dip: x[5] });
  if (u < W.wind) { const w = smooth(u / W.wind); return key(a.map((x, i) => x + (b[i] - x) * w)); }
  if (u < W.release) return key(hermite(b, zero, c, vel.map((x) => x * d1), (u - W.wind) / d1));
  if (u < W.follow) return key(hermite(c, vel.map((x) => x * d2), end, zero, (u - W.release) / d2));
  const back = smooth((u - W.follow) / (1 - W.follow)), e = key(end);
  return { mid: e.mid, twist: e.twist * (1 - back), pitch: e.pitch * (1 - back), dip: e.dip * (1 - back) };
}

/** The velocity that lands a thing let go at `from` on `to` at the style's angle (steeper when the target is high),
 * no faster than the body throws a thing of that load that way (an adult's most, less for a child's height and less
 * the heavier it is): too far, and it falls short. */
export function launch(from: Vec3, to: Vec3, style: ThrowStyle, load: number, height: number): Vec3 {
  const dx = to[0] - from[0], dz = to[2] - from[2], d = Math.hypot(dx, dz), dy = to[1] - from[1];
  const most = LAUNCH[style].most * Math.min(1, height / 1.75) ** 1.5 * Math.max(0.15, 1 - 2.2 * load);
  let a = LAUNCH[style].angle;
  while (d * Math.tan(a) - dy < 0.05 && a < 1.3) a += 0.05;
  const v = d < 0.01 ? 1 : Math.min(most, Math.sqrt((G * d * d) / (2 * Math.cos(a) ** 2 * Math.max(0.01, d * Math.tan(a) - dy))));
  const ux = d < 0.01 ? 0 : dx / d, uz = d < 0.01 ? 0 : dz / d;
  return [ux * v * Math.cos(a), v * Math.sin(a), uz * v * Math.cos(a)];
}

/** how long a thing launched at `vel` from `from` takes to come down to `floor` */
export function flightTime(from: Vec3, vel: Vec3, floor: number): number {
  const dy = from[1] - floor, disc = vel[1] * vel[1] + 2 * G * dy;
  return disc < 0 ? 0 : (vel[1] + Math.sqrt(disc)) / G;
}

/** the upright turn (about up only) nearest a turn: the way a box landing on its bottom faces */
export function uprightNear(q: Quat): Quat {
  const up = qrot(q, [0, 1, 0]), ax = Math.abs(up[0]), ay = Math.abs(up[1]), az = Math.abs(up[2]);
  // whichever of its axes is nearest up stays up; a box lands on any face, so the yaw is taken from a level axis
  const f = ay >= ax && ay >= az ? qrot(q, [0, 0, 1]) : qrot(q, [0, 1, 0]);
  return qyaw(Math.atan2(f[0], f[2]));
}
