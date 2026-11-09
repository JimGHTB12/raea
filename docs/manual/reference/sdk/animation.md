---
title: "raea/animation"
summary: "Declared animation: named moves, staggers, sequences and timelines, and a model's own clips."
order: 23
sources: ["sdk/raea/src/animation.ts"]
---

`raea/animation` describes motion as data instead of per-frame code. A move (`fade`, `rise`, `pop`, `slide`, `slam`,
`wipe` and the rest) says how something enters or leaves; `sequence`, `parallel` and `stagger` combine moves; and
`<Animate>` plays a declared animation on what it wraps. Because the motion is data, the engine can play it without
running React every frame, and a lint can read it. `useAnimation3d` plays a glTF model's own clips.

```tsx
import { Animate, Stagger, pop } from "raea/animation";

// each line rises in 4 frames after the one before, and fades out at frame 80
function Lines({ lines }: { lines: string[] }) {
  return (
    <Stagger each={4} enter="rise" exit="fade" exitAt={80}>
      {lines.map((l) => <p key={l}>{l}</p>)}
    </Stagger>
  );
}

// one element, with a move built in code
const Badge = () => <Animate enter={pop({ frames: 10 })}>NEW</Animate>;
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { Animate, clipEvents, curve, curveFn } from "raea/animation"
```

`raea/animation` exports 2 components, 1 hook, 27 functions, 2 values, 18 types. Bevy's counterpart: `bevy_animation`. Source: [sdk/raea/src/animation.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/animation.ts).

## Components

### `Animate`

<Animate>, memoised by its props' content (React skips a render whose props fingerprint matches the last). The
fingerprint is on it as `Animate.fingerprint` for a runtime's own bailout. Its declaration is the engine's
Animation (@bevy raea_core_animate::spec::Animation), typed from the registry (animate.gen.ts), sent as data-raea-anim and
read back as that type by sdk/react animate.rs.

