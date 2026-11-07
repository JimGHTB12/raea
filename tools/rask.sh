#!/bin/bash
# rask (tools/rask, a submodule: github.com/shinyobjectz/rask): tasks are feature-file scenarios, marked in the Rust
# with `// rask: ID what`, claimed by lanes, closed only when checked (docs/agents.md, "Tasks"). Built on first use and
# whenever its source is newer than the binary, in a lane of its own: a small compile that every L0 needs shouldn't queue
# behind the release builds (10-28: two L1s waited on it behind 40-minute builds).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
R=$ROOT/tools/rask
[ -f "$R/Cargo.toml" ] || git -C "$ROOT" submodule update --init -q tools/rask
bin=$R/target/release/rask
if [ ! -x "$bin" ] || [ -n "$(find "$R/src" "$R/Cargo.toml" -newer "$bin" 2>/dev/null | head -1)" ]; then
  if command -v lane >/dev/null; then lane rask -- cargo build --release -q --manifest-path "$R/Cargo.toml" >&2
  else cargo build --release -q --manifest-path "$R/Cargo.toml" >&2; fi
fi
exec "$bin" "$@"
