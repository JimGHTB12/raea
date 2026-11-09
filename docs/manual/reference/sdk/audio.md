---
title: "raea/audio"
summary: "Sound: Audio, the master Mix, speech, sound in 3D space, and a song's energy map."
order: 43
sources: ["sdk/raea/src/audio.ts"]
---

`<Audio>` plays sound in step with the frame. `<Mix>` masters everything under it to a loudness and a true-peak
ceiling (-14 LUFS and -1 dBTP suit the major platforms). `song()` and `useSong()` read the energy map
[`raea song`](../cli/song.md) writes, so cuts can land on beats and drops. `<Speech>` speaks a line, and
`<Sound3D>` places a sound in a 3D scene.

```tsx
import { staticFile } from "raea";
import { Audio, Mix } from "raea/audio";

function Soundtrack() {
  return (
    <Mix lufs={-14} ceiling={-1}>
      <Audio src={staticFile("media/voice.wav")} bus="voice" />
      <Audio src={staticFile("media/bed.m4a")} bus="music" gainDb={-12} fadeOut={1.5} />
    </Mix>
  );
}
```

<!-- generated:begin by tools/manual-ref/sdk.ts from the SDK's exports and doc comments; edit outside this block -->

```ts
import { Audio, Mix, music, prepareSpeech } from "raea/audio"
```

`raea/audio` exports 6 components, 1 hook, 7 functions, 1 value, 25 types. Bevy's counterpart: `bevy_audio; raea plays it with seedling on its own mixer`. Source: [sdk/raea/src/audio.ts](https://github.com/shinyobjectz/raea/blob/main/sdk/raea/src/audio.ts).

## Components

### `Audio`

Sound in step with the frame; no picture. A muted or ended Audio is silent.

| Prop | Type | What it does |
| --- | --- | --- |
| `src` (required) | `string \| GeneratedAsset` | a file URL, or a generated asset from lyria() or sao() |

