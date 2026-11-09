---
title: "raea mezz"
summary: "Make an intra-only ProRes copy of each long-GOP source over 1080p, so scrubbing it is smooth."
order: 67
sources: ["engine/cli/src/verbs.rs", "io/media/src/mezz.rs"]
---

Footage from phones and cameras is long-GOP: most frames are stored as changes from others, so jumping around in it
is slow. `raea mezz` makes an intra-only ProRes copy of each such source over 1080p, which raea then reads instead
when you scrub. Given an edit, it also copies just the stretches the edit shows of sources too long to copy whole.
It takes files and folders, and looks a few levels down.

```sh
raea mezz --check media/
raea mezz media/ edit.tsx
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea mezz [--check] PATH...   (an intra-only ProRes copy of each long-GOP source over 1080p, for smooth scrubbing; --check says which would get one)
```

Headless: it needs nothing running and opens no window. Takes any number of arguments besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--check` | – | Say which sources would get a copy, and which already have one, without making any. |

<!-- generated:end -->
