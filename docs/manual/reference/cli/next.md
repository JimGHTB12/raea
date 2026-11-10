---
title: "raea next"
summary: "The two or three most useful next steps for a project, read from its brief, compositions, renders and checks."
order: 47
sources: ["apps/workspace/src/next.rs"]
---

`raea next` reads a project's files (what its brief still needs, the storyboard its modes want, its compositions,
the renders in `out/` and their checks) and prints the two or three steps worth doing next. It's a menu for the agent
or the person to pick from; nothing runs.

```sh
raea next
raea next ~/Documents/raea/acme/projects/spring-launch
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea next [DIR]   (the two or three useful next steps for the project, from its brief, compositions, renders and checks)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

<!-- generated:end -->
