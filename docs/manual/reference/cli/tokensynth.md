---
title: "raea tokensynth"
summary: "Make an instrument's notes from a description of its timbre with TokenSynth, on this machine."
order: 85
sources: ["apps/tools/src/sound.rs", "io/assets/src/local.rs"]
---

`raea tokensynth` plays notes on an instrument described in words, with the TokenSynth model running locally, and
writes one file per note: a sample set for a Sampler. Like `raea sao`, it shares the asset store and defaults with
`Tone.tokensynth("…", notes)` in a composition.

```sh
raea tokensynth "warm analog bass" --notes C1,C2,C3 -o media/bass/
raea tokensynth "glassy bell pad" --seconds 4 --velocity 80
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea tokensynth "TIMBRE" [--notes C2,C3,C4] [--seconds 2] [--velocity 100] [--seed 1234] [--precision fp16] [--dir EDIT_DIR] [-o OUT_DIR]   (an instrument's notes, local)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--dir` | `EDIT_DIR` | The edit's folder, whose asset store keeps the notes (the current folder by default). |
| `--notes` | `C2,C3,C4` | Which notes to play, comma-separated. |
| `--precision` | `fp16` | The model's precision: `fp32`, `fp16` (the default), `bf16`, `int8` or `coreml`. |
| `--seconds` | `2` | How long each note is. |
| `--seed` | `1234` | The seed; another seed gives another take. |
| `--velocity` | `100` | How hard each note is played, as MIDI velocity up to 127. |
| `-o` | `OUT_DIR` | Also write the notes to this folder. |

<!-- generated:end -->
