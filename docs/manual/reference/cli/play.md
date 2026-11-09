---
title: "raea play"
summary: "Play a game live in a window, or headless with scripted keys, snapshots and a timing report."
order: 3
sources: ["apps/play/src/lib.rs", "apps/play/src/offscreen.rs"]
---

`raea play` runs a composition live: its Clock follows the wall clock, the keyboard, pointer and gamepads reach it,
and its sound plays as it goes. It opens a window by default. With `--offscreen` it plays headless, which is how an
agent plays a game: keys come from a script or a random hand, snapshots land at given seconds, and a JSON report
says how each frame went.

```sh
raea play game.tsx                                            # in a window, with your keyboard
raea play game.tsx --offscreen --input --seconds 10 --json report.json
raea play game.tsx --offscreen --input-script keys.json --snapshot 2 at-2s.png --snapshot 5 at-5s.png
raea play game.tsx --offscreen --input --input-log play.json  # record what was pressed ...
raea render game.tsx --input play.json -o same-run.mp4        # ... and render the very same run
```

A key script is `{"FRAME": [keys]}`, each set held from its frame until the next one, such as
`{"0": [], "30": ["ArrowRight"], "60": ["ArrowRight", "Space"]}`. Keys can also be `Mouse:Right`, `Wheel:Y 100`,
`Pad0:South`, `Touch:0:16:16` and `Pointer:X:Y` (with `:down` while pressed). `--input` and
`--input-script` both drive the keys, so they can't be given together.

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea play EDIT.tsx [--input | --input-script KEYS.json] [--size WxH] [--seconds N|end]   (a game, live; also: --input-log FILE --offscreen --snapshot T FILE --json FILE --seek F FILE --scrub N --baked --autoplay --no-vsync --master --3d)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--3d` | – | Draw the composition's 3D scene under its page, as its render draws it. With snapshots, the 3D is waited for before the clock starts. |
| `--autoplay` | – | Start playing at once instead of waiting for the first key or click. |
| `--baked` | – | Play a timeline from its bake: React steps through every frame once first, then no JavaScript runs while it plays. |
| `--input` | – | Press and release keys at random, 50 to 250 ms apart, like a player's hand, and measure input-to-photon latency. Arrow keys by default; `RAEA_INPUT_KEYS` names the sets to cycle through. |
| `--input-log` | `VALUE` | Write each frame's input as played, in the timeline form `raea render --input` takes, so a render can repeat the play exactly. |
| `--input-script` | `KEYS.json` | Drive the keys from a script, `{"FRAME": [keys]}`, each set held until the next. |
| `--json` | `VALUE` | Write the play's report (frame times, gaps, latency, memory) to this file. |
| `--master` | – | Master a timeline's sound the way its render masters it. |
| `--no-vsync` | – | Present frames as fast as they're drawn rather than at the display's rate, to measure the work itself. |
| `--offscreen` | – | Play headless, with no window: what agents and checks use. |
| `--scrub` | `VALUE` | Jump to N frames spread over the timeline in shuffled order and report what each seek costs. Headless only. |
| `--seconds` | `N\|end` | How long to play, in seconds, or `end` to play a timeline to its last frame. |
| `--seek` | `V1 V2` | Jump straight to frame F and save it as a PNG. Repeatable; the report is the seeks alone. Headless only. |
| `--size` | `WxH` | The stage's size in pixels (1080x1920 by default); a responsive composition lays out at it. |
| `--snapshot` | `V1 V2` | Save the stage at T seconds as a PNG. Repeatable. Headless only. |

<!-- generated:end -->

## See also

- [Your first game](../../start/first-game.md) writes a game, plays it and renders the same run.
- [`raea export-game`](export-game.md) turns a game into a program of its own.
