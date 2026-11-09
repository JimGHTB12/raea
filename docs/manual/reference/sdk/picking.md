---
title: "raea/picking"
summary: "What the pointer is over: the nearest part under it, and the ray it casts."
order: 69
sources: ["sdk/raea/src/picking.ts"]
---

`pick` finds the nearest part under the pointer, given the camera and the parts, and `pointerRay` gives the ray the
pointer casts into the scene. Both are pure functions, so a game's step function can use them and still replay
exactly.

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { pick, pointerRay } from "raea/picking"
```

`raea/picking` exports 2 functions, 3 types. Bevy's counterpart: `bevy_picking`. Source: [sdk/raea/src/picking.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/picking.ts).

## Functions

### `pick`

The nearest part under the pointer, or null. `size` is the frame's (useVideoConfig's width and height), whose
pixels the pointer is in.

```ts
function pick(pointer: { x: number; y: number; }, camera: PickCamera, parts: readonly Pickable[], size: { width: number; height: number; }): Hit | null
```

### `pointerRay`

The pointer's ray: from the camera through the pointer (x, y in the frame's pixels, y down), as the engine draws it
(Bevy's looking_at, with +Z for up when the camera looks straight up or down, as rdb does).

```ts
function pointerRay(pointer: { x: number; y: number; }, camera: PickCamera, size: { width: number; height: number; }): { origin: Vec3; direction: Vec3; }
```

## Types

### `Hit`

| Field | Type | What it is |
| --- | --- | --- |
| `name` (required) | `string` | – |
| `distance` (required) | `number` | metres from the camera (from its near plane's point, for an orthographic one) along the ray |
| `point` (required) | `Vec3` | where the ray first meets it, in world space |

### `Pickable`

Something the pointer can hit: an axis-aligned box or a sphere, in world space.

```ts
type Pickable = { name: string; box: { min: Vec3; max: Vec3 } } | { name: string; sphere: { center: Vec3; radius: number } }
```

### `PickCamera`

The camera as a scene sets it: a PerspectiveCamera's props (fov in degrees, vertical) or an OrthographicCamera's.

| Field | Type | What it is |
| --- | --- | --- |
| `position` | `Vec3` | – |
| `lookAt` | `Vec3` | – |
| `fov` | `number` | vertical field of view in degrees (50 by default); ignored when `height` is set |
| `height` | `number` | an orthographic camera's view height in metres |
| `up` | `Vec3` | – |

<!-- generated:end -->
