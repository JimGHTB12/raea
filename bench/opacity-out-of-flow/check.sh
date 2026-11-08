#!/bin/bash
# #39: an absolutely positioned child is faded with an unpositioned parent's opacity, as Chrome paints it. index.html
# draws four black 160 px boxes in groups at opacity 0.3 on white; each box's centre must be Chrome's grey (178), and
# the fourth box must stay where its containing block puts it though its wrapper is moved down (white above it, at
# y 30, and grey at its top edge, y 45). Exits 1 on a failure.
#   bench/opacity-out-of-flow/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-opacity-oof.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
"$BIN" render bench/opacity-out-of-flow/index.html -o "$OUT/a.png" > "$OUT/log" 2>&1 || { echo "FAIL render (log below)"; sed 's/^/    /' "$OUT/log"; exit 1; }
grey() { magick "$OUT/a.png" -format "%[fx:int(255*p{$1}.r+0.5)]" info:; }
status=0
want() { # X,Y GREY WHAT: within 3
  local g; g=$(grey "$1")
  (( g - $2 <= 3 && $2 - g <= 3 )) || { echo "FAIL at $1: $g, wanted $2 ($3)"; status=1; }
}
want 120,120 178 "1: an absolute child of an unpositioned wrapper"
want 320,120 178 "2: an in-flow child"
want 520,120 178 "3: an absolute child of a positioned wrapper"
want 720,120 178 "4: as 1, its wrapper moved down"
want 720,30 255 "4: above the box, white"
want 720,45 178 "4: the box's top edge, where its containing block puts it"
[ $status = 0 ] && echo "PASS opacity over out-of-flow children"
exit $status
