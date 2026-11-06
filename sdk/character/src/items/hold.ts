// hold: the things fitted to the hands that have them, the last pass of interact. Each frame, a thing in hand is
// moved with the hand (rigidly on a guide, by the palms' miss otherwise), sat in the palm (a ball) or pushed out of the
// body and the hands, its grip locked once the fingers have closed on it, and the free hand moved out of its way;
// the dodges eased over the frames after. It returns how far the hands missed their goals, frame by frame.
import type { Vec3 } from "raea/3d";
import { B, add, ballInHand, cross, dot, handAlong, qbasis, qinv, handSurface, joint, len, norm, palmAt, palmFacing, pose, qaxis, qmul, qnorm, qrot, reach, scale3, setWorld, slerp, sub, type HandGoal, type HandShape, type Pose, type Side, type Skeleton } from "raea/character";
import { handling, shapeOf, type Handling } from "../things";
import { type Attach, type Goals, GRASP, ItemPlan, PINCHED, Track, depthIn, easeDodges, easeOff, fieldNormal, graspThing, leadOf, lockOn, mix, outOfBody, outOfHands, pinchLock, reachLocked, riddenSliding, settled, shapedHand, thingSdf, type Lock } from "./all";

/** a frame of interact's work: the walk's pose, the goals laid over it, the hips' drop and stoop it needs, the things
 * riding the hands, whether a beat is on, and a thing sat in a palm */
export interface Work { p: Pose; goals: Goals; drop: number; stoop: number; attach: Attach[]; beat: boolean; palmed?: { id: string; hand: Side; w: number } }

