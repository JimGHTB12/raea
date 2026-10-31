# Settings

raea reads each setting from the environment first, then from `~/.raea/env` (or the file `RAEA_ENV_FILE` names):
one `KEY=value` per line, `#` for comments. Nothing has a hidden default host: a feature that needs a service says
which setting is missing.

```sh
mkdir -p ~/.raea && touch ~/.raea/env && chmod 600 ~/.raea/env
```

## The agent's model

The raea app's agent streams the Anthropic Messages API, so any provider that serves it works. Pick one with
`RAEA_LLM_PROVIDER`:

| `RAEA_LLM_PROVIDER` | Endpoint | Key variable | Model |
|---|---|---|---|
| `openrouter` (default) | `https://openrouter.ai/api` | `OPENROUTER_API_KEY` | `RAEA_LLM_MODEL`, default `z-ai/glm-5.3-flash` |
| `anthropic` | `https://api.anthropic.com` | `ANTHROPIC_API_KEY` | `RAEA_LLM_MODEL` (required) |
| `zai` | `https://api.z.ai/api/anthropic` | `ZAI_API_KEY` | `RAEA_LLM_MODEL` (required) |
| `deepseek` | `https://api.deepseek.com/anthropic` | `DEEPSEEK_API_KEY` | `RAEA_LLM_MODEL` (required) |
| `moonshot` | `https://api.moonshot.ai/anthropic` | `MOONSHOT_API_KEY` | `RAEA_LLM_MODEL` (required) |
| `custom` | `RAEA_LLM_BASE_URL` | `RAEA_LLM_API_KEY` | `RAEA_LLM_MODEL` (required) |

`RAEA_LLM_API_KEY` overrides the provider's own key variable. For a `custom` endpoint that wants
`Authorization: Bearer` instead of `x-api-key`, set `RAEA_LLM_AUTH=bearer`.

```sh
# ~/.raea/env
RAEA_LLM_PROVIDER=anthropic
RAEA_LLM_MODEL=claude-sonnet-5-5
ANTHROPIC_API_KEY=...
```

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
