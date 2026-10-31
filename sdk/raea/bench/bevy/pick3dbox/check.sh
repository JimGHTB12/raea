#!/bin/sh
# The pick3dbox fixture's measurements (docs/sdk.feature S3a: on* on 3D parts under a camera in a page element), as
# pick3d's, its parts at their places in the box (440, 240, 480 x 270): a render of its 30 frames with
# input.json's pointer: over the cube from frame 4, pressed at 8 and let go at 10, over the ball from 16, away from 24.
#   click   the cube is its first colour through frame 9 (pressed, not yet let go) and the other from frame 11 on
#   hover   the ball is its first colour before 16, the other while the pointer is over it, its first again after
# then the same as an MP4 (the fed render: the view drawn into the box on the GPU, the rows stepped again in it)
#   sh sdk/raea/bench/bevy/pick3dbox/check.sh [RAEA_BIN]     exits 1 on any failure
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../../../.." && pwd)
bin=${1:-${RAEA_BIN:-$root/target/release/raea}}
out=$(mktemp -d "${TMPDIR:-/tmp}/raea-pick3dbox-check-XXXXXX")
trap 'rm -rf "$out"' EXIT
RAEA_SIGN=0 "$bin" render "$here/edit.tsx" --input "$here/input.json" -o "$out/" > "$out/render.log" 2>&1 || { echo "pick3dbox: the render failed ($out/render.log)"; cat "$out/render.log"; exit 1; }
px() { ffmpeg -v error -i "$out/frame-$(printf %05d "$1").png" -vf "crop=1:1:$2:375" -f rawvideo -pix_fmt rgb24 - | xxd -p; }
cube() { px "$1" 588; }
ball() { px "$1" 772; }
status=0
check() { if [ "$2" = "$3" ]; then echo "pick3dbox: PASS  $1"; else echo "pick3dbox: FAIL  $1 ($2, want $3)"; status=1; fi; }
c0=$(cube 2); b0=$(ball 2)
check "the cube pressed, not yet let go (frame 9)" "$(cube 9)" "$c0"
[ "$(cube 11)" != "$c0" ] && echo "pick3dbox: PASS  the cube clicked at frame 10 (frame 11 $(cube 11), was $c0)" || { echo "pick3dbox: FAIL  the cube's click (frame 11 still $c0)"; status=1; }
check "the cube stays clicked (frame 28)" "$(cube 28)" "$(cube 11)"
check "the ball before the pointer's over it (frame 15)" "$(ball 15)" "$b0"
[ "$(ball 20)" != "$b0" ] && echo "pick3dbox: PASS  the ball with the pointer over it (frame 20 $(ball 20))" || { echo "pick3dbox: FAIL  the ball's hover (frame 20 still $b0)"; status=1; }
check "the ball after the pointer left (frame 28)" "$(ball 28)" "$b0"
# the fed render (an MP4): its frames as PNGs, the same probes (H.264 moves a colour by a step or two, so a part is
# "the other colour" when it's off its first by more than 24 in any channel)
RAEA_SIGN=0 "$bin" render "$here/edit.tsx" --input "$here/input.json" -o "$out/fed.mp4" > "$out/fed.log" 2>&1 || { echo "pick3dbox: the fed render failed ($out/fed.log)"; cat "$out/fed.log"; exit 1; }
mkdir -p "$out/fed" && ffmpeg -v error -i "$out/fed.mp4" -start_number 0 "$out/fed/frame-%05d.png"
fpx() { ffmpeg -v error -i "$out/fed/frame-$(printf %05d "$1").png" -vf "crop=1:1:$2:375" -f rawvideo -pix_fmt rgb24 - | xxd -p; }
far() { a=$1; b=$2; for i in 0 2 4; do d=$(( 0x${a:$i:2} - 0x${b:$i:2} )); [ ${d#-} -gt 24 ] && return 0; done; return 1; }
fc0=$(fpx 2 588); fb0=$(fpx 2 772)
far "$(fpx 9 588)" "$fc0" && { echo "pick3dbox: FAIL  fed: the cube changed before it was let go (frame 9)"; status=1; } || echo "pick3dbox: PASS  fed: the cube pressed, not yet let go (frame 9)"
far "$(fpx 11 588)" "$fc0" && echo "pick3dbox: PASS  fed: the cube clicked at frame 10 (frame 11 $(fpx 11 588), was $fc0)" || { echo "pick3dbox: FAIL  fed: the cube's click (frame 11 $(fpx 11 588), was $fc0)"; status=1; }
far "$(fpx 20 772)" "$fb0" && echo "pick3dbox: PASS  fed: the ball with the pointer over it (frame 20 $(fpx 20 772))" || { echo "pick3dbox: FAIL  fed: the ball's hover (frame 20 $(fpx 20 772), was $fb0)"; status=1; }
far "$(fpx 28 772)" "$fb0" && { echo "pick3dbox: FAIL  fed: the ball after the pointer left (frame 28)"; status=1; } || echo "pick3dbox: PASS  fed: the ball after the pointer left (frame 28)"
exit $status
