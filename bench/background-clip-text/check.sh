#!/bin/bash
# #37: background-clip: text clips a background to the element's glyphs, as Chrome does. index.html draws "Gradient"
# three times in a pink-to-cyan gradient on #111, clipped by background-clip (first line), -webkit-background-clip
# alone (second), and background-clip on a block holding the word in a block of its own (third). In each line the G's
# stem must be the gradient's pink, and the box beside and above the word the page's #111 (it was the gradient: the
# box was filled and the text was invisible). Chrome 148 gives these same pixels.
# scrolled.tsx scrolls two boxes of "One Two Three Four" down a line. Only "Three" reaches past x 112, so a strip at x
# 112-140 tells the lines apart: the plain box must show "Two" over "Three" as Chrome does (#46: raea scrolled inline
# text twice, "Three" over "Four"), and the text-clipped one "One" over "Two", Chrome's mask being its text unscrolled.
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
ink() { # X,Y,W,H: whether the strip has text in it (its brightest pixel well above the page's #111)
  [ "$(magick "$OUT/b.png" -crop "$3x$4+$1+$2" +repage -format '%[fx:maxima.r>0.5?1:0]' info:)" = 1 ]
}
strip() { # Y WANT WHAT
  local got=empty; ink 112 "$1" 28 40 && got=ink
  [ "$got" = "$2" ] || { echo "FAIL at x 112-140, y $1: $got, wanted $2 ($3)"; status=1; }
}
status=0
for y in 60 180 300; do
  near 8,$y 251 2 204 "the G's stem, in the gradient"
  near 600,$y 17 17 17 "beside the word: the page"
done
near 200,10 17 17 17 "above the first line's glyphs"
near 200,130 17 17 17 "above the second line's glyphs"
near 200,250 17 17 17 "above the third line's glyphs"
"$BIN" render bench/background-clip-text/scrolled.tsx -o "$OUT/b.png" > "$OUT/log" 2>&1 || { echo "FAIL render scrolled.tsx (log below)"; sed 's/^/    /' "$OUT/log"; exit 1; }
strip 10 empty "the plain scroller's first line: Two"
strip 70 ink "the plain scroller's second line: Three"
strip 130 empty "the clipped scroller's first line: One"
strip 190 empty "the clipped scroller's second line: Two"
[ $status = 0 ] && echo "PASS background-clip: text"
exit $status
