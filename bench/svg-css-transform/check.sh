#!/bin/bash
# #60: an SVG shape's transform, transform-box and transform-origin from a <style> inside its <svg> turn it, as Chrome
# does. A composition's <style> there goes to usvg in the SVG's markup, and usvg applied that sheet's transform and
# transform-origin over the transform raea had computed from the cascade (`rotate(30deg)`, which SVG's syntax can't
# read: the shape drew unturned). sheet.tsx's class-turned square (red) must be the inline-turned one (green) moved
# 600 px (each recoloured black, so only its shape counts), and mixed.tsx's class box and origin with an inline turn
# must be the all-inline square. Before: 8827 and 5598 px differed. Exits 1 on a failure.
#   bench/svg-css-transform/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-svg-css-transform.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
status=0
for c in sheet mixed; do
  "$BIN" render bench/svg-css-transform/$c.tsx --at 0 -o "$OUT/$c" > "$OUT/$c.log" 2>&1 && [ -f "$OUT/$c/frame-00000.png" ] || { echo "FAIL $c: the render failed"; sed 's/^/    /' "$OUT/$c.log" | tail -4; exit 1; }
done
same() { # PNG LEFT-COLOUR RIGHT-COLOUR MAX WHAT
  magick "$1" -crop 300x300+0+0 +repage -fuzz 35% -fill black -opaque "$2" "$OUT/a.png"
  magick "$1" -crop 300x300+600+0 +repage -fuzz 35% -fill black -opaque "$3" "$OUT/b.png"
  local d; d=$(magick compare -metric AE -fuzz 10% "$OUT/a.png" "$OUT/b.png" null: 2>&1 | cut -d' ' -f1)
  (( ${d%.*} <= $4 )) && echo "ok   $5 ($d px differ)" || { echo "FAIL $5: $d px differ, at most $4"; status=1; }
}
same "$OUT/sheet/frame-00000.png" "#e0245e" "#17bf63" 300 "a class's transform, box and origin turn it as inline ones do"
same "$OUT/mixed/frame-00000.png" "#000000" "#000000" 50 "a class's box and origin with an inline turn"
[ $status = 0 ] && echo "PASS SVG transforms from a stylesheet"
exit $status
