---
title: "raea/video"
summary: "Footage and time-shifted sequences: Video, Sequence, Series, Loop and Freeze."
order: 42
sources: ["sdk/raea/src/video.ts"]
---

`<Video>` shows a clip in step with the frame: the renderer decodes it natively and draws the frame the composition
asks for, so scrubbing and rendering agree. Trim it with `trimBefore` and `trimAfter`, set its level with `volume`
or `gainDb`, and frame a talking take on its face with `framing="face"`. `<Sequence>` shifts time for what it holds,
`<Series>` plays its children one after another, `<Loop>` repeats, and `<Freeze>` holds a frame.

```tsx
import { staticFile } from "raea";
import { Series, Video } from "raea/video";

function Cut() {
  return (
    <Series>
      <Series.Sequence durationInFrames={90}>
        <Video src={staticFile("media/interview.mp4")} trimBefore={120} framing="face" />
      </Series.Sequence>
      <Series.Sequence durationInFrames={60}>
        <Video src={staticFile("media/broll.mp4")} muted />
      </Series.Sequence>
    </Series>
  );
}
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { Freeze, Loop, Sequence, Series } from "raea/video"
```

`raea/video` exports 5 components, 7 types. Bevy's counterpart: `raea's own; Bevy has no video crate`. Source: [sdk/raea/src/video.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/video.ts).

## Components

### `Freeze`

Shows its children as they are at `frame`, whatever the parent's frame.

| Prop | Type | What it does |
| --- | --- | --- |
| `frame` (required) | `number` | – |
| `children` | `ReactNode` | – |

### `Loop`

Repeats its children every `durationInFrames` frames, `times` times (forever by default).

| Prop | Type | What it does |
| --- | --- | --- |
| `durationInFrames` (required) | `number` | – |
| `times` | `number` | – |
| `name` | `string` | – |
| `layout` | `SequenceLayout` | – |
| `style` | `CSSProperties` | – |
| `className` | `string` | – |
| `children` | `ReactNode` | – |

### `Sequence`

Shifts time by `from` and shows its children only for `durationInFrames` frames.

| Prop | Type | What it does |
| --- | --- | --- |
| `from` | `number` | – |
| `durationInFrames` | `number` | – |
| `name` | `string` | – |
| `layout` | `SequenceLayout` | – |
| `width` | `number` | – |
| `height` | `number` | – |
| `style` | `CSSProperties` | – |
| `className` | `string` | – |
| `children` | `ReactNode` | – |

### `Series`

Plays its Series.Sequence children back to back, each shifted by its own offset.

| Prop | Type | What it does |
| --- | --- | --- |
| `children` | `ReactNode` | – |

### `Video`

A video clip in step with the frame; `data-source-time` is the source time (seconds) to show.

| Prop | Type | What it does |
| --- | --- | --- |
| `framing` | `"face"` | ours: frame the take on the largest face |
| `punchIn` | `number` | ours: multiplies the framing's scale (1.15 is the usual) |

Also takes the props of [`MediaProps`](#mediaprops).

## Types

### `FreezeProps`

The props of [`Freeze`](#freeze), tabled there.

### `LoopProps`

The props of [`Loop`](#loop), tabled there.

### `MediaProps`

| Field | Type | What it is |
| --- | --- | --- |
| `src` (required) | `string` | – |
| `startFrom` | `number` | – |
| `endAt` | `number` | – |
| `trimBefore` | `number` | – |
| `trimAfter` | `number` | – |
| `volume` | `VolumeProp` | linear gain, 1 being unity (0.5 is about -6 dB, 1.78 about +5 dB), or a function of the media's own frame |
| `gainDb` | `VolumeProp` | ours: the gain in decibels (or a function of the media's own frame returning it); wins over volume |
| `fadeIn` | `number` | ours: seconds to ramp the sound in from silence at its start (applied by the renderer at the audio's rate) |
| `fadeOut` | `number` | ours: seconds to ramp the sound out to silence at its end |
| `playbackRate` | `number` | – |
| `muted` | `boolean` | – |
| `loop` | `boolean` | – |
| `bus` | `string` | the bus it plays into: a named channel of the edit's Tone graph (music, sfx, voice, ambience, ui), so it takes that bus's volume, effects and ducking (raea/tone `buses`) |
| `name` | `string` | – |
| `className` | `string` | – |
| `style` | `CSSProperties` | – |

### `SequenceLayout`

```ts
type SequenceLayout = "absolute-fill" | "none"
```

### `SequenceProps`

The props of [`Sequence`](#sequence), tabled there.

### `SeriesSequenceProps`

| Field | Type | What it is |
| --- | --- | --- |
| `offset` | `number` | – |
| `height` | `number` | – |
| `style` | `CSSProperties` | – |
| `children` | `ReactNode` | – |
| `className` | `string` | – |
| `name` | `string` | – |
| `width` | `number` | – |
| `durationInFrames` | `number` | – |

### `VideoProps`

The props of [`Video`](#video), tabled there.

<!-- generated:end -->
