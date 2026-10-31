// clipEvents(): a clip's events fired between two frames' clip times (docs/bevy-ts.feature G8b).
import { describe, expect, test } from "bun:test";
import { clipEvents } from "../src/3d";

const steps = [{ time: 0, name: "left" }, { time: 0.5, name: "right" }];
const names = (xs: { name: string }[]) => xs.map((x) => x.name);

describe("clipEvents", () => {
  test("an event fires on the frame whose time first reaches it", () => {
    expect(names(clipEvents(steps, 0.4, 0.5, 1))).toEqual(["right"]);
    expect(names(clipEvents(steps, 0.5, 0.6, 1))).toEqual([]);
  });
  test("the first frame (prev -1) fires an event at 0", () => {
    expect(names(clipEvents(steps, -1, 0, 1))).toEqual(["left"]);
  });
  test("a looping clip fires each event once per loop, even across a long step", () => {
    expect(names(clipEvents(steps, 0.9, 1.1, 1))).toEqual(["left"]);
    expect(names(clipEvents(steps, 0.2, 2.2, 1))).toEqual(["right", "left", "right", "left"]);
  });
  test("a clip that doesn't loop stops at its end", () => {
    expect(names(clipEvents(steps, 0.4, 3, 1, false))).toEqual(["right"]);
    expect(names(clipEvents(steps, 1, 3, 1, false))).toEqual([]);
  });
  test("nothing fires going backwards or standing still", () => {
    expect(clipEvents(steps, 0.6, 0.6, 1)).toEqual([]);
    expect(clipEvents(steps, 0.6, 0.1, 1)).toEqual([]);
  });
});
