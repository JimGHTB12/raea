---
title: "raea frame"
summary: "Say how a take fits the edit: a talking take for <FaceFrame>, a screen recording for <ScreenFit>, from where its face is."
order: 64
sources: ["engine/cli/src/index.ts"]
---

`raea frame` looks at a take and says how it should sit in the composition. A talking take gets a `<FaceFrame>`
that keeps the face where the frame wants it; a screen recording gets a `<ScreenFit>`. It reads where the face is
and says which of the two fits, and how.

```sh
raea frame media/interview.mp4
raea frame media/demo.mov --at 8
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea frame TAKE [--at S]   (how a take fits the edit: a talking take for <FaceFrame>, a screen for <ScreenFit>, from where its face is)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--at` | `S` | Look at the take at this second rather than at its start. |

<!-- generated:end -->
