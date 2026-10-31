# react

`raea-runtime-react` and its CLI, `raea`: the engine that runs [`raea`](../motion) compositions. React runs on
JavaScriptCore inside Bevy; each frame's tree is read back and lowered to [`raea-render`](../../engine/render) rows, which Bevy draws.

    raea render EDIT.tsx -o OUT.mp4 | OUT.png | DIR   [--frame N | --at A,B]
    raea play EDIT.tsx           play live in a window (games take keyboard and pointer input)
    raea run SCRIPT.ts           run a script's default export inside the engine
    raea test FILE...            run bun:test-style files inside the engine
    raea types -o FILE.ts        write the native calls' TypeScript types

- `src/`: the engine (`motion.rs` and its modules; page, sound, live and export are runtime/ crates); the `raea` binary is engine/cli (crate raea-cli).
- `js/`: the JavaScript side of the bridge (bundles are made by raea-bundle's library: Rolldown, in process).
- `ios/`: the same engine on the iPhone, as a library a probe app links.
- `bench/`: example games (`games/`), a render-vs-play corpus (`corpus/`), timeline checks and a playtest prototype.

Build with `cargo build --release -p raea-cli` from the repository root. Depends on [`runtime/js/`](../../runtime/js),
[`engine/render/`](../../engine/render), [`io/media/`](../../io/media), [`engine/native/`](../../engine/native), [`engine/modules/`](../../engine/modules), and optionally
the sandbox's `wsx`.

Status: in use; the main way to render with raea.
