---
title: "raea hook"
summary: "Claude Code's PostToolUse hook: the quick text check after an agent saves a composition."
order: 48
sources: ["apps/check/src/quick.rs"]
---

`raea hook claude` is meant to be run by Claude Code, not by hand. Installed as a PostToolUse hook (`raea skills
install` sets it up), it runs `raea check --quick` on a composition each time the agent saves one, and hands the
findings back to the agent, so clipped or cramped text is caught on the edit that caused it.

```sh
raea skills install --agent claude     # installs the skill and this hook
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea hook claude   (Claude Code's PostToolUse hook: the quick check after a composition is saved)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

<!-- generated:end -->
