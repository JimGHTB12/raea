---
title: "raea controller"
summary: "Read and change the workspace's controllers: its settings and workflows, each a TypeScript file."
order: 25
sources: ["apps/workspace/src/controllers.rs"]
---

Controllers are a workspace's settings and workflows, each `controllers/ID.ts` exporting `controller({...})` from
`raea/brief`. A field is named by its path, `controller.group.field`. A project can override a field for itself,
and its value then wins, as git config's scopes do. Every change is a commit, by "you" unless `--by` names someone.

```sh
raea controller ls
raea controller get delivery.format.aspect-ratio
raea controller set delivery.format.aspect-ratio 9:16 --by agent
raea controller set delivery.format.aspect-ratio 16:9 --project     # this project only
raea controller write controllers/delivery.ts
raea controller rm delivery
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea controller ls [--json] | raea controller get PATH [--json] | raea controller set PATH VALUE [--project] [--by NAME] | raea controller write FILE.ts [--by NAME] | raea controller rm ID [--by NAME]   (the workspace's controllers, controllers/ID.ts; PATH is controller.group.field; every change a commit)
```

Headless: it needs nothing running and opens no window. Takes up to 3 arguments besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--by` | `NAME` | Who made the change, for its commit (`you` by default). An agent passes its own name. |
| `--json` | – | With `ls` and `get`: print JSON instead of text. |
| `--project` | – | With `set`: change the value for the current project only, in its `.raea/project`. |

<!-- generated:end -->
