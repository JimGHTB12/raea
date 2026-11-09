---
title: "raea splat"
summary: "Train footage of a real place or thing into a 3D Gaussian splat for <Splat>, on this Mac's GPU."
order: 66
sources: ["apps/tools/src/splat.rs", "io/assets/src/splat.rs"]
---

`raea splat train` turns a video, or a folder of photos, of a real place or object into a Gaussian splat that
`<Splat src>` draws in a 3D scene. It picks frames, solves the cameras with COLMAP and trains with Brush on Metal.
`raea splat setup` installs those tools. Like a solve, a splat is kept in the asset store by its footage and
settings, so the same footage trains once.

```sh
raea splat setup
raea splat train media/shoe.mp4 -o shoe.ply --matte black
raea splat train media/room.mp4 -o room.ply --from 3 --seconds 20 --frames 300 --steps 30000
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea splat train VIDEO|IMAGE_DIR -o OUT.ply [--frames 150] [--steps 20000] [--max-res 1600] [--from S] [--seconds N] [--max-splats 2000000] [--matte none|black] [--match sequential|exhaustive] [--credit TEXT] [--license TEXT] [--dir EDIT_DIR]
raea splat setup   (footage trained into a Gaussian splat for <Splat>; setup installs its tools)
```

Headless: it needs nothing running and opens no window. Takes up to 2 arguments besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--credit` | `TEXT` | Who made the footage, written into `OUT.ply.json` beside the splat. |
| `--dir` | `EDIT_DIR` | The edit's folder, whose asset store keeps the splat (the current folder by default). |
| `--frames` | `150` | How many pictures to train on, 8 to 2000. |
| `--from` | `S` | Start this many seconds into a video. |
| `--license` | `TEXT` | The footage's licence, written into `OUT.ply.json`. |
| `--match` | `sequential\|exhaustive` | How COLMAP matches pictures: `sequential` (neighbours only, the default for a video over 60 frames) or `exhaustive` (every pair). |
| `--matte` | `none\|black` | `black` for an object filmed on black, trained alone without its background. |
| `--max-res` | `1600` | The long side the pictures are scaled to, in pixels (256 to 4096). |
| `--max-splats` | `2000000` | The most splats the result may have. |
| `--seconds` | `N` | How many seconds of a video to use (to its end by default). |
| `--steps` | `20000` | Training steps, 100 to 200000: more is sharper and slower. |
| `-o` | `OUT.ply` | Where the splat goes, with its sidecar `OUT.ply.json`. |

<!-- generated:end -->