Also takes the props of [`AnimateOwnProps`](#animateownprops).

### `Stagger`

Each child in its own <Animate>, entrances `each` frames apart (or by grid distance). The same content as last
render gives the same element back.

Also takes the props of [`StaggerProps`](#staggerprops) (`each`, `grid`, `exitEach`); [`AnimateOwnProps`](#animateownprops).

## Hooks

### `useAnimation3d`

For a 3D node (<Mesh animate>, materials): the declaration's text, or under plain bun the props it sets now.

```ts
function useAnimation3d(spec: Record<string, TrackSpec<string | number> | TrackSpec<string | number>[] | Keys> | Move | undefined): { attr?: string; values?: Record<string, string | number> | undefined; }
```

## Functions

### `clipEvents`

The clip events that fire between two frames (Bevy's AnimationClip events, as data): an event `time` seconds into
the clip fires on the frame whose clip time first reaches it, that is when it lies in (prev, now], once per loop
when the clip loops (`duration` its length). Give the previous frame's clip time and this frame's, as a <Model>'s
`time` runs; on the first frame pass prev -1 so an event at 0 fires.

```ts
function clipEvents<E extends { time: number; }>(events: E[], prev: number, now: number, duration: number, loop?: boolean): E[]
```

### `curveFn`

A curve name as a function (the engine's easing::Easing::parse, the same names).

```ts
function curveFn(name: string, fps?: number): (t: number) => number
```

### `delay`

The same move later by `frames`.

```ts
function delay(m: MoveLike, frames: number): Move
```

### `fade`

opacity 0 to 1 over `frames` (6).

```ts
function fade(o?: { frames?: number; ease?: Curve; }): Move
```

### `fill`

a bar filling: width 0 to `to` (100%) over `frames` (24), out-cubic.

```ts
function fill(o?: { to?: number; frames?: number; unit?: "px" | "%"; ease?: Curve; }): Move
```

### `fingerprint`

FNV-1a over a value's numbers and strings. Functions (components, handlers) count by identity, React elements by
type, key and props; an inline `animate={{ y: track(...) }}` or `style={{...}}` is a new object every render, and this
tells it's the same content. Assumes props aren't mutated in place (React's own assumption).

```ts
function fingerprint(v: unknown, h: number, depth?: number): number
```

### `gridDelay`

A grid item's distance from an origin, in steps (raea-animate's StaggerPattern::Grid, the same rule).

```ts
function gridDelay(cols: number, rows: number, from: GridFrom, index: number): number
```

### `move`

A move from its tracks, by property: `move({ opacity: track({ 0: 0, 6: 1 }), y: track({ 0: 30, 6: 0 }) })`. Its
length is its last key. The named moves (rise, pop and the rest) are built with it.

```ts
function move(tracks: Record<string, TrackSpec<string | number> | TrackSpec<string | number>[] | Keys<string | number>>): Move
```

### `parallel`

Every step at once.

```ts
function parallel(...steps: Step[]): Group
```

### `pop`

scale `from` (0.6), `peak` (1.08) at `frames`/2, 1 at `frames` (8); in over 3 frames.

```ts
function pop(o?: { from?: number; peak?: number; frames?: number; ease?: Curve; }): Move
```

### `propsFingerprint`

A props object's fingerprint: what <Animate> and <Stagger> compare to know a render would give the same element,
and what a runtime's bailout can key on (`Animate.fingerprint`).

```ts
function propsFingerprint(props: object): number
```

### `pushIn`

a slow push-in on footage: scale 1 to 1 + `amount` (0.04) across `frames` (90), linear.

```ts
function pushIn(o?: { amount?: number; frames?: number; }): Move
```

### `reverse`

A move played backwards: it leaves the way it came (each curve reversed, so an out-curve becomes the matching
in-curve). An exit is its entrance reversed.

```ts
function reverse(m: MoveLike): Move
```

### `rise`

y offset `distance` to 0 over `frames` (30 px over 6), fading in over `fade` (5).

```ts
function rise(o?: { distance?: number; frames?: number; fade?: number; ease?: Curve; }): Move
```

### `sampleAnimation`

What an Animation sets at composition frame `frame`: page style keys, a counter's text, or 3D props
(raea-animate's Sampled, here for plain bun).

```ts
function sampleAnimation(a: Animation, frame: number): { style: Record<string, string | number>; text?: string; values: Record<string, string | number>; }
```

### `sampleMove`

What a move sets at local frame `frame` when it starts at `start`: the same rules the engine samples <Animate>
with (x, y, opacity, scale, rotate, colours...), for parts that are animated one by one in TypeScript (a 3D text's
glyphs: <RichText enter>). An empty object before it starts is its first key's values (an entrance holds them).

```ts
function sampleMove(m: MoveLike, start: number, frame: number, fps?: number): Record<string, string | number>
```

### `sequence`

Each step when the one before it ends (`gap` frames apart: sequenceGap).

```ts
function sequence(...steps: Step[]): Group
```

### `sequenceGap`

Each step when the one before it ends, `gap` frames later (a negative gap overlaps them).

```ts
function sequenceGap(gap: number, ...steps: Step[]): Group
```

### `slam`

a hit: scale `from` (1.3), `dip` (0.96) at `frames`/2, 1 at `frames` (8); in over 2 frames.

```ts
function slam(o?: { from?: number; dip?: number; frames?: number; ease?: Curve; }): Move
```

### `slide`

arrives from a side: `distance` px (30) over `frames` (7), fading in over 5. `from` is where it comes from.

```ts
function slide(o?: { from?: Dir; distance?: number; frames?: number; ease?: Curve; }): Move
```

### `stagger`

The parts `each` frames apart, each with `m`. With `grid`, delays spread by grid distance from an origin.

```ts
function stagger(parts: string[], each: number, m: MoveLike, grid?: { cols: number; rows?: number; from?: GridFrom; } | undefined): Group
```

### `strike`

a strike-through line drawn: width 0 to 100% over `frames` (8).

```ts
function strike(o?: { frames?: number; ease?: Curve; }): Move
```

### `tick`

a number counting from `from` (0) to `to` over `frames` (24), written as the element's text: `digits` decimals,
`separator` between its thousands ("," or true writes 12,345; none by default), `prefix` and `suffix` around it
("$", " pts"). Hidden until it starts, then in over `fade` frames (2; 0 shows it at `from` the whole time).

```ts
function tick(o: { to: number; from?: number; frames?: number; digits?: number; ease?: Curve; prefix?: string; suffix?: string; separator?: string | boolean; fade?: number; }): Move
```

### `timeline`

Steps placed in time (a sequence of them). Each part's start is decided here, once.

```ts
function timeline(...steps: Step[]): Plan
```

### `toMove`

A name, a move or several moves together, as one move.

```ts
function toMove(m: MoveLike): Move
```

### `track`

A track: `track({ 0: 30, 6: 0 })` is a 30 px rise over 6 frames, out-cubic by default.

```ts
function track<V = string | number>(keys: Keys<V>, ease?: Curve | Curve[], opts?: Omit<TrackSpec<string | number>, "ease" | "keys">): TrackSpec<V>
```

### `wipe`

a reveal wiping across from `from` (left) over `frames` (8), in-out-cubic, by clipping.

```ts
function wipe(o?: { from?: Dir; frames?: number; ease?: Curve; }): Move
```

## Values

### `curve`

Curve helpers: `curve.bezier(0.05, 0.7, 0.1, 1)`, `curve.spring({ damping: 15, stiffness: 120 })`.

### `moves`

The named moves by name, as `enter="rise"` finds them.

## Types

### `AnimateOwnProps`

| Field | Type | What it is |
| --- | --- | --- |
| `enter` | `MoveLike` | the entrance: a move's name, a move, or several together |
| `exit` | `MoveLike` | the exit, played at `exitAt` (an entrance reversed: `exit="rise"` sinks back out) |
| `animate` | `Record<string, TrackSpec \| TrackSpec[] \| Keys> \| Move` | free tracks by property, in local frames: `{ backgroundColor: track({ 0: "#fff", 20: "#000" }) }` |
| `at` | `number` | local frame the entrance starts (0) |
| `exitAt` | `number` | local frame the exit starts |
| `timeline` | `Plan` | a timeline's plan and this element's part in it: the part's move, at its start (plus `at`) |
| `part` | `string` | – |
| `mount` | `"during" \| "always"` | "always" (default): in the tree, and its layout, the whole time (invisible before an entrance that fades in); "during": only from the entrance's start until the exit ends (it takes its space when it arrives) |
| `as` | `string` | the element: "div" by default |
| `style` | `CSSProperties` | – |
| `className` | `string` | – |
| `children` | `ReactNode` | – |

### `AnimateProps`

```ts
type AnimateProps = AnimateOwnProps & { [attr: string]: unknown }
```

### `Animation`

raea_core_animate::spec::Animation

| Field | Type | What it is |
| --- | --- | --- |
| `origin` (required) | `number` | – |
| `freeze` (required) | `number \| null` | – |
| `fps` (required) | `number` | – |
| `tracks` (required) | `(AnimTrack)[]` | – |
| `show_from` (required) | `number \| null` | – |
| `show_until` (required) | `number \| null` | – |

### `AnimGraphNode`

A node of a <Model graph>.

```ts
type AnimGraphNode = | { clip: string | number; time?: number; weight?: number; mask?: string[] } | { blend: AnimGraphNode[]; weight?: number; mask?: string[] } | { add: AnimGraphNode[]; weight?: number; mask?: string[] }
```

### `AnimKey`

raea_core_animate::spec::AnimKey

| Field | Type | What it is |
| --- | --- | --- |
| `at` (required) | `number` | – |
| `value` (required) | `number` | – |
| `color` (required) | `string` | – |
| `ease` (required) | `string` | – |

### `AnimTrack`

raea_core_animate::spec::AnimTrack

| Field | Type | What it is |
| --- | --- | --- |
| `prop` (required) | `string` | – |
| `keys` (required) | `(AnimKey)[]` | – |
| `unit` (required) | `string` | – |
| `repeat` (required) | `string` | – |
| `digits` (required) | `number` | – |
| `prefix` (required) | `string` | – |
| `suffix` (required) | `string` | – |
| `separator` (required) | `string` | – |

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

### `GridFrom`

```ts
type GridFrom = "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center" | "top" | "bottom" | "left" | "right"
```

### `Group`

| Field | Type | What it is |
| --- | --- | --- |
| `kind` (required) | `"sequence" \| "parallel"` | – |
| `steps` (required) | `Step[]` | – |
| `gap` (required) | `number` | frames between steps of a sequence (negative overlaps) |

### `Keys`

Keyframes by local frame: numbers, or colour text for colour properties (a material's `animate` takes other
values for its own fields, LinearRgba for an emissive).

```ts
type Keys = Record<number, V>
```

### `Move`

Tracks by property, in frames from the move's own start.

| Field | Type | What it is |
| --- | --- | --- |
| `tracks` (required) | `Record<string, TrackSpec \| TrackSpec[]>` | – |
| `frames` (required) | `number` | how long it runs (its last key) |

### `MoveLike`

```ts
type MoveLike = MoveName | Move | MoveLike[]
```

### `MoveName`

```ts
type MoveName = "rise" | "pop" | "slam" | "slide" | "wipe" | "fill" | "strike" | "pushIn" | "fade"
```

### `Plan`

A timeline's plan: each part's start frame and move, and the whole length.

| Field | Type | What it is |
| --- | --- | --- |
| `parts` (required) | `Record<string, { at: number; move: Move }>` | – |
| `frames` (required) | `number` | – |

### `StaggerProps`

| Field | Type | What it is |
| --- | --- | --- |
| `each` (required) | `number` | frames between one child's entrance and the next |
| `grid` | `{ cols: number; rows?: number; from?: GridFrom }` | spread by grid distance instead of order |
| `exitEach` | `number` | the exit staggered too, `exitEach` frames apart (0: all together) |
| `style` | `CSSProperties` | – |
| `animate` | `Record<string, TrackSpec \| TrackSpec[] \| Keys> \| Move` | free tracks by property, in local frames: `{ backgroundColor: track({ 0: "#fff", 20: "#000" }) }` |
| `as` | `string` | the element: "div" by default |
| `exit` | `MoveLike` | the exit, played at `exitAt` (an entrance reversed: `exit="rise"` sinks back out) |
| `children` | `ReactNode` | – |
| `enter` | `MoveLike` | the entrance: a move's name, a move, or several together |
| `at` | `number` | local frame the entrance starts (0) |
| `exitAt` | `number` | local frame the exit starts |
| `mount` | `"during" \| "always"` | "always" (default): in the tree, and its layout, the whole time (invisible before an entrance that fades in); "during": only from the entrance's start until the exit ends (it takes its space when it arrives) |
| `className` | `string` | – |

### `Step`

A timeline step: [part, move], a group from sequence/parallel/stagger, or a pause in frames.

```ts
type Step = [part: string, move: MoveLike] | { part: string; move: MoveLike; offset?: number } | Group | number
```

### `TrackSpec`

One property's keyframes: values by frame, the curve between each pair (one for all, or one per segment).

| Field | Type | What it is |
| --- | --- | --- |
| `keys` (required) | `[number, V][]` | – |
| `ease` (required) | `Curve \| Curve[]` | – |
| `unit` | `"px" \| "%"` | – |
| `repeat` | `"hold" \| "loop" \| "pingpong" \| `${"loop" \| "pingpong"}:${number}`` | – |
| `digits` | `number` | – |
| `prefix` | `string` | a `text` counter's text before and after the number, and between its thousands ("," writes 12,345) |
| `suffix` | `string` | – |
| `separator` | `string` | – |

<!-- generated:end -->
