#!/usr/bin/env bash
# raea-web parity (docs/web-export.md, "raea's own kinds"): each case in test/parity/cases.ts, the kinds three.js can't
# draw (sdk/r3f/src/three/kinds.ts "engine"), rendered by `raea render` and by raea-web in headless Chrome on WebGPU
# (tools/shoot.ts), compared by SSIM. Fails under $MIN (0.92). Needs raea, Chrome, ImageMagick and sdk/web/wasm
# (tools/wasm.sh builds it).
#   tools/parity.sh [OUT_DIR]
set -euo pipefail
cd "$(dirname "$0")/.."
OUT=${1:-$(mktemp -d)}
MIN=${MIN:-0.92}
[ -f wasm/raea_web_bg.wasm ] || tools/wasm.sh
mkdir -p "$OUT"
export RAEA_SIGN=0 RAEA_MARK=0
CASES=$(bun -e 'const { CASES } = await import("./test/parity/cases.ts"); for (const c of CASES) console.log(`${c.name} ${c.edit} ${c.frame}`)')
while read -r name edit frame; do
  (cd test/parity && raea render "$edit" --composition main --frame "$frame" -o "$OUT/$name-native.png" >/dev/null)
done <<< "$CASES"
bun tools/shoot.ts "$OUT"
fail=0
while read -r name _ _; do
  # ImageMagick 7.1.2 reports SSIM's distance (1 - SSIM)
  d=$(magick compare -metric SSIM "$OUT/$name-native.png" "$OUT/$name-web.png" null: 2>&1 | sed -E 's/.*\(([0-9.e-]+)\).*/\1/' || true)
  s=$(echo "1 - $d" | bc -l)
  # a blank web frame fails whatever its SSIM (a mostly dark native frame would otherwise pass)
  [ "$(magick "$OUT/$name-web.png" -format "%[fx:maxima]" info:)" = 0 ] && s=0
  ok=$(echo "$s >= $MIN" | bc -l)
  printf '%-15s SSIM %.4f %s\n' "$name" "$s" "$([ "$ok" = 1 ] && echo ok || echo LOW)"
  [ "$ok" = 1 ] || fail=1
done <<< "$CASES"
exit $fail
