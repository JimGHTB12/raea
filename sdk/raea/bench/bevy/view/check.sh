#!/bin/sh
# The view fixture's measurements (docs/sdk.feature S3a, a camera inside a page element), at frames 0, 15 and 29:
#   box     the 400 x 300 box's view above the label equals ref.tsx (the same scene as a 400 x 300 frame of its own):
#           SSIM >= 0.999 (they differ by at most 2/255: the tonemapper's deband dither is keyed by screen position)
#   page    outside the box the page is the page alone: its background (#e8eef6), not the 3D
#   over    the label after the camera's element is drawn over the view (white inside its I)
#   sh sdk/raea/bench/bevy/view/check.sh [RAEA_BIN]     exits 1 on any failure
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../../../.." && pwd)
bin=${1:-${RAEA_BIN:-$root/target/release/raea}}
out=$(mktemp -d "${TMPDIR:-/tmp}/raea-view-check-XXXXXX")
trap 'rm -rf "$out"' EXIT
export RAEA_SIGN=0
px() { ffmpeg -v error -i "$1" -vf "crop=1:1:$2:$3" -f rawvideo -pix_fmt rgb24 - | xxd -p; }
status=0
for f in 0 15 29; do
  "$bin" render "$here/edit.tsx" --frame $f -o "$out/edit-$f.png" > /dev/null 2>&1 && "$bin" render "$here/ref.tsx" --frame $f -o "$out/ref-$f.png" > /dev/null 2>&1 || { echo "view: frame $f didn't render"; exit 1; }
  s=$(ffmpeg -v error -i "$out/edit-$f.png" -i "$out/ref-$f.png" -filter_complex "[0]crop=400:250:120:80,format=rgb24[a];[1]crop=400:250:0:0,format=rgb24[b];[a][b]ssim=stats_file=-" -f null - 2>&1 | grep -o 'All:[0-9.]*' | tail -1 | cut -d: -f2)
  page=$(px "$out/edit-$f.png" 60 450)
  over=$(px "$out/edit-$f.png" 161 354)
  ok=PASS
  awk -v s="$s" 'BEGIN { exit !(s >= 0.999) }' || ok=FAIL
  [ "$page" = e8eef6 ] || ok=FAIL
  [ "$over" = ffffff ] || ok=FAIL
  [ $ok = PASS ] || status=1
  echo "view: $ok  frame $f: box SSIM $s against ref, page $page, label $over"
done
exit $status
