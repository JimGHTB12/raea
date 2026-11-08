#!/bin/bash
# #45: an inline SVG's fill and stroke from a stylesheet reach usvg, as Chrome draws them. index.html draws five
# squares: a and b styled by stylesheet rules, c by attributes, each a green outline with a white inside; d a filled
# orange square (its attribute beats the fill its svg's rule gives it); e a filled green one (its rule beats its
# attribute). Chrome 148 gives these same pixels. Exits 1 on a failure.
#   bench/svg-css-paint/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-svg-css-paint.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
"$BIN" render bench/svg-css-paint/index.html -o "$OUT/a.png" > "$OUT/log" 2>&1 || { echo "FAIL render (log below)"; sed 's/^/    /' "$OUT/log"; exit 1; }
px() { magick "$OUT/a.png" -format "%[fx:int(255*p{$1}.r+.5)] %[fx:int(255*p{$1}.g+.5)] %[fx:int(255*p{$1}.b+.5)]" info:; }
status=0
near() { # X,Y R G B WHAT: each channel within 8
  local got; read -ra got <<< "$(px "$1")"
  (( ${got[0]} - $2 <= 8 && $2 - ${got[0]} <= 8 && ${got[1]} - $3 <= 8 && $3 - ${got[1]} <= 8 && ${got[2]} - $4 <= 8 && $4 - ${got[2]} <= 8 )) && return
  echo "FAIL at $1: ${got[*]}, wanted $2 $3 $4 ($5)"; status=1
}
for x in 0 120 240; do
  near $((x + 20)),70 19 80 43 "square at $x: its green stroke"
  near $((x + 50)),70 255 255 255 "square at $x: white inside (fill: none)"
done
near 410,70 240 128 0 "d: its attribute's orange over the fill its svg's rule gives it"
near 530,70 19 80 43 "e: its rule's green over its attribute"
grep -q "vector-effect" "$OUT/log" && { echo "FAIL the render says vector-effect wasn't applied"; status=1; }
[ $status = 0 ] && echo "PASS SVG paint from stylesheets"
exit $status
