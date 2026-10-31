#!/bin/bash
# S2d's done line on the engine (docs/sdk.feature): mixed.tsx's query, which mixes a declared component and a
# plugin's generated one, run on the engine's World in a raea render; its bars' ends read back. Under bun the same
# query is conformance/data.test.ts's. Needs target/release/raea.
set -u
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
WORK=$(mktemp -d "${TMPDIR:-/tmp}/data-bench.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
"$ROOT/target/release/raea" render "$ROOT/sdk/raea/bench/bevy-plugins/mixed.tsx" -o "$WORK/f.png" --frame 0 > "$WORK/render.log" 2>&1 || { echo "mixed: FAIL (render: $(tail -1 "$WORK/render.log"))"; exit 1; }
px() { ffmpeg -v error -i "$WORK/f.png" -vf "crop=1:1:$1:$2" -f rawvideo -pix_fmt rgb24 - 2>/dev/null | xxd -p; }
green() { case "$1" in 0[0-9a-f]f[0-9a-f]0[0-9a-f]|0[0-9a-f]e[0-9a-f]0[0-9a-f]) return 0 ;; *) return 1 ;; esac; }
dark() { case "$1" in 0[0-9a-f]0[0-9a-f]0[0-9a-f]) return 0 ;; *) return 1 ;; esac; }
# Position.x is 1 (a 100 px bar), LinearVelocity[2] is 9 (90 px), both green (the engine's World)
if green "$(px 98 30)" && dark "$(px 102 30)" && green "$(px 88 70)" && dark "$(px 92 70)"; then
  echo "mixed: PASS one query over a declared and a generated component on the engine's World"
else
  echo "mixed: FAIL (row 30 at 98/102: $(px 98 30)/$(px 102 30); row 70 at 88/92: $(px 88 70)/$(px 92 70))"; exit 1
fi
