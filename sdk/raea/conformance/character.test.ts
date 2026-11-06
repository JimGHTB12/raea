// raea/character, clip and matching under plain bun: the skeleton's forward kinematics, the goals landing
// where they are sent, clips surviving their file format, routes keeping clear, and a performance being a pure
// function of its inputs. The skeleton is a small stand-in on raea's game_engine joints (stick.ts).
import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { BevyApp } from "raea/bevy";
import { captured, flatHand, withFingers, len, lookAt, palmAt, plant, pose, qaxis, qrot, reach, rest, softElbows, sub, twoBone, type Skeleton } from "raea/character";
import { readBvh, readClips, retarget, writeClips } from "raea/clip";
import { before, libraryPose, libraryTable, mirror, motionLibrary, perform, planFloor, route, stateMix, through, walkPlan } from "raea/matching";
import { applyEffect, clipMeans, learnEffect, mirrorEffect, mixEffects } from "raea/effects";
import { styleOf } from "raea/style";
import { interact, planItems } from "raea/items";
import { footDrift } from "raea/feet";
import { motionLint } from "raea/lint";
import { obstacleDistance } from "raea/things";
import { defineRoom, type LaidRun, type Plan } from "raea/room";
import type { Vec3 } from "../src/3d";
import { mirrorX, names, skel, stickBvh } from "./stick";

const at = (n: string) => names.indexOf(n);
const near = (a: Vec3, b: Vec3, tol: number) => expect(len(sub(a, b))).toBeLessThan(tol);

describe("the skeleton", () => {
  test("at rest every joint is at its head", () => {
    const w = withFingers(skel, pose(skel, rest(skel)));
    skel.joints.forEach((j, i) => near(w.pos[i], j.head, 1e-9));
  });
  test("a turn at the shoulder carries the arm round it", () => {
    const p = rest(skel);
    p.turns[at("upperarm_l")] = qaxis([0, 0, 1], -Math.PI / 2); // the left arm down
    const w = pose(skel, p);
    near(w.pos[at("hand_l")], [0.12, 0.98 - 0.42, 0], 1e-9);
  });
  test("a mirrored pose mirrors the joints", () => {
    const p = rest(skel);
    p.turns[at("upperarm_l")] = qaxis([0, 0, 1], -1);
    const w = pose(skel, p), m = pose(skel, mirror(skel, p));
    near(m.pos[at("hand_r")], mirrorX(w.pos[at("hand_l")]), 1e-9);
  });
});

