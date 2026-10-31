#!/bin/sh
# The pick3d fixture's measurements (docs/sdk.feature S3a: on* on 3D parts), from a render of its 30 frames with
# input.json's pointer: over the cube from frame 4, pressed at 8 and let go at 10, over the ball from 16, away from 24.
#   click   the cube is its first colour through frame 9 (pressed, not yet let go) and the other from frame 11 on
#   hover   the ball is its first colour before 16, the other while the pointer is over it, its first again after
#   sh sdk/raea/bench/bevy/pick3d/check.sh [RAEA_BIN]     exits 1 on any failure
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../../../.." && pwd)
bin=${1:-${RAEA_BIN:-$root/target/release/raea}}
out=$(mktemp -d "${TMPDIR:-/tmp}/raea-pick3d-check-XXXXXX")
trap 'rm -rf "$out"' EXIT
RAEA_SIGN=0 "$bin" render "$here/edit.tsx" --input "$here/input.json" -o "$out/" > "$out/render.log" 2>&1 || { echo "pick3d: the render failed ($out/render.log)"; cat "$out/render.log"; exit 1; }
px() { ffmpeg -v error -i "$out/frame-$(printf %05d "$1").png" -vf "crop=1:1:$2:270" -f rawvideo -pix_fmt rgb24 - | xxd -p; }
cube() { px "$1" 295; }
ball() { px "$1" 665; }
status=0
check() { if [ "$2" = "$3" ]; then echo "pick3d: PASS  $1"; else echo "pick3d: FAIL  $1 ($2, want $3)"; status=1; fi; }
c0=$(cube 2); b0=$(ball 2)
check "the cube pressed, not yet let go (frame 9)" "$(cube 9)" "$c0"
[ "$(cube 11)" != "$c0" ] && echo "pick3d: PASS  the cube clicked at frame 10 (frame 11 $(cube 11), was $c0)" || { echo "pick3d: FAIL  the cube's click (frame 11 still $c0)"; status=1; }
check "the cube stays clicked (frame 28)" "$(cube 28)" "$(cube 11)"
check "the ball before the pointer's over it (frame 15)" "$(ball 15)" "$b0"
[ "$(ball 20)" != "$b0" ] && echo "pick3d: PASS  the ball with the pointer over it (frame 20 $(ball 20))" || { echo "pick3d: FAIL  the ball's hover (frame 20 still $b0)"; status=1; }
check "the ball after the pointer left (frame 28)" "$(ball 28)" "$b0"
exit $status
