// <Gizmo>'s colours as runtime/plugins fields.rs gizmo_asset reads them (docs/control.feature FC0a: mesh_ray_cast's
// laser is HDR, Color::srgb(30, 0, 0); align's arrows srgb(0.65, 0, 0)): a hex as it is, Bevy's LinearRgba as linear
// [r, g, b, a] above 1 kept, Srgba linearised as bevy_color does, and a list one per point.
import { expect, test } from "bun:test";
import type { ReactElement } from "react";
import { Gizmo } from "raea/gizmos";
import { LinearRgba, Srgba } from "raea/color";

type El = ReactElement<{ children: ReactElement<{ value: { handle: Record<string, unknown> } }> }>;
const handle = (el: unknown) => (el as El).props.children.props.value.handle;
const line: [[number, number, number], [number, number, number]][] = [[[0, 0, 0], [1, 0, 0]]];
const near = (a: unknown, b: number[]) => (a as number[]).every((x, i) => Math.abs(x - b[i]!) < 1e-6) && (a as number[]).length === b.length;

test("one colour: hex as written, LinearRgba as linear (HDR kept), Srgba linearised, [r, g, b, a] as given", () => {
  expect(handle(Gizmo({ lines: line, color: "#ff0000" })).color).toBe("#ff0000");
  expect(handle(Gizmo({ lines: line, color: new LinearRgba(30, 0, 0, 1) })).color).toEqual([30, 0, 0, 1]);
  // Srgba(0.65, 0, 0): 0.65 is 0.380056 linear (((0.65 + 0.055) / 1.055) ^ 2.4)
  expect(near(handle(Gizmo({ lines: line, color: new Srgba(0.65, 0, 0, 1) })).color, [0.380056, 0, 0, 1])).toBe(true);
  // a dark channel is linear below 0.04045: 0.03 / 12.92
  expect(near(handle(Gizmo({ lines: line, color: new Srgba(0.03, 0, 0, 1) })).color, [0.002322, 0, 0, 1])).toBe(true);
  expect(handle(Gizmo({ lines: line, color: [0.5, 0.25, 0, 1] })).color).toEqual([0.5, 0.25, 0, 1]);
});

test("a list is one colour per point, each read the same way", () => {
  const h = handle(Gizmo({ lines: line, strip: [[0, 1, 0]], color: ["#00ff00", new LinearRgba(2, 2, 2, 1), [0, 0, 1, 1]] }));
  expect(h.list_colors).toEqual(["#00ff00", [2, 2, 2, 1]]);
  expect(h.strip_colors).toEqual([[0, 0, 1, 1]]);
});

test("a colour the engine can't read is refused here", () => {
  expect(() => Gizmo({ lines: line, color: "rgb(1, 2, 3)" })).toThrow(/#rrggbb/);
  expect(() => Gizmo({ lines: line, color: [1, 0, 0] as unknown as [number, number, number, number] })).toThrow(/\[r, g, b, a\]/);
  expect(() => Gizmo({ lines: line, color: ["#ffffff"] })).toThrow(/one colour per point/);
});
