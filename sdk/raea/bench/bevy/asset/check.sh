#!/bin/sh
# The asset fixture's measurement (docs/sdk.feature S2b: useAsset): dot.png's size (64 x 32), read with useAsset, sizes
# the bar to 256 x 128 from x 40 on the first frame: frame 0's pixel at x 295 (the bar's last column) is the bar's
# orange, x 297 (past it) is the background.
#   sh sdk/raea/bench/bevy/asset/check.sh [RAEA_BIN]     exits 1 on any failure
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../../../.." && pwd)
bin=${1:-${RAEA_BIN:-$root/target/release/raea}}
out=$(mktemp -d "${TMPDIR:-/tmp}/raea-asset-check-XXXXXX")
trap 'rm -rf "$out"' EXIT
RAEA_SIGN=0 "$bin" render "$here/edit.tsx" -o "$out/" > "$out/render.log" 2>&1 || { echo "asset: the render failed"; cat "$out/render.log"; exit 1; }
px() { ffmpeg -v error -i "$out/frame-00000.png" -vf "crop=1:1:$1:$2" -f rawvideo -pix_fmt rgb24 - | xxd -p; }
status=0
check() { if [ "$2" = "$3" ]; then echo "asset: PASS  $1 ($2)"; else echo "asset: FAIL  $1 ($2, want $3)"; status=1; fi; }
check "the bar's last column on frame 0 (x 295)" "$(px 295 100)" "ff6a00"
check "past the bar (x 297)" "$(px 297 100)" "e8eef6"
check "the bar's last row on frame 0 (y 167)" "$(px 100 167)" "ff6a00"
check "below the bar (y 169)" "$(px 100 169)" "e8eef6"
exit $status
