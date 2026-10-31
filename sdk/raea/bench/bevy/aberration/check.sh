#!/bin/bash
# The aberration fixture's checks (edit.tsx beside this): a <Grade aberration> keyed 0 -> 0.03 -> 0 over a paint
# Look's white sky with depth of field on the camera.
#   sky     the sky stays white (every channel >= 200 at a corner) on every frame, the spike's included: Bevy's effect
#           stack wrote alpha 1 on every pixel, so once the aberration passed its 1e-4 cut-off the paint look's sky came
#           out black (raea-render engine/effect_stack.rs)
#   pure    frames 30 (the spike) and 50 (after it) rendered in one run with 5 and 26 equal each rendered alone: the
#           aberration is each frame's own, not the first frame's of whatever the render held (scene3d.rs, keyed)
#   fringes the spike's frame differs from frame 5 (no aberration) around the shapes: the aberration is drawn at all
#   bash sdk/raea/bench/bevy/aberration/check.sh [RAEA_BIN]     exits 1 on any failure
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../../.." && pwd)
BIN=${1:-${RAEA_BIN:-$ROOT/target/release/raea}}
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-aberration-check.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
status=0
fail() { echo "FAIL $*"; status=1; }

render() { # FRAMES DIR
  "$BIN" render "$HERE/edit.tsx" --at "$1" -o "$OUT/$2" > "$OUT/$2.log" 2>&1 || { echo "render of $1 failed (log below)"; tail -20 "$OUT/$2.log"; exit 1; }
}
render 5,26,30,50 seq
render 30 alone30
render 50 alone50

# sky: the top-left corner of every frame, light
for f in "$OUT"/seq/frame-*.png "$OUT"/alone*/frame-*.png; do
  px=$(magick "$f" -format '%[fx:int(255*min(min(p{20,20}.r,p{20,20}.g),p{20,20}.b))]' info:)
  [ "$px" -ge 200 ] || fail "sky: $(basename "$(dirname "$f")")/$(basename "$f") corner is $px (black sky)"
done

ssim() { ffmpeg -nostats -i "$1" -i "$2" -lavfi "[0]format=rgb24[a];[1]format=rgb24[b];[a][b]ssim" -f null - 2>&1 | grep -o 'All:[0-9.]*' | cut -d: -f2; }
for pair in "frame-00002 alone30 30" "frame-00003 alone50 50"; do
  set -- $pair
  s=$(ssim "$OUT/seq/$1.png" "$OUT/$2/frame-00000.png")
  [ "$(echo "${s:-0} >= 0.9999" | bc)" = 1 ] || fail "pure: frame $3 in the run vs alone SSIM ${s:-none}"
done

s=$(ssim "$OUT/seq/frame-00000.png" "$OUT/seq/frame-00002.png")
[ "$(echo "${s:-1} < 0.999" | bc)" = 1 ] || fail "fringes: the spike's frame equals frame 5 (SSIM ${s:-none}): no aberration drawn"

[ $status = 0 ] && echo "aberration: PASS (sky light on every frame; frames 30 and 50 equal alone; fringes drawn, SSIM to frame 5 $s)"
exit $status
