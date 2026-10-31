// A live host's input beyond keys and the pointer (other mouse buttons, the wheel, gamepads, touches) reaches a game's
// useInput() as fields: docs/bevy-ts.feature G3.
import { expect, test } from "bun:test";
import { wireKeys } from "raea/games";

test("plain keys pass through untouched", () => {
  const i = { keys: ["ArrowLeft", "a"] };
  expect(wireKeys(i)).toEqual({ keys: ["ArrowLeft", "a"] });
});

test("marked keys become buttons, wheel, gamepads and touches", () => {
  const i = wireKeys({ keys: ["a", "Mouse:Right", "Wheel:Y 120", "Wheel:X -40", "Pad0:South", "Pad0:LeftStickX=0.5", "Pad1:Start", "Touch:3:10:20", ":"] });
  expect(i.keys).toEqual(["a", ":"]);
  expect(i.buttons).toEqual(["Right"]);
  expect(i.wheel).toEqual({ x: -40, y: 120 });
  expect(i.gamepads).toEqual([{ buttons: ["South"], axes: { LeftStickX: 0.5 } }, { buttons: ["Start"], axes: {} }]);
  expect(i.touches).toEqual([{ id: 3, x: 10, y: 20 }]);
});

test("a scripted entry's own fields are kept", () => {
  const i = wireKeys({ keys: ["Mouse:Middle"], wheel: { x: 0, y: 10 }, gamepads: [{ buttons: ["East"], axes: {} }] });
  expect(i.buttons).toEqual(["Middle"]);
  expect(i.wheel).toEqual({ x: 0, y: 10 });
  expect(i.gamepads?.[0].buttons).toEqual(["East"]);
});
