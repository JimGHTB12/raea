// What interact asks of the body and the plan, beat by beat.
import type { Vec3 } from "raea/3d";
import { B, add, cross, qinv, joint, len, norm, pinchPoint, pose, qmul, qrot, qyaw, rest, scale3, sub, type Side, type Skeleton } from "raea/character";
import { type Performance } from "../matching";
import { handOn, pickGuide } from "../grasp";
import { shapeOf, surfacesOf, toLocal } from "../things";
import { Beat, D, FRONT, GUIDED, ItemPlan, SEGMENTS, T, lerp, thingSdf, trunkTurn, turnGuide } from "./all";

/** A performance of an item plan with its reaches, grips, holds, releases and throws laid over it. */
/** what interact asks of the body and the plan, beat by beat: where the body stands, where hands go on a thing, how a
 * hold leans and shifts the body, and where a thing lands */
export function beatGeometry(skel: Skeleton, perf: Performance, plan: ItemPlan) {
  const h = skel.height, dt = 1 / plan.fps;
  const J = skel.joints;
  const chestY = J[joint(skel, B.chest)].head[1], shoulderY = J[joint(skel, B.upperarm("L"))].head[1], hipsY = J[0].head[1];
  // the shoulders at rest, in the body's frame (its middle over the floor), and an arm's length
  const shoulder: Record<Side, Vec3> = { L: J[joint(skel, B.upperarm("L"))].head, R: J[joint(skel, B.upperarm("R"))].head };
  const armLength = len(sub(J[joint(skel, B.lowerarm("L"))].head, J[joint(skel, B.upperarm("L"))].head)) + len(sub(J[joint(skel, B.hand("L"))].head, J[joint(skel, B.lowerarm("L"))].head));
  const beatAt = new Map(plan.beats.map((b) => [b.step, b]));
  // the body's ground frame at a frame: the hips over the floor, its heading the hips' (over half a second, so the
  // hold doesn't sway with each step)
  const ground = (f: number): { at: Vec3; yaw: number } => {
    let sx = 0, sz = 0;
    for (let k = -7; k <= 7; k++) {
      const v = qrot(perf.pose(Math.max(0, Math.min(perf.frames - 1, f + k))).turns[0], [0, 0, 1]);
      sx += v[0]; sz += v[2];
    }
    const r = perf.pose(f).root;
    return { at: [r[0], 0, r[2]], yaw: Math.atan2(sx, sz) };
  };
  const toScene = (g: { at: Vec3; yaw: number }, v: Vec3): Vec3 => add(g.at, qrot(qyaw(g.yaw), v));
  // the thing's half sizes in the grip's frame (its across, high, deep once turned by `turn`)
  const halves = (b: Beat): Vec3 => {
    const s = plan.items[b.item].size, odd = Math.abs(Math.round(b.turn / (Math.PI / 2))) % 2 === 1;
    return odd ? [s[2] / 2, s[1] / 2, s[0] / 2] : [s[0] / 2, s[1] / 2, s[2] / 2];
  };
  // each hand on the thing for its grip, in the grip's frame from the thing's middle: where, the palm facing, the
  // fingers pointing
  const handsOn = (b: Beat): Partial<Record<Side, { at: Vec3; facing: Vec3; point: Vec3 }>> => {
    if (b.how.grip === "hug" || b.how.grip === "cradle" || b.how.grip === "edges") return bigHold(b).on;
    const [ex, ey, ez] = halves(b), it = plan.items[b.item], out: Partial<Record<Side, { at: Vec3; facing: Vec3; point: Vec3 }>> = {};
    for (const s of b.hands) {
      const sd = s === "L" ? 1 : -1;
      // a guide the grip takes: the hand planned on it in its own frame (grasp.ts), the guide turned into the grip's
      const on = guided(b, s);
      if (on) { out[s] = on; continue; }
      switch (b.how.grip) {
        // (every hand comes to the thing from the body's side of it: on its near half, the fingers reaching forward
        // and down round it, never round its far side)
        // the sides' near halves, low, the fingers down and forward
        case "sides": out[s] = { at: [sd * (ex + 0.008), -0.25 * ey, -0.3 * ez], facing: [-sd, 0, 0], point: [0, -0.6, 1] }; break;
        // the lower edges of its sides: the palm low on the side, turned up a little to the edge, the fingers down
        // past it to curl under the bottom
        case "wrap": out[s] = { at: [sd * (ex + 0.008), 0, -0.35 * ez], facing: [-sd, 0, 0.3], point: [0, 0.1, 1] }; break;
        // from above, the fingers across its length (along it, it went between the index and middle fingers), the
        // thing where the captured pinch's pads meet (pinchPoint) rather than under the palm
        case "pinch": {
          const n: Vec3 = [0, -1, 0], f = norm(ez >= ex ? [-sd, -0.3, 0.3] : [-sd * 0.3, -0.3, 1]);
          const c = s === "R" ? cross(f, n) : cross(n, f), [pa, pp, pc] = pinchPoint(skel, s);
          const target: Vec3 = ez >= ex ? [0, 0, -0.25 * ez] : [sd * 0.25 * ex, 0, 0];
          const mid = sub(target, add(add(scale3(f, pa), scale3(n, pp)), scale3(c, pc)));
          out[s] = { at: add(mid, scale3(n, 0.009 * h)), facing: n, point: f };
          break;
        }
        // near its back end, from above and the side as it is written with, the fingers along it toward the point
        case "tripod": out[s] = { at: [sd * (ex + 0.006), 0.6 * ey, -0.25 * ez], facing: [-sd * 0.7, -0.7, 0], point: [0, -0.35, 1] }; break;
        // round the stem from its side, the fingers closing round it, the bowl over the hand
        case "stem": {
          const g = it.grips![0].at[1] - it.size[1] / 2;
          out[s] = { at: [sd * 0.012, g, -0.012], facing: [-sd, 0, 0.3], point: [0, 0, 1] };
          break;
        }
        // the palm on its top, a little to the body's side of its middle, the fingers forward and down round it
        case "top": out[s] = { at: [0, ey + 0.004, -0.15 * ez], facing: [0, -1, 0], point: [-sd * 0.25, 0, 1] }; break;
        // round the ball's upper side, the fingers over it
        case "palm": out[s] = { at: [sd * 0.6 * ex, 0.45 * ey, -0.45 * ez], facing: [-sd * 0.6, -0.7, 0.5], point: [0, -0.5, 1] }; break;
        case "handle": {
          const g = it.grips![0].at, at = qrot(qyaw(b.turn), [g[0], g[1] - it.size[1] / 2, g[2]]);
          out[s] = { at, facing: [-at[0], 0, -at[2]], point: [0, -0.2, 1] };
          break;
        }
        case "pole": out[s] = b.hands.length === 2 ? { at: [sd * 0.5 * ex, ey + 0.005, 0], facing: [0, -1, 0], point: [0, 0, 1] } : { at: [0, ey + 0.005, 0], facing: [0, -1, 0], point: [-sd, 0, 0.2] }; break;
      }
    }
    return out;
  };
  // a hand on the thing's guide for its grip (grasp.ts), in the grip's frame from its middle: a hand round a can comes
  // at it from its own side and a little behind (the palm on that side, the axis diagonal across it, the fingers round
  // one way and the thumb the other); one from above takes it by its end (the palm on its top, the fingers down its far
  // side and the thumb down its near one)
  const guideMemo = new Map<Beat, Partial<Record<Side, { at: Vec3; facing: Vec3; point: Vec3 } | null>>>();
  const guided = (b: Beat, s: Side): { at: Vec3; facing: Vec3; point: Vec3 } | undefined => {
    const m = guideMemo.get(b) ?? {};
    guideMemo.set(b, m);
    if (m[s] !== undefined) return m[s] ?? undefined;
    const it = plan.items[b.item], sd = s === "L" ? 1 : -1, grip = b.how.grip;
    const g = GUIDED.has(grip) ? pickGuide(it, grip, b.tag) : undefined;
    let r: { at: Vec3; facing: Vec3; point: Vec3 } | null = null;
    // (a round thing's body wrapped, or taken by its end from above: the guides that beat the per-grip places on the
    // grips bench; a ball's sphere in the palm or from above held on as many frames as the places below, or fewer)
    if (g?.kind === "cylinder") {
      const T = qyaw(b.turn), half: Vec3 = [0, it.size[1] / 2, 0];
      const toGrip = (v: Vec3) => qrot(T, sub(v, half)), dir = (v: Vec3) => qrot(T, v);
      const gg = turnGuide(g, toGrip, dir);
      const approach: Vec3 = grip === "top" ? [0, 1, -0.15] : [sd, 0, -0.35];
      const on = handOn(gg, s, approach, { hand: 0.1 * h, ahead: [0, 0, 1] });
      r = { at: on.at, facing: on.facing, point: on.point };
    }
    m[s] = r;
    return r ?? undefined;
  };
  // the hold, in the body's frame: the thing's middle. One hand carries it at the side, the arm hanging a little bent;
  // two hold it before the chest, lower and closer the heavier it is; a hug holds it against the belly; a pole for two
  // hands lies across before the hips
  // a big thing (cradled or hugged) against the body: its near side on the belly or chest, as low as the hands can
  // still bear it, and where on it each hand goes, the most bearing place in reach (as far under it as the arms go;
  // a kid's arms round a big ball reach only its near sides, a man's under it). Places on it, each with how well it
  // bears the thing: a ball's surface from under it up past its middle and round toward the body; a box's bottom (the
  // further under the better), its sides' lower edges, and its sides
  type Place = { at: Vec3; facing: Vec3; point: Vec3; bear: number };
  const bigHolds = new Map<Beat, { at: Vec3; on: Partial<Record<Side, Place>> }>();
  const bigHold = (b: Beat) => {
    if (bigHolds.has(b)) return bigHolds.get(b)!;
    const [ex, ey, ez] = halves(b), heavy = Math.min(1, b.how.load / 0.2), ball = shapeOf(plan.items[b.item]) === "ball";
    const places = (sd: number): Place[] => {
      const out: Place[] = [];
      if (b.how.grip === "edges") {
        // (a box held by its lower edges: those alone, from its near end to its far, the nearer its middle the better: the
        // hands either side of its weight, as on its slab guide. At a third as much, a man took a shoe box lying lengthwise
        // by its near end, its middle 14 cm past his hands, which held it on a quarter of the frames)
        for (let t = -0.6; t <= 0.61; t += 0.2) out.push({ at: [sd * (ex + 0.008), -ey + Math.min(0.5 * ey, 0.035 * h), t * ez], facing: norm([-sd, 0.35, 0]), point: norm([0, -0.8, 0.6]), bear: 1 - Math.abs(t) });
      } else if (ball) {
        for (let ph = -1.2; ph <= 0.31; ph += 0.1) for (let ps = 0; ps <= 1.01; ps += 0.1) {
          const d: Vec3 = [sd * Math.cos(ph) * Math.cos(ps), Math.sin(ph), -Math.cos(ph) * Math.sin(ps)];
          // (the fingers along the surface, forward)
          const fw = sub([0, 0, 1], scale3(d, d[2]));
          out.push({ at: scale3(d, ex + 0.004), facing: scale3(d, -1), point: len(fw) > 0.2 ? norm(fw) : [0, 1, 0], bear: -ph - 0.3 * ps });
        }
      } else {
        for (let t = -0.6; t <= 0.61; t += 0.2) {
          for (const u of [0.55, 0.75, 0.9]) out.push({ at: [sd * u * ex, -ey - 0.004, t * ez], facing: [0, 1, 0], point: norm([-sd * 0.35, 0, 1]), bear: 1.4 + 0.3 * t });
          out.push({ at: [sd * (ex + 0.008), -ey + Math.min(0.5 * ey, 0.035 * h), t * ez], facing: norm([-sd, 0.35, 0]), point: norm([0, -0.8, 0.6]), bear: 0.9 + 0.2 * t });
          for (const k of [-0.3, 0, 0.3]) out.push({ at: [sd * (ex + 0.008), k * ey, t * ez], facing: [-sd, 0, 0], point: [0, -0.3, 1], bear: 0.3 - 0.2 * k + 0.1 * t });
        }
      }
      return out;
    };
    const cand: Record<Side, Place[]> = { L: places(1), R: places(-1) };
    // (an arm reaching a place round the thing's side, not through it: the line from the shoulder to the hand, the
    // elbow bending out from it, kept out of the thing but for its last tenth)
    const sdf = thingSdf(plan.items[b.item]), inThing = (q: Vec3, c: Vec3) => sdf(qrot(qyaw(-b.turn), [q[0] - c[0], q[1] - c[1] + ey, q[2] - c[2]]));
    const armClear = (s: Side, at: Vec3, pl: Place) => {
      const hand = add(at, pl.at);
      for (let k = 1; k < 9; k++) if (inThing(add(shoulder[s], scale3(sub(hand, shoulder[s]), k / 10)), at) < -0.015) return false;
      return true;
    };
    // (one held by its edges is held out before the body as far as is easy, up to a fifth of the height, rather than
    // against it)
    const y0 = hipsY + 0.06 * h - 0.03 * h * heavy, z0 = FRONT * h + ez, zs = b.how.grip === "edges" ? [0, 0.03, 0.06, 0.09].map((k) => z0 + k * h) : [z0];
    let best: { score: number; at: Vec3; on: Partial<Record<Side, Place>> } | undefined, near: typeof best;
    for (const z of zs) for (let y = y0; y <= chestY + 1e-6; y += 0.01) {
      const at: Vec3 = [0, y, z], on: Partial<Record<Side, Place>> = {};
      let bear = Infinity, far = 0;
      for (const s of b.hands) {
        const ok = cand[s].filter((p) => len(sub(add(at, p.at), shoulder[s])) <= 0.88 * armLength && armClear(s, at, p)).sort((p, q) => q.bear - p.bear);
        const nearest = cand[s].reduce((p, q) => (len(sub(add(at, q.at), shoulder[s])) < len(sub(add(at, p.at), shoulder[s])) ? q : p));
        on[s] = ok[0] ?? nearest;
        if (!ok.length) far = Math.max(far, len(sub(add(at, nearest.at), shoulder[s])));
        bear = Math.min(bear, on[s]!.bear);
      }
      // (lower is easier, by a bear's worth over a tenth of the height; and held out a fifth of the height before the
      // body when light, drawn in against it as it gets heavy: its weight's lever shrinks, at a tenth of the body's
      // mass all the way)
      const score = bear - (y - y0) / (0.1 * h) - Math.abs(z - lerp(Math.max(z0, 0.2 * h), z0, Math.min(1, b.how.load / 0.1))) / (0.2 * h);
      if (!far && (!best || score > best.score)) best = { score, at, on };
      if (far && (!near || -far > near.score)) near = { score: -far, at, on };
    }
    const r = best ?? near!;
    bigHolds.set(b, r);
    return r;
  };
  const holdOf = (b: Beat): Vec3 => {
    if (b.how.grip === "hug" || b.how.grip === "cradle" || b.how.grip === "edges") return bigHold(b).at;
    const [ex, ey, ez] = halves(b), heavy = Math.min(1, b.how.load / 0.2);
    const on = handsOn(b);
    let at: Vec3;
    if (b.hands.length === 1) {
      const sd = b.hands[0] === "L" ? 1 : -1;
      // (a wine glass carried before the body, upright, the upper arm hanging and the forearm out: hanging at the
      // side it spills)
      at = sub(b.how.grip === "stem" || plan.items[b.item].upright ? [sd * 0.12 * h, shoulderY - 0.17 * h, 0.14 * h] : [sd * 0.12 * h, shoulderY - 0.78 * armLength, 0.1 * h], on[b.hands[0]]!.at);
    } else if (b.how.grip === "pole") at = [0, hipsY + 0.04 * h, 0.15 * h];
    else at = [0, chestY - 0.16 * h - ey - 0.05 * h * heavy, Math.max(0.2 * h - 0.04 * h * heavy, FRONT * h + ez + 0.02)];
    // raised and drawn in until the hands are in easy reach of the shoulders (a big thing is held high against the
    // body), no higher than the chest and no closer than its back against the belly
    const arm = armLength;
    for (let k = 0; k < 60; k++) {
      const far = Math.max(...b.hands.map((s) => len(sub(add(at, on[s]!.at), shoulder[s]))));
      if (far <= 0.88 * arm) break;
      if (at[1] < chestY) at[1] += 0.01;
      at[2] = Math.max(b.hands.length === 1 ? 0.04 * h : FRONT * h + ez, at[2] - 0.006);
    }
    return at;
  };
  // A load balanced as people carry one (statics, no learned model): the body and the load together keep their
  // centre of mass where the body's own was, over the feet, so the load's weight on its lever (its mass to the body's
  // times how far it sits out from the body's centre) is offset by the body's own moved back the other way. A load
  // before the body: the trunk extends back from the lower back and the hips go back over the feet, the load going
  // with the chest. One in one hand at the side: the trunk bends away from it (the load hanging from the shoulder,
  // which comes in over the feet with it) and the free arm lifts out. Each takes its share by the same amount `t`
  // (the lean back up to 0.2 rad, the hips up to a twentieth of the height, the bend up to 0.25), solved on the body
  // at rest, so a ball or a mug moves nothing. `moved`: the load's middle moved with the trunk (the body's frame).
  type Statics = { pitch: number; back: number; roll: number; abduct: number; moved: Vec3 };
  const restPose = rest(skel), restW = pose(skel, restPose), spineJ = joint(skel, B.spine[1]);
  const comOf = (w: ReturnType<typeof pose>): Vec3 => {
    let c: Vec3 = [0, 0, 0], sum = 0;
    for (const [a, b2, k] of SEGMENTS) { const A = w.pos[joint(skel, a)], B = w.pos[joint(skel, b2)]; c = add(c, scale3(add(A, B), 0.5 * k)); sum += k; }
    return scale3(c, 1 / sum);
  };
  // (the hips moved back over planted feet carry the legs back less: the thighs half as far, the shins a tenth)
  const LEGS_BACK = 1 - 2 * (0.1 * 0.5 + 0.0465 * 0.9 + 0.0145);
  const c0 = comOf(restW), staticsMemo = new Map<Beat, Statics>();
  const staticsOf = (b: Beat): Statics => {
    if (staticsMemo.has(b)) return staticsMemo.get(b)!;
    const r = b.how.load, L0 = holdOf(b), one = b.hands.length === 1 && b.how.grip !== "stem", sd = b.hands[0] === "L" ? 1 : -1;
    const shJ = joint(skel, B.upperarm(b.hands[0]));
    const at = (t: number) => {
      const pitch = -0.2 * t * (one ? 0 : 1), back = 0.05 * h * t * (one ? 0 : 1), roll = one ? sd * 0.25 * t : 0;
      const w = pose(skel, trunkTurn(skel, restPose, 0, 0, pitch, roll));
      // (before the body: rigid with the middle of the back; at the side: hanging under the shoulder)
      const moved: Vec3 = one ? [w.pos[shJ][0] - restW.pos[shJ][0], 0, w.pos[shJ][2] - restW.pos[shJ][2] - back]
        : sub(add(w.pos[spineJ], qrot(qmul(w.rot[spineJ], qinv(restW.rot[spineJ])), sub(L0, restW.pos[spineJ]))), add(L0, [0, 0, back]));
      const c = add(comOf(w), [0, 0, -LEGS_BACK * back]), L = add(L0, moved);
      // how far the two together sit from where the body's own centre was, ahead (`z`) or toward the load's side
      const err = one ? sd * ((c[0] + r * L[0]) / (1 + r) - c0[0]) : (c[2] + r * L[2]) / (1 + r) - c0[2];
      return { st: { pitch, back, roll, abduct: one ? Math.min(0.35, 1.5 * Math.abs(roll)) : 0, moved }, err };
    };
    let st = at(0).st;
    if (r > 0 && b.how.grip !== "stem" && at(0).err > 0) {
      let lo = 0, hi = 1;
      if (at(1).err > 0) lo = 1;
      else for (let k = 0; k < 16; k++) { const m = (lo + hi) / 2; if (at(m).err > 0) lo = m; else hi = m; }
      st = at(lo).st;
    }
    staticsMemo.set(b, st);
    return st;
  };
  // a thing held up before the eyes is looked past: the head turned as little as lets the eyes see the floor two
  // body-heights ahead round its side (radians, + to the left; 0 when the way ahead is clear over it)
  const pastOf = new Map<Beat, number>();
  const lookPast = (b: Beat): number => {
    if (pastOf.has(b)) return pastOf.get(b)!;
    const c = holdOf(b), [ex, ey, ez] = halves(b), e = skel.landmarks.eyes, D = 2 * h;
    const blocked = (th: number) => {
      const d: Vec3 = [D * Math.sin(th) - e[0], -e[1], D * Math.cos(th) - e[2]];
      let t0 = 0, t1 = 1;
      // (the line of sight through the thing's box, a centimetre bigger all round)
      for (let k = 0; k < 3; k++) {
        const lo = c[k] - [ex, ey, ez][k] - 0.01 - e[k], hi = c[k] + [ex, ey, ez][k] + 0.01 - e[k];
        if (Math.abs(d[k]) < 1e-9) { if (lo > 0 || hi < 0) return false; continue; }
        const a = lo / d[k], z = hi / d[k];
        t0 = Math.max(t0, Math.min(a, z)); t1 = Math.min(t1, Math.max(a, z));
      }
      return t0 <= t1;
    };
    let th = 0;
    for (let k = 0; k <= 24 && blocked(th); k++) th = k % 2 ? (k + 1) * 0.025 : -k * 0.025;
    pastOf.set(b, th);
    return th;
  };
  // how much of a carrying arm is left to the walk: a thing light and small in one hand swings with the stride (held
  // steady at the hold, a ball carried on a run looked like a steadicam's), all of it at a tenth of the light limit and
  // a hand's size, none at the limit or twice the hand; it rides the palm as the hand has it
  const freeOf = (b: Beat) => {
    // (a wine glass carried steady before the waist, never swung)
    if (b.hands.length !== 1 || b.how.grip === "stem") return 0;
    // (one carried upright swings a third as freely: steady, but not a steadicam's)
    const up = plan.items[b.item].upright ? 0.35 : 1;
    const small = 1 - Math.min(1, Math.max(0, (Math.max(...plan.items[b.item].size) - 0.1 * h) / (0.1 * h)));
    return up * 0.85 * small * (1 - Math.min(1, Math.max(0, (b.how.load - 0.003) / 0.027)));
  };
  const holdW = (b: Beat) => 1 - freeOf(b);
  const grip = (b: Beat) => ({ pinch: 0.55, tripod: 0.6, top: 1, wrap: 1, handle: 1, stem: 1.1, palm: 0.35, sides: 0.45, edges: 0.9, cradle: 0.3, hug: 0.5, pole: 1 })[b.how.grip];

  // the furniture's surfaces things land on, as scene heights over rectangles
  // (one well below the thing's top is inside it: a bin's floor, a drawer's)
  const landing = (p: Vec3, below: number): { y: number; inside: boolean } => {
    let best = { y: 0, inside: false };
    for (const t of plan.things) for (const s of surfacesOf(t)) {
      const l = toLocal(t, p), y = t.at[1] + s.y;
      if (y <= below + 1e-3 && y > best.y && l[0] >= s.min[0] && l[0] <= s.max[0] && l[2] >= s.min[1] && l[2] <= s.max[1]) best = { y, inside: s.y < t.size[1] - 0.1 };
    }
    return best;
  };
  const supportAt = (p: Vec3, below: number) => landing(p, below).y;
  return { beatAt, chestY, dt, freeOf, grip, ground, guided, h, halves, handsOn, hipsY, holdOf, holdW, landing, lookPast, shoulderY, staticsOf, supportAt, toScene };
}
