---
title: "raea check"
summary: "Check a composition's text in a second, or an export against the floor: text, contrast, safe areas, freezes, flashing, cues and loudness."
order: 42
sources: ["apps/check/src/lib.rs", "apps/check/src/quick.rs"]
---

`raea check` has two speeds. On a composition with `--quick`, it renders three stills and measures the text:
clipped, cramped, inside the safe areas, large enough to read. That takes about a second, so run it after every save.
On an exported video, it measures everything the floor asks of a finished piece. Each finding is ranked P0 (broken)
to P3 (polish), says where it is and how to fix it, and the command exits 1 while a P0 or P1 stands.

```sh
raea check piece.tsx --quick --platform tiktok
raea render piece.tsx -o piece.mp4 --manifest render.json
raea check piece.mp4 --manifest render.json --words words.json --platform tiktok -o check.json
```

What a full check can see depends on what you give it. The file alone gives loudness, true peak, sound that stops
dead, frozen stretches and blotching in dark areas. The render's manifest adds text clipped off the frame, text under
the platform's buttons, captions held too briefly and caption contrast. The transcript adds cues away from their
word and music fighting the voice; the perception adds text over a face. A taste file adds the person's own rules,
each a P1 when broken, since their rule beats the floor's default.

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea check EDIT.tsx --quick [--shape S] [--platform P] [--taste T] [-o check.json]   (three stills of a composition checked for text: clipped, cramped, safe areas, sizes; about a second)
raea check VIDEO.mp4 [--manifest render.json] [--words W.json] [--perception P.json] [--shape 16:9|9:16|4:5|1:1] [--platform tiktok|reels|shorts] [--taste taste.ts|DIR] [-o check.json]   (an export against the floor: clipped text, contrast, safe areas, freezes, flashing, cues, loudness, blotching, a placeholder brand kit; P0-P3)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--manifest` | `render.json` | The manifest `raea render --manifest` wrote for this export: where each element was on each frame, for the text, contrast and safe-area checks. |
| `--perception` | `P.json` | Where the face is on each frame (the `perception.json` `raea cut` writes), to find text over a face. |
| `--platform` | `P` | Check the safe areas against this platform's buttons and captions. |
| `--quick` | – | Check a composition's text from three stills instead of checking an export. |
| `--shape` | `S` | The delivery's aspect ratio, for its safe areas. |
| `--taste` | `T` | The person's taste files: the rules in them that carry a `check` are applied too. |
| `--words` | `W.json` | The transcript, to check cues against their words and the voice against the music. |
| `-o` | `check.json` | Write the findings as JSON, for a later review or for `raea next`. |

<!-- generated:end -->

## See also

- [Rendering and checking](../../concepts/rendering.md) lists what the full check measures and how loudness is set.
