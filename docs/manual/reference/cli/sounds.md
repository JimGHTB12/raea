---
title: "raea sounds"
summary: "Search open sample libraries for instruments and drums, and fetch one for a Sampler."
order: 83
sources: ["apps/tools/src/sound/library.rs"]
---

`raea sounds` searches open, freely licensed sample libraries by the words producers use ("kick", "hihat", "upright
piano") and lists what matches. `raea sounds get` fetches one instrument's samples into the project, ready for a
Tone.js `Sampler`. For a pitched instrument it takes one sample per note, at the middle dynamic.

```sh
raea sounds piano
raea sounds get vcsl/upright-piano --notes C2,C3,C4,C5 -o media/piano/
raea sounds get vcsl/bass-drum --all
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea sounds [WORDS] | sounds get PACK/INSTRUMENT [--notes ..|--all] [-o DIR]   (open sample libraries for a Sampler)
```

Headless: it needs nothing running and opens no window. Takes up to 2 arguments besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--all` | – | Fetch every sample the instrument has: every note, dynamic and round robin. |
| `--notes` | `..\|--all` | Fetch only these notes, comma-separated (`C2,C4`). |
| `-o` | `DIR` | The folder the samples go to. |

<!-- generated:end -->
