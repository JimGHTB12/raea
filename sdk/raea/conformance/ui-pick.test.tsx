// A bevy_ui node's picking handlers and bevy_ui_widgets' Popover (docs/control.feature FC0a, FC3c: ui/widgets/
// standard_widgets' menu): what the node's element carries for the page.
import { describe, expect, test } from "bun:test";
import { Node, px } from "raea/ui";
import { renderFrame } from "../src/test";

const html = (el: React.ReactElement) => renderFrame(el, { frame: 0, width: 320, height: 240, fps: 60, durationInFrames: 1 });

describe("Popover", () => {
  test("carries its window margin and placements, defaults filled, and is absolute", () => {
    const out = html(<Node><Node width={px(10)} popover={{ positions: [{ side: "Bottom", align: "Start", gap: 2 }, { side: "Top" }], window_margin: 10 }} /></Node>);
    expect(out).toContain('data-raea-bevy-popover="10;Bottom Start 2;Top Start 0"');
    expect(out).toMatch(/position:\s*absolute[^"]*"[^>]*data-raea-bevy-popover/);
  });
  test("refuses no placements and an unknown side", () => {
    expect(() => html(<Node popover={{ positions: [] }} />)).toThrow(/popover/);
    expect(() => html(<Node popover={{ positions: [{ side: "Middle" as never }] }} />)).toThrow(/positions\[0\]/);
  });
});

describe("picking handlers", () => {
  test("refuses a handler that isn't a function", () => {
    expect(() => html(<Node onClick={"go" as never} />)).toThrow(/onClick is a function/);
  });
});
