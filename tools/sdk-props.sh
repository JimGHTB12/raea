#!/bin/bash
# Every exported prop of the SDK's engine components is read (tools/sdk-props, docs/sdk.feature S8c).
#
#   tools/sdk-props.sh            the unread, one a line, then the count
#   tools/sdk-props.sh --check    the ratchet (tools/agents/check): no more unread than tools/sdk-props.max
#
# Built on first use and whenever its source is newer than the binary, in a lane and target dir of its own (as
# tools/rask.sh): a small compile every L0 needs shouldn't queue behind the release builds on target/.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
T=$ROOT/target/sdk-props
bin=$T/release/raea-sdk-props
if [ ! -x "$bin" ] || [ -n "$(find "$ROOT/tools/sdk-props" -newer "$bin" 2>/dev/null | head -1)" ]; then
  if command -v lane >/dev/null; then lane sdk-props -- cargo build --release -q -p raea-sdk-props --target-dir "$T" --manifest-path "$ROOT/Cargo.toml" >&2
  else cargo build --release -q -p raea-sdk-props --target-dir "$T" --manifest-path "$ROOT/Cargo.toml" >&2; fi
fi
exec "$bin" "$@" "$PWD"
