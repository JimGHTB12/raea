---
title: "raea init"
summary: "Add a project's starting files where they're missing: edit.tsx, cuts.ts, components/ and AGENTS.md."
order: 22
sources: ["engine/cli/src/index.ts"]
---

`raea init` writes the files a project starts from, in the current folder, and only the ones that aren't there yet:
`edit.tsx` (the composition), `cuts.ts`, `components/` and `AGENTS.md` (what an agent working here should know).
Running it again in a project that has them all says there's nothing to make. It never overwrites a file.

```sh
mkdir spring && cd spring && raea init
```

For a project inside a workspace, with its brief and folders, use [`raea project new`](project.md).

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea init   (a project's files where they're missing: edit.tsx, cuts.ts, components/, AGENTS.md)
```

Headless: it needs nothing running and opens no window. Takes no arguments besides its flags.

<!-- generated:end -->
