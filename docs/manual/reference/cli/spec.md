---
title: "raea spec"
summary: "Print a spec file's default export as JSON, checked against its schema: briefs, scripts, taste, plans and controllers."
order: 26
sources: ["engine/cli/src/spec.rs"]
---

raea's specs are TypeScript files whose default export is data: `brief.ts`, `NAME.script.ts`, `taste.ts`,
`plan.ts` and `controllers/ID.ts`. `raea spec` evaluates one, checks it against its schema and prints it as JSON,
so an agent can read a brief without running anything else, and a wrong field is caught before it matters.

```sh
raea spec brief.ts
raea spec launch.script.ts
raea spec notes.ts --as taste
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea spec FILE.ts [--as brief|script|taste|plan|controller]   (a spec's default export as JSON, checked against its schema: brief.ts, script.ts, taste.ts, plan.ts, controllers/ID.ts)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--as` | `brief\|script\|taste\|plan\|controller` | Which schema to check it against. By default from the file's name (`brief`, `script`, `plan`, a `controllers/` folder), and `taste` otherwise. |

<!-- generated:end -->
