---
title: "raea render"
summary: "Render a composition to an MP4, a PNG or a folder of frames, on the GPU and headless."
order: 2
sources: ["engine/cli/src/cli.rs", "runtime/export/src/render.rs", "runtime/bundle/src/lib.rs"]
---

`raea render` bundles a TSX composition, draws it on the GPU without opening a window, and writes a video, one still
or a folder of frames. It draws the same tree the viewer gets in the raea app, so its stills are the truth about
layout. It also renders an HTML composition and a bundle made by [`raea bundle`](bundle.md).

```sh
raea render piece.tsx -o piece.mp4                       # the whole video
raea render piece.tsx --frame 120 -o frame-120.png       # one frame
raea render piece.tsx --at 15,63,144 -o stills/          # some frames, as stills/frame-00015.png and on
raea render piece.tsx --alpha -o frames/                 # every frame as a transparent PNG
raea render piece.tsx --quality master -o final.mp4      # constant quality for type and thin lines
raea render scene.tsx --review review/                   # a 3D scene from six fixed cameras, in colour and clay
```

What `-o` names decides what you get: a name ending in `.mp4` is a video, `.png` is one frame, and anything else is
a folder of `frame-NNNNN.png`. The folder an output goes in must exist.

A composition's size, frame rate and length come from its `<Composition>` (or an `export const config`); `--w`,
`--h`, `--fps` and `--frames` only apply to a file that states neither. A `words.json` and a `perception.json` next
to the composition are loaded on their own, as `raea play` loads them; a flag naming another file wins.

A whole video made from the same inputs as an earlier one is copied from the export cache rather than drawn again,
and the command says so.

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea render EDIT.tsx -o OUT.mp4|OUT.png|DIR [--frame N | --at A,B] [--composition ID] [--w W] [--h H] [--fps FPS] [--frames N] [--words W.json] [--perception P.json] [--input I.json|inputs.rrl] [--manifest render.json] [--layers] [--stage WxH] [--10bit] [--aov] [--alpha] [--kbps N | --quality draft|high|master|0..1] [--clay]   (also: --lite --comps)
raea render EDIT.tsx --review DIR [--frame N]   (fixed cameras hero, front, right, top, iso, low in colour and clay: DIR/colour.png, clay.png, review.json)
raea render PAGE.html -o OUT.mp4|OUT.png|DIR [--at A,B] [--alpha]   (an HTML composition)
raea render --bundle NAME.comp.js -o OUT.png|DIR [--dir DIR] [--frame N]   (a bundle from raea bundle, as render_bundle draws it)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--10bit` | – | Encode the video as 10-bit HEVC (Main 10), whose decoded colours match the render exactly; H.264 8-bit otherwise. |
| `--alpha` | – | Keep the background transparent. Writes PNGs only (`-o OUT.png` or a folder): H.264 and HEVC hold no alpha, so encode the folder to ProRes 4444 afterwards if you need a video. |
| `--aov` | – | Write each frame's motion vectors and depth beside its colour, in `DIR/motion` and `DIR/depth`, for [`raea neural`](neural.md). `-o` must name a folder. |
| `--at` | `A,B` | Render only these frame numbers, comma-separated. |
| `--bundle` | `NAME.comp.js` | Render a bundle made by `raea bundle` instead of a TSX file, as `render_bundle` on the iPhone draws it. |
| `--clay` | – | Draw every standard 3D surface in plain grey, to judge shapes, light and contact without colour. |
| `--composition` | `ID` | Which registered `<Composition>` to render when the file holds several; the first one otherwise. |
| `--comps` | – | Record which components drew each frame in the frame's marker. Used by bakes and the app's live view, rarely by hand. |
| `--dir` | `DIR` | With `--bundle`: the folder its relative media paths are read from (the bundle's own folder by default). |
| `--fps` | `FPS` | Frame rate for a file with no `<Composition>` or `config` (30 by default). |
| `--frame` | `N` | Render this one frame. |
| `--frames` | `N` | Length in frames for a file with no `<Composition>` or `config` (60 by default). |
| `--h` | `H` | Height in pixels for a file with no `<Composition>` or `config` (1920 by default). |
| `--input` | `I.json\|inputs.rrl` | A game's input over time: a JSON timeline, or the step keys of an `inputs.rrl`, so a recorded play-through renders the same every time. |
| `--kbps` | `N` | A fixed video bitrate in kbit/s, in place of the default that follows size and frame rate. |
| `--layers` | – | Write `NAME.layers` beside the export: which layer (a 3D part, an element, its text, an image or a video, by where its JSX is written) each pixel of each frame shows, for raea's player to select layers by clicking. The manifest (`--manifest`, or `NAME.render.json` beside it) names it. The export cache is skipped. |
| `--lite` | – | Draw with raea's live view runtime instead of React, as a live game does. Used by the app when it opens a game. |
| `--manifest` | `render.json` | Write a manifest of the render to this file: every frame's element boxes as drawn, the 3D's clipping and the sounds heard, which `raea check` reads. |
| `--perception` | `P.json` | The perception the composition's Clock reads (faces, boxes, one for every frame or by frame), when it isn't the `perception.json` beside it. |
| `--quality` | `draft\|high\|master\|0..1` | Constant-quality encoding: `draft` for previews, `high` for most deliveries, `master` for type and hard edges, or a number from 0 to 1. |
| `--review` | `DIR` | Render a 3D scene from six fixed cameras (hero, front, right, top, iso, low), in colour and in clay, to `DIR/colour.png`, `DIR/clay.png` and `DIR/review.json`. |
| `--stage` | `WxH` | Render a responsive composition, one that lays out from `useVideoConfig()`, at this size. A fixed-size composition ignores it and says so. |
| `--w` | `W` | Width in pixels for a file with no `<Composition>` or `config` (1080 by default). |
| `--words` | `W.json` | A transcript for the Clock's words (captions and cuts read them), when it isn't the `words.json` beside the composition. |
| `-o` | `OUT.mp4\|OUT.png\|DIR` | Where it goes: an `.mp4` video, one `.png`, or a folder of frames. |

<!-- generated:end -->

## See also

- [Rendering and checking](../../concepts/rendering.md) covers stills, quality, transparent frames and dark material in depth.
- [`raea check`](check.md) measures a render against the floor; [`raea play`](play.md) runs a game live.
