---
title: "raea physics"
summary: "Make the bakes <Physics> and <Goo> play: rigid bodies, soft goo, and parts cut out of a .glb."
order: 103
sources: ["runtime/react/src/scene/physics.rs", "systems/bake/src"]
---

A `<Physics>` or `<Goo>` in a composition plays a bake: the simulation run once ahead of time, so every render of it
is the same. A render makes the bakes it needs on its own; `raea physics` makes one by hand from its request, in the
composition's asset store. `part` cuts named nodes out of a `.glb` so each can be its own body.

```sh
raea physics bake request.json -o bake.json
raea physics goo spec.json
raea physics part media/kit.glb Wheel_L Wheel_R
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea physics bake REQUEST.json [-o OUT.json] | physics goo SPEC.json | physics part SRC.glb NODE...
```

Headless: it needs nothing running and opens no window. Its arguments go to a parser of its own, whose forms are listed above.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `-o` | `OUT.json` | Also write the result to this file, in place of printing it. |

<!-- generated:end -->

## See also

- [3D](../../guides/3d.md) shows physics in a scene.
