---
title: "raea sao"
summary: "Make a sound effect from a description with Stable Audio Open, on this machine."
order: 84
sources: ["apps/tools/src/sound.rs", "io/assets/src/local.rs"]
---

`raea sao` makes foley and sound effects from words with Stable Audio Open Small (on Core ML), running locally. It goes through the same
asset store a render does, with the same defaults as `sao("…", {seconds})` in a composition, so a sound made here is
the one the composition finds already made, and the other way round. The model installs itself the first time (or
with [`raea setup-audio`](setup-audio.md)).

```sh
raea sao "a heavy wooden door creaks open" --seconds 3 -o door.wav
raea sao "rain on a tin roof" --seconds 8 --seed 4
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea sao "PROMPT" [--seconds 2] [--seed 1] [--dir EDIT_DIR] [-o OUT.wav]   (foley, local)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--dir` | `EDIT_DIR` | The edit's folder, whose asset store keeps the sound (the current folder by default). |
| `--seconds` | `2` | How long the sound is. |
| `--seed` | `1` | The seed; another seed gives another take of the same prompt. |
| `-o` | `OUT.wav` | Also write the sound to this file. |

<!-- generated:end -->
