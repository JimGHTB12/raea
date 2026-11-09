---
title: "raea song"
summary: "Read a song's energy map: beats, bars, sections, builds, drops, stops, hits and cuts per bar."
order: 82
sources: ["apps/tools/src/song.rs"]
---

`raea song` listens to a song and writes its energy map, the file `song()`, `useSong()` and `<SongPlot>` read: the
beats, downbeats and bars, each beat's loudness, onsets, brightness and low end, the sections, and the moments that
matter to an edit (builds, drops, stops, hits, dips and breaks) with how many cuts each bar can carry. Cut to it and
the edit moves with the music.

```sh
raea song media/track.m4a                                # media/track.song.json beside it
raea song media/track.m4a --plot track.png               # and a picture of the map, to check it by eye
raea song media/track.wav --srt none -o map.json
```

Sections come from the song's own `[Section]` markers when it has them: an `.srt` named by `--srt`, else one beside
the file, else the file's timed-text track (an `.m4a`'s lyrics). Without markers they're found from where the music
changes.

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea song FILE [-o OUT.song.json] [--srt S.srt|none] [--plot OUT.png] [--size 2400x1350]   (the song's energy map: beats, sections, builds, drops, stops, hits, cuts per bar; also: --backend NAME)
```

Headless: it needs nothing running and opens no window. Takes up to 1 argument besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--backend` | `VALUE` | Which of raea's decoders reads the audio (`apple` on a Mac by default). Rarely needed. |
| `--plot` | `OUT.png` | Also draw the map with `<SongPlot>` to a PNG, so a person can check it against the song. |
| `--size` | `2400x1350` | The plot's size in pixels. |
| `--srt` | `S.srt\|none` | The subtitle file whose `[Section]` markers name the sections, or `none` to ignore markers and find sections from the music. |
| `-o` | `OUT.song.json` | Where the map goes (the song's name with `.song.json`, beside it, by default). |

<!-- generated:end -->
