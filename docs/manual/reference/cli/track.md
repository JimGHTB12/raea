---
title: "raea track"
summary: "Solve a clip's camera on this machine, so 3D can stand in its world with <Matchmove>."
order: 65
sources: ["runtime/react/src/scene/track.rs", "io/assets/src/lib.rs"]
---

`raea track` solves the camera that filmed a clip: where it was and where it looked on each frame, and where the
ground is. `<Matchmove src=...>` in a composition uses the solve to put 3D into the shot. The solve is kept in the
project's asset store with its settings, so the render that asks for the same clip and settings finds it already made,
and the other way round. The solver installs itself the first time it's used.

```sh
raea track media/street.mp4 --from 2 --seconds 6
raea track media/table.mp4 --subject 0.3,0.4,0.4,0.3 --height 1.2 -o table.solve.json
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea track CLIP.mp4 [--subject X,Y,W,H] [--from S] [--seconds N] [--height M] [--fov DEG] [--smooth F] [--dir EDIT_DIR] [-o OUT.json]   (the clip's camera solved, for <Matchmove>)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--dir` | `EDIT_DIR` | The edit's folder, whose asset store keeps the solve (the current folder by default). |
| `--fov` | `DEG` | The camera's vertical field of view, for a shot with too little parallax to solve it (0, the solver's own, by default; 40 when it has none). |
| `--from` | `S` | Start the solve this many seconds in. |
| `--height` | `M` | The camera's height over the ground in metres, which sets the scene's scale (1.6 by default). |
| `--seconds` | `N` | How many seconds to solve (to the clip's end by default). |
| `--smooth` | `F` | Frames of Gaussian smoothing on the camera's path (1 by default; 0 for none). |
| `--subject` | `X,Y,W,H` | The subject's box on the first frame, in 0 to 1 of the frame, to solve around. |
| `-o` | `OUT.json` | Also write the solve to this file. |

<!-- generated:end -->
