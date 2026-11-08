#!/bin/bash
# #37: background-clip: text clips a background to the element's glyphs, as Chrome does. index.html draws "Gradient"
# three times in a pink-to-cyan gradient on #111, clipped by background-clip (first line), -webkit-background-clip
# alone (second), and background-clip on a block holding the word in a block of its own (third). In each line the G's
# stem must be the gradient's pink, and the box beside and above the word the page's #111 (it was the gradient: the
# box was filled and the text was invisible). Chrome 148 gives these same pixels.
# Exits 1 on a failure.
#   bench/background-clip-text/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-bg-clip-text.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
"$BIN" render bench/background-clip-text/index.html -o "$OUT/a.png" > "$OUT/log" 2>&1 || { echo "FAIL render (log below)"; sed 's/^/    /' "$OUT/log"; exit 1; }
px() { magick "$OUT/a.png" -format "%[fx:int(255*p{$1}.r)] %[fx:int(255*p{$1}.g)] %[fx:int(255*p{$1}.b)]" info:; }
near() { # X,Y R G B: each channel within 6
  local got; read -ra got <<< "$(px "$1")"
  (( ${got[0]} - $2 <= 6 && $2 - ${got[0]} <= 6 && ${got[1]} - $3 <= 6 && $3 - ${got[1]} <= 6 && ${got[2]} - $4 <= 6 && $4 - ${got[2]} <= 6 )) && return
  echo "FAIL at $1: ${got[*]}, wanted $2 $3 $4 ($5)"; status=1
}
status=0
for y in 60 180 300; do
  near 8,$y 251 2 204 "the G's stem, in the gradient"
  near 600,$y 17 17 17 "beside the word: the page"
done
near 200,10 17 17 17 "above the first line's glyphs"
near 200,130 17 17 17 "above the second line's glyphs"
near 200,250 17 17 17 "above the third line's glyphs"
[ $status = 0 ] && echo "PASS background-clip: text"
exit $status
