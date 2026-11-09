---
title: "raea/input"
summary: "The frame's input: keys, pointer, wheel and gamepads, the same live and in a scripted render."
order: 26
sources: ["sdk/raea/src/input.ts"]
---

`useInput()` returns this frame's input: the keys held, the pointer, the wheel and Bevy's gamepads. Live, it comes
from the window; in a render, from the input timeline `raea render --input` reads, so a recorded play renders
exactly. `keyEdges` tells which keys went down or up this frame.

```tsx
import { useInput } from "raea/input";

function Ship() {
  const { keys } = useInput();
  const dx = (keys.includes("ArrowRight") ? 1 : 0) - (keys.includes("ArrowLeft") ? 1 : 0);
  return <div style={{ transform: `translateX(${dx * 40}px)` }}>▲</div>;
}
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { keyEdges, useInput, wireKeys } from "raea/input"
```

`raea/input` exports 1 hook, 2 functions, 2 types. Bevy's counterpart: `bevy_input, bevy_input_focus`. Source: [sdk/raea/src/input.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/input.ts).

## Hooks

### `useInput`

This frame's input ({keys: []} when there was none). Its `gamepads` are Bevy's (bevy_gilrs's Gamepad entities, read
by engine/render's gamepad_keys):

Bevy: `bevy_input::gamepad::Gamepad`.

```ts
function useInput(): FrameInput
```

## Functions

### `keyEdges`

The input edges between two frames' keys, as events for a machine's `on`: "keydown:K" for each key pressed this
frame, "keyup:K" for each released (keep last frame's keys in the game state to have `prev`).

```ts
function keyEdges(prev: readonly string[] | undefined, now: readonly string[] | undefined): string[]
```

### `wireKeys`

A live host's input beyond keys and the pointer, which reaches the page as marked keys, made fields:
"Mouse:Right" a button held; "Wheel:Y DY" (and "Wheel:X DX") the wheel's turn this frame; "Pad0:South" a
gamepad's button, "Pad0:LeftStickX=0.42" its axis; "Touch:ID:X:Y" a finger. A scripted entry (render --input) can
give the fields themselves.

```ts
function wireKeys(input: FrameInput): FrameInput
```

## Types

### `FrameInput`

One frame's input for a game.

| Field | Type | What it is |
| --- | --- | --- |
| `keys` (required) | `string[]` | the keys held (KeyboardEvent.key names: "ArrowLeft", "a", " ") |
| `pointer` | `{ x: number; y: number; down: boolean }` | the primary pointer in composition px, and whether it's pressed |
| `buttons` | `string[]` | the mouse buttons held besides the primary one: "Right", "Middle", "Back", "Forward" (Bevy's MouseButton names) |
| `wheel` | `{ x: number; y: number }` | the wheel turned this frame, in CSS px, down and right positive (the DOM's deltaX and deltaY) |
| `gamepads` | `{ buttons: string[]; axes: Record<string, number> }[]` | each connected gamepad: the buttons held and the sticks' and triggers' axes, by Bevy's GamepadButton and GamepadAxis names ("South", "DPadUp", "RightTrigger2"; "LeftStickX" -1..1, "LeftZ") |
| `touches` | `{ id: number; x: number; y: number }[]` | the fingers down, in composition px |

### `InputTimeline`

Recorded input by composition frame; frames without an entry had no input.

```ts
type InputTimeline = Record<number, Partial<FrameInput> | undefined>
```

<!-- generated:end -->
