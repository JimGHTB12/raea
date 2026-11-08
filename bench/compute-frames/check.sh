#!/bin/bash
# An inline <Compute> whose uniform changes every frame (level.tsx: `level` f / 20, the grey it fills a render image
# with). Each frame must show its own frame's grey, both in a PNG render (-o DIR) and in an mp4 (the fed render, whose
# 3D is live): frame 10 is level 0.5, sRGB 188; frame 0 is black. Before, a compute's uniforms were its first frame's
# (the spec is held from frame 0), and the mp4's live rows were read from the load's columns, so an mp4 of a compute
# driven by its uniforms stayed black. Exits 1 on a failure.
#   bench/compute-frames/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-compute-frames.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
grey() { magick "$1" -format "%[fx:round(255*p{64,64}.r)]" info:; }
status=0
"$BIN" render bench/compute-frames/level.tsx -o "$OUT/dir" > "$OUT/dir.log" 2>&1 || { echo "FAIL: the PNG render failed"; grep -v WARN "$OUT/dir.log" | tail -3; exit 1; }
for k in 0 10; do
  want=$([ $k = 0 ] && echo 0 || echo 188)
  got=$(grey "$OUT/dir/$(printf 'frame-%05d.png' $k)")
  [ "$got" = "$want" ] || { echo "FAIL: PNG frame $k is grey $got, not $want"; status=1; }
done
"$BIN" render bench/compute-frames/level.tsx -o "$OUT/out.mp4" > "$OUT/mp4.log" 2>&1 || { echo "FAIL: the mp4 render failed"; grep -v WARN "$OUT/mp4.log" | tail -3; exit 1; }
for k in 0 10; do
  want=$([ $k = 0 ] && echo 0 || echo 188)
  ffmpeg -v error -i "$OUT/out.mp4" -vf "select=eq(n\,$k)" -frames:v 1 -y "$OUT/mp4-$k.png" || { echo "FAIL: no mp4 frame $k"; status=1; continue; }
  got=$(grey "$OUT/mp4-$k.png")
  # the mp4's YUV round trip moves a grey by a few levels
  [ $(( got > want ? got - want : want - got )) -le 4 ] || { echo "FAIL: mp4 frame $k is grey $got, not $want (within 4)"; status=1; }
done
[ $status = 0 ] && echo "PASS: a compute's per-frame uniform reaches frame 10 in a PNG render (188) and an mp4 (within 4)"
exit $status
