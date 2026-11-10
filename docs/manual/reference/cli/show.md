---
title: "raea show"
summary: "Show the person something in the raea app, as an item they can use and answer: stills, a video, a sound, a script, an interactive .tsx or a question."
order: 124
sources: ["engine/cli/src/show.ts"]
---

`raea show` is how an agent working outside the app, in a terminal session such as Claude Code or Codex, shows the
person its work. It posts the files to the project's feed in the raea app and brings the window up on the new item:
stills become a look (several, a storyboard), a video a watch, a sound a listen, a `NAME.script.ts` or `.md` a read,
and a `.tsx` an interactive artifact (one of the project's compositions opens in its editor). `--question` asks,
with words or stills to pick from. It starts the app if it's installed and not running.

What the person sends back (an answer, a comment, a pick) arrives as JSON lines in the session's inbox file, which
the command prints. Without the app on this Mac, the files open in the Mac's own viewer and it says nothing was
posted. While the app is installed, an agent shows renders, stills, sheets and questions this way, never in Preview
or a browser.

```sh
raea show out/look-1.png out/look-2.png out/look-3.png --ask "Which of these three?"
raea show out/cut.mp4 --replaces i-show-2f9c
raea show --question "Warmer or cooler grade?" --choice Warmer --choice Cooler
```

<!-- generated:begin by tools/manual-ref/cli.ts from `raea help --json`; edit only the flag descriptions -->

## Usage

```sh
raea show FILE... [--title T] [--ask LINE] [--project P] [--replaces ITEM] [--as NAME] [--role ROLE] [--inbox NAME] | raea show --question Q --choice A --choice B [STILL...]   (shown in the raea app as an item the person can use and answer, the window brought up on it: stills (several, a storyboard), a video, a sound, a NAME.script.ts or .md, a .tsx (an interactive artifact; a project's composition opens in its editor), or a question; what they send comes to ~/.raea/inbox/INBOX.jsonl; never open media in Preview while the app is installed)
```

Needs the raea app's host running on this machine; then waits for a person's click in the raea app (to answer). Takes any number of arguments besides its flags.

## Flags

| Flag | Takes | What it does |
| --- | --- | --- |
| `--as` | `NAME` | Who's showing it, as the feed names them (your harness's name when left out: Claude Code, Codex, Gemini). |
| `--ask` | `LINE` | What you want from the person, shown with the item. |
| `--choice` | `A` | With `--question`: one answer to pick; give it once per answer. Stills given instead are the choices. |
| `--inbox` | `NAME` | The inbox the answers go to, `~/.raea/inbox/NAME.jsonl` (one per session when left out). |
| `--project` | `P` | The project to post to: its id, title or `Workspace/project`. The project this folder is in when left out. |
| `--question` | `Q` | Ask the person this, with `--choice` answers or stills to pick from. |
| `--replaces` | `ITEM` | The item this is the next version of, so the feed shows it as that item's new version. |
| `--role` | `ROLE` | With `--as`: the role shown beside the name. |
| `--title` | `T` | The item's title in the feed. |

<!-- generated:end -->
