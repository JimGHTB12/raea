#!/bin/sh
# The drag3d fixture's measurements (docs/sdk.feature S2c: drag and drop on 3D parts), from a render of its 30 frames
# with input.json's pointer: over the cube from frame 4, pressed at 6, dragged right from 8 onto the ball (over it from
# about 13), let go at 18.
#   drag   the cube is its first colour before the drag (frame 7), another while dragged (frame 12), its first again after
#   drop   the ball is its first colour before the drag reaches it (frame 10), another once entered (frame 16), a third
#          once the cube is dropped on it (frame 22)
#   sh sdk/raea/bench/bevy/drag3d/check.sh [RAEA_BIN]     exits 1 on any failure
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../../../.." && pwd)
bin=${1:-${RAEA_BIN:-$root/target/release/raea}}
out=$(mktemp -d "${TMPDIR:-/tmp}/raea-drag3d-check-XXXXXX")
trap 'rm -rf "$out"' EXIT
RAEA_SIGN=0 "$bin" render "$here/edit.tsx" --input "$here/input.json" -o "$out/" > "$out/render.log" 2>&1 || { echo "drag3d: the render failed ($out/render.log)"; cat "$out/render.log"; exit 1; }
px() { ffmpeg -v error -i "$out/frame-$(printf %05d "$1").png" -vf "crop=1:1:$2:270" -f rawvideo -pix_fmt rgb24 - | xxd -p; }
cube() { px "$1" 295; }
ball() { px "$1" 690; }
status=0
same() { if [ "$2" = "$3" ]; then echo "drag3d: PASS  $1"; else echo "drag3d: FAIL  $1 ($2, want $3)"; status=1; fi; }
differs() { if [ "$2" != "$3" ]; then echo "drag3d: PASS  $1 ($2)"; else echo "drag3d: FAIL  $1 (still $3)"; status=1; fi; }
c0=$(cube 2); b0=$(ball 2)
same "the cube pressed, not yet dragged (frame 7)" "$(cube 7)" "$c0"
differs "the cube while dragged (frame 12)" "$(cube 12)" "$c0"
same "the cube after the drag ends (frame 22)" "$(cube 22)" "$c0"
same "the ball before the drag reaches it (frame 10)" "$(ball 10)" "$b0"
differs "the ball once the drag entered it (frame 16)" "$(ball 16)" "$b0"
differs "the ball once the cube is dropped on it (frame 22)" "$(ball 22)" "$(ball 16)"
exit $status