export function holdThings(skel: Skeleton, plan: ItemPlan, h: number, poses: Pose[], work: Work[], tracks: Map<string, Track[]>): number[] {
  const misses: number[] = [];
  // how far each hand not on a thing moves out of one in hand, frame by frame (dodged after, smoothed)
  const dodge: Record<Side, (Vec3 | undefined)[]> = { L: [], R: [] };
  // (and the thing a dodging hand carries, which goes with it)
  const carried: (string | undefined)[] = [];
  // (each frame's grasps of a thing in hand, to close again on it where the dodge moves the hand)
  const grasped: { id: string; hands: Side[]; grip: Handling["grip"]; close: number }[][] = [];
  // (and each hand's fingers' shape on it, frame by frame: a hand's fingers move on from the last frame's)
  const shapes: Record<Side, (HandShape | undefined)[]> = { L: [], R: [] };
  // (each thing's grip once the hands have closed on it, fixed while it is held: lock.ts)
  const locks = new Map<string, Lock>(), ready = new Map<string, number>();
  // (whether a thing's grip at a frame is one a guide placed the hand for)
  const guidedGrip = (id: string, f: number) => work[f].goals.grasps.some((g) => g.id === id && g.guide);
  for (let f = 0; f < work.length; f++) {
    const p = poses[f], { goals } = work[f];
    let miss = 0;
    if (goals.weight >= 1) {
      const w = pose(skel, p);
      for (const [s, g] of Object.entries(goals.hands) as [Side, HandGoal][]) {
        const at = g.point ? sub(g.at, scale3(norm(g.facing ?? [0, -1, 0]), 0.009 * h)) : g.at;
        miss = Math.max(miss, len(sub(palmAt(skel, w, s), at)));
      }
    }
    misses.push(miss);
    // (where each thing was sent, before it is fitted to the hands: a locked one goes from there)
    const sent = new Map([...tracks].map(([id, t]) => [id, t[f]]));
    if (work[f].attach.length) {
      const w = pose(skel, p);
      for (const a of work[f].attach) {
        if (locks.has(a.id)) continue;
        // (a flat hand's palm rests half its thickness off the goal, as reach puts it)
        const goalOf = (g: HandGoal) => (g.point ? sub(g.at, scale3(norm(g.facing ?? [0, -1, 0]), 0.009 * h)) : g.at);
        const tr = tracks.get(a.id)!, g1 = goals.hands[a.hands[0]]!;
        if (a.hands.length === 1 && g1.point && guidedGrip(a.id, f)) {
          // a thing on a guide in one hand rides it rigidly, turned with the hand as well as moved with it: the hand's
          // frame where it is against where it was sent (a can carried by a hand swinging with the walk tips with it,
          // and stays on the palm, not just as far from it)
          const s = a.hands[0], n = norm(g1.facing ?? [0, -1, 0]), fp = norm(sub(g1.point, scale3(n, dot(g1.point, n))));
          const na = palmFacing(skel, w, s), al = handAlong(skel, w, s), fa = norm(sub(al, scale3(na, dot(al, na))));
          const turn = slerp([0, 0, 0, 1], qmul(qbasis(fa, na, cross(fa, na)), qinv(qbasis(fp, n, cross(fp, n)))), a.w);
          const from = goalOf(g1), to = add(from, scale3(sub(palmAt(skel, w, s), from), a.w));
          tr[f] = { ...tr[f], at: add(to, qrot(turn, sub(tr[f].at, from))), rot: qnorm(qmul(turn, tr[f].rot)) };
        } else {
          const off = a.hands.reduce<Vec3>((acc, s) => add(acc, scale3(sub(palmAt(skel, w, s), goalOf(goals.hands[s]!)), 1 / a.hands.length)), [0, 0, 0]);
          tr[f] = { ...tr[f], at: add(tr[f].at, scale3(off, a.w)) };
        }
      }
    }
    // (a ball sat in the palm as it swings; anything else stays in its grip, a cup wrapped)
    const pm = work[f].palmed && shapeOf(plan.items[work[f].palmed!.id]) === "ball" ? work[f].palmed : undefined;
    if (pm && !locks.has(pm.id)) {
      const it = plan.items[pm.id], tr = tracks.get(pm.id)!, r = Math.max(...it.size) / 2;
      // (sat in the palm as far as it is in it: graspThing closes the fingers on it)
      const w = pose(skel, poses[f]), mid = ballInHand(skel, w, pm.hand, r);
      tr[f] = { ...tr[f], at: mix(tr[f].at, sub(mid, qrot(tr[f].rot, [0, it.size[1] / 2, 0])), pm.w) };
    }
    // each hand on a thing: the thing out of the hand's skin (a thing in hand moves; one still where it lies is
    // reached to its surface), then each finger closed until it touches it, as far as the grip closes them
    const grasps = goals.grasps.length ? goals.grasps : pm ? [{ id: pm.id, hands: [pm.hand], grip: "palm" as const, w: pm.w, close: 1 }] : [];
    for (const gr of grasps) {
      const tr = tracks.get(gr.id)!, planned = sent.get(gr.id)!.at, held = tr[f].held || work[f].attach.some((a) => a.id === gr.id) || pm?.id === gr.id;
      const sdf = thingSdf(plan.items[gr.id]);
      const field = (q: Vec3) => sdf(qrot(qinv(tr[f].rot), sub(q, tr[f].at)));
      if (!held) { locks.delete(gr.id); ready.delete(gr.id); }
      const lk = held ? locks.get(gr.id) : undefined, ride = work[f].attach.find((a) => a.id === gr.id)?.w ?? (pm?.id === gr.id ? 1 : 0);
      if (lk) {
        // locked: a light thing in one hand where the hand has it; anything else where it goes, the hands on it
        if (lk.lead === "hand") {
          // (a thing carried upright kept so by the wrist, RAEA_CARRY_UPRIGHT=0 not: a cup riding a hand that swings with
          // the walk tipped with it, 60 to 96 degrees off upright all the way to the desk; the hand turned about the wrist by the
          // shortest turn that stands it up, no more than a wrist bends (RAEA_UPRIGHT_MAX, 70 degrees), as far as it rides)
          if (process.env.RAEA_CARRY_UPRIGHT !== "0" && plan.items[gr.id].upright && ride > 0) {
            const s = gr.hands[0], w0 = pose(skel, poses[f]), hi = joint(skel, B.hand(s));
            const up = qrot(riddenSliding(skel, w0, s, { ...lk, slide: undefined }).rot, [0, 1, 0]);
            const ax = cross(up, [0, 1, 0]), sn = len(ax), a = Math.atan2(sn, up[1]);
            const most = (Number(process.env.RAEA_UPRIGHT_MAX ?? 70) * Math.PI) / 180;
            if (sn > 1e-6 && a > 1e-3) poses[f] = setWorld(skel, poses[f], w0, hi, qnorm(qmul(qaxis(scale3(ax, 1 / sn), Math.min(a, most) * ride), w0.rot[hi])));
          }
          // (let go of by the hand as it is put down: where it goes, the hand brought to its locked place on it)
          const r = riddenSliding(skel, pose(skel, poses[f]), gr.hands[0], lk);
          lk.off = easeOff(lk.off, [0, 0, 0]);
          tr[f] = { ...tr[f], at: mix(add(tr[f].at, lk.off), r.at, ride), rot: qnorm(slerp(tr[f].rot, r.rot, ride)) };
          if (ride < 0.999) poses[f] = reachLocked(skel, poses[f], lk, tr[f].at, tr[f].rot, { L: goals.hands.L?.elbow, R: goals.hands.R?.elbow }).p;
          const v = outOfBody(skel, poses[f], field, false);
          if (len(v) > 0.002) { dodge[gr.hands[0]][f] = add(v, scale3(norm(v), 0.006)); carried[f] = gr.id; }
        } else {
          lk.off = easeOff(lk.off, gr.hands.length === 2 ? outOfBody(skel, poses[f], field) : [0, 0, 0]);
          const at0 = add(tr[f].at, lk.off), r = reachLocked(skel, poses[f], lk, at0, tr[f].rot, { L: goals.hands.L?.elbow, R: goals.hands.R?.elbow });
          poses[f] = r.p;
          tr[f] = { ...tr[f], at: add(at0, scale3(r.miss, ride)) };
        }
        // (the fingers go on closing on it from where they were: the hand fixed on it, they meet it the same way
        // each frame, and settle)
        const fieldL = (q: Vec3) => sdf(qrot(qinv(tr[f].rot), sub(q, tr[f].at)));
        for (const s of gr.hands) {
          const hand = { shape: shapes[s][f - 1] };
          poses[f] = graspThing(skel, poses[f], s, fieldL, gr.grip, gr.close, 1, hand);
          shapes[s][f] = hand.shape;
        }
        (grasped[f] ??= []).push(gr);
      } else {
        if (held) {
          // (a thing against the body out of its trunk, head and thighs, the hands brought with it; a small thing in one
          // hand swings past the thigh as the arm does)
          const off = gr.hands.length === 2 ? outOfBody(skel, poses[f], field) : [0, 0, 0] as Vec3;
          // (one in one hand swung into the thigh or the waist: the hand and it dodged out, smoothed after)
          // (a ball swung with the stride isn't: dodging it frame by frame jerked the arm)
          if (gr.hands.length === 1 && !(pm && pm.id === gr.id)) {
            const v = outOfBody(skel, poses[f], field, false);
            if (len(v) > 0.002) { dodge[gr.hands[0]][f] = add(v, scale3(norm(v), 0.006)); carried[f] = gr.id; }
          }
          if (len(off) > 5e-4) {
            tr[f] = { ...tr[f], at: add(tr[f].at, off) };
            for (const s of gr.hands) { const g = goals.hands[s]; if (g) poses[f] = reach(skel, poses[f], s, { ...g, at: add(g.at, off) }); }
          }
          const f1 = (q: Vec3) => sdf(qrot(qinv(tr[f].rot), sub(q, tr[f].at)));
          if (gr.hands.length === 2) {
            // (a thing between two palms can't be pushed out of both: each hand drawn back out of it instead)
            // (a few times over: a reach lands within a centimetre, and the man's palms stayed that far into a shoe box)
            for (const s of gr.hands) {
              const g = goals.hands[s];
              let goal = g?.at;
              for (let k = 0; k < 3 && g && goal; k++) {
                let deep = 0, at: Vec3 | undefined;
                for (const x of handSurface(skel, pose(skel, poses[f]), s)) if (x.part === "palm" || x.part === "thumb1") { const d = -f1(x.at); if (d > deep) { deep = d; at = x.at; } }
                if (!at || deep <= 5e-4) break;
                goal = add(goal, scale3(fieldNormal(f1, at), deep + 0.001));
                poses[f] = reach(skel, poses[f], s, { ...g, at: goal });
              }
            }
          } else if (gr.guide && shapes[gr.hands[0]][f - 1]) {
            // (a hand on a guide: out of the hand as it last closed on it, its fingers round it; laid straight, the
            // knuckles' pads stand off the palm and pushed a can a centimetre and a half off it, out of the palm's reach)
            const hs = gr.hands[0], shaped = shapedHand(skel, poses[f], hs, gr.grip, shapes[hs][f - 1]!);
            tr[f] = { ...tr[f], at: add(tr[f].at, outOfHands(skel, shaped, gr.hands, f1, false)) };
          } else tr[f] = { ...tr[f], at: add(tr[f].at, outOfHands(skel, poses[f], gr.hands, f1, GRASP[gr.grip].on.length === 5)) };
      }
      const field2 = (q: Vec3) => sdf(qrot(qinv(tr[f].rot), sub(q, tr[f].at)));
      // (a thing in hand gripped fully, however freely the arm swings with it)
      for (const s of gr.hands) {
        const hand = { shape: shapes[s][f - 1] };
        poses[f] = graspThing(skel, poses[f], s, field2, gr.grip, gr.close, held ? 1 : gr.w, hand);
        shapes[s][f] = hand.shape;
      }
      if (held) (grasped[f] ??= []).push(gr);
      // (the hand's place on it locked as the thing is first held: the fingers closed and still (or six frames on), the
      // hand clear of it (not one taken from above to be looked at: turned palm up under it, locked, its fingers let a
      // ball go)
      const pinched = PINCHED.has(gr.grip) && gr.hands.length === 1;
      if (held && !locks.has(gr.id) && pinched) locks.set(gr.id, pinchLock(skel, pose(skel, poses[f]), tr[f].at, tr[f].rot, plan.items[gr.id].size, gr.hands[0]));
      else if (held && !pinched && gr.close >= 0.999 && (ready.set(gr.id, (ready.get(gr.id) ?? 0) + 1), depthIn(skel, poses[f], gr.hands, field2) <= 0.002)
        && gr.grip !== "top" && (gr.hands.every((s) => settled(shapes[s][f], shapes[s][f - 1])) || ready.get(gr.id)! >= 6)) {
        // (with the thing's push off where it was sent as it locked, let go over the frames after: dropped at once, a
        // shoe box jumped 16 cm in the hands)
        const l = lockOn(skel, pose(skel, poses[f]), tr[f].at, tr[f].rot, gr.hands, { L: shapes.L[f], R: shapes.R[f] }, leadOf(gr.hands, handling(plan.items[gr.id], h)));
        l.off = sub(tr[f].at, planned);
        locks.set(gr.id, l);
      }
      }
      const fieldNow = (q: Vec3) => sdf(qrot(qinv(tr[f].rot), sub(q, tr[f].at)));
      // the other hand out of its way (hanging where a wine glass is brought down to the table)
      for (const s of (["L", "R"] as Side[]).filter((x) => !gr.hands.includes(x))) {
        const w = pose(skel, poses[f]);
        let deep = 0, at: Vec3 | undefined;
        for (const x of handSurface(skel, w, s)) { const d = -fieldNow(x.at); if (d > deep) { deep = d; at = x.at; } }
        if (at && deep > 0.002) dodge[s][f] = scale3(fieldNormal(fieldNow, at), deep + 0.008);
      }
    }
  }
  easeDodges(skel, plan, poses, dodge, carried, tracks, grasped, shapes);
  return misses;
}
