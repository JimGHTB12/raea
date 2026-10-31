<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="content/logo/cat-white.svg">
    <img src="content/logo/cat-black.svg" width="72" alt="raea's cat">
  </picture>
</p>

<h1 align="center">raea</h1>

<p align="center"><b>Render Anything Engine</b> <i>(for Anyone)</i></p>

<p align="center">
  One GPU engine for video, motion, games, 3D and documents, written as React and TypeScript,<br>
  rendered faster than it plays, on the device you already own.
</p>

<p align="center">
  <a href="https://raea.io">raea.io</a> ·
  <a href="https://raea.io/why/">Why we built it</a> ·
  <a href="https://raea.io/learn/">How it works</a> ·
  <a href="docs/settings.md">Settings</a> ·
  <a href="sdk/skills/raea/SKILL.md">Agent skill</a>
</p>

<p align="center">
  <img alt="licence FSL-1.1-ALv2" src="https://img.shields.io/badge/licence-FSL--1.1--ALv2-black">
  <img alt="status canary" src="https://img.shields.io/badge/status-canary-black">
  <img alt="macOS, Apple Silicon" src="https://img.shields.io/badge/macOS-Apple%20Silicon-black">
</p>

---

> **Canary.** raea is in active development and changes daily. It builds and renders on a Mac with Apple Silicon,
> and parts of it are rough. Expect breaking changes, missing docs, and things that only work the way we use them.

## What it is

raea renders compositions: React components that draw a frame for a given moment, written with `raea`.
The engine runs them inside [Bevy](https://bevyengine.org) on the GPU, composites video with fused compute kernels,
encodes in hardware, and keeps everything it drew as data you can query. The same file renders to an MP4, plays live
as a game, or renders one frame for an agent to look at.

```tsx
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";

function Hello() {
  const f = useCurrentFrame();
  return <AbsoluteFill style={{ background: "#000", color: "#fff", fontSize: 120, opacity: interpolate(f, [0, 20], [0, 1]) }}>Hello</AbsoluteFill>;
}

registerRoot(() => <Composition id="hello" component={Hello} width={1920} height={1080} fps={30} durationInFrames={90} />);
```

```sh
raea render hello.tsx -o hello.mp4
```

## Get started

Requirements: macOS on Apple Silicon, [Rust](https://rustup.rs) (stable), [bun](https://bun.sh), Xcode command-line
tools.

```sh
git clone https://github.com/shinyobjectz/raea && cd raea

# the CLI: render, play, run and test compositions (one Cargo workspace, one target/)
cargo build --release -p raea-cli
target/release/raea render sdk/react/bench/games/pong/edit.tsx -o pong.mp4
```

The raea app's agent needs a model. Put a key in `~/.raea/env`; OpenRouter is the default, and Anthropic, Z.ai, DeepSeek,
Moonshot or any Anthropic-compatible endpoint work through `RAEA_LLM_PROVIDER`:

```sh
mkdir -p ~/.raea && echo "OPENROUTER_API_KEY=sk-or-..." >> ~/.raea/env
```

Everything else is optional; [docs/settings.md](docs/settings.md) lists every setting, and how to run the small API
behind generated music and speech yourself.

## Use it from your agent

- **Agent skill.** `sdk/skills/raea/SKILL.md` teaches any agent to write, render and check raea compositions.
  `sdk/skills/raea-bevy` covers 3D and games drawn by Bevy.
- **Claude Code plugin.** `/plugin marketplace add shinyobjectz/raea`, then `/plugin install raea@raea`.
- **MCP.** With the raea app running, `claude mcp add --transport http raea-app http://127.0.0.1:7457/mcp` gives an agent its
  tools: `state`, `screenshot`, `send`, `act`, `wait`, `logs`, `hear`, `say`.

## What's in here

| Folder | What |
|---|---|
| [`engine/render/`](engine/render) | `raea-render`: the Bevy renderer an edit's rows draw through, the GPU encode path, the media verbs |
| [`runtime/js/`](runtime/js) | React and HTML on bevy_ui: the page a composition's 2D draws on, tested against Chrome |
| [`core/k/`](core/k), [`systems/k/`](systems/k) | The render path's kernels in a small K dialect: interpreted on the CPU, compiled to WGSL, bit-exact |
| [`io/media/`](io/media) | Decode and encode behind one seam: VideoToolbox, software codecs, AV1, a muxer |
| [`systems/mix/`](systems/mix), [`systems/tone/`](systems/tone), [`engine/seedling/`](engine/seedling) | One mixer core for live play and offline render, Tone's instruments and effects on it, and bevy_seedling on it |
| [`core/animate/`](core/animate), [`engine/bevy-plugins/`](engine/bevy-plugins) | Frame-pure animation (easing, tweens, springs), and Bevy plugins compiled into the engine |
| [`core/calls/`](core/calls), [`engine/native/`](engine/native), [`io/shell/`](io/shell), [`engine/modules/`](engine/modules) | The native-call registry, its JavaScriptCore binding, the shell's natives, and the in-process TypeScript loader |
| [`engine/bundle/`](engine/bundle) | The in-process TypeScript bundler (Rolldown) |
| [`engine/sandbox/`](engine/sandbox) | A WASI sandbox for command-line tools on the iPhone (experimental) |
| [`gpu/device/`](gpu/device), [`io/trace/`](io/trace) | The one shared GPU device and its telemetry, and the flight recorder each run leaves |
| [`sdk/raea/`](sdk/raea) | `raea`, the TypeScript SDK: its spec, conformance suite and source |
| [`sdk/react/`](sdk/react) | `raea`: runs `raea` compositions in Bevy; render, play, run, test; the iPhone build |
| [`sdk/shell/`](sdk/shell) | The agent's shell |
| [`sdk/skills/`](sdk/skills) | Agent skills for working in raea |
| [`engine/cli/`](engine/cli) | The agent's `raea` command |
| [`systems/footage/`](systems/footage) | Reading footage the way an editor does |
| [`systems/ml/`](systems/ml), [`systems/speech/`](systems/speech) | Models on ONNX Runtime, and Pocket TTS speech on it |
| [`tools/`](tools) | Build helpers: the JIT-signing linker wrapper, setup scripts |
| [`docs/`](docs) | Cross-cutting docs and [settings](docs/settings.md) |

Each folder has a README saying what it is and what it depends on. Behaviour specs written in Gherkin (`.feature`)
sit beside the code they specify.

## Licence

raea is under the [Functional Source License 1.1, ALv2 Future License](LICENSE) (FSL-1.1-ALv2), copyright Shane
Murphy: use it, change it, build on it and pass it on for anything except a product that competes with raea. Each
version becomes Apache-2.0 two years after its release. Third-party parts keep their own licences; see [NOTICE](NOTICE).
