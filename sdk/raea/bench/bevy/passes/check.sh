#!/bin/bash
# The composition passes fixture's checks (edit.tsx beside this): a painted, inked character in front of a <Screen>
# page in the pass "ui", under a paint <Look>, <Outline>, <Bloom>, a <Grade> (vignette, aberration) and <DepthOfField>,
# each skipping "ui" (composition "passes"), against the same scene skipping nothing ("plain").
#   page    points on the page's background and on its card come out at the page's own colours (within 2 of 255) in
#           "passes": no grain, rag, bands, ink, glow, fringes or vignette on them; and in "plain" at least one doesn't
#           (the effects are drawn there at all)
#   paint   the character keeps its paint where it stands over the page: its torso and leg equal "plain"'s (within 3),
#           and its ink over the page is drawn (dark pixels in the overlap)
#   pure    frames 0 and 20 rendered in one run equal each rendered alone
#   play    live play's snapshots of frames 0 and 20 equal the render's (SSIM >= 0.99)
#   bash sdk/raea/bench/bevy/passes/check.sh [BIN]     exits 1 on any failure
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../../.." && pwd)
BIN=${1:-${RAEA_BIN:-$ROOT/target/release/raea}}
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-passes-check.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
status=0
fail() { echo "FAIL $*"; status=1; }

render() { # COMPOSITION FRAMES DIR
  "$BIN" render "$HERE/edit.tsx" --composition "$1" --at "$2" -o "$OUT/$3" > "$OUT/$3.log" 2>&1 || { echo "render of $1 at $2 failed (log below)"; tail -20 "$OUT/$3.log"; exit 1; }
}
render passes 0,20 seq
render passes 20 alone20
render plain 0 plain

rgb() { magick "$1" -format "%[fx:int(255*p{$2}.r)] %[fx:int(255*p{$2}.g)] %[fx:int(255*p{$2}.b)]" info:; }
near() { # "R G B" "R G B" TOLERANCE
  set -- $1 $2 $3
  local d=$(( ${1}-${4} )) e=$(( ${2}-${5} )) f=$(( ${3}-${6} ))
  [ ${d#-} -le $7 ] && [ ${e#-} -le $7 ] && [ ${f#-} -le $7 ]
}

# page: the page's blue (#2a6df4) at its centre, right edge and top; its card (#ffd23f)
PAGE="42 109 244" CARD="255 210 63"
plain_differs=0
for pt in "300,230 $PAGE" "540,240 $PAGE" "400,80 $PAGE" "480,130 $CARD"; do
  set -- $pt; at=$1; shift; want="$*"
  for f in "$OUT"/seq/frame-*.png; do
    got=$(rgb "$f" "$at")
    near "$got" "$want" 2 || fail "page: $(basename "$f") at $at is $got, the page's is $want"
  done
  near "$(rgb "$OUT/plain/frame-00000.png" "$at")" "$want" 4 || plain_differs=1
done
[ $plain_differs = 1 ] || fail "page: the plain scene's page is at its own colours too (are the effects drawn at all?)"

# paint: the character over the page, as the plain scene draws it
for at in 240,200 232,250; do
  near "$(rgb "$OUT/seq/frame-00000.png" "$at")" "$(rgb "$OUT/plain/frame-00000.png" "$at")" 3 || fail "paint: the character at $at is $(rgb "$OUT/seq/frame-00000.png" "$at"), plain $(rgb "$OUT/plain/frame-00000.png" "$at")"
done
ink=$(magick "$OUT/seq/frame-00000.png" -crop 140x60+200+150 +repage -colorspace gray -threshold 25% -negate -format '%[fx:int(mean*w*h)]' info:)
[ "$ink" -ge 300 ] || fail "paint: only $ink ink pixels where the character stands over the page"

ssim() { ffmpeg -nostats -i "$1" -i "$2" -lavfi "[0]format=rgb24[a];[1]format=rgb24[b];[a][b]ssim" -f null - 2>&1 | grep -o 'All:[0-9.]*' | cut -d: -f2; }
s=$(ssim "$OUT/seq/frame-00001.png" "$OUT/alone20/frame-00000.png")
[ "$(echo "${s:-0} >= 0.9999" | bc)" = 1 ] || fail "pure: frame 20 in the run vs alone SSIM ${s:-none}"

RAEA_AUDIO_MUTE=1 "$BIN" play "$HERE/edit.tsx" --offscreen --seconds 0.8 --snapshot 0 "$OUT/play-0.png" --snapshot 0.6666667 "$OUT/play-20.png" > "$OUT/play.log" 2>&1 || fail "play failed: $(tail -3 "$OUT/play.log")"
for pair in "frame-00000 0" "frame-00001 20"; do
  set -- $pair
  p=$(ssim "$OUT/seq/$1.png" "$OUT/play-$2.png")
  [ "$(echo "${p:-0} >= 0.99" | bc)" = 1 ] || fail "play: frame $2 played vs rendered SSIM ${p:-none}"
done

[ $status = 0 ] && echo "passes: PASS (page at its own colours under every effect; the character painted and inked over it; frame 20 equals alone; play equals render, SSIM $p)"
exit $status
