---
title: "raea/math"
summary: "Numbers through time: interpolation, easing, springs, curves and the vector types Bevy uses."
order: 24
sources: ["sdk/raea/src/math.ts"]
---

`interpolate` maps a frame onto a range of values, with an easing and with clamping at either end. `spring` gives a
physically damped value for a frame, and `measureSpring` says how long one takes to settle. `Easing` holds the
standard curves; `curveFn` turns a declared curve into a function. The quaternion and affine helpers match Bevy's
math types.

```ts
import { Easing, interpolate, spring } from "raea/math";

const x = interpolate(30, [0, 45], [-200, 0], { easing: Easing.out(Easing.cubic), extrapolateRight: "clamp" });
const scale = spring({ frame: 30, fps: 30, config: { damping: 12 } });
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { Affine2, curve, curveFn, Easing } from "raea/math"
```

`raea/math` exports 7 functions, 3 values, 7 types. Bevy's counterpart: `bevy_math, bevy_curve`. Source: [sdk/raea/src/math.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/math.ts).

## Functions

### `curveFn`

A curve name as a function (the engine's easing::Easing::parse, the same names).

```ts
function curveFn(name: string, fps?: number): (t: number) => number
```

### `hash01`

0..1 from integers, the same on every run

```ts
function hash01(...xs: number[]): number
```

### `interpolate`

`x` mapped from the `input` range onto the `output` range, piece by piece: `input` rises, and the two have the same
length. `options.easing` shapes each piece; `extrapolateLeft` and `extrapolateRight` say what happens outside the
range (`extend`, the default, carries on; `clamp` holds the end value; `identity` returns `x`; `wrap` repeats).

```ts
function interpolate(x: number, input: readonly number[], output: readonly number[], options?: InterpolateOptions): number
```

### `measureSpring`

How many frames a spring with these settings takes to settle (before any stretching).

```ts
function measureSpring(opts: { fps: number; config?: SpringConfig; durationRestThreshold?: number; }): number
```

### `quatAxis`

The quaternion [x, y, z, w] turning `angle` radians about `axis`, as Bevy's Quat::from_axis_angle.

```ts
function quatAxis(axis: Vec3, angle: number): Quat
```

### `quatMul`

The product a × b of two quaternions [x, y, z, w], `b` applied first, as Bevy's Quat * Quat.

```ts
function quatMul(a: Quat, b: Quat): Quat
```

### `spring`

A damped spring's value at `frame`: from `from` (0) to `to` (1), started `delay` frames late, by its `config`
(mass 1, damping 10, stiffness 100). It overshoots and settles unless `overshootClamping` is set; `durationInFrames`
stretches it to end at that frame. `measureSpring` says how long one takes to settle.

```ts
function spring(opts: SpringOptions): number
```

## Values

### `Affine2`

glam's Affine2 as its parts: scaled, then turned `angle` radians, then moved by `translation` (from_scale_angle_translation).
glam's Affine2 constructors, by their Rust names in camelCase.

Used in: [2d/mesh2d_repeated_texture](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/bench/bevy-parity/2d/mesh2d_repeated_texture.tsx).

### `curve`

Curve helpers: `curve.bezier(0.05, 0.7, 0.1, 1)`, `curve.spring({ damping: 15, stiffness: 120 })`.

### `Easing`

The easing curves, each a function of progress 0..1: `linear`, `quad`, `cubic`, `poly(n)`, `sin`, `circle`, `exp`,
`back`, `elastic`, `bounce` and `bezier(x1, y1, x2, y2)`, eased in as they are; `Easing.out(f)` and `Easing.inOut(f)`
turn one round. Pass one as `interpolate`'s `easing`.

## Types

### `Curve`

```ts
type Curve = CurveName | (string & {})
```

### `CurveName`

Curve names. Any of these, or `bezier(x1,y1,x2,y2)`, `steps(n)`, `spring(mass,damping,stiffness)`,
`reverse(<curve>)`.

```ts
type CurveName = | "linear" | "ease" | `${"in" | "out" | "in-out"}-${"quad" | "cubic" | "quart" | "quint" | "sine" | "expo" | "circ" | "back" | "elastic" | "bounce"}`
```

### `EasingFn`

```ts
type EasingFn = (p: number) => number
```

### `ExtrapolateMode`

```ts
type ExtrapolateMode = "extend" | "clamp" | "identity" | "wrap"
```

### `InterpolateOptions`

| Field | Type | What it is |
| --- | --- | --- |
| `easing` | `EasingFn \| EasingFn[]` | the curve between keyframes: one for all, or one a segment |
| `extrapolateLeft` | `ExtrapolateMode` | below the first keyframe: extend the line (default), clamp to it, identity (the input itself) or wrap |
| `extrapolateRight` | `ExtrapolateMode` | past the last keyframe: extend (default), clamp, identity or wrap |

### `SpringConfig`

| Field | Type | What it is |
| --- | --- | --- |
| `mass` | `number` | how heavy the spring's weight is (1): heavier is slower and swings further |
| `damping` | `number` | how fast the swing dies (10): lower bounces more |
| `stiffness` | `number` | how hard the spring pulls (100): stiffer is quicker |
| `overshootClamping` | `boolean` | stop at `to` the first time it gets there, with no overshoot (false) |

### `SpringOptions`

| Field | Type | What it is |
| --- | --- | --- |
| `frame` (required) | `number` | the frame to sample, usually useCurrentFrame() |
| `fps` (required) | `number` | frames a second, from useVideoConfig() |
| `from` | `number` | the value it starts at (0) |
| `to` | `number` | the value it settles at (1) |
| `config` | `SpringConfig` | the spring: mass, damping, stiffness, overshootClamping |
| `durationInFrames` | `number` | stretch or squeeze the spring to settle in this many frames |
| `durationRestThreshold` | `number` | how close to `to`, as a share of the travel, counts as settled when timing it (0.005) |
| `delay` | `number` | frames before it starts (0) |
| `reverse` | `boolean` | play it backwards: from `to` back to `from` |

<!-- generated:end -->
