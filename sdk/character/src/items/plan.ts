// planItems: the walks, reaches and waits that do the tasks.
import type { Vec3 } from "raea/3d";
import { B, bodyScale, joint, len, qrot, qyaw, sub, type Side } from "raea/character";
import { route, type MotionLibrary, type Obstacle, type Step } from "../matching";
import { handling, obstacleDistance, placeOn, shapeOf, standAt, surfacesOf, thingObstacles, toLocal, toWorld, under, type Handling, type Thing } from "../things";
import { Beat, D, E, Item, ItemPlan, ItemTask, P, S, T, THROW_LEN, autoStyle, clipMove, gripTurn, reachAhead, wrap } from "./all";

/** The steps for a list of tasks with things, from where the body starts. `things` are the furniture: what items rest
 * on and land on, and what walks go round (inside `bounds`). */
export function planItems(lib: MotionLibrary, start: { at: Vec3; facing: number }, items: Record<string, Item>, tasks: ItemTask[], o: { things?: Record<string, Thing> | Thing[]; bounds?: Obstacle; clear?: number; lead?: number } = {}): ItemPlan {
  const h = lib.skel.height, steps: Step[] = [], beats: Beat[] = [];
  const looks: number[] = [];
  const named = o.things && !Array.isArray(o.things) ? o.things : {};
  const furniture = [...(Array.isArray(o.things) ? o.things : Object.values(named))];
  // every seat the plan sits on is in the way of its walks (RAEA_SEAT_OBSTACLE=0 not): unknown to the routes, errands'
  // walk to the chair's front from behind it went straight through it, the feet 15 cm into its base
  if (process.env.RAEA_SEAT_OBSTACLE !== "0") for (const t of tasks) {
    const sit = "step" in t ? (t.step as { sit?: { at: Vec3; facing: number | Vec3 } }).sit : undefined;
    if (!sit) continue;
    const fy = typeof sit.facing === "number" ? sit.facing : Math.atan2(sit.facing[0] - sit.at[0], sit.facing[2] - sit.at[2]);
    furniture.push({ at: [sit.at[0], 0, sit.at[2]], size: [0.48, sit.at[1], 0.48], yaw: fy });
  }
  let x = start.at[0], z = start.at[2], yaw = start.facing;
  const holding = new Map<string, Beat>();
  const carried = () => [...holding.values()].reduce((a, b) => a + b.how.load, 0);
  // (on a database library, docs/clips.feature: while a thing is held the upper body plays carry captures, the legs
  // walk from walks, and a heavy thing takes the whole body from carries; a library without tags plays the step whole)
  // (a thing in one hand: that hand low at its side, as interact's hold has it, so a suitcase's carry is played rather
  // than a box's, the hands before the chest)
  const SJ = lib.skel.joints, shY = SJ[joint(lib.skel, B.upperarm("L"))].head[1];
  const armL = len(sub(SJ[joint(lib.skel, B.lowerarm("L"))].head, SJ[joint(lib.skel, B.upperarm("L"))].head)) + len(sub(SJ[joint(lib.skel, B.hand("L"))].head, SJ[joint(lib.skel, B.lowerarm("L"))].head));
  const held = (): Pick<Extract<Step, { walk: unknown }>, "tags" | "limbs"> => {
    if (!holding.size) return {};
    const hands: Partial<Record<Side, Vec3>> = {}, clear: Partial<Record<Side, number>> = {};
    for (const b of holding.values()) {
      if (b.hands.length !== 1) continue;
      const sd = b.hands[0] === "L" ? 1 : -1, it = items[b.item];
      // (the room it needs as the arm swings past the thigh: its half width)
      if (b.how.grip !== "stem") clear[b.hands[0]] = Math.max(it.size[0], it.size[2]) / 2;
      // (a wine glass before the waist, the forearm out; anything else low at the side, the arm hanging)
      hands[b.hands[0]] = b.how.grip === "stem" ? [sd * 0.12 * h, shY - 0.17 * h, 0.14 * h] : [sd * 0.12 * h, shY - 0.78 * armL, 0.1 * h];
    }
    return { limbs: { upper: { tags: { want: ["carry"] }, ...(Object.keys(hands).length ? { hands } : {}) }, ...(Object.keys(clear).length ? { clear } : {}) }, ...(carried() > 0.1 ? { tags: { want: ["carry"] } } : {}) };
  };
  // a carry slows the walk by its load, a little: a load of a fifth of the body slows people's own pace by about a
  // twentieth (shorter strides, longer on both feet), and no further. Slowed by a quarter, as it was, the matched walk
  // crept with the hips 22 cm ahead of the feet (a man's 15 kg box on parkour.json, against 4 cm for a ball)
  // (a child's paces, a bigger body's by its size: its captures are scaled so)
  const zb = bodyScale(lib.skel), walkSpeed = () => { const l = carried(); return l > 0.02 ? Math.max(0.95, 1 - 1.2 * l) * zb : undefined; };
  // where to stand to reach `target` on `self`: at the thing `self` rests on (else the item itself), clear of the rest
  // A thing low down is stood back from as far as the way down to it needs, worked out as the body sees it coming:
  // a stoop reaches down close before the toes, a squat further out (the knees come forward, the hands reach past
  // them). A thing on furniture is stood back from at an easy arm's length, the body clear of the edge.
  // (a heavy thing is stood closer: it is lifted near the body). The target is kept an easy arm's length from the
  // body's middle, further the lower it is and the more of the way down is a squat (the knees come forward, the hands
  // reach past them), and clear of the things lying about.
  const spotFor = (target: Vec3, self: Thing, hand?: Side, bend = 0.5, load = 0, skip?: string, how?: Handling) => {
    const on = under(furniture, self.at);
    const light = Math.max(0, 1 - load / 0.12), low = Math.min(1, Math.max(0, (0.45 * h - target[1]) / (0.45 * h)));
    // (no further than the bend reaches easily from where the hips will be; off furniture, the knees bent at it need
    // room before its edge, and before a solid thing's more)
    const comfy = Math.min((0.17 + 0.06 * bend * low * (0.4 + 0.6 * light)) * h, low > 0 ? 0.9 * reachOf(target[1], bend) : Infinity);
    const others = [...thingObstacles(furniture.filter((t) => t !== on)), ...lying(skip)];
    return standAt(on ?? self, target, { from: [x, z], height: h, others, bounds: o.bounds, hand, gap: on ? (0.06 + 0.15 * bend * low + (on.solid ? 0.16 * bend : 0)) * h : low > 0.3 ? 0 : undefined, comfy, behind: (0.1 + 0.12 * bend) * low * h,
      elbows: how && (how.grip === "sides" || how.grip === "edges" || how.grip === "cradle" || how.grip === "hug") ? Math.max(self.size[0], self.size[2]) / 2 + 0.08 * h : undefined });
  };
  const reaches = new Map<string, number>();
  const reachOf = (y: number, bend: number) => {
    const k = `${Math.round(y * 50)}:${Math.round(bend * 20)}`;
    if (!reaches.has(k)) reaches.set(k, reachAhead(lib.skel, Math.round(y * 50) / 50, Math.round(bend * 20) / 20));
    return reaches.get(k)!;
  };
  // the items lying on the floor (where the plan has left them), as obstacles: walks go round them, bodies don't
  // stand on them
  const restAt = new Map(Object.entries(items).map(([id, it]) => [id, it as Thing]));
  const lying = (skip?: string) => thingObstacles([...restAt].filter(([id, t]) => id !== skip && !holding.has(id) && t.at[1] < 0.02).map(([, t]) => ({ ...t, blocks: true })));
  const goTo = (s: { at: [number, number]; face: number }, skip?: string) => {
    if (Math.hypot(s.at[0] - x, s.at[1] - z) > 0.05) {
      const way = route([x, z], s.at, [...thingObstacles(furniture), ...lying(skip)], { bounds: o.bounds, clear: o.clear, arrive: s.face, lead: o.lead });
      // (straight on from a walk before it, slowed to this one's pace rather than stopped and started again)
      const prev = steps[steps.length - 1], on = prev && "walk" in prev && prev.face === undefined ? Math.min(prev.speed ?? zb, walkSpeed() ?? zb) : 0;
      if (on && "walk" in prev) prev.leave = on;
      steps.push({ walk: way, face: s.face, speed: walkSpeed(), ...(on ? { enter: on } : {}), ...held() });
    }
    x = s.at[0]; z = s.at[1]; yaw = s.face;
  };
  // heavier is slower, and lower: a thing on the floor takes half as long again to reach for, lift or put down
  const slowOf = (how: Handling) => 1 + 2 * Math.min(0.35, how.load);
  // how a body gets down to a low thing, unless the task says: a heavy thing is squatted to (the back kept straight);
  // a lighter one any mix of the knees and a bend at the hips, varied from one reach to the next (as people do)
  const bendOf = (how: Handling, id: string, k: number, said?: number) => {
    if (said !== undefined) return Math.min(1, Math.max(0, said));
    let hsh = k * 2654435761;
    for (const c of id) hsh = Math.imul(hsh ^ c.charCodeAt(0), 2246822519) >>> 0;
    const r = (hsh >>> 8) / 16777216;
    return how.weight === "light" ? 0.25 + 0.5 * r : how.weight === "medium" ? 0.55 + 0.3 * r : 0.85 + 0.15 * r;
  };
  const lowOf = (y: number) => 1 + 0.5 * Math.max(0, 1 - y / (0.45 * h));
  let slack: number | undefined;
  for (const t of tasks) {
    if ("pick" in t) {
      const it = items[t.pick];
      if (!it) throw new Error(`planItems: no item ${t.pick}`);
      let how = handling(it, h);
      if (t.hand === "both" && how.hands === 1) how = { ...how, grip: ({ pole: "pole", ball: "cradle", box: "edges", round: "sides" } as const)[shapeOf(it)], hands: 2 };
      // (one that fits in the hand, from above: a ball, a can, a small box, light)
      if ((t.from ?? (t.manner === "patient" ? "above" : "side")) === "above" && how.hands === 1 && how.weight === "light" && (how.grip === "palm" || how.grip === "wrap" || how.grip === "pinch") && !t.clip) how = { ...how, grip: "top" };
      const hand: Side = t.hand === "L" ? "L" : "R";
      const mid: Vec3 = [it.at[0], it.at[1] + it.size[1] / 2, it.at[2]];
      const bend = bendOf(how, t.pick, steps.length + 1, t.bend);
      // (a pole from one of its long sides, never its end)
      // (beside it: the hand's side a quarter of the height out from the body's middle, a little behind the thing)
      const side = (k: number) => (hand === "L" ? -k : k);
      const s = t.face !== undefined
        ? { at: [mid[0] + Math.cos(t.face) * side(0.25 * h) - Math.sin(t.face) * 0.04 * h, mid[2] - Math.sin(t.face) * side(0.25 * h) - Math.cos(t.face) * 0.04 * h] as [number, number], face: t.face }
        : spotFor(mid, shapeOf(it) === "pole" ? { ...it, use: it.use ?? ["left", "right"] } : it, how.hands === 1 ? hand : undefined, bend, how.load, t.pick, how);
      // a handle facing away from the hand is no use: a hand round it instead
      if (how.grip === "handle") {
        const g = it.grips![0].at, dir = qrot(qyaw((it.yaw ?? 0) - s.face), [g[0], 0, g[2]]);
        // (out to the hand's side or toward the body: a handle on the far side would be reached round for)
        const l = Math.hypot(dir[0], dir[2]) || 1;
        if ((hand === "L" ? dir[0] : -dir[0]) < -0.2 * l || dir[2] > 0.35 * l) how = { ...how, grip: shapeOf(it) === "ball" ? "palm" : "wrap" };
      }
      if (t.clip && how.weight !== "immovable") {
        // played from a capture: placed so its palm comes lowest over the thing (the hips facing it then, or `face`),
        // walked to where it starts
        const m = clipMove(lib, t.clip, "pick", hand), face = t.face ?? Math.atan2(mid[0] - x, mid[2] - z);
        const R = (v: Vec3) => qrot(qyaw(face), v), pk = R(m.palm), at: [number, number] = [mid[0] - pk[0], mid[2] - pk[2]], s0 = R(m.start), e = R(m.end);
        // (the walk before it, going this way, goes on to where the capture starts: stopped short, it stepped on to it
        // from standing, a foot held up a second)
        const startAt: [number, number] = [at[0] + s0[0], at[1] + s0[2]], prev = steps[steps.length - 1];
        if (prev && "walk" in prev && prev.face === undefined && Math.hypot(startAt[0] - x, startAt[1] - z) < 1.5 && (startAt[0] - x) * Math.sin(yaw) + (startAt[1] - z) * Math.cos(yaw) > 0) {
          prev.walk[prev.walk.length - 1] = startAt; prev.face = face + m.startYaw;
          x = startAt[0]; z = startAt[1]; yaw = face + m.startYaw;
        } else goTo({ at: startAt, face: face + m.startYaw }, t.pick);
        beats.push({ step: steps.length, kind: "pick", item: t.pick, hands: [hand], how, turn: gripTurn(it, how, hand, face), slow: 1, bend, tag: t.tag, clip: { key: m.key - m.from, len: m.to - m.from + 1 } });
        steps.push({ act: m.act, from: m.from, to: m.to, anchor: m.key, at, face, yaw: face + m.startYaw });
        x = at[0] + e[0]; z = at[1] + e[2]; yaw = face + m.endYaw;
        holding.set(t.pick, beats[beats.length - 1]); restAt.delete(t.pick);
        slack = undefined;
        continue;
      }
      // (a patient one stood as a quick one is: walked round to face a thing beside the way, a 90° turn on the spot
      // splayed the legs in a sidestep no capture makes; the trunk turns to it instead)
      const patient = t.manner === "patient", sp = s;
      // (a patient one looks about first: a capture of someone looking round, played where it takes the thing from,
      // stood back from it 5 cm (the capture's turning hips brushed the plinth), and reached from where it ends; the
      // head swept by a made-up gaze glitched, Shane 10-22: "the looking is very glitchy", and walked up to the thing
      // after looking about from where the run stopped, the stop and the start between matched runs and floated)
      goTo(sp, t.pick);
      if (patient && t.look) {
        const m = clipMove(lib, t.look, "throw", hand), R = (v: Vec3) => qrot(qyaw(yaw), v), s0 = R(m.start), e = R(m.end);
        const ax = sp.at[0] - it.at[0], az = sp.at[1] - it.at[2], al = Math.hypot(ax, az) || 1;
        const at: [number, number] = [x - s0[0] + (0.05 * zb * ax) / al, z - s0[2] + (0.05 * zb * az) / al];
        looks.push(steps.length);
        steps.push({ act: m.act, from: m.from, to: m.to, anchor: m.key, at, face: yaw, yaw: yaw + m.startYaw });
        x = at[0] + e[0]; z = at[1] + e[2]; yaw = yaw + m.endYaw;
      }
      const strain = how.weight === "immovable";
      const b: Beat = { step: steps.length, kind: "pick", item: t.pick, hands: how.hands === 2 ? ["L", "R"] : [hand], how, turn: gripTurn(it, how, hand, sp.face), slow: slowOf(how) * lowOf(it.at[1]), strain, bend, tag: t.tag, manner: patient ? "patient" : "quick" };
      beats.push(b);
      // (taken in passing, beside it: wherever the stop comes to within 10 cm, the hand reaching the rest)
      slack = t.face !== undefined ? 0.1 : undefined;
      steps.push({ wait: (strain ? S.strain : T.pick) * b.slow, face: sp.face, slack });
      if (!strain) { holding.set(t.pick, b); restAt.delete(t.pick); }
      if (patient && !strain) {
        beats.push({ ...b, step: steps.length, kind: "examine" });
        steps.push({ wait: E.look, face: sp.face, ...held() });
      }
      continue;
    } else if ("place" in t || "drop" in t || "throw" in t) {
      const id = "place" in t ? t.place : "drop" in t ? t.drop : t.throw, held = holding.get(id);
      if (!held) throw new Error(`planItems: ${id} isn't held (picked up first, and light enough to lift)`);
      const it = items[id], base = { item: id, hands: held.hands, how: held.how, turn: held.turn, tag: held.tag, slow: slowOf(held.how), bend: bendOf(held.how, id, steps.length, "place" in t ? t.bend : undefined) };
      if ("place" in t) {
        let at = t.at ?? (t.on && named[t.on] ? placeOn(named[t.on], { at: t.spot }) : undefined);
        if (!at) throw new Error(`planItems: place ${id} needs at, or on a thing given in things`);
        const hand1 = held.hands.length === 1 ? held.hands[0] : undefined;
        let s = spotFor([at[0], at[1] + it.size[1] / 2, at[2]], { ...it, at }, hand1, base.bend, held.how.load, id, held.how);
        // on a thing with no spot said: put down near the edge the body comes to, as someone does who sees where it
        // will go from an easy arm's length rather than walking up against the furniture to reach its middle
        if (!t.at && t.on && !t.spot) {
          const host = named[t.on], sf = surfacesOf(host)[0], body = toLocal(host, [s.at[0], 0, s.at[1]]), inset = Math.max(it.size[0], it.size[2]) / 2 + 0.03;
          const lx = Math.min(sf.max[0] - inset, Math.max(sf.min[0] + inset, body[0])), lz = Math.min(sf.max[1] - inset, Math.max(sf.min[1] + inset, body[2]));
          const c = toLocal(host, at), near: Vec3 = [Math.abs(body[0]) > host.size[0] / 2 ? lx : c[0], 0, Math.abs(body[2]) > host.size[2] / 2 ? lz : c[2]];
          at = toWorld(host, [near[0], sf.y, near[2]]);
          s = spotFor([at[0], at[1] + it.size[1] / 2, at[2]], { ...it, at }, hand1, base.bend, held.how.load, id, held.how);
        }
        goTo(s, id);
        holding.delete(id);
        restAt.set(id, { ...it, at, yaw: t.yaw ?? s.face + held.turn });
        const slow = base.slow * lowOf(at[1]);
        beats.push({ ...base, slow, step: steps.length, kind: "place", to: { at, yaw: t.yaw ?? s.face + held.turn } });
        steps.push({ wait: P.place * slow, face: s.face });
      } else if ("drop" in t) {
        holding.delete(id);
        restAt.set(id, { ...it, at: [x + Math.sin(yaw) * 0.2 * h, 0, z + Math.cos(yaw) * 0.2 * h] });
        beats.push({ ...base, step: steps.length, kind: "drop" });
        steps.push({ wait: D.drop });
      } else {
        const target = typeof t.to === "string" ? (named[t.to] ? placeOn(named[t.to]) : undefined) : t.to;
        if (!target) throw new Error(`planItems: throw ${id} to ${String(t.to)}: no such thing in things`);
        const face = Math.atan2(target[0] - x, target[2] - z), dist = Math.hypot(target[0] - x, target[2] - z);
        const style = !t.style || t.style === "auto" ? autoStyle(held.how, dist) : t.style;
        if (t.clip) {
          // played from a capture, from where the body stands, stepping toward the target (turned first to the way
          // the capture starts if that is well round: less, the capture turns into its wind-up, the act easing it in;
          // stood turning a third of a radian, the feet left the floor)
          const m = clipMove(lib, t.clip, "throw", held.hands[0]), R = (v: Vec3) => qrot(qyaw(face), v), s0 = R(m.start), e = R(m.end);
          const at: [number, number] = [x - s0[0], z - s0[2]], from = face + m.startYaw;
          if (Math.abs(wrap(from - yaw)) > 0.7) steps.push({ wait: 0.3 + Math.abs(wrap(from - yaw)) * 0.6, face: from });
          holding.delete(id);
          restAt.set(id, { ...it, at: target });
          beats.push({ ...base, slow: 1, step: steps.length, kind: "throw", throw: { style, target }, clip: { key: m.key - m.from, len: m.to - m.from + 1 } });
          steps.push({ act: m.act, from: m.from, to: m.to, anchor: m.key, at, face, yaw: from });
          x = at[0] + e[0]; z = at[1] + e[2]; yaw = face + m.endYaw;
          continue;
        }
        // turned to the target first if it is off to the side
        if (Math.abs(wrap(face - yaw)) > 0.3) steps.push({ wait: 0.3 + Math.abs(wrap(face - yaw)) * 0.6, face });
        yaw = face;
        holding.delete(id);
        restAt.set(id, { ...it, at: target });
        beats.push({ ...base, step: steps.length, kind: "throw", throw: { style, target } });
        steps.push({ wait: THROW_LEN[style] * base.slow, face });
      }
    } else if ("walk" in t) {
      // (round anything in its way, a leg at a time: straight where it is clear)
      const obs = [...thingObstacles(furniture), ...lying()], clear = o.clear ?? 0.3, way: [number, number][] = [[x, z]];
      for (const q of t.walk) {
        const a = way[way.length - 1], n = Math.ceil(Math.hypot(q[0] - a[0], q[1] - a[1]) / 0.05);
        const blocked = Array.from({ length: n + 1 }, (_, i) => [a[0] + ((q[0] - a[0]) * i) / Math.max(1, n), a[1] + ((q[1] - a[1]) * i) / Math.max(1, n)] as [number, number]).some((pt) => obs.some((b) => obstacleDistance(pt, b) < clear));
        way.push(...(blocked ? route(a, q, obs, { bounds: o.bounds, clear }).slice(1) : [q]));
      }
      steps.push({ walk: way, speed: t.speed ?? walkSpeed(), enter: t.enter, leave: t.leave, ...held() });
      const e = t.walk[t.walk.length - 1], p = t.walk.length > 1 ? t.walk[t.walk.length - 2] : [x, z];
      yaw = Math.atan2(e[0] - p[0], e[1] - p[1]);
      x = e[0]; z = e[1];
    } else if ("step" in t) {
      steps.push(t.step);
      const sit = (t.step as { sit?: { at: Vec3; facing: number | Vec3 } }).sit;
      if (sit && process.env.RAEA_SEAT_OBSTACLE !== "0") {
        // stood up, a step forward clear of it before turning anywhere (turned on the spot at its front, the turn's
        // steps went back into it)
        const out: [number, number] = [t.at[0] + Math.sin(t.face) * 0.45, t.at[1] + Math.cos(t.face) * 0.45];
        steps.push({ walk: [[t.at[0], t.at[1]], out], speed: 0.8, ...held() });
        x = out[0]; z = out[1]; yaw = t.face;
      } else { x = t.at[0]; z = t.at[1]; yaw = t.face; }
    } else if (t.face !== undefined) {
      steps.push({ wait: t.wait, face: t.face, slack, ...held() });
      yaw = t.face;
      continue;
    } else steps.push({ wait: t.wait, ...held() });
    slack = undefined;
  }
  // a thing thrown or dropped last is watched until it lands
  const last = tasks[tasks.length - 1];
  if (last && ("throw" in last || "drop" in last)) steps.push({ wait: 2 });
  return { spec: { at: start.at, facing: start.facing, steps }, items, things: furniture, beats, looks, fps: lib.fps };
}
