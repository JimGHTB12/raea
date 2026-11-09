---
title: "raea guide"
summary: "Print raea's craft guides: the floor read before any edit, the modes, the notes and the 3D guides."
order: 122
sources: ["engine/cli/src/guide.rs", "sdk/guides"]
---

raea ships its craft as guides an agent reads before it works. `raea guide` alone lists them; `raea guide NAME`
prints one. `craft` is the floor, read before any edit. The modes (`pitch`, `explain`, `story`, `play`, `tool`)
are read when a brief names one. The notes (`quieter`, `bolder`, `layout`, `polish` and the rest) say what a note
from a person asks for, and are read before acting on it. `bevy` and `bevy/NAME` cover 3D.

```sh
raea guide
raea guide craft
raea guide pitch
raea guide quieter
raea guide bevy/lights
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea guide [NAME]   (craft, the floor read before any edit; the modes: pitch, explain, story, play, tool; motion, music, layout and the rest; bevy and bevy/NAME for 3D)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

<!-- generated:end -->
