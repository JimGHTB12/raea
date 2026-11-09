---
title: "raea export-game"
summary: "Package a game as a program of its own: a Mac app or a command-line program."
order: 4
sources: ["apps/game/src/export.rs"]
---

`raea export-game` packs a game's bundle, its media and fonts, and raea's game player into one program that runs
without raea installed. Name the output `Something.app` for a Mac app; anything else becomes a folder with a command
in it. The program is signed ad hoc so it runs on this Mac; to hand it to others it needs a Developer ID signature and
notarization.

```sh
raea export-game game.tsx -o "Ball Run.app" --icon icon.png
raea export-game game.tsx -o ball-run --target mac-cli
```

The exported program can test itself: it plays its own run with the key sets of `--keys` and reports how it went.
A game that plays notes or instruments must declare `@raea/audio` in its `raea.plugins`, or the export stops
rather than ship a silent game.

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea export-game EDIT.tsx -o OUT [--target mac-app|mac-cli] [--name NAME] [--icon PNG] [--keys SETS] [--player BIN] [--no-strip]   (a game as a program of its own: a Mac app or a command)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--icon` | `PNG` | A square PNG made into the app's icon (an `.icns` in the bundle). Mac apps only. |
| `--keys` | `SETS` | The key sets the program's self-test cycles through: sets split by commas, keys in a set joined by `+`, an empty set for none, such as `ArrowRight,ArrowRight+ArrowUp,`. Arrows, space and a run-and-jump by default. |
| `--name` | `NAME` | The program's name, as the Dock and menu bar show it (from the file name by default). |
| `--no-strip` | – | Keep the player's symbols instead of stripping them, for debugging a crash. The symbols are kept in `~/Library/Caches/raea/dsym` either way. |
| `--player` | `BIN` | The game player binary to ship, in place of the newest `raea-game` beside this raea. |
| `--target` | `mac-app\|mac-cli` | A Mac app bundle or a command-line program; from `-o` by default (`.app` is an app). |
| `-o` | `OUT` | Where the program goes: `NAME.app`, or a folder. |

<!-- generated:end -->
