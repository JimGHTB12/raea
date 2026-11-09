---
title: "raea gen"
summary: "Generate images, video, music, sound, speech, 3D models and text through made.app, fal, Replicate, OpenRouter, Modal or ComfyUI."
order: 102
sources: ["io/gen/src", "io/gen-comfy/src"]
---

`raea gen` makes generated media through the services you've connected: made.app, fal, Replicate, OpenRouter, Modal
or a ComfyUI server. One prompt makes one file; a flow (`FLOW.ts` or `FLOW.json`) chains steps, and `plan` says
what a flow will cost before `run` spends anything. `--max-cents` caps what a call may spend. A generated 3D model
is checked and fitted: its real height, its origin at the bottom centre. ComfyUI workflows go both ways: import one
as a flow, export a flow as one, or run a workflow as it is.

```sh
raea gen models --kind image
raea gen image "a ceramic mug on a sunlit table, product photo" --aspect 4:5 -o media/
raea gen mesh "a mid-century armchair" --h 0.8
raea gen check chair.glb --h 0.8 --solid -o chair-fixed.glb
raea gen plan flow.ts && raea gen run flow.ts --max-cents 200
raea gen comfy import workflow.json -o flow.json
raea gen balance
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea gen models [--kind image|video|audio] [--query WORDS] | balance | types [-o FILE] | flow FLOW.ts [-o FLOW.json]
raea gen image|video|music|sfx|speech|mesh|text "PROMPT" [--model M] [--image FILE]... [--seconds S] [--aspect W:H] [-o DIR] [--max-cents N] [--plan]   (mesh: --model tripo:H3.1|tripo:P2|fal:<endpoint>)
raea gen mesh "PROMPT" [--image FILE] [--h METRES] [--dims W,D,H] [--turn DEG]   (the model checked and fitted: its real height, origin at the bottom centre)
raea gen check MODEL.glb [--h METRES] [--dims W,D,H] [--turn DEG] [--faces N | --seen METRES] [--parts a,b] [--rig] [--solid] [-o FIXED.glb] [--json]
raea gen plan FLOW.json|FLOW.ts [--exact] | raea gen run FLOW.json|FLOW.ts [-o DIR] [--max-cents N] [--exact] [--comfy-on-replicate] [--no-local] [--no-tripo]
raea gen comfy import WORKFLOW.json [-o FLOW.json] | comfy export FLOW.json|FLOW.ts [-o WORKFLOW.json] | comfy needs WORKFLOW.json [-o NEEDS.json]
raea gen comfy run WORKFLOW.json [-o DIR]   (any workflow as it is, on the ComfyUI server: its input files uploaded, every output fetched)
raea gen comfy view WORKFLOW.json [-o FILE] | comfy set WORKFLOW.json CHANGES.json   (nodes, settings by name, links, groups; edits written back)
raea gen comfy schema [URL] | comfy coverage DIR | comfy up
```

Headless: it needs nothing running and opens no window. Its arguments go to a parser of its own, whose forms are listed above.

<!-- generated:end -->
