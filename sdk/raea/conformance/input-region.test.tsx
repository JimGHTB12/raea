// InputRegion: panels sharing one page each hear the pointer in their own box, as a page of their own would (the
// phone's compact shell, docs/platform.feature: The iOS app is raea).
import { describe, expect, test } from "bun:test";
import { InputRegion, useGameState } from "../src";
import type { FrameInput } from "../src";
import { renderFrame } from "../src/test";

type Seen = { at: { x: number; y: number; down: boolean } | null; presses: number; down: boolean };
const seen = (s: Seen, input: FrameInput): Seen => {
  const p = input.pointer ?? null;
  return { at: p, presses: s.presses + (p?.down && !s.down ? 1 : 0), down: !!p?.down };
};
const got: Record<string, Seen> = {};
function Panel({ name }: { name: string }) {
  got[name] = useGameState<Seen>({ at: null, presses: 0, down: false }, seen);
  return null;
}
const page = (
  <>
    <InputRegion x={0} y={0} w={100} h={100} scale={2}><Panel name="top" /></InputRegion>
    <InputRegion x={0} y={100} w={100} h={100} scale={2}><Panel name="bottom" /></InputRegion>
  </>
);
const base = { fps: 30, width: 200, height: 400, durationInFrames: 60 };
const at = (frame: number, input: Record<number, Partial<FrameInput>>) => renderFrame(page, { ...base, frame, input });

describe("InputRegion", () => {
  test("the pointer in a box's own points, and only that box hears it", () => {
    // page pixels at 2 per point: (60, 300) is (30, 150) in points, the bottom box's (30, 50)
    at(2, { 0: { pointer: { x: 60, y: 300, down: true } }, 1: { pointer: { x: 60, y: 300, down: true } } });
    expect(got.bottom.at).toEqual({ x: 30, y: 50, down: true });
    expect(got.bottom.presses).toBe(1);
    expect(got.top.at).toBeNull();
    expect(got.top.presses).toBe(0);
  });
  test("a press stays the box's until it's let go, wherever the finger goes", () => {
    // pressed in the top box, dragged down into the bottom one, then let go
    const drag = { 0: { pointer: { x: 20, y: 20, down: true } }, 1: { pointer: { x: 20, y: 300, down: true } }, 2: { pointer: { x: 20, y: 300, down: false } } };
    at(2, drag);
    expect(got.top.at).toEqual({ x: 10, y: 150, down: true });
    expect(got.bottom.at).toBeNull();
    expect(got.bottom.presses).toBe(0);
    at(3, drag);
    // let go over the bottom box: the top box has no press now, and the bottom one hovers
    expect(got.top.at).toBeNull();
    expect(got.bottom.at).toEqual({ x: 10, y: 50, down: false });
  });
});
