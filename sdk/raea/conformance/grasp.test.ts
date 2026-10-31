// Grasp guides (docs/grasp-guides.feature): a thing's guides from its shape and size, a pick's tag choosing one, a hand
// planned on a guide in its own frame, and whether contacts can hold a thing up against gravity; on the kid's can, the
// guide puts the palm and the fingers on opposite sides, so it holds.
import { describe, expect, test } from "bun:test";
import { add, dot, len, norm, pose, qinv, qrot, sub, type Vec3 } from "raea/character";
import { autoGuides, contactsOf, guidesOf, handOn, holds, pickGuide, type Contact, type CylinderGuide, type SlabGuide } from "raea/grasp";
import { interact, thingSdf } from "raea/items";
import { perform } from "raea/matching";
import type { Thing } from "raea/things";
import { SCENES, gripRun } from "../../../examples/character/grip-scenes";
import { parkour } from "../../../examples/character/parkour-levels";

const can: Thing = { at: [0, 0, 0], size: [0.066, 0.122, 0.066], shape: "round", mass: 0.36 };
const close = (a: Vec3, b: Vec3, e = 1e-9) => expect(len(sub(a, b))).toBeLessThan(e);

describe("auto-fit", () => {
  test("a can gets a cylinder of its radius round its body", () => {
    const [g] = autoGuides(can) as CylinderGuide[];
    expect(g.kind).toBe("cylinder");
    expect(g.radius).toBeCloseTo(0.033, 9);
    expect(g.length).toBeCloseTo(0.122, 9);
    close(g.at, [0, 0.061, 0]);
    close(g.axis, [0, 1, 0]);
    expect(g.tags).toContain("body");
  });
  test("a box gets a slab across each pair of its upright faces, two hands where it's wider than a hand spans", () => {
    const gs = autoGuides({ at: [0, 0, 0], size: [0.33, 0.12, 0.2] }) as SlabGuide[];
    expect(gs.map((g) => g.kind)).toEqual(["slab", "slab"]);
    close(gs[0].normal, [1, 0, 0]);
    expect(gs[0].width).toBeCloseTo(0.33, 9);
    expect(gs[0].hands).toBe(2);
    close(gs[1].normal, [0, 0, 1]);
    expect(gs[1].width).toBeCloseTo(0.2, 9);
    expect((autoGuides({ at: [0, 0, 0], size: [0.12, 0.2, 0.03] }) as SlabGuide[])[1].hands).toBe(1);
  });
  test("a handle and a stem come first, a ball is a sphere, a thing's own guides replace them all", () => {
    const mug: Thing = { at: [0, 0, 0], size: [0.085, 0.1, 0.085], shape: "round", grips: [{ at: [0.06, 0.05, 0], along: [0, 1, 0] }] };
    expect(autoGuides(mug).map((g) => g.tags?.[0])).toEqual(["handle", "body"]);
    expect(pickGuide(mug, "handle")?.tags).toContain("handle");
    expect(pickGuide(mug, "wrap")?.tags).toContain("body");
    expect(pickGuide(mug, "wrap", "handle")?.tags).toContain("handle");
    const glass: Thing = { at: [0, 0, 0], size: [0.08, 0.2, 0.08], shape: "round", grips: [{ at: [0, 0.05, 0], kind: "stem" }] };
    const gs = autoGuides(glass);
    expect(gs[0].kind).toBe("pinch");
    expect((gs[1] as CylinderGuide).length).toBeCloseTo(0.1, 9);
    expect(autoGuides({ at: [0, 0, 0], size: [0.067, 0.067, 0.067], shape: "ball" })[0].kind).toBe("sphere");
    const own: Thing = { ...can, guides: [{ kind: "pinch", at: [0, 0.122, 0], across: [1, 0, 0], width: 0.066, tags: ["rim"] }] };
    expect(guidesOf(own).length).toBe(1);
    expect(pickGuide(own, "wrap", "rim")?.kind).toBe("pinch");
  });
});

