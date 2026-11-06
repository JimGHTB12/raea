# Settings

raea reads each setting from the environment first, then from `~/.raea/env` (or the file `RAEA_ENV_FILE` names):
one `KEY=value` per line, `#` for comments. Nothing has a hidden default host: a feature that needs a service says
which setting is missing.

```sh
mkdir -p ~/.raea && touch ~/.raea/env && chmod 600 ~/.raea/env
```

## Network

| Setting | What |
|---|---|
| `RAEA_OFFLINE=1` | raea makes no network requests: a remote URL not already in raea's cache, a model or sample library not yet installed, a composition's own `fetch()` or WebSocket, Bevy's http asset source and every `raea gen` step are refused with what wanted out. What's on disk keeps working |

## Generation (`raea gen`, optional)

| Setting | What |
|---|---|
| `FISH_API_KEY` | Your Fish Audio key, for an `audio.speech` step on `fish:<model>` (`fish:` alone is s2.1-pro); a plan quotes Fish's price for the words first |

## Local audio models

Two models make sound on your Mac, free, with no account. Each installs itself the first time a composition or a
command asks for it (a few minutes, once), or ahead of time with `raea setup-audio`.

| Setting | What |
|---|---|
| `RAEA_TOKENSYNTH` | Where TokenSynth lives (default `~/.cache/raea-tokensynth`, about 6 GB): an instrument described in words, one note at a time, for a Tone.js `Sampler` |
| `RAEA_SAO` | Where Stable Audio Open Small lives (default `~/.cache/raea-sao`, about 2 GB, Core ML): foley and sound effects |
| `RAEA_AUDIO_AUTOSETUP=0` | Don't install on first use; a missing model's error names the command (`tools/sao/setup.sh`, `tools/tokensynth/setup.sh`) |

From the command line, `raea sao "PROMPT" -o hit.wav` and `raea tokensynth "TIMBRE" --notes C3,C4 -o
notes/` make sounds through the same cache a composition's render uses, and `raea sounds` searches open sample
libraries (pianos, General MIDI instruments, orchestral and percussion samples, each with its licence).

## Checks and guides (optional)

| Setting | What |
|---|---|
| `CLOUDFLARE_ACCOUNT_ID`, `CLOUDFLARE_API_TOKEN` | The checks' judge model (clef-omni on Workers AI) |
| `RAEA_GUIDES` | A folder of craft references for the agent's `guide` tool (`captions.md`, `motion-rules.md`, ...) |
