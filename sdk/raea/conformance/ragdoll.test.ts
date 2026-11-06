// raea/ragdoll under plain bun: a body going limp mid-run falls onto the ground or a box, keeps its joints in
// their limits, never sinks through, comes to rest and stays there, and is the same whichever frame is asked first.
// The skeleton is the stick stand-in (stick.ts, game_engine joints) with no skin samples (the ragdoll rings its bones instead).
import { describe, expect, test } from "bun:test";
import { rest, type Pose } from "raea/character";
import { ragdoll } from "raea/ragdoll";
import type { Vec3 } from "../src/3d";
import { skel } from "./stick";

// running along +x at 2.5 m/s, stood straight (the stand-in's feet just off the floor)
const still = rest(skel);
const run = (f: number): Pose => ({ ...still, root: [2.5 * (f / 30), 0.72, 0] });
const FROM = 30;

/** how far each joint is past its limits (radians) */
const over = (js: ReturnType<ReturnType<typeof ragdoll>["joints"]>) =>
  Math.max(0, ...js.map((j) => Math.max(j.lim.x[0] - j.x, j.x - j.lim.x[1], j.lim.z[0] - j.z, j.z - j.lim.z[1], j.lim.y[0] - j.twist, j.twist - j.lim.y[1])));

describe("ragdoll", () => {
  const fall = ragdoll(skel, run, FROM, { push: [0.5, 0, 0] });
  const restAt = fall.restsAt();

  test("is the motion before it goes limp", () => {
    expect(fall(FROM - 1)).toEqual(run(FROM - 1));
  });
  test("falls, carrying the run on, and comes to rest lying down", () => {
    expect(restAt).toBeGreaterThan(FROM);
    expect(restAt).toBeLessThan(FROM + 5 * 30);
    const end = fall(restAt);
    expect(end.root[1]).toBeLessThan(0.3);
    expect(end.root[0]).toBeGreaterThan(run(FROM).root[0] + 0.3);
  });
  test("stays exactly still once at rest", () => {
    expect(fall(restAt + 1)).toBe(fall(restAt));
    expect(fall(restAt + 200)).toBe(fall(restAt));
  });
  test("never goes through the ground, and keeps its joints in their limits", () => {
    for (let f = FROM; f <= restAt; f++) {
      expect(fall.lowest(f)).toBeGreaterThan(-0.005);
      // (the first frames may still be pulling a pose from the motion back inside them)
      if (f > FROM + 2) expect(over(fall.joints(f))).toBeLessThan(0.05);
    }
  });
  test("is the same whichever frame is asked first", () => {
    const again = ragdoll(skel, (f) => run(f), FROM, { push: [0.5, 0, 0] });
    expect(again(restAt - 5)).toEqual(fall(restAt - 5));
    expect(again(FROM + 7)).toEqual(fall(FROM + 7));
  });
  test("lands on a box in its way", () => {
    const top = 0.25, box = { min: [0, 0, -1] as Vec3, max: [6, top, 1] as Vec3 };
    const onBox = ragdoll(skel, (f) => ({ ...run(f), root: [run(f).root[0], run(f).root[1] + top, 0] }), FROM, { solids: [box] });
    const r = onBox.restsAt();
    for (let f = FROM; f <= r; f++) expect(onBox.lowest(f)).toBeGreaterThan(top - 0.005);
    expect(onBox(r).root[1]).toBeLessThan(top + 0.3);
  });
});
