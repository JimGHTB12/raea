---
title: "raea/gizmos"
summary: "Lines drawn for looking: Bevy's gizmos and debug overlays."
order: 70
sources: ["sdk/raea/src/gizmos.ts"]
---

`<Gizmo>` draws debug lines with Bevy's gizmo renderer: a path, a box, an axis, a ray. They take and cast no light.
`<Overlay>` and `overlays` draw the debug overlays raea's checks use, such as safe areas and element boxes, over a
frame you're looking at.

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { Gizmo, Overlay, overlays } from "raea/gizmos"
```

`raea/gizmos` exports 2 components, 1 function, 3 types. Bevy's counterpart: `bevy_gizmos`. Source: [sdk/raea/src/gizmos.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/gizmos.ts).

## Components

### `Gizmo`

Debug lines drawn by Bevy's gizmo renderer (a retained bevy_gizmos Gizmo and its GizmoAsset), placed like a part:
a path, a bounding box, an axis, a ray. They cast and take no light, and draw over nothing unless depthBias says.

Bevy: `bevy_gizmos::retained::Gizmo`.

Takes [`GizmoProps`](3d.md#gizmoprops)'s props.

Used in: [3d/mesh_ray_cast](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/mesh_ray_cast.tsx), [3d/rect_light](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/3d/rect_light.tsx), [transforms/align](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/transforms/align.tsx).

### `Overlay`

Its children only in a render asked for the overlay `name`.

| Prop | Type | What it does |
| --- | --- | --- |
| `name` (required) | `string` | the overlay's name, as the render asks for it |
| `children` | `ReactNode` | what the overlay draws |

## Functions

### `overlays`

The overlays this render was asked for ([] in a clean render and in live play).

```ts
function overlays(): string[]
```

## Types

### `GizmoColor`

a gizmo point's colour: "#rrggbb", linear [r, g, b, a], or Bevy's LinearRgba / Srgba (bevy3d's colours)

```ts
type GizmoColor = string | [number, number, number, number] | { red: number; green: number; blue: number; alpha?: number }
```

### `GizmoProps`

| Field | Type | What it is |
| --- | --- | --- |
| `lines` | `[Vec3, Vec3][]` | segments, each [from, to] |
| `strip` | `(Vec3 \| null)[]` | one line through these points (null breaks it) |
| `color` | `GizmoColor \| GizmoColor[]` | one colour for every point, or one per point of `lines` (two per segment) then `strip`: "#rrggbb", Bevy's LinearRgba (above 1 for HDR, as bevy_gizmos takes it) or Srgba, or linear [r, g, b, a] |
| `width` | `number` | line width in pixels (Bevy's default 2) |
| `depthBias` | `number` | -1 always in front of geometry, 0 depth-tested as it is (Bevy's GizmoLineConfig and depth_bias) |
| `position` | `Vec3` | where it stands, metres (x, y, z) |
| `rotation` | `Vec3` | its rotation, Euler angles in radians (x, y, z) |
| `scale` | `Vec3 \| number` | its scale: one number, or x, y and z |
| `name` | `string` | the entity's name, for picking, the inspector and world.* |

### `OverlayProps`

The props of [`Overlay`](#overlay), tabled there.

<!-- generated:end -->
