// interact: the hands, the lean, the squat, the trunk, the gaze and the grip laid over a performance (the things then
// fitted to the hands that have them: hold.ts).
import type { Vec3 } from "raea/3d";
import { B, add, ballInHand, flatHand, blend as blendPose, joint, norm, palmAt, pose, qaxis, qmul, qrot, qyaw, scale3, setWorld, slerp, solve, sub, type Pose, type Quat, type Side, type Skeleton } from "raea/character";
import { type Performance } from "../matching";
import { shapeOf } from "../things";
import { type Attach, type Goals, HAND, halfOf, Beat, D, E, Flight, Interaction, ItemPlan, P, QUICK_TWIST, S, T, THROW_HAND, THROW_LEN, Track, W, beatGeometry, clearWay, crouchDrop, crouched, eased, fingers, flightTime, fly, holdThings, launch, leanFor, smooth, smoothCarries, span, throwAt, throwKeys, trunkTurn, type Work } from "./all";

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
  const work: Work[] = [];

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
        const up = qrot(pose(skel, p).rot[joint(skel, B.chest)], [0, 1, 0]), fw = qrot(qyaw(g.yaw), [0, 0, 1]);
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
          // (the goal's weight over the reach's own span (RAEA_REACH_FAST=1 for the old fifth of a second): taken up at
          // once, the hand's turn onto the thing went whole in 0.2 s, the wrist turning 20 degrees a frame on the cup)
          handsTo(b, mid0, rot0, process.env.RAEA_REACH_FAST === "1" ? smooth(s / 0.2) : span(s, [0, T.reach[1]]), smooth((s - T.grip + 0.2) / 0.2));
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
          // (let go over the whole way back (RAEA_RELEASE_LATE=1 for over its last 30%): the hand's turn on the thing,
          // held to the end and dropped in a fifth of a second, swung the wrist 24 degrees a frame back to the walk's)
          const back0 = process.env.RAEA_RELEASE_LATE === "1" ? P.back[0] + 0.7 * (P.back[1] - P.back[0]) : P.back[0];
          handsTo(b, mid1, rot1, 1 - span(s, [back0, P.back[1]]), 1 - smooth((s - P.release) / 0.15));
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
    fly(flying, holding, now, placed, plan, h, dt, landing, supportAt);
    const hasHands = Object.keys(goals.hands).length > 0;
    // (a throw's knees are its style's: it doesn't squat to reach its wind-up)
    const need = hasHands && goals.weight > 0 && goals.squat ? crouchDrop(skel, p, goals.hands, goals.lean, goals.bend) : { drop: 0, stoop: 0 };
    work.push({ p, goals, drop: need.drop, stoop: need.stoop, attach, beat: !!b, palmed });
    for (const [id, tr] of tracks) tr.push(now.get(id) ?? { ...placed.get(id)!, held: false });
  }
  // the hips' drop and the stoop over the frames, eased (eased, below)
  // the stoop too, the same way (its least is as jumpy as the drop's, and tips the pelvis the legs hang from)
  const drops = eased(work.map((x) => x.drop)), stoops = eased(work.map((x) => x.stoop));
  const poses: Pose[] = [];
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
        const i = joint(skel, B.upperarm(s)), w = pose(skel, p);
        p = setWorld(skel, p, w, i, qmul(qaxis(qrot(qyaw(g.yaw), [0, 0, 1]), s === "L" ? a : -a), w.rot[i]));
      }
      if (goals.look && goals.lookWeight > 0) p = solve(skel, p, { look: goals.look, lookWeight: goals.lookWeight });
      p = fingers(skel, p, goals.fingers);
    }
    poses.push(p);
  }
  smoothCarries(skel, poses, work);
  const misses = holdThings(skel, plan, h, poses, work, tracks);
  const at = <X,>(a: X[], f: number) => a[Math.max(0, Math.min(a.length - 1, Math.round(f)))];
  return {
    frames: perf.frames,
    pose: (f) => at(poses, f),
    item: (id, f) => { const v = at(tracks.get(id)!, f); return { at: v.at, rot: v.rot }; },
    held: (id, f) => at(tracks.get(id)!, f).held,
    miss: (f) => at(misses, f),
  };
}
