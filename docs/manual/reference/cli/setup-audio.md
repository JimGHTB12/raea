---
title: "raea setup-audio"
summary: "Install or check the local audio models: Stable Audio Open for foley and TokenSynth for instruments."
order: 86
sources: ["apps/tools/src/sound.rs"]
---

`raea sao` and `raea tokensynth` install their models the first time they're used. `raea setup-audio` does it on
purpose, ahead of time, and says where each lives and how much disk it takes. Name one model to set up only that one.

```sh
raea setup-audio
raea setup-audio --check
raea setup-audio tokensynth
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea setup-audio [--check] [sao|tokensynth]   (install or check the local audio models)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--check` | – | Say whether each model is installed, where it lives and how much disk it takes, without installing anything. |

<!-- generated:end -->
