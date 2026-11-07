#!/bin/bash
# The TypeScript table of native calls (sdk/raea/src/native-calls.ts) against the registry it's generated from:
# exits 1 when `raea types` (every host's registered calls, from the raea binary) prints something else, so a call
# added or changed in Rust can't go untyped in TS. It reads the built binary, so it belongs after a build of main
# (L2), not in L0. `tools/native-types.sh --write` regenerates the file.
set -u
ROOT=$(cd "$(dirname "$0")/.." && pwd)
F=$ROOT/sdk/raea/src/native-calls.ts
RAEA=${RAEA_BIN:-$ROOT/target/release/raea}
[ -x "$RAEA" ] || { echo "native-types: no raea binary at $RAEA (build -p raea-cli, or set RAEA_BIN)"; exit 2; }
out=$("$RAEA" types 2>/dev/null) && [ -n "$out" ] || { echo "native-types: raea types printed nothing"; exit 2; }
if [ "${1:-}" = --write ]; then printf '%s\n' "$out" > "$F"; echo "native-types: wrote $F"; exit 0; fi
if d=$(diff -B <(printf '%s\n' "$out") "$F"); then echo "native-types: ok"; else
  echo "native-types: sdk/raea/src/native-calls.ts is out of date with the registry (tools/native-types.sh --write):"
  echo "$d" | grep -E '^[<>] +"' | sed 's/^< /  missing in TS: /; s/^> /  only in TS:    /' | cut -c1-120
  exit 1
fi
