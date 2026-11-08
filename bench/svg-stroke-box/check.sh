#!/bin/bash
# #50: an SVG shape's transform-box: stroke-box as Chrome resolves it. index.html moves two stroked rects by
# translateX(50%) of their stroke box: a's stroke from a stylesheet makes that box 120 wide (it moves 60); b's
# vector-effect: non-scaling-stroke makes it the fill box (it moves 50), and b keeps its 20-wide stroke under
# scaleY(0.5) though it has an id of its own. Chrome 148 gives these same pixels. Exits 1 on a failure.
#   bench/svg-stroke-box/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-svg-stroke-box.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
"$BIN" render bench/svg-stroke-box/index.html -o "$OUT/a.png" > "$OUT/log" 2>&1 || { echo "FAIL render (log below)"; sed 's/^/    /' "$OUT/log"; exit 1; }
px() { magick "$OUT/a.png" -format "%[fx:int(255*p{$1}.r+.5)] %[fx:int(255*p{$1}.g+.5)] %[fx:int(255*p{$1}.b+.5)]" info:; }
status=0
near() { # X,Y R G B WHAT: each channel within 8
  local got; read -ra got <<< "$(px "$1")"
  (( ${got[0]} - $2 <= 8 && $2 - ${got[0]} <= 8 && ${got[1]} - $3 <= 8 && $3 - ${got[1]} <= 8 && ${got[2]} - $4 <= 8 && $4 - ${got[2]} <= 8 )) && return
  echo "FAIL at $1: ${got[*]}, wanted $2 $3 $4 ($5)"; status=1
}
near 65,90 255 255 255 "a: white left of its stroke (moved 60, not 50)"
near 75,90 0 0 0 "a: its left stroke"
near 185,90 0 0 0 "a: its right stroke"
near 195,90 255 255 255 "a: white right of its stroke"
near 305,90 0 0 0 "b: its left stroke (moved 50: the fill box)"
near 425,90 255 255 255 "b: white right of its stroke"
near 360,128 0 0 0 "b: its bottom stroke, 20 wide under scaleY(0.5)"
near 360,105 19 80 43 "b: its green fill"
[ $status = 0 ] && echo "PASS transform-box: stroke-box on SVG shapes"
exit $status
