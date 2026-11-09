---
title: "raea part"
summary: "Copy a ready-made part into the project's components/, to use and change as your own."
order: 27
sources: ["engine/cli/src/index.ts", "engine/cli/src/registry.gen.ts"]
---

Parts are ready-made components (captions, lower thirds, end cards and the like). `raea part list` shows them;
`raea part add` copies one into `components/NAME/`, where it's yours: raea never updates or overwrites it unasked.

```sh
raea part list
raea part add captions end-card
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea part list | raea part add NAME ... [--force]   (a ready part copied into components/, yours to change)
```

Headless: it needs nothing running and opens no window. Takes any number of arguments besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--force` | – | Replace a part's files that are already in `components/`. Without it, an existing part is left alone and the command stops. |

<!-- generated:end -->
