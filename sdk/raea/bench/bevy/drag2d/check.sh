#!/bin/sh
# The drag2d fixture's measurements (docs/sdk.feature S2c: drag and drop on 2D parts (<Shape2D>)), from a render of its 30 frames
# with input.json's pointer: over the square from frame 4, pressed at 6, dragged right from 8 onto the circle (over it from
# about 13), let go at 18.
#   drag   the square is its first colour before the drag (frame 7), another while dragged (frame 12), its first again after
#   drop   the circle is its first colour before the drag reaches it (frame 10), another once entered (frame 16), a third
#          once the square is dropped on it (frame 22)
#   sh sdk/raea/bench/bevy/drag2d/check.sh [RAEA_BIN]     exits 1 on any failure
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../../../.." && pwd)
bin=${1:-${RAEA_BIN:-$root/target/release/raea}}
out=$(mktemp -d "${TMPDIR:-/tmp}/raea-drag2d-check-XXXXXX")
trap 'rm -rf "$out"' EXIT
RAEA_SIGN=0 "$bin" render "$here/edit.tsx" --input "$here/input.json" -o "$out/" > "$out/render.log" 2>&1 || { echo "drag2d: the render failed ($out/render.log)"; cat "$out/render.log"; exit 1; }
px() { ffmpeg -v error -i "$out/frame-$(printf %05d "$1").png" -vf "crop=1:1:$2:270" -f rawvideo -pix_fmt rgb24 - | xxd -p; }
square() { px "$1" 295; }
circle() { px "$1" 690; }
status=0
same() { if [ "$2" = "$3" ]; then echo "drag2d: PASS  $1"; else echo "drag2d: FAIL  $1 ($2, want $3)"; status=1; fi; }
differs() { if [ "$2" != "$3" ]; then echo "drag2d: PASS  $1 ($2)"; else echo "drag2d: FAIL  $1 (still $3)"; status=1; fi; }
c0=$(square 2); b0=$(circle 2)
same "the square pressed, not yet dragged (frame 7)" "$(square 7)" "$c0"
differs "the square while dragged (frame 12)" "$(square 12)" "$c0"
same "the square after the drag ends (frame 22)" "$(square 22)" "$c0"
same "the circle before the drag reaches it (frame 10)" "$(circle 10)" "$b0"
differs "the circle once the drag entered it (frame 16)" "$(circle 16)" "$b0"
differs "the circle once the square is dropped on it (frame 22)" "$(circle 22)" "$(circle 16)"
exit $status
