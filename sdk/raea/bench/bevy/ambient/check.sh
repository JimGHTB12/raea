#!/bin/sh
# The ambient fixture's measurement (docs/sdk.feature S2a: an AmbientLight on a camera overrides the global one): the
# cube's centre in edit.tsx (a bright GlobalAmbientLight, the camera's own dim AmbientLight) is its centre in only.tsx
# (the dim light alone), and darker than with the bright one counted.
#   sh sdk/raea/bench/bevy/ambient/check.sh [RAEA_BIN]     exits 1 on any failure
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../../../.." && pwd)
bin=${1:-${RAEA_BIN:-$root/target/release/raea}}
out=$(mktemp -d "${TMPDIR:-/tmp}/raea-ambient-check-XXXXXX")
trap 'rm -rf "$out"' EXIT
for f in edit only; do
  RAEA_SIGN=0 "$bin" render "$here/$f.tsx" -o "$out/$f/" > "$out/$f.log" 2>&1 || { echo "ambient: the $f render failed"; cat "$out/$f.log"; exit 1; }
done
px() { ffmpeg -v error -i "$out/$1/frame-00000.png" -vf "crop=1:1:160:120" -f rawvideo -pix_fmt rgb24 - | xxd -p; }
a=$(px edit); b=$(px only)
if [ "$a" = "$b" ] && [ "$a" != "000000" ]; then echo "ambient: PASS  the camera's own ambient light alone lights the cube ($a)"; else echo "ambient: FAIL  the cube is $a, want $b (the camera's dim light alone)"; exit 1; fi