describe("a hand on a guide", () => {
  const g = autoGuides(can)[0] as CylinderGuide;
  test("from the side: the palm on the approach's side facing the axis, the fingers round it, the thumb opposite them", () => {
    for (const side of ["L", "R"] as const) {
      const on = handOn(g, side, [side === "L" ? 1 : -1, 0, -0.35], { hand: 0.126 });
      const toPalm = norm(sub(on.contacts.palm, g.at));
      expect(dot(on.facing, toPalm)).toBeLessThan(-0.99);
      // (the axis diagonal across the palm: the fingers neither along it nor square to it)
      expect(Math.abs(dot(on.point, g.axis))).toBeGreaterThan(0.5);
      expect(Math.abs(dot(on.point, g.axis))).toBeLessThan(0.9);
      // the fingers' contact and the thumb's on opposite sides of the palm's
      const flat = (v: Vec3) => norm(sub(sub(v, g.at), [0, dot(sub(v, g.at), [0, 1, 0]), 0]));
      const cross = (u: Vec3, v: Vec3) => u[2] * v[0] - u[0] * v[2];
      expect(Math.sign(cross(toPalm, flat(on.contacts.fingers)))).toBe(-Math.sign(cross(toPalm, flat(on.contacts.thumb))));
    }
  });
  test("from above: the palm on its end facing down, the fingers across it", () => {
    const on = handOn(g, "R", [0, 1, -0.15], { hand: 0.126 });
    close(on.facing, [0, -1, 0]);
    expect(Math.abs(dot(on.point, [0, 1, 0]))).toBeLessThan(1e-9);
    expect(on.contacts.palm[1]).toBeCloseTo(0.122, 9);
  });
});

describe("holding against gravity", () => {
  // contacts round a can's side (radius 3.3 cm, its middle at the origin), pushing in
  const at = (deg: number, y = 0): Contact => { const a = (deg * Math.PI) / 180, p: Vec3 = [0.033 * Math.cos(a), y, 0.033 * Math.sin(a)]; return { part: `${deg}`, at: p, push: norm([-p[0], 0, -p[2]]) }; };
  test("opposed contacts hold it; contacts on one side don't, however many", () => {
    expect(holds([at(0), at(180)], [0, 0, 0])).toBe(true);
    expect(holds([at(0, 0.02), at(0, -0.02), at(150, 0.02), at(150, -0.02), at(210, 0)], [0, 0, 0])).toBe(true);
    expect(holds([at(0), at(30), at(60), at(90)], [0, 0, 0])).toBe(false);
    expect(holds([], [0, 0, 0])).toBe(false);
    // (a thing standing on a palm turned up holds by itself)
    expect(holds([{ part: "palm", at: [0, -0.061, 0], push: [0, 1, 0] }], [0, 0, 0])).toBe(true);
  });
  test("the kid's can, carried on its guide, is held from opposed sides", () => {
    const { skel, lib } = parkour("kid"), sc = SCENES.find((s) => s.name === "can")!;
    const r = gripRun("kid", sc, lib), live = interact(skel, perform(lib, r.plan.spec, { quiet: true }), r.plan), sdf = thingSdf(r.plan.items.it);
    let n = 0, held = 0, opposed = 0;
    for (let f = 0; f < live.frames; f += 8) {
      if (!live.held("it", f)) continue;
      const t = live.item("it", f), field = (q: Vec3) => sdf(qrot(qinv(t.rot), sub(q, t.at)));
      const cs = contactsOf(skel, pose(skel, live.pose(f)), field), mid = add(t.at, qrot(t.rot, [0, 0.061, 0]));
      n++;
      if (holds(cs, mid)) held++;
      // (some two of its contacts push against each other: across it, not side by side)
      if (cs.some((a) => cs.some((b) => dot(a.push, b.push) < -0.5))) opposed++;
    }
    expect(n).toBeGreaterThan(20);
    expect(held / n).toBeGreaterThan(0.9);
    expect(opposed / n).toBeGreaterThan(0.9);
  });
});
