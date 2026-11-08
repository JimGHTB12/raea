// <Camera3d order clear_color viewport> (Bevy's Camera order, ClearColorConfig and Viewport, docs/control.feature FC0a:
// 3d/two_passes, 3d/split_screen): what the camera row takes from them.
import { expect, test } from "bun:test";
import type { ReactElement } from "react";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { LinearRgba, Srgba } from "raea/color";

type El = ReactElement<Record<string, unknown>>;
/** the props of the camera node a <Camera3d> makes */
function row(props: Parameters<typeof Camera3d>[0]): Record<string, unknown> {
  const out = Camera3d(props) as El;
  const kids = ([] as unknown[]).concat((out.props as { children?: unknown }).children);
  const cam = kids.find((k) => (k as El | null)?.props && "lookAt" in ((k as El).props as object)) as El;
  return cam.props;
}
const at = Transform.fromXyz(0, 0, 5);

test("order and clear_color: None draws over without clearing, Custom clears to its colour, Default as before", () => {
  expect(row({ transform: at })).not.toHaveProperty("order");
  expect(row({ transform: at, clear_color: "Default" })).not.toHaveProperty("clear");
  const second = row({ transform: at, order: 1, clear_color: "None" });
  expect([second.order, second.clear, second.bg]).toEqual([1, "none", undefined]);
  expect(row({ transform: at, clear_color: { Custom: new Srgba(1, 0, 0, 1) } }).bg).toBe("#FF0000");
  expect(row({ transform: at, clear_color: { Custom: new LinearRgba(0, 0, 1, 0.5) } }).bg).toBe("#0000FF80");
});

test("refuses a fractional order and an unknown clear_color", () => {
  expect(() => row({ transform: at, order: 0.5 })).toThrow(/whole number/);
  expect(() => row({ transform: at, clear_color: "Black" as never })).toThrow(/clear_color/);
});

test("viewport (3d/split_screen): the row carries it as given; refuses fractions, negatives and a zero size", () => {
  const vp = { physical_position: [640, 360] as [number, number], physical_size: [640, 360] as [number, number] };
  expect(row({ transform: at, viewport: vp }).viewport).toEqual(vp);
  expect(row({ transform: at })).not.toHaveProperty("viewport");
  for (const bad of [{ physical_position: [0.5, 0], physical_size: [1, 1] }, { physical_position: [-1, 0], physical_size: [1, 1] }, { physical_position: [0, 0], physical_size: [0, 1] }])
    expect(() => row({ transform: at, viewport: bad as never })).toThrow(/viewport/);
});
