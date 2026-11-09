---
title: "raea neural"
summary: "Run DLSS 5 neural rendering over a render's frames on the Mac, through your own MLX-DLSS build and weights."
order: 68
sources: ["engine/neural/src/lib.rs"]
---

`raea neural mlxdlss` feeds a render's frames, with the motion vectors and depth raea writes beside them, to
MLX-DLSS, which runs DLSS 5's neural rendering on Apple silicon. It's an optional integration: the `mlxdlss` binary
and the weights are yours. The weights are NVIDIA's, extracted from your own DLSS install, and never ship with raea.
Set `RAEA_MLXDLSS` to the binary and `RAEA_DLSS_MODEL` (or `--model`) to the weights. Each shot runs on its own,
so a face never carries over a cut.

```sh
raea render piece.tsx --aov -o frames/
raea neural mlxdlss frames/ -o neural/ --steady 0.25
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea neural mlxdlss DIR -o OUT [--model M.dlssmodel] [--intensity X] [--profile P] [--skin-structure X] [--local-structure X] [--steady A]   (DLSS 5 neural rendering on a `render --aov` folder, with its motion and depth: your own MLX-DLSS build and weights, RAEA_MLXDLSS and RAEA_DLSS_MODEL)
```

Headless: it needs nothing running and opens no window. Takes up to 2 arguments besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--intensity` | `X` | How strongly the network repaints, passed to MLX-DLSS. |
| `--local-structure` | `X` | How much local detail it keeps, passed to MLX-DLSS. |
| `--model` | `M.dlssmodel` | The weights to use, in place of `RAEA_DLSS_MODEL`. |
| `--profile` | `P` | MLX-DLSS's profile. |
| `--skin-structure` | `X` | How much of skin's own structure it keeps, passed to MLX-DLSS. |
| `--steady` | `A` | Steady the shimmer the network adds: its change to each frame is carried along raea's motion vectors and blended in at this rate, 0 to 1. Lower is steadier; 1 turns it off (0.25 by default). |
| `-o` | `OUT` | The folder the frames go to. |

<!-- generated:end -->
