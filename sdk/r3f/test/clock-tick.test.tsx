// The Clock's state is one object across frames, its frame moved on in place, so a frame's change doesn't change the
// context React would look through every part for (docs/control.feature FC1b, many_cubes): what reads the frame
// subscribes instead. A reader its parent doesn't render again still moves on with the frame; one its parent does
// render renders once a frame, not twice; and a host stepping frames back to back calls tickClocks in the frame's own
// update (runtime/bundle entry.rs). Rendered through a reconciler frame by frame, as a bake steps them (R3F's here).
import { describe, expect, test } from "bun:test";
import { memo } from "react";
import { Clock, tickClocks, useCurrentFrame } from "@raea/app/clock";
import { end, stepTo } from "../src/input/session";

let renders: string[] = [];
function Reader({ name }: { name: string }) {
  renders.push(`${name}@${useCurrentFrame()}`);
  return null;
}
// a part whose parent never renders it again: only the clock moves it on
const Still = memo(function Still() {
  renders.push("still");
  return <Reader name="deep" />;
});

function run(key: string, tick: boolean): string[] {
  renders = [];
  stepTo(key, 3, 30, 64, 64, {}, (i) => {
    if (tick) tickClocks();
    return (
      <Clock frame={i} fps={30} width={64} height={64} durationInFrames={100}>
        <Reader name="top" />
        <Still />
      </Clock>
    );
  });
  end(key);
  return renders;
}

const FRAMES = ["top@0", "still", "deep@0", "top@1", "deep@1", "top@2", "deep@2", "top@3", "deep@3"];

describe("the Clock's frame readers", () => {
  test("moved on as its frame commits, the memoized one too, each once a frame", () => {
    expect(run("tick-commit", false)).toEqual(FRAMES);
  });
  test("told in the frame's own update by tickClocks: the same, once each", () => {
    expect(run("tick-ahead", true)).toEqual(FRAMES);
  });
});