describe("goals", () => {
  test("twoBone brings the wrist to a point in reach", () => {
    const target: Vec3 = [0.3, 0.75, 0.25];
    const w = pose(skel, twoBone(skel, rest(skel), ["upperarm_l", "lowerarm_l", "hand_l"], target, [0, -1, 0]));
    near(w.pos[at("hand_l")], target, 1e-3);
  });
  test("a limb never locks straight: out of reach it stops short, and a straight limb bends its own way", () => {
    const bend = (p: ReturnType<typeof rest>, a: string, b: string, c: string) => {
      const w = pose(skel, p), u = sub(w.pos[at(b)], w.pos[at(a)]), v = sub(w.pos[at(c)], w.pos[at(b)]);
      return (Math.acos((u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / len(u) / len(v)) * 180) / Math.PI;
    };
    // a wrist sent a metre out: the elbow still about 11° bent
    const far = twoBone(skel, rest(skel), ["upperarm_l", "lowerarm_l", "hand_l"], [1.2, 0.98, 0]);
    expect(bend(far, "upperarm_l", "lowerarm_l", "hand_l")).toBeGreaterThan(9);
    // an ankle pulled a little up from a straight leg: the knee goes ahead, not out or back
    const w = pose(skel, twoBone(skel, rest(skel), ["thigh_l", "calf_l", "foot_l"], [0.1, 0.12, 0]));
    const knee = w.pos[at("calf_l")];
    expect(knee[2]).toBeGreaterThan(0.05);
    // a straight arm relaxed to 12°, one bent 30° left alone
    expect(bend(softElbows(skel, rest(skel)), "upperarm_l", "lowerarm_l", "hand_l")).toBeCloseTo(12, 0);
    const bent = rest(skel);
    bent.turns[at("lowerarm_l")] = qaxis([0, 1, 0], -(30 * Math.PI) / 180);
    expect(bend(softElbows(skel, bent), "upperarm_l", "lowerarm_l", "hand_l")).toBeCloseTo(30, 1);
    // relaxing bends the forearm ahead of the body
    expect(pose(skel, softElbows(skel, rest(skel))).pos[at("hand_l")][2]).toBeGreaterThan(0.02);
  });
  test("reach lays the hand flat and puts the palm on the point", () => {
    const target: Vec3 = [0.25, 0.7, 0.3];
    const p = reach(skel, rest(skel), "L", { at: target, point: [0, 0, 1] });
    const w = withFingers(skel, pose(skel, p));
    // the palm resting on the point: its middle half a hand's thickness over it
    near(palmAt(skel, w, "L"), [target[0], target[1] + 0.009 * skel.height, target[2]], 2e-3);
    // the fingers ahead and level
    const f = sub(w.pos[at("middle_01_l")], w.pos[at("hand_l")]);
    expect(f[2] / len(f)).toBeGreaterThan(0.99);
    void flatHand;
  });
  test("a planted foot's ankle stands over its spot at its height", () => {
    // on a step 10 cm up, a little ahead
    const w = pose(skel, plant(skel, rest(skel), "L", { at: [0.1, 0.1, 0.1] }));
    near(w.pos[at("foot_l")], [0.1, 0.16, 0.1], 1e-3);
  });
  test("lookAt turns the face to the target", () => {
    const target: Vec3 = [1, 1.14, 1];
    const w = pose(skel, lookAt(skel, rest(skel), target));
    const h = at("head");
    const fwd = qrot(w.rot[h], [0, 0, 1]);
    const eyes = sub(w.pos[h], qrot(w.rot[h], sub(skel.joints[h].head, skel.landmarks.eyes)));
    const want = sub(target, eyes);
    expect((fwd[0] * want[0] + fwd[1] * want[1] + fwd[2] * want[2]) / len(want)).toBeGreaterThan(0.99);
  });
});

describe("clips", () => {
  const b = readBvh(stickBvh(361));
  const clip = retarget(skel, b, { name: "walk" });
  test("a BVH reads its joints and frames", () => {
    expect(b.joints.length).toBe(skel.joints.length);
    expect(b.frames.length).toBe(361);
  });
  test("retargeting onto the same skeleton keeps the walk's pace", () => {
    // (3 s at 30 fps, both ends: a BVH's T-pose frames are trimmed by pose, not by a skipped first frame)
    expect(clip.frames).toBe(91);
    const d = clip.root[(clip.frames - 1) * 3 + 2] - clip.root[2];
    expect(d / ((clip.frames - 1) / 30)).toBeCloseTo(1.1, 2);
  });
  test("a clip file reads back what was written (quaternions to a part in 32767)", () => {
    const back = readClips(JSON.parse(JSON.stringify(writeClips(skel, [clip])))).get("walk")!;
    expect(back.frames).toBe(clip.frames);
    let err = 0;
    for (let i = 0; i < clip.turns.length; i++) err = Math.max(err, Math.abs(Math.abs(back.turns[i]) - Math.abs(clip.turns[i])));
    expect(err).toBeLessThan(2e-4);
  });
});

describe("routes and performances", () => {
  test("a route goes round an obstacle, clear of it", () => {
    const box = { min: [1, -0.5] as [number, number], max: [2, 0.5] as [number, number] };
    const r = route([0, 0], [3, 0], [box], { clear: 0.3 });
    expect(r.length).toBeGreaterThan(2);
    for (let k = 1; k < r.length; k++) {
      for (let u = 0; u <= 1; u += 0.05) {
        const x = r[k - 1][0] + (r[k][0] - r[k - 1][0]) * u, z = r[k - 1][1] + (r[k][1] - r[k - 1][1]) * u;
        const inside = x > box.min[0] - 0.2 && x < box.max[0] + 0.2 && z > box.min[1] - 0.2 && z < box.max[1] + 0.2;
        expect(inside).toBe(false);
      }
    }
  });
  test("a performance arrives where it is sent and is the same every time", () => {
    const walk = retarget(skel, readBvh(stickBvh(1201)), { name: "walk" }), stand = retarget(skel, readBvh(stickBvh(361, 120, true)), { name: "stand" });
    const clips = readClips(JSON.parse(JSON.stringify(writeClips(skel, [walk, stand]))));
    const lib = motionLibrary(skel, clips);
    const spec = { at: [0, 0, 0] as Vec3, facing: 0, steps: [{ walk: [[0, 0], [0, 2.5]] as [number, number][] }, { wait: 0.5 }] };
    const a = perform(lib, spec), b = perform(lib, JSON.parse(JSON.stringify(spec)));
    const end = pose(skel, a.pose(a.frames - 1)).pos[0];
    expect(Math.hypot(end[0], end[2] - 2.5)).toBeLessThan(0.15);
    expect(JSON.stringify(a.pose(40))).toBe(JSON.stringify(b.pose(40)));
  });
  test("planted feet stay put while they stand, and a walk asked faster than the library's skates less for the pace match", () => {
    const walk = retarget(skel, readBvh(stickBvh(1201)), { name: "walk" }), stand = retarget(skel, readBvh(stickBvh(361, 120, true)), { name: "stand" });
    const lib = motionLibrary(skel, readClips(JSON.parse(JSON.stringify(writeClips(skel, [walk, stand])))));
    const spec = { at: [0, 0, 0] as Vec3, facing: 0, steps: [{ walk: [[0, 0], [0, 3], [2, 4]] as [number, number][], speed: 1.2 }, { wait: 0.5 }] };
    const free = perform(lib, spec, { pin: false, gaze: false }), planted = perform(lib, spec, { gaze: false, trace: true, quiet: true });
    const a = footDrift(skel, free.pose, free.frames), b = footDrift(skel, planted.pose, planted.frames);
    expect(b.skateCms).toBeLessThan(a.skateCms);
    // (a knee kept from buckling lets its foot give a little)
    expect(b.slideCm).toBeLessThan(4);
    // and the lint finds no buckle: no standing knee folded past the clip's
    expect(motionLint(lib, planted).findings.filter((f) => f.kind === "buckle")).toEqual([]);
    expect(b.kneeMax).toBeLessThan(15);
  });
  test("a route's corners are rounded clear of the obstacles, and it can come in to its end along a way", () => {
    const box = { min: [1, 1] as [number, number], max: [2, 2] as [number, number] };
    const way = route([0, 0], [3, 3], [box], { clear: 0.3, arrive: 0 });
    for (const p of way) expect(obstacleDistance(p, box)).toBeGreaterThan(0.24);
    // the last leg comes up the +z way to the end
    const a = way[way.length - 2], b = way[way.length - 1];
    expect(Math.abs(b[0] - a[0])).toBeLessThan(0.05 * Math.hypot(b[0] - a[0], b[1] - a[1]) + 1e-6);
    // no corner sharper than a gentle bend between the arc's points
    for (let k = 1; k < way.length - 2; k++) {
      const u = [way[k][0] - way[k - 1][0], way[k][1] - way[k - 1][1]], v = [way[k + 1][0] - way[k][0], way[k + 1][1] - way[k][1]];
      expect(Math.acos(Math.max(-1, Math.min(1, (u[0] * v[0] + u[1] * v[1]) / Math.hypot(u[0], u[1]) / Math.hypot(v[0], v[1]))))).toBeLessThan(0.8);
    }
  });
});

describe("the library's table, styles and effects", () => {
  const walk = retarget(skel, readBvh(stickBvh(1201)), { name: "walk" }), stand = retarget(skel, readBvh(stickBvh(361, 120, true)), { name: "stand" });
  const clips = readClips(JSON.parse(JSON.stringify(writeClips(skel, [walk, stand]))));
  test("a stored table loads the library it was written from", () => {
    const a = motionLibrary(skel, clips, { weights: { feet: 0.7 } });
    const b = motionLibrary(skel, new Map(clips), { weights: { feet: 0.7 }, table: JSON.parse(JSON.stringify(libraryTable(skel, clips))) });
    expect(b.size).toBe(a.size);
    let d = 0;
    for (let i = 0; i < a.features.length; i++) if (!Number.isNaN(a.features[i])) d = Math.max(d, Math.abs(a.features[i] - b.features[i]));
    expect(d).toBeLessThan(1e-3);
    expect([...b.contact]).toEqual([...a.contact]);
  });
  test("a mirrored frame is its clip's frame in the mirror", () => {
    const lib = motionLibrary(skel, clips), r = lib.clips.find((c) => c.mirrored && c.name === "walk")!;
    expect(JSON.stringify(libraryPose(lib, r.start + 10))).toBe(JSON.stringify(mirror(skel, libraryPose(lib, lib.clips[0].start + 10))));
  });
  test("a walk's style: it travels, steps and swings its legs; a stand doesn't", () => {
    const w = styleOf(skel, walk, 0, 30), s = styleOf(skel, stand, 0, 30);
    expect(w.speed).toBeCloseTo(1.1, 1);
    expect(w.cadence).toBeGreaterThan(1);
    expect(s.speed).toBeLessThan(0.01);
    expect(s.cadence).toBe(0);
  });
  test("an effect learned from a clip against itself changes nothing, and at weight 0 nothing either", () => {
    const e = learnEffect(skel, walk, walk), base = clipMeans(skel, walk), p = readClips(writeClips(skel, [walk])).get("walk")!;
    const q = { root: [p.root[30], p.root[31], p.root[32]] as Vec3, turns: Array.from({ length: captured(skel) }, (_, i) => Array.from(p.turns.subarray((30 * captured(skel) + i) * 4, (30 * captured(skel) + i) * 4 + 4)) as [number, number, number, number]) };
    const out = applyEffect(skel, q, e, { base });
    for (let i = 0; i < q.turns.length; i++) for (let a = 0; a < 4; a++) expect(Math.abs(Math.abs(out.turns[i][a]) - Math.abs(q.turns[i][a]))).toBeLessThan(1e-3);
    expect(applyEffect(skel, q, mixEffects([[e, 1]]), { base, weight: 0 })).toBe(q);
    expect(mirrorEffect(skel, mirrorEffect(skel, e)).posture).toEqual(e.posture);
  });
  test("an act step plays its clip with its anchor over the spot, along the way it faces", () => {
    const hop = retarget(skel, readBvh(stickBvh(481)), { name: "hop" });
    const cl = readClips(JSON.parse(JSON.stringify(writeClips(skel, [walk, stand, hop]))));
    const lib = motionLibrary(skel, cl, { locomotion: ["walk", "stand"], actions: ["hop"] });
    const p = perform(lib, { at: [0, 0, 0], facing: Math.PI / 2, steps: [
      { walk: [[0, 0], [1, 0]], speed: 1.1, leave: 1.1 }, { act: "hop", anchor: 30, at: [2.6, 0], face: Math.PI / 2 }, { walk: [[4.4, 0], [6, 0]], enter: 1.1, speed: 1.1 }] });
    const at = p.guide.findIndex((g) => g.act?.j === 30);
    // the hips over the spot (the start's error eased out by then), and the clip's way along +x
    const hips = p.pose(at).root;
    expect(Math.hypot(hips[0] - 2.6, hips[2])).toBeLessThan(0.05);
    expect(p.pose(at + 10).root[0]).toBeGreaterThan(hips[0] + 0.2);
    expect(p.played[at].clip).toBe("hop");
  });
  test("a state plays the frames of its own style where the library has them, and Neutral's where it hasn't", () => {
    // two styles walking alike: only the label tells them apart
    const named = (c: typeof walk, name: string) => ({ ...c, name });
    const cl = readClips(JSON.parse(JSON.stringify(writeClips(skel, [named(walk, "Neutral_FW"), named(stand, "Neutral_ID"), named(walk, "Old_FW")]))));
    const lib = motionLibrary(skel, cl);
    const styles = (state?: string) => new Set(perform(lib, { at: [0, 0, 0], facing: 0, steps: [{ walk: [[0, 0], [0, 3]] }], state }).played.slice(5).map((x) => x.clip.split("_")[0]));
    expect(styles("Old")).toEqual(new Set(["Old"]));
    expect(styles("Neutral")).toEqual(new Set(["Neutral"]));
    expect(styles("Zombie")).toEqual(new Set(["Neutral"]));
  });
  test("states read as mixes of styles", () => {
    expect(stateMix("Old")).toEqual({ Old: 1 });
    expect(stateMix("sad")).toEqual({ Depressed: 1 });
    expect(stateMix({ sad: 0.5, LimpLeft: 1 })).toEqual({ Depressed: 0.5, LimpLeft: 1 });
    expect(stateMix({ trunkPitch: 20 })).toBeNull();
  });
});

describe("items", () => {
  test("a thing picked up is in hand while held and ends where it is put down", () => {
    const walk = retarget(skel, readBvh(stickBvh(1201)), { name: "walk" }), stand = retarget(skel, readBvh(stickBvh(361, 120, true)), { name: "stand" });
    const lib = motionLibrary(skel, readClips(JSON.parse(JSON.stringify(writeClips(skel, [walk, stand])))));
    const items = { cup: { at: [1.2, 0.55, 0.3] as Vec3, size: [0.07, 0.09, 0.07] as Vec3 } };
    const plan = planItems(lib, { at: [0, 0, 0], facing: Math.PI / 2 }, items, [{ pick: "cup" }, { place: "cup", at: [0.2, 0.7, 2] }]);
    const live = interact(skel, perform(lib, plan.spec), plan);
    const heldFrames = Array.from({ length: live.frames }, (_, f) => f).filter((f) => live.held("cup", f));
    expect(heldFrames.length).toBeGreaterThan(30);
    // the hand on the cup's side all the while it's held
    for (const f of heldFrames) {
      const c = live.item("cup", f), palm = palmAt(skel, pose(skel, live.pose(f)), "R");
      expect(len(sub(palm, [c.at[0], c.at[1] + 0.045, c.at[2]]))).toBeLessThan(0.1);
    }
    near(live.item("cup", live.frames - 1).at, [0.2, 0.7, 2], 1e-6);
    expect(live.held("cup", live.frames - 1)).toBe(false);
  });
  const itemLib = () => {
    const walk = retarget(skel, readBvh(stickBvh(1201)), { name: "walk" }), stand = retarget(skel, readBvh(stickBvh(361, 120, true)), { name: "stand" });
    return motionLibrary(skel, readClips(JSON.parse(JSON.stringify(writeClips(skel, [walk, stand])))));
  };
  test("a thrown thing lands where it was thrown, a ball in a bin stays there, a dropped thing falls onto what is under it", () => {
    const lib = itemLib(), h = skel.height;
    const things = { table: { at: [2, 0, 0] as Vec3, size: [0.6, 0.4 * h, 0.5] as Vec3 }, bin: { at: [-1.5, 0, 1.5] as Vec3, size: [0.4, 0.3, 0.4] as Vec3, surfaces: [{ y: 0.02, min: [-0.18, -0.18] as [number, number], max: [0.18, 0.18] as [number, number] }] } };
    const items = {
      ball: { at: [0.8, 0, 0.6] as Vec3, size: [0.06, 0.06, 0.06] as Vec3, shape: "ball" as const, mass: 0.1 },
      box: { at: [1.9, 0.4 * h, 0] as Vec3, size: [0.2, 0.12, 0.15] as Vec3, mass: 1 },
    };
    const plan = planItems(lib, { at: [0, 0, 0], facing: 0 }, items, [{ pick: "ball" }, { throw: "ball", to: "bin" }, { pick: "box" }, { throw: "box", to: [2.6, 0, 1.4], style: "chest" }], { things });
    const live = interact(skel, perform(lib, plan.spec), plan);
    const end = (id: string) => live.item(id, live.frames - 1).at;
    near(end("ball"), [-1.5, 0.02, 1.5], 0.12);
    near(end("box"), [2.6, 0, 1.4], 0.15);
    expect(plan.beats.find((b) => b.kind === "throw" && b.item === "ball")!.throw!.style).toBe("underhand");
    // dropped where it was picked up, it falls back onto the table under the hold
    const plan2 = planItems(lib, { at: [0, 0, 0], facing: 0 }, items, [{ pick: "box" }, { drop: "box" }], { things });
    const live2 = interact(skel, perform(lib, plan2.spec), plan2);
    const last = live2.item("box", live2.frames - 1).at;
    expect(Math.abs(last[1] - 0.4 * h)).toBeLessThan(1e-6);
    expect(live2.held("box", live2.frames - 1)).toBe(false);
  });
  test("a heavy thing is slower to lift and carried slower; one too heavy is strained at and never held", () => {
    const lib = itemLib(), h = skel.height;
    const light = { at: [1, 0.5 * h, 0] as Vec3, size: [0.2, 0.15, 0.15] as Vec3, mass: 0.5 };
    const heavy = { ...light, size: [0.3, 0.25, 0.25] as Vec3, mass: 5 };
    const safe = { ...light, size: [0.4, 0.4, 0.4] as Vec3, mass: 80 };
    const plan = planItems(lib, { at: [0, 0, 0], facing: 0 }, { light, heavy, safe }, [{ pick: "light" }, { place: "light", at: [1, 0, 2] }, { pick: "heavy" }, { place: "heavy", at: [-1, 0, 2] }, { pick: "safe" }]);
    const pick = (id: string) => plan.beats.find((b) => b.item === id && b.kind === "pick")!;
    expect(pick("heavy").slow).toBeGreaterThan(pick("light").slow * 1.3);
    expect(pick("heavy").how.grip).toBe("hug");
    expect(pick("safe").strain).toBe(true);
    const carry = plan.spec.steps.filter((st) => "walk" in st) as { speed?: number }[];
    // (slowed a little, as people are: a twentieth for a fifth of the body's mass, not the quarter it was)
    expect(carry[3].speed!).toBeLessThan(0.95);
    const live = interact(skel, perform(lib, plan.spec), plan);
    for (let f = 0; f < live.frames; f++) expect(live.held("safe", f)).toBe(false);
    near(live.item("safe", live.frames - 1).at, safe.at, 1e-6);
    expect(() => planItems(lib, { at: [0, 0, 0], facing: 0 }, { safe }, [{ pick: "safe" }, { throw: "safe", to: [0, 0, 3] }])).toThrow();
  });
});

describe("the scene's room", () => {
  // a 4 x 3.5 m kitchen: a run along the back wall ending in a fridge, an island in the middle, a door in the front wall
  const room = defineRoom({ width: 4, depth: 3.5, height: 2.6 });
  const unit = (name: string, u0: number, u1: number, role?: "fridge" | "sink") => ({ src: "", name, role, u0, u1, depth: 0.6, rotation: [0, 0, 0] as Vec3, position: [0, 0, 0] as Vec3, height: 0.9, bottom: 0 });
  const run: LaidRun = { wall: room.walls.back, units: [unit("base", 0.5, 2.9), unit("fridge", 2.9, 3.6, "fridge")], end: 3.6 };
  const plan: Plan = { room, runs: [run], islands: [{ name: "island", min: [1.4, 1.5], max: [2.6, 2.2] }] };
  test("a doorway's spot is inside the room, a unit's before its front", () => {
    const door = through(room, { wall: "front", at: 2, width: 0.9, bottom: 0, top: 2.1, kind: "door" });
    near([door.at[0], 0, door.at[1]], [2, 0, 3.05], 1e-9);
    expect(door.face[2]).toBeLessThan(door.at[1]);
    const f = before(run, "fridge");
    near([f.at[0], 0, f.at[1]], [3.25, 0, 1.0], 1e-9);
    expect(f.face[2]).toBeCloseTo(0.3, 9);
  });
  test("a walk from the doorway to the fridge goes round the island and keeps inside the walls", () => {
    const door = through(room, { wall: "front", at: 2, width: 0.9, bottom: 0, top: 2.1, kind: "door" });
    const st = walkPlan(plan, door, before(run, "fridge")) as { walk: [number, number][] };
    const { obstacles } = planFloor(plan);
    expect(obstacles.length).toBe(2);
    expect(st.walk.length).toBeGreaterThan(2);
    for (let k = 1; k < st.walk.length; k++) {
      for (let u = 0; u <= 1; u += 0.05) {
        const x = st.walk[k - 1][0] + (st.walk[k][0] - st.walk[k - 1][0]) * u, z = st.walk[k - 1][1] + (st.walk[k][1] - st.walk[k - 1][1]) * u;
        expect(x > 1.2 && x < 2.8 && z > 1.3 && z < 2.4).toBe(false);
        expect(x > 0.2 && x < 3.8 && z > 0.2 && z < 3.3).toBe(true);
      }
    }
  });
});

// S6e: the motion engine as an App system. The engine is stubbed as plugins.test.ts stubs it: a fake __raea_call
// keeps a World of entities and their components by path and answers BRP's spawn, insert, query and get.
describe("CharacterPlugin", () => {
  const g = globalThis as { __raea_call?: (name: string, json: string) => string | undefined };
  let had: typeof g.__raea_call;
  const world = new Map<number, Record<string, unknown>>();
  let frame = 0;
  beforeAll(() => {
    had = g.__raea_call;
    g.__raea_call = (name, json) => {
      const a = JSON.parse(json);
      const p = a.params;
      switch (name) {
        case "bevy.app.open": return JSON.stringify({ app: 1, methods: [] });
        case "bevy.app.schedules": return JSON.stringify(["First", "PreUpdate", "Update", "PostUpdate", "Last"]);
        case "bevy.app.run": if (a.until === undefined) frame++; return JSON.stringify({ frame });
        case "bevy.app.step": frame += a.frames; return JSON.stringify([frame]);
      }
      switch (a.method) {
        case "world.spawn_entity": { const e = world.size + 1; world.set(e, { ...p.components }); return JSON.stringify({ entity: e }); }
        case "world.insert_components": Object.assign(world.get(p.entity)!, p.components); return JSON.stringify(null);
        case "world.get_components": return JSON.stringify({ components: Object.fromEntries(p.components.filter((c: string) => c in world.get(p.entity)!).map((c: string) => [c, world.get(p.entity)![c]])) });
        case "world.query": return JSON.stringify([...world].filter(([, cs]) => p.data.components.every((c: string) => c in cs)).map(([entity, cs]) => ({ entity, components: Object.fromEntries(p.data.components.map((c: string) => [c, cs[c]])) })));
      }
      return JSON.stringify(null);
    };
  });
  afterAll(() => void (g.__raea_call = had));

  test("a goal on an entity is performed by the plugin: its pose each frame is the performance's, and so are its joints", async () => {
    const { CharacterPlugin, CharacterPose, goal, poseAt, registerMotion } = await import("@raea/character/plugin");
    const walk = retarget(skel, readBvh(stickBvh(1201)), { name: "walk" }), stand = retarget(skel, readBvh(stickBvh(361, 120, true)), { name: "stand" });
    const lib = motionLibrary(skel, readClips(JSON.parse(JSON.stringify(writeClips(skel, [walk, stand])))));
    registerMotion("stick", lib);
    const spec = { at: [0, 0, 0] as Vec3, facing: 0, steps: [{ walk: [[0, 0], [0, 2.5]] as [number, number][] }, { wait: 0.5 }] };
    const ref = perform(lib, spec);
    const app = BevyApp.open({ plugins: [CharacterPlugin] });
    const e = app.spawn(goal("stick", spec, 2));
    app.step(1);
    // before its start the character holds the performance's first frame
    expect(app.get(e, CharacterPose)).toEqual({ frame: 0, translation: [ref.pose(0).root[0], 0, ref.pose(0).root[2]], yaw: ref.guide[0].yaw });
    app.step(40);
    const at = app.get(e, CharacterPose)!;
    expect(at.frame).toBe(40 - 2);
    expect(at.translation[2]).toBeCloseTo(ref.pose(38).root[2], 9);
    expect(JSON.stringify(poseAt({ motion: "stick", spec: JSON.stringify(spec), start: 2 }, 40))).toBe(JSON.stringify(ref.pose(38)));
    // past its end it holds the last frame
    app.step(ref.frames + 10);
    expect(app.get(e, CharacterPose)!.frame).toBe(ref.frames - 1);
    expect(() => poseAt({ motion: "nobody", spec: "{}", start: 0 }, 0)).toThrow(/no motion library "nobody".*stick/);
  });
});
