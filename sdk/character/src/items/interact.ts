// interact: the hands, the lean, the squat, the trunk, the gaze and the grip laid over a performance.
import type { Vec3 } from "raea/3d";
import { add, ballInHand, cross, dot, flatHand, handAlong, qbasis, blend as blendPose, qinv, handPose, handSurface, joint, len, norm, palmAt, palmFacing, pose, qaxis, qmul, qnorm, qrot, qyaw, reach, scale3, setWorld, slerp, solve, sub, type HandGoal, type HandShape, type Pose, type Quat, type Side, type Skeleton } from "raea/character";
import { type Performance } from "../matching";
import { handling, shapeOf, toLocal, type Handling, type Thing } from "../things";
import { type Attach, type Goals, type Grasp, HAND, halfOf, Beat, D, E, Flight, G, GRASP, Interaction, ItemPlan, P, QUICK_TWIST, S, T, THROW_HAND, THROW_LEN, Track, W, beatGeometry, clearWay, crouchDrop, crouched, easeDodges, eased, fieldNormal, fingers, flightTime, graspThing, launch, leanFor, mix, outOfBody, outOfHands, smooth, smoothCarries, span, stoop, thingSdf, throwAt, throwKeys, trunkTurn, uprightNear } from "./all";

export function interact(skel: Skeleton, perf: Performance, plan: ItemPlan): Interaction {
  const { beatAt, chestY, dt, freeOf, grip, ground, guided, h, halves, handsOn, hipsY, holdOf, holdW, landing, lookPast, shoulderY, staticsOf, supportAt, toScene } = beatGeometry(skel, perf, plan);

  const tracks = new Map<string, Track[]>();
  for (const id of Object.keys(plan.items)) tracks.set(id, []);
  const placed = new Map(Object.entries(plan.items).map(([id, it]) => [id, { at: it.at, rot: qyaw(it.yaw ?? 0) }]));
  const flying = new Map<string, Flight>();
  const holding = new Map<string, Beat>();
  // where each hand was as a beat began (from the walk's own pose), for a reach to start from where the hand is
  const startPalm = new Map<number, Record<Side, Vec3>>();
  // the frame each thing looked past was first carried from
  const heldFrom = new Map<Beat, number>();
  const work: { p: Pose; goals: Goals; drop: number; stoop: number; attach: Attach[]; beat: boolean; palmed?: { id: string; hand: Side; w: number } }[] = [];

  for (let f = 0; f < perf.frames; f++) {
    const p = perf.pose(f), st = perf.step(f), b = st ? beatAt.get(st.index) : undefined, g = ground(f);
    const goals: Goals = { hands: {}, weight: 0, lookWeight: 0, fingers: {}, twist: 0, pitch: 0, roll: 0, back: 0, abduct: {}, dip: 0, lean: 0.6, squat: true, bend: 0.5, grasps: [] };
    const now = new Map<string, Track>(), attach: Attach[] = [];
    // a thing sitting in a palm as the hand has it (a light thing carried, a capture's throw), `w` of the way from where
    // it rode at the hold
    let palmed: { id: string; hand: Side; w: number } | undefined;

    // the hands on a thing whose middle is at `mid`, the grip's frame turned `rot`, `w` of the way
    const handsTo = (hb: Beat, mid: Vec3, rot: Quat, w: number, fingers = 1, elbow: Vec3 = [0.8, -0.45, -0.4]) => {
      for (const [s, o] of Object.entries(handsOn(hb)) as [Side, { at: Vec3; facing: Vec3; point: Vec3 }][]) {
        // the elbows out to the side and a little back (down and out, the default, lines up with an arm hanging to the
        // floor, and the elbow flips about it)
        goals.hands[s] = { at: add(mid, qrot(rot, o.at)), facing: qrot(rot, o.facing), point: qrot(rot, o.point), elbow: qrot(qyaw(g.yaw), [s === "L" ? elbow[0] : -elbow[0], elbow[1], elbow[2]]) };
        goals.fingers[s] = grip(hb) * fingers;
      }
      goals.grasps = goals.grasps.filter((x) => x.id !== hb.item);
      goals.grasps.push({ id: hb.item, hands: hb.hands, grip: hb.how.grip, w, close: fingers, guide: hb.hands.length === 1 && !!guided(hb, hb.hands[0]) });
      goals.weight = Math.max(goals.weight, w);
    };
    // a thing in the hold: its middle and the grip's frame (the body's heading); its own turn adds `turn`
    // (raised with the hips as far as they rise over standing: a thing carried over a hurdle goes over it with the body)
    // (eased onto standing height as the hips come down, not stopped at it: stopped, a hurdle's landing jolted the
    // carrying hand three times as hard as any capture's)
    // (over the frames either side, a third of a second)
    let rise = 0;
    for (let k = -4; k <= 4; k++) rise += Math.max(0, perf.pose(Math.max(0, Math.min(perf.frames - 1, f + k))).root[1] - hipsY) * (5 - Math.abs(k)) / 25;
    // (where the body balancing it has it: moved back with the trunk)
    const hold = (hb: Beat) => ({ mid: toScene({ at: [g.at[0], rise, g.at[2]], yaw: g.yaw }, add(holdOf(hb), staticsOf(hb).moved)), rot: qyaw(g.yaw) });
    const track = (hb: Beat, mid: Vec3, rot: Quat, held: boolean, ride = held ? 1 : 0) => {
      const it = plan.items[hb.item];
      now.set(hb.item, { at: sub(mid, qrot(qmul(rot, qyaw(hb.turn)), [0, it.size[1] / 2, 0])), rot: qmul(rot, qyaw(hb.turn)), held });
      for (let k = attach.length - 1; k >= 0; k--) if (attach[k].id === hb.item) attach.splice(k, 1);
      if (ride > 0) attach.push({ id: hb.item, hands: hb.hands, w: ride });
    };
    // the body balances the load (`u` of the way): the back leans back from one before it, the hips go back, and the
    // trunk bends away from one at the side, the free arm out
    const carrying = (hb: Beat, u = 1) => {
      const st = staticsOf(hb);
      goals.pitch = Math.min(goals.pitch, st.pitch * u);
      goals.back = Math.max(goals.back, st.back * u);
      if (Math.abs(st.roll * u) > Math.abs(goals.roll)) goals.roll = st.roll * u;
      if (st.abduct > 0) { const free: Side = hb.hands[0] === "L" ? "R" : "L"; goals.abduct[free] = Math.max(goals.abduct[free] ?? 0, st.abduct * u); }
      // a hold is in easy reach, so the back stays as the walk has it, and the knees (a big ball carried crouched over
      // put the head into it)
      goals.lean = 0;
      if (u >= 1) goals.squat = false;
      // a thing carried keeps the trunk upright: the walk's own stoop (a turn's, a stop's) stood back up, or the head
      // goes into a big thing and a wine glass is tipped at the face (a ball swung in one hand goes as the walk has it)
      if (hb.how.grip === "hug" || hb.how.grip === "cradle" || hb.how.grip === "edges" || hb.how.grip === "stem") {
        const up = qrot(pose(skel, p).rot[joint(skel, "Spine1")], [0, 1, 0]), fw = qrot(qyaw(g.yaw), [0, 0, 1]);
        const stoop = Math.atan2(up[0] * fw[0] + up[2] * fw[2], up[1]);
        goals.pitch = Math.min(goals.pitch, (st.pitch - Math.max(0, stoop - 0.05)) * u);
      }
      // (looking past a big thing, turned to over a third of a second from when it is first held)
      const th = lookPast(hb);
      if (th !== 0) {
        if (!heldFrom.has(hb)) heldFrom.set(hb, f);
        const k = smooth((f - heldFrom.get(hb)!) / 10) * u;
        goals.look = toScene(g, [2 * h * Math.sin(th), 0, 2 * h * Math.cos(th)]);
        goals.lookWeight = Math.max(goals.lookWeight, 0.9 * k);
      }
    };
    for (const hb of holding.values()) {
      const t = hold(hb);
      handsTo(hb, t.mid, t.rot, holdW(hb));
      track(hb, t.mid, t.rot, true);
      carrying(hb);
      if (freeOf(hb) > 0) palmed = { id: hb.item, hand: hb.hands[0], w: freeOf(hb) };
    }
    if (b && st) {
      const s = st.since / b.slow, it = plan.items[b.item];
      if (!startPalm.has(b.step)) { const w = pose(skel, p); startPalm.set(b.step, { L: palmAt(skel, w, "L"), R: palmAt(skel, w, "R") }); }
      // the hands where the walk has them now (where they go back to)
      const natural = () => { const w = pose(skel, p); return { L: palmAt(skel, w, "L"), R: palmAt(skel, w, "R") } as Record<Side, Vec3>; };
      if (b.kind === "examine") {
        // raised before the chest, turned over in the hand (about the up axis, and tipped toward the face and away),
        // looked at, and lowered back to the hold
        const len = E.look, u = st.since / len, t0 = hold(b), sd = b.hands.length === 1 ? (b.hands[0] === "L" ? 1 : -1) : 0;
        const r = smooth(u / 0.25) * (1 - smooth((u - 0.78) / 0.22));
        const up = toScene({ at: [g.at[0], rise, g.at[2]], yaw: g.yaw }, [sd * 0.05 * h, shoulderY - 0.12 * h, 0.22 * h]);
        const at: Vec3 = [t0.mid[0] + (up[0] - t0.mid[0]) * r, t0.mid[1] + (up[1] - t0.mid[1]) * r, t0.mid[2] + (up[2] - t0.mid[2]) * r];
        const v = smooth((u - 0.2) / 0.6);
        let rot = qmul(t0.rot, qmul(qyaw(-sd * 1.2 * Math.sin(Math.PI * v)), qaxis([1, 0, 0], 0.5 * Math.sin(2 * Math.PI * v))));
        // one taken from above: the forearm turns as it comes up (the thumb out), tipped toward the eyes, and back over
        // it as it goes down: a ball the palm up under it, anything else a quarter turn, on its side with its face to
        // the eyes (a can turned palm up stood on its head) (Shane, 10-22: "to look at it youd rotate forearm to see it
        // as you lift up")
        if (b.how.grip === "top") {
          const fw = qrot(qyaw(g.yaw), [0, 0, 1]), side = qrot(qyaw(g.yaw), [1, 0, 0]), turn = shapeOf(it) === "ball" ? 0.95 * Math.PI : 0.5 * Math.PI;
          rot = qmul(qaxis(side, -0.35 * r), qmul(qaxis(fw, -sd * turn * r), qmul(t0.rot, qyaw(-sd * 0.5 * Math.sin(Math.PI * v)))));
        }
        handsTo(b, at, rot, holdW(b) + (1 - holdW(b)) * r);
        track(b, at, rot, true);
        carrying(b);
        goals.look = at;
        goals.lookWeight = 0.9 * r;
      } else if (b.kind === "pick") {
        const was = placed.get(b.item)!, rot0 = qmul(was.rot, qyaw(-b.turn));
        const mid0 = add(was.at, qrot(was.rot, [0, it.size[1] / 2, 0]));
        // a quick pick turns the trunk toward the thing as the hand goes out (the rest of the way round from where the
        // feet face), and back as it comes in to the hold
        const toward = (() => { let a = Math.atan2(mid0[0] - g.at[0], mid0[2] - g.at[2]) - g.yaw; a = Math.atan2(Math.sin(a), Math.cos(a)); return Math.max(-0.7, Math.min(0.7, QUICK_TWIST * a)); })();
        goals.lean = leanFor(b.bend); goals.bend = b.bend;
        goals.look = mid0;
        goals.lookWeight = smooth(s / T.look) * (1 - (b.strain ? span(s, S.off) : span(s, T.lift)));
        if (b.clip) {
          // a capture's pick: it bends and looks; the arm alone is brought onto the thing over the half second before
          // its hand comes lowest, and after brought in to the hold by the capture's end, let go back to the capture's
          // own lift on the way (the thing riding the hand as it is)
          const { key, len: n } = b.clip, j = Math.round(st.since * plan.fps), pre = Math.round(0.45 * plan.fps);
          goals.squat = false; goals.lean = 0; goals.lookWeight = 0;
          // (turned toward it over the half second before the hand comes lowest, back by the capture's end)
          goals.twist = toward * smooth((j - key + pre) / pre) * (1 - smooth((j - key) / Math.max(1, n - 1 - key)));
          if (j < key) {
            handsTo(b, mid0, rot0, smooth((j - key + pre) / pre), smooth((j - key + 6) / 6));
            track(b, mid0, rot0, false);
          } else {
            const u = smooth((j - key) / Math.max(1, n - 1 - key)), to = hold(b), rot = slerp(rot0, to.rot, u);
            const at = clearWay(mid0, to.mid, plan.things, halfOf(it), 0.03 * h)(u);
            handsTo(b, at, rot, (1 - 0.6 * Math.sin(Math.PI * u)) * (1 - u) + holdW(b) * u);
            if (freeOf(b) > 0) palmed = { id: b.item, hand: b.hands[0], w: freeOf(b) * u };
            track(b, at, rot, true);
            if (j >= n - 1) holding.set(b.item, b);
          }
        } else if (b.strain) {
          // too heavy: hands on, pulled at twice (the back leaning back, the thing rocking a centimetre), let go
          const pull = s < S.pull[0] ? 0 : Math.sin(Math.PI * 2 * Math.min(1, (s - S.pull[0]) / (S.pull[1] - S.pull[0]))) ** 2;
          const rock: Vec3 = [0, 0.01 * pull, 0];
          handsTo(b, add(mid0, rock), rot0, span(s, T.reach) * (1 - span(s, S.off)));
          goals.pitch = -0.2 * pull;
          goals.dip = 0.03 * h * pull;
          track(b, add(mid0, rock), rot0, false);
        } else if (s < T.grip) {
          goals.twist = toward * smooth(s / 0.4);
          // reaching: each hand from where it is to its place on the thing, round the furniture (over the edge of
          // a shelf, not through it), taken up at once (its goal starts where the hand is)
          handsTo(b, mid0, rot0, smooth(s / 0.2), smooth((s - T.grip + 0.2) / 0.2));
          for (const hs of b.hands) { const gh = goals.hands[hs]!; gh.at = clearWay(startPalm.get(b.step)![hs], gh.at, plan.things, HAND)(span(s, T.reach)); }
          track(b, mid0, rot0, false);
        } else {
          // taken: lifted clear of what it stood on, then in to the hold (turned square to the body on the way)
          const u = span(s, T.lift), to = hold(b);
          goals.twist = toward * (1 - smooth((s - T.grip) / (T.pick - T.grip)));
          // (up off what it stood on before it comes in, and over any edge between)
          const lifted = clearWay(mid0, to.mid, plan.things, halfOf(it), 0.03 * h)(u);
          const rot = slerp(rot0, to.rot, u);
          // (eased onto the carry's own hold by the end)
          const free = freeOf(b) * span(s, [T.lift[0], T.pick]);
          handsTo(b, lifted, rot, 1 - free);
          // (riding the hand from as it starts up: eased in over the lift's whole way, a cup the arm let go back to the
          // walk's swing was left 11 cm behind the palm)
          track(b, lifted, rot, true, smooth(u / 0.15));
          if (free > 0) palmed = { id: b.item, hand: b.hands[0], w: free };
          carrying(b, u);
          goals.lean = leanFor(b.bend); goals.bend = b.bend;
          if (s >= T.lift[1]) holding.set(b.item, b);
        }
      } else if (b.kind === "place") {
        const to = b.to!, rot1 = qmul(qyaw(to.yaw), qyaw(-b.turn)), from = hold(b);
        const mid1 = add(to.at, [0, it.size[1] / 2, 0]);
        holding.delete(b.item);
        goals.look = mid1;
        goals.lookWeight = smooth(s / T.look) * (1 - span(s, P.back));
        goals.lean = leanFor(b.bend); goals.bend = b.bend;
        if (s < P.release) {
          const u = span(s, P.lower);
          // (over the edge of what it goes on and down onto it, never through its front)
          const mid = clearWay(from.mid, mid1, plan.things, halfOf(it), 0.03 * h)(u), rot = slerp(from.rot, rot1, u);
          const settle = span(s, [0, 0.3]);
          handsTo(b, mid, rot, holdW(b) + (1 - holdW(b)) * settle);
          track(b, mid, rot, true, 1 - u);
          if (freeOf(b) > 0) palmed = { id: b.item, hand: b.hands[0], w: freeOf(b) * (1 - settle) };
          carrying(b, 1 - u);
          goals.lean = leanFor(b.bend); goals.bend = b.bend;
        } else {
          // let go: the hands open, come up off it and back out over the edge to where the walk has them, and only
          // then are let go of (a hand let go of straight away swings down through the furniture)
          placed.set(b.item, { at: to.at, rot: qyaw(to.yaw) });
          handsTo(b, mid1, rot1, 1 - span(s, [P.back[0] + 0.7 * (P.back[1] - P.back[0]), P.back[1]]), 1 - smooth((s - P.release) / 0.15));
          const home = natural();
          for (const hs of b.hands) { const gh = goals.hands[hs]!; gh.at = clearWay(add(gh.at, [0, 0.01, 0]), home[hs], plan.things, HAND)(span(s, P.back)); }
          now.set(b.item, { at: to.at, rot: qyaw(to.yaw), held: false });
        }
      } else if (b.kind === "drop") {
        const t = hold(b);
        holding.delete(b.item);
        if (st.since < D.release) {
          handsTo(b, t.mid, t.rot, holdW(b)); track(b, t.mid, t.rot, true); carrying(b);
          if (freeOf(b) > 0) palmed = { id: b.item, hand: b.hands[0], w: freeOf(b) };
        }
        else {
          carrying(b, 1 - span(st.since, D.back));
          // the hands open where they held it and fall back
          handsTo(b, t.mid, t.rot, holdW(b) * (1 - span(st.since, D.back)), 1 - smooth((st.since - D.release) / 0.1));
          if (!flying.has(b.item)) {
            track(b, t.mid, t.rot, false);
            const v = now.get(b.item)!;
            flying.set(b.item, { at: v.at, vel: [0, 0, 0], rot: v.rot, spin: [0, 0, 0], landed: false, settle: 0, bounces: 0 });
          }
          goals.look = flying.get(b.item)!.at;
          goals.lookWeight = 0.7 * (1 - span(st.since, [0.6, D.drop]));
        }
      } else if (b.clip) {
        // a capture's throw: the arm let go from the hold to the capture's over its first third of a second, the thing
        // in the palm (its middle off it by its size) until the hand is fastest, then flying
        const th = b.throw!, { key } = b.clip, j = Math.round(st.since * plan.fps), hs = b.hands[0];
        holding.delete(b.item);
        goals.squat = false; goals.lean = 0;
        const give = 1 - smooth(j / Math.round(0.35 * plan.fps)), t = hold(b);
        if (j < key) {
          handsTo(b, t.mid, t.rot, give * holdW(b));
          track(b, t.mid, t.rot, true);
          palmed = { id: b.item, hand: hs, w: Math.max(freeOf(b), smooth(j / 6)) };
        } else {
          handsTo(b, t.mid, t.rot, 0, 1 - smooth((j - key) / 4));
          if (!flying.has(b.item)) {
            const w = pose(skel, p), r = Math.max(...halves(b)), it2 = plan.items[b.item];
            const mid = ballInHand(skel, w, hs, r);
            track(b, mid, qyaw(g.yaw), false);
            const v = now.get(b.item)!, vel = launch(v.at, th.target, th.style, b.how.load, h);
            const left = qrot(qyaw(g.yaw), [1, 0, 0]), flight = flightTime(v.at, vel, supportAt(th.target, th.target[1] + 0.01));
            const rate = shapeOf(it2) === "ball" ? 8 : flight > 0 ? (Math.round((flight * 5) / (2 * Math.PI)) * 2 * Math.PI) / flight : 0;
            flying.set(b.item, { at: v.at, vel, rot: v.rot, spin: scale3(left, rate), landed: false, settle: 0, bounces: 0 });
          }
        }
      } else {
        // a throw: the thing's middle along the style's keys in the body's frame, the trunk and knees with it
        const th = b.throw!, len = THROW_LEN[th.style], u = st.since / (len * b.slow);
        holding.delete(b.item);
        const keys = throwKeys(th.style, b, holdOf(b), { h, chestY, shoulderY, hipsY });
        const k = throwAt(keys, u);
        // (the trunk is the style's: no stoop toward the hands, which would fight it; the elbows out to the side)
        goals.twist = k.twist; goals.pitch = k.pitch; goals.dip = k.dip; goals.lean = 0; goals.squat = false;
        const out: Vec3 = [1, -0.35, -0.15];
        goals.look = th.target;
        goals.lookWeight = 0.8 * smooth(u / 0.15) * (1 - smooth((u - W.follow) / (1 - W.follow)));
        // the grip's frame turns with the trunk, so the hands stay square to the shoulders
        const rot = qyaw(g.yaw + k.twist * 0.8), mid = toScene(g, k.mid);
        // the hands turned for the style (an underhand palm up under the thing, an overhand palm forward behind it,
        // a push palms forward, a toss palms up under it), a one-hand throw's thing sitting in the palm
        const throwHands = () => {
          for (const hs of b.hands) {
            const gh = goals.hands[hs]!, sd = hs === "L" ? 1 : -1;
            const turn = THROW_HAND[th.style](sd), facing = qrot(rot, turn.facing), point = qrot(rot, turn.point);
            gh.facing = facing; gh.point = point;
            if (b.hands.length === 1) gh.at = sub(mid, scale3(norm(facing), Math.max(...halves(b)) + 0.008));
          }
        };
        if (u < W.release) {
          const settle = smooth(u / 0.15);
          handsTo(b, mid, rot, holdW(b) + (1 - holdW(b)) * settle, 1, out);
          if (freeOf(b) > 0) palmed = { id: b.item, hand: b.hands[0], w: freeOf(b) * (1 - settle) };
          throwHands();
          track(b, mid, rot, true);
        } else {
          handsTo(b, mid, rot, 1 - smooth((u - W.follow) / (1 - W.follow)), 1 - smooth((u - W.release) / 0.06), out);
          throwHands();
          if (!flying.has(b.item)) {
            track(b, mid, rot, false);
            const v = now.get(b.item)!, it2 = plan.items[b.item];
            const vel = launch(v.at, th.target, th.style, b.how.load, h);
            // a box turns over a whole number of times in the air, so it lands the way up it left; a ball spins freely
            const flight = flightTime(v.at, vel, supportAt(th.target, th.target[1] + 0.01));
            const left = qrot(qyaw(g.yaw), [1, 0, 0]), turns = shapeOf(it2) === "ball" ? 1.5 : Math.round((flight * 5) / (2 * Math.PI));
            const rate = shapeOf(it2) === "ball" ? 8 : flight > 0 ? (turns * 2 * Math.PI) / flight : 0;
            flying.set(b.item, { at: v.at, vel, rot: v.rot, spin: scale3(left, rate), landed: false, settle: 0, bounces: 0 });
          }
        }
      }
    }
    // the things let go: falling and flying until they land, then a light ball bounces and rolls to a stop
    for (const [id, fl] of flying) {
      if (holding.has(id) || now.get(id)?.held) { flying.delete(id); continue; }
      const it = plan.items[id], ball = shapeOf(it) === "ball", r = it.size[0] / 2;
      if (!fl.landed) {
        const next: Vec3 = [fl.at[0] + fl.vel[0] * dt, fl.at[1] + fl.vel[1] * dt - 0.5 * G * dt * dt, fl.at[2] + fl.vel[2] * dt];
        const { y: floor, inside } = landing(next, fl.at[1]);
        fl.vel = [fl.vel[0], fl.vel[1] - G * dt, fl.vel[2]];
        const w = Math.hypot(...fl.spin);
        if (w > 0) fl.rot = qmul(qaxis(scale3(fl.spin, 1 / w), w * dt), fl.rot);
        if (next[1] <= floor) {
          next[1] = floor;
          const light = handling(it, h).weight === "light";
          if (ball && light && !inside && fl.vel[1] < -1.2 && fl.bounces < 4) {
            fl.vel = [fl.vel[0] * 0.7, -fl.vel[1] * 0.45, fl.vel[2] * 0.7];
            fl.bounces++;
          } else {
            fl.landed = true;
            fl.vel = ball && !inside ? [fl.vel[0] * 0.7, 0, fl.vel[2] * 0.7] : [0, 0, 0];
          }
        }
        fl.at = next;
      } else {
        // a ball rolls (slowing), turning by the way it goes; anything else settles flat over a few frames
        const sp = Math.hypot(fl.vel[0], fl.vel[2]);
        if (ball && sp > 0.02) {
          const k = Math.max(0, sp - 1.5 * dt) / sp;
          fl.vel = [fl.vel[0] * k, 0, fl.vel[2] * k];
          const next: Vec3 = [fl.at[0] + fl.vel[0] * dt, fl.at[1], fl.at[2] + fl.vel[2] * dt];
          // it stops against anything standing in its way
          if (plan.things.some((t) => t.at[1] < 0.02 && toLocal(t, next)[1] < t.size[1] && Math.abs(toLocal(t, next)[0]) < t.size[0] / 2 + r && Math.abs(toLocal(t, next)[2]) < t.size[2] / 2 + r)) fl.vel = [0, 0, 0];
          else {
            const axis: Vec3 = [fl.vel[2] / sp, 0, -fl.vel[0] / sp];
            fl.rot = qmul(qaxis(axis, (sp * k * dt) / Math.max(0.01, r)), fl.rot);
            fl.at = [next[0], supportAt(next, fl.at[1] + 0.01), next[2]];
          }
        } else if (!ball && fl.settle < 1) {
          fl.settle = Math.min(1, fl.settle + 0.2);
          fl.rot = slerp(fl.rot, uprightNear(fl.rot), fl.settle);
        }
      }
      now.set(id, { at: fl.at, rot: fl.rot, held: false });
      placed.set(id, { at: fl.at, rot: fl.rot });
    }
    const hasHands = Object.keys(goals.hands).length > 0;
    // (a throw's knees are its style's: it doesn't squat to reach its wind-up)
    const need = hasHands && goals.weight > 0 && goals.squat ? crouchDrop(skel, p, goals.hands, goals.lean, goals.bend) : { drop: 0, stoop: 0 };
    work.push({ p, goals, drop: need.drop, stoop: need.stoop, attach, beat: !!b, palmed });
    for (const [id, tr] of tracks) tr.push(now.get(id) ?? { ...placed.get(id)!, held: false });
  }
  // the hips' drop and the stoop over the frames, eased (eased, below)
  // the stoop too, the same way (its least is as jumpy as the drop's, and tips the pelvis the legs hang from)
  const drops = eased(work.map((x) => x.drop)), stoops = eased(work.map((x) => x.stoop));
  const poses: Pose[] = [], misses: number[] = [];
  for (let f = 0; f < work.length; f++) {
    let { p } = work[f];
    const { goals } = work[f];
    const trunk = goals.twist !== 0 || goals.pitch !== 0 || goals.roll !== 0 || goals.dip !== 0;
    if (goals.weight > 0 || goals.lookWeight > 0 || trunk || Object.keys(goals.fingers).length) {
      const g = ground(f);
      // the trunk first (the shoulders the hands reach from move with it), then the squat, the lean and the hands at
      // their weight, then the gaze at its own
      const body = trunkTurn(skel, p, g.yaw, goals.twist, goals.pitch, goals.roll);
      p = goals.weight > 0 ? blendPose(body, crouched(skel, body, goals.hands, drops[f] + goals.dip, goals.lean, stoops[f], goals.back), goals.weight) : goals.dip > 0 ? crouched(skel, body, {}, goals.dip, 0) : body;
      // a hand on a guide keeps its turn on it however much of the arm is left to the walk (the wrist taking up the
      // swing): a can rides the hand rigidly, and a hand half let go back to the walk's tipped it 45° toward spilling
      for (const gr of goals.grasps) {
        const gh = gr.hands.length === 1 ? goals.hands[gr.hands[0]] : undefined;
        const ride = work[f].attach.find((a) => a.id === gr.id)?.w ?? 0;
        if (gh?.point && gr.guide && ride > 0 && goals.weight < 1) p = blendPose(p, flatHand(skel, p, gr.hands[0], gh.point, gh.facing), ride);
      }
      // the free arm lifted out from the side, away from a load in the other hand
      for (const [s, a] of Object.entries(goals.abduct) as [Side, number][]) {
        if (!a || goals.hands[s]) continue;
        const i = joint(skel, s === "L" ? "LeftArm" : "RightArm"), w = pose(skel, p);
        p = setWorld(skel, p, w, i, qmul(qaxis(qrot(qyaw(g.yaw), [0, 0, 1]), s === "L" ? a : -a), w.rot[i]));
      }
      if (goals.look && goals.lookWeight > 0) p = solve(skel, p, { look: goals.look, lookWeight: goals.lookWeight });
      p = fingers(skel, p, goals.fingers);
    }
    poses.push(p);
  }
  smoothCarries(skel, poses, work);
  // how far each hand not on a thing moves out of one in hand, frame by frame (dodged after, smoothed)
  const dodge: Record<Side, (Vec3 | undefined)[]> = { L: [], R: [] };
  // (and the thing a dodging hand carries, which goes with it)
  const carried: (string | undefined)[] = [];
  // (each frame's grasps of a thing in hand, to close again on it where the dodge moves the hand)
  const grasped: { id: string; hands: Side[]; grip: Handling["grip"]; close: number }[][] = [];
  // (and each hand's fingers' shape on it, frame by frame: a hand's fingers move on from the last frame's)
  const shapes: Record<Side, (HandShape | undefined)[]> = { L: [], R: [] };
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
    if (work[f].attach.length) {
      const w = pose(skel, p);
      for (const a of work[f].attach) {
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
    if (pm) {
      const it = plan.items[pm.id], tr = tracks.get(pm.id)!, r = Math.max(...it.size) / 2;
      // (sat in the palm as far as it is in it: graspThing closes the fingers on it)
      const w = pose(skel, poses[f]), mid = ballInHand(skel, w, pm.hand, r);
      tr[f] = { ...tr[f], at: mix(tr[f].at, sub(mid, qrot(tr[f].rot, [0, it.size[1] / 2, 0])), pm.w) };
    }
    // each hand on a thing: the thing out of the hand's skin (a thing in hand moves; one still where it lies is
    // reached to its surface), then each finger closed until it touches it, as far as the grip closes them
    const grasps = goals.grasps.length ? goals.grasps : pm ? [{ id: pm.id, hands: [pm.hand], grip: "palm" as const, w: pm.w, close: 1 }] : [];
    for (const gr of grasps) {
      const tr = tracks.get(gr.id)!, held = tr[f].held || work[f].attach.some((a) => a.id === gr.id) || pm?.id === gr.id;
      const sdf = thingSdf(plan.items[gr.id]);
      const field = (q: Vec3) => sdf(qrot(qinv(tr[f].rot), sub(q, tr[f].at)));
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
          const hs = gr.hands[0], shaped = handPose(skel, poses[f], hs, shapes[hs][f - 1]!);
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
      // the other hand out of its way (hanging where a wine glass is brought down to the table)
      for (const s of (["L", "R"] as Side[]).filter((x) => !gr.hands.includes(x))) {
        const w = pose(skel, poses[f]);
        let deep = 0, at: Vec3 | undefined;
        for (const x of handSurface(skel, w, s)) { const d = -field2(x.at); if (d > deep) { deep = d; at = x.at; } }
        if (at && deep > 0.002) dodge[s][f] = scale3(fieldNormal(field2, at), deep + 0.008);
      }
    }
  }
  easeDodges(skel, plan, poses, dodge, carried, tracks, grasped, shapes);
  const at = <X,>(a: X[], f: number) => a[Math.max(0, Math.min(a.length - 1, Math.round(f)))];
  return {
    frames: perf.frames,
    pose: (f) => at(poses, f),
    item: (id, f) => { const v = at(tracks.get(id)!, f); return { at: v.at, rot: v.rot }; },
    held: (id, f) => at(tracks.get(id)!, f).held,
    miss: (f) => at(misses, f),
  };
}
