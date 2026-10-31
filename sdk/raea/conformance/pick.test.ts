// pick(): the pointer's ray from a scene's camera, and the nearest part it meets (docs/bevy-ts.feature G4).
import { describe, expect, test } from "bun:test";
import { pick, pointerRay } from "@raea/render/pick";

const SIZE = { width: 1920, height: 1080 };
const CAM = { position: [0, 0, 10] as [number, number, number], lookAt: [0, 0, 0] as [number, number, number], fov: 40 };

describe("pick", () => {
  test("the middle of the frame looks down the camera's axis", () => {
    const r = pointerRay({ x: 960, y: 540 }, CAM, SIZE);
    expect(r.direction[2]).toBeCloseTo(-1, 6);
  });
  test("the top edge is half the vertical field of view up", () => {
    const r = pointerRay({ x: 960, y: 0 }, CAM, SIZE);
    expect(Math.atan2(r.direction[1], -r.direction[2]) * (180 / Math.PI)).toBeCloseTo(20, 4);
  });
  test("the nearest of two boxes along the ray wins, with its distance and point", () => {
    const parts = [
      { name: "far", box: { min: [-1, -1, -6], max: [1, 1, -4] } as { min: [number, number, number]; max: [number, number, number] } },
      { name: "near", box: { min: [-1, -1, -1], max: [1, 1, 1] } as { min: [number, number, number]; max: [number, number, number] } },
    ];
    const hit = pick({ x: 960, y: 540 }, CAM, parts, SIZE);
    expect(hit?.name).toBe("near");
    expect(hit?.distance).toBeCloseTo(9, 6);
    expect(hit?.point[2]).toBeCloseTo(1, 6);
  });
  test("a sphere off to the right is hit only where it is on screen", () => {
    const parts = [{ name: "ball", sphere: { center: [3, 0, 0] as [number, number, number], radius: 0.5 } }];
    // x = 3 at 10 m: tan(atan(3/10)) / (tan(20 deg) * 16/9) of the half-width right of the middle
    const x = 960 + 960 * (0.3 / (Math.tan((20 * Math.PI) / 180) * (16 / 9)));
    expect(pick({ x, y: 540 }, CAM, parts, SIZE)?.name).toBe("ball");
    expect(pick({ x: 960, y: 540 }, CAM, parts, SIZE)).toBeNull();
  });
  test("an orthographic camera's rays are parallel, spread over its height", () => {
    const ortho = { position: [0, 0, 10] as [number, number, number], lookAt: [0, 0, 0] as [number, number, number], height: 4 };
    const r = pointerRay({ x: 960, y: 0 }, ortho, SIZE);
    expect(r.origin[1]).toBeCloseTo(2, 6);
    expect(r.direction[2]).toBeCloseTo(-1, 6);
  });
  test("a camera looking straight down still has a right and an up (+Z up, as the engine)", () => {
    const r = pointerRay({ x: 1920, y: 540 }, { position: [0, 10, 0], lookAt: [0, 0, 0], fov: 40 }, SIZE);
    expect(Number.isFinite(r.direction[0]) && Math.abs(r.direction[0]) > 0.1).toBe(true);
  });
});
