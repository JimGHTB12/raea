---
title: "raea cut"
summary: "Cut a take to the lines it says, transcribing it first, and write cuts.ts and words.json."
order: 63
sources: ["engine/cli/src/index.ts"]
---

`raea cut` cuts a take under the project's `media/` folder by what it says: give it the lines you want and it cuts
in the pauses around each one. A take without a `TAKE.words.json` is transcribed first, on this machine. It writes
`cuts.ts` (the cut) and `words.json` (every word kept, with its timing) beside the edit, and `raea render` reads
both from there, so captions and cues follow the cut without any wiring.

```sh
raea cut interview.mp4 "We started in a garage" "Now we ship to forty countries"
raea cut interview.mp4 --at 12.4-18.9 --at 31-36.2
raea cut broll.mp4 --at 4-7 --append
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea cut TAKE "line" ... | --at A-B ... [--append]   (cuts.ts and words.json: a take under media/ cut in the pauses around each line it says, transcribed first if it has no TAKE.words.json; or at those seconds)
```

Headless: it needs nothing running and opens no window. Takes any number of arguments besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--append` | – | Add these lines or stretches to the end of the existing cut, instead of replacing it. |
| `--at` | `A-B` | Cut at these seconds (`12.4-18.9`) instead of by lines. Repeatable. |

<!-- generated:end -->

## See also

- [Cutting by words](../../guides/words.md) cues parts on phrases and builds captions that follow the cut.