Also takes the props of [`MediaProps`](video.md#mediaprops).

### `Mix`

The master: every sound in the composition brought to `lufs` integrated loudness (-14) under a true-peak limiter at
`ceiling` dBTP (-1.5), as the platforms play back. Put one in the composition, around the soundtrack or anywhere.

| Prop | Type | What it does |
| --- | --- | --- |
| `lufs` | `number` | integrated loudness the mix is brought to (EBU R128, LUFS) |
| `ceiling` | `number` | the limiter's true-peak ceiling (dBTP) |
| `children` | `ReactNode` | what it masters, when it's written around the soundtrack (<Mix><Voice /><Bed /></Mix>): rendered as they are |

### `Seedling`

A game's sample, played by bevy_seedling on raea's mixer from the frame it first shows until it leaves the page,
with its effects in its chain. A render and a live play hear the same. Give a repeated one-shot a new `id` each
time: `{hit && <Seedling src={staticFile("hit.wav")} id={`hit-${hits}`} />}`.

| Prop | Type | What it does |
| --- | --- | --- |
| `src` (required) | `string \| GeneratedAsset` | the sample: a media file (staticFile), or a generated sound (sao(...), tokensynth(...)) from the asset store |
| `id` | `string` | the player's identity while it plays: a new id starts a new player (default: the src) |
| `volume` | `number` | linear gain, from the first frame (default 1) |
| `speed` | `number` | playback speed (default 1); may change frame to frame |
| `loop` | `boolean` | start again from the beginning when it ends |
| `fx` | `SeedlingFx[]` | a serial chain of effects on this player |
| `at` | `[number, number, number]` | where it sounds from in the 3D scene (metres, the Canvas3D's frame): heard from the scene's camera, panned, fading and dulling with distance, pitched by its speed toward or away (doppler); may move frame to frame. Unset: a sound with no place |
| `muffle` | `number` | a lowpass (Hz) for what stands between it and the camera: a wall, a window |

### `SongPlot`

The map drawn for a human to check: section bands named, the energy curve with builds, stops, dips and drops on
it, hits, the feature lanes, and the pacing (cuts per bar). `raea song FILE --plot OUT.png` renders it.

| Prop | Type | What it does |
| --- | --- | --- |
| `map` (required) | `SongMap` | the song's map, from \`raea song\` |
| `width` | `number` | in px |
| `height` | `number` | in px |
| `title` | `string` | the heading (the song's name, tempo, metre and length by default) |

### `Sound3D`

A sound in the 3D scene: a <Seedling> at a place (`at`), heard from the scene's camera as it moves, in a render and
in a game alike. A moving emitter (a car, a fly, a pan sliding) takes its place frame by frame.

 <Sound3D src={staticFile("kettle.wav")} at={onRun(right, 2.7, 0.35, 1.1)} loop volume={0.6} />

Also takes the props of [`SeedlingProps`](#seedlingprops).

### `Speech`

A line said on the frames it's rendered: heard from `start` (made ahead while `start` is null), stopped at the next
word after `cut`. Keep it rendered until the line ends (speechAt's `ended`): taken off the page it stops at once
(faded over 10 ms).

| Prop | Type | What it does |
| --- | --- | --- |
| `start` (required) | `number \| null` | the frame it's heard from; null: not yet (made ahead, not heard) |
| `cut` | `number \| null` | the frame it's interrupted at: it stops at the next word's start; null: said to its end |
| `volume` | `number` | gain (1) |

Also takes the props of [`Line`](#line) (`text`, `voice`, `seed`, `temperature`).

## Hooks

### `useSong`

Where the song is at the current frame (frame 0 of the enclosing Sequence is `offset` seconds into the song), with
the map bound to the composition's fps as `song`.

```ts
function useSong(map: SongMap, options?: UseSongOptions): SongMoment & { song: Song; }
```

## Functions

### `prepareSpeech`

Starts making a line now (or finds it made): call it as soon as the text is known.

```ts
function prepareSpeech(line: Line): void
```

### `song`

A song map bound to a frame rate (and where frame 0 sits in the song): frames from song times, and the reverse.

```ts
function song(map: SongMap, options: SongOptions): Song
```

### `songAt`

Where the song is at `t` seconds: a pure function of the map.

```ts
function songAt(map: SongMap, t: number): SongMoment
```

### `speechAt`

Where `line` (heard from `start`, cut at `cut`) is at `frame`: which word, whether it has ended, and where it ends.
Pure: a function of the line and the frame, so a game may fold it into its state. Waits for the engine if the line
hasn't been made that far yet (a render always waits; live play rarely has to).

```ts
function speechAt(line: Line & { start: number; cut?: number | null; }, frame: number, fps: number): SpeechState
```

### `speechDuration`

A line's length in seconds (waits until all of it is made; pure).

```ts
function speechDuration(line: Line): number
```

### `speechMade`

How much of a line has been made so far. Display-only: it depends on the machine and the moment.

```ts
function speechMade(line: Line): { seconds: number; done: boolean; }
```

### `speechStart`

The frame a line asked for at `frame` should start: SPEECH_LEAD later.

```ts
function speechStart(frame: number, fps: number, lead?: number): number
```

## Values

### `SPEECH_LEAD`

How long after a request a line should start, so live play never waits for it (seconds).

## Types

### `AudioProps`

The props of [`Audio`](#audio), tabled there.

### `CutOptions`

| Field | Type | What it is |
| --- | --- | --- |
| `by` | `"section" \| "bar"` | "section" (default) takes each section's rate, steady across it; "bar" each bar's own |
| `scale` | `number` | multiplies every rate (2: twice as many cuts), kept on the ladder 0.25..4 |
| `drops` | `boolean` | cut on every drop (default true) |
| `stops` | `boolean` | cut on a stop's hits (default true) |
| `builds` | `boolean` | double the rate through a build's second half (default true) |
| `minGap` | `number` | cuts closer than this many frames are merged, the earlier kept (default a quarter second) |
| `from` | `number` | first frame (inclusive), default 0 |
| `to` | `number` | last frame (exclusive), default the song's end |

### `FrameRange`

| Field | Type | What it is |
| --- | --- | --- |
| `from` | `number` | first frame (inclusive), default 0 |
| `to` | `number` | last frame (exclusive), default the song's end |

### `Line`

| Field | Type | What it is |
| --- | --- | --- |
| `text` (required) | `string` | the words to say |
| `voice` | `string` | a Pocket voice ("alba", "marius",...), a clip in ~/.raea-feed/voices by name ("watts"), or a WAV ("/media/narrator.wav" or staticFile("narrator.wav")); the default is "watts" when that clip's there, else alba |
| `seed` | `number` | which take: another seed says it another way (0) |
| `temperature` | `number` | Pocket's sampling temperature: lower is calmer and more even, higher livelier (0.7) |

### `MixProps`

The props of [`Mix`](#mix), tabled there.

### `music`

### `SeedlingFx`

| Field | Type | What it is |
| --- | --- | --- |
| `kind` (required) | `string` | one of raea/tone's effect classes by name: Filter, Reverb, Freeverb, FeedbackDelay, Compressor, EQ3,... |
| `options` | `Record<string, unknown>` | its options, by Tone's names |

### `SeedlingProps`

The props of [`Seedling`](#seedling), tabled there.

### `Song`

| Field | Type | What it is |
| --- | --- | --- |
| `map` (required) | `SongMap` | the song's map, from \`raea song\` |
| `fps` (required) | `number` | the composition's frame rate, that frames are counted in |
| `offset` (required) | `number` | seconds into the song frame 0 falls on (0) |
| `frame` (required) | `(seconds: number) => number` | the frame a song time falls on |
| `seconds` (required) | `(frame: number) => number` | the song's seconds at a frame |
| `at` (required) | `(frame: number) => SongMoment` | where the song is at a frame |
| `onBeat` (required) | `(every?: number \| undefined, range?: FrameRange \| undefined) => number[]` | frames of every `every`-th beat (1: each beat), counted from the song's first |
| `onDownbeat` (required) | `(every?: number \| undefined, range?: FrameRange \| undefined) => number[]` | frames of every `every`-th downbeat |
| `eventsOf` (required) | `(kind: SongEventKind, range?: FrameRange \| undefined) => { frame: number; event: SongEv…` | frames of every event of a kind (a build's or stop's start), with the event |
| `cutsAt` (required) | `(options?: CutOptions \| undefined) => number[]` | frames to cut on, paced to the energy: each section's (or bar's) rate, on its beats, plus drops, a stop's hits, and a build's second half at double rate |

### `SongBar`

| Field | Type | What it is |
| --- | --- | --- |
| `time` (required) | `number` | seconds from the song's start where the bar begins (its downbeat) |
| `beat` (required) | `number` | the index of its first beat |
| `beats` (required) | `number` | how many beats it holds |
| `energy` (required) | `number` | its mean energy, 0..1 |
| `cuts` (required) | `number` | suggested cuts in this bar: 0.25, 0.5, 1, 2 or 4 |

### `SongEvent`

| Field | Type | What it is |
| --- | --- | --- |
| `kind` (required) | `SongEventKind` | what the event is: build, drop, stop, hit, dip or break |
| `time` (required) | `number` | seconds from the song's start: a beat (drop, a downbeat) or the grid's nearest beat or half beat (hit) |
| `end` | `number` | spans (build, stop, dip, break) end here |
| `beat` (required) | `number` | the index of the beat it starts on, in `SongMap.beats` |
| `bar` (required) | `number` | the index of the bar it starts in |
| `beats` | `number` | a span: how many beats it lasts |
| `strength` (required) | `number` | 0..1 |
| `into` | `number \| null` | a build or stop: the drop it leads into, seconds |
| `hits` | `SongHit[]` | a stop: its sparse hits |
| `rise` | `number` | a drop: the energy of the bar after minus the bar before (negative when the band cuts after one big hit) |
| `holds` | `boolean` | a drop: the bar after stays up (false: one hit, then the band cuts out) |
| `afterStop` | `boolean` | a drop: it comes straight out of a stop |
| `afterBuild` | `boolean` | a drop: it lands at the end of a build (within two bars of it) |
| `gapBefore` | `boolean` | a drop: a breath of near-silence just before it |
| `at` | `number` | a hit: the onset's own time; how far over the groove it stands |
| `over` | `number` | a hit: how far over the groove it stands |

### `SongEventKind`

```ts
type SongEventKind = "build" | "drop" | "stop" | "hit" | "dip" | "break"
```

### `SongFeatures`

| Field | Type | What it is |
| --- | --- | --- |
| `loudness` (required) | `number[]` | loudness (RMS) a beat, 0..1 over the song |
| `onsets` (required) | `number[]` | onset strength (spectral flux) a beat, 0..1: how much is struck |
| `brightness` (required) | `number[]` | brightness (spectral centroid) a beat, 0..1 |
| `low` (required) | `number[]` | the low end (kick and bass, under 150 Hz) a beat, 0..1 |
| `high` (required) | `number[]` | the high band (over 3 kHz) a beat, 0..1 |
| `energy` (required) | `number[]` | the combined energy a beat, 0..1: loudness, low end, onsets and highs together |

### `SongHit`

A hit inside a stop: `time` on the beat grid (a beat or half beat), `at` where the onset is.

| Field | Type | What it is |
| --- | --- | --- |
| `time` (required) | `number` | seconds from the song's start, snapped to the beat grid (a beat or half beat) |
| `at` (required) | `number` | seconds from the song's start where the onset itself is |
| `strength` (required) | `number` | how hard the hit is, 0..1 |

### `SongLevel`

```ts
type SongLevel = "intro" | "low" | "mid" | "high" | "outro"
```

### `SongMap`

| Field | Type | What it is |
| --- | --- | --- |
| `version` (required) | `number` | the map format's version (1) |
| `duration` (required) | `number` | the song's length in seconds |
| `tempo` (required) | `number \| null` | beats per minute, or null when no beat was found |
| `beatsPerBar` (required) | `number` | beats in a bar (4 for 4/4) |
| `beats` (required) | `number[]` | every beat's time in seconds, ascending |
| `downbeats` (required) | `number[]` | every bar's first beat, in seconds |
| `features` (required) | `SongFeatures` | one value a beat, 0..1 over the song |
| `bars` (required) | `SongBar[]` | the bars in order, each with its energy and suggested cut rate |
| `sections` (required) | `SongSection[]` | the song's sections in order (intro, verse, chorus and so on) |
| `events` (required) | `SongEvent[]` | builds, drops, stops, hits, dips and breaks, in time order |
| `sectionsFrom` | `"markers" \| "novelty"` | where the sections came from: the song file's markers, or novelty in the music itself |
| `boundaries` | `{ time: number; beat: number; bar: number; strength: number }[]` | the candidate section boundaries novelty found, each with its strength 0..1 |
| `source` | `string \| null` | the song file's name |

### `SongMoment`

Where a moment of the song is. Seconds are the song's; infinities when there's no such event.

| Field | Type | What it is |
| --- | --- | --- |
| `time` (required) | `number` | the moment, seconds from the song's start |
| `beat` (required) | `number` | the beat at or before `time` (-1 before the first) |
| `bar` (required) | `number` | the bar at or before `time` (-1 before the first beat) |
| `beatInBar` (required) | `number` | 0-based position in the bar |
| `phase` (required) | `number` | 0..1 through the beat, and through the bar |
| `barPhase` (required) | `number` | 0..1 through the bar |
| `energy` (required) | `number` | the features at `time`, interpolated between beats |
| `loudness` (required) | `number` | loudness at `time`, 0..1 |
| `onsets` (required) | `number` | onset strength at `time`, 0..1 |
| `brightness` (required) | `number` | brightness at `time`, 0..1 |
| `low` (required) | `number` | the low end at `time`, 0..1 |
| `section` (required) | `SongSection \| null` | the section `time` is in (null before the first) |
| `sectionIndex` (required) | `number` | that section's index in `SongMap.sections` (-1 for none) |
| `sectionProgress` (required) | `number` | 0..1 through the section |
| `cutsPerBar` (required) | `number` | the bar's suggested cut rate (cuts per bar) |
| `drop` (required) | `SongEvent \| null` | the last drop at or before `time`, the next after it |
| `nextDrop` (required) | `SongEvent \| null` | the next drop after `time` |
| `sinceDrop` (required) | `number` | seconds since the last drop (Infinity when there's none) |
| `untilDrop` (required) | `number` | seconds until the next drop (Infinity when there's none) |
| `build` (required) | `SongEvent \| null` | the build `time` is in, and 0..1 through it |
| `inBuild` (required) | `boolean` | whether `time` is inside a build |
| `buildProgress` (required) | `number` | 0..1 through that build |
| `stop` (required) | `SongEvent \| null` | the stop `time` is in |
| `inStop` (required) | `boolean` | whether `time` is inside a stop |
| `dip` (required) | `SongEvent \| null` | the dip or break `time` is in |
| `next` (required) | `SongEvent \| null` | the next event of any kind after `time` |

### `SongOptions`

| Field | Type | What it is |
| --- | --- | --- |
| `fps` (required) | `number` | the composition's frame rate |
| `offset` | `number` | seconds into the song at frame 0 (where an <Audio> of it starts: its startFrom over fps) |

### `SongPlotProps`

The props of [`SongPlot`](#songplot), tabled there.

### `SongSection`

| Field | Type | What it is |
| --- | --- | --- |
| `name` (required) | `string` | the section's marker name when the song file has markers, else its level |
| `level` (required) | `SongLevel` | how high the section sits in the song: intro, low, mid, high or outro |
| `time` (required) | `number` | seconds from the song's start where the section begins |
| `end` (required) | `number` | seconds from the song's start where it ends |
| `beat` (required) | `number` | the index of its first beat |
| `bar` (required) | `number` | the index of its first bar |
| `energy` (required) | `number` | its mean energy, 0..1 |
| `cuts` (required) | `number` | suggested cuts per bar: 0.25, 0.5, 1, 2 or 4 |

### `SpeechProps`

The props of [`Speech`](#speech), tabled there.

### `SpeechState`

Where a line is at a frame.

| Field | Type | What it is |
| --- | --- | --- |
| `started` (required) | `boolean` | the frame is at or past its start |
| `word` (required) | `number` | the word being spoken (an index into `words`), -1 before the first |
| `ended` (required) | `boolean` | it has finished (said to its end, or stopped at its cut) |
| `endFrame` (required) | `number \| null` | the frame it ends on, once that's known (always known for a cut line, and for any line once it's all made) |
| `words` (required) | `string[]` | the line's words as Pocket says them (so far: a long line's later sentences are added as they're reached) |

### `Tone`

### `UseSongOptions`

| Field | Type | What it is |
| --- | --- | --- |
| `offset` | `number` | seconds into the song at frame 0 of the enclosing Sequence |

### `VolumeProp`

```ts
type VolumeProp = number | ((mediaFrame: number) => number)
```

<!-- generated:end -->
