#!/bin/bash
# An <R3F> scene that suspends on its first frame (first-frame.tsx): frame 0 of an mp4 and a --frame 0 still must be
# the red box. Before the frame hold (sdk/r3f hold.tsx, runtime/bundle entry.rs HOLD) frame 0 was the background
# alone, so a looping film opened on a blank frame. Exits 1 on a failure.
#   sdk/r3f/test/hold/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../../../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-r3f-hold.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
T=sdk/r3f/test/hold/first-frame.tsx
red() { magick "$1" -format "%[fx:int(255*p{80,80}.r)] %[fx:int(255*p{80,80}.g)]" info: | awk '{ exit !($1 > 200 && $2 < 40) }'; }
"$BIN" render $T -o "$OUT/v.mp4" > "$OUT/v.log" 2>&1 || { echo "FAIL: the mp4 render failed"; tail -3 "$OUT/v.log"; exit 1; }
ffmpeg -loglevel error -y -i "$OUT/v.mp4" -frames:v 1 "$OUT/m0.png" || { echo "FAIL: no frame 0 in the mp4"; exit 1; }
red "$OUT/m0.png" || { echo "FAIL: the mp4's frame 0 is $(magick "$OUT/m0.png" -format '%[pixel:p{80,80}]' info:), not the red box"; exit 1; }
"$BIN" render $T --frame 0 -o "$OUT/f0.png" > "$OUT/f0.log" 2>&1 || { echo "FAIL: the --frame 0 render failed"; tail -3 "$OUT/f0.log"; exit 1; }
red "$OUT/f0.png" || { echo "FAIL: --frame 0 is $(magick "$OUT/f0.png" -format '%[pixel:p{80,80}]' info:), not the red box"; exit 1; }
echo "PASS: a suspended <R3F> scene's frame 0 waits for its load (mp4 and still)"
