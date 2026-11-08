#!/bin/bash
# #26: a 3D composition's 3D made live, frame by frame as it's stepped (as live play and a game's render make it),
# draws what its rows made beforehand draw (a render's program), pixel for pixel. Each composition below is rendered
# to per-frame hashes with RAEA_3D=program and with RAEA_3D=live (RAEA_LIVE3D_CHECK=1: each live frame's fast rows
# also held against its full rows), and the two must be identical. They caught what live frames sent otherwise than
# a render keys it: a spot's cone (spotlight), a zero scale's NaN rotation and a bulb's f32 place
# (spherical_area_lights, lighting), a colour as rounded floats against its hex (screenshake), a rotation's sign
# (shapes); atmospheric_fog keeps the camera's own spawns, the at-3d scene a ShaderMesh's uniforms. Exits 1 on a failure.
#   bench/live-3d/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-live-3d.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
status=0
for f in bench/at-3d/scene.tsx sdk/raea/bench/bevy-parity/3d/spotlight.tsx sdk/raea/bench/bevy-parity/3d/spherical_area_lights.tsx \
  sdk/raea/bench/bevy-parity/3d/atmospheric_fog.tsx examples/games/screenshake.tsx examples/games/shapes.tsx examples/games/lighting.tsx; do
  n=$(basename "$f" .tsx)
  RAEA_3D=program "$BIN" render "$f" -o "$OUT/$n.program.txt" > "$OUT/$n.program.log" 2>&1 || { echo "FAIL $f: the programmed render failed"; tail -3 "$OUT/$n.program.log"; status=1; continue; }
  RAEA_3D=live RAEA_LIVE3D_CHECK=1 "$BIN" render "$f" -o "$OUT/$n.live.txt" > "$OUT/$n.live.log" 2>&1 || { echo "FAIL $f: the live render failed: $(grep -o 'RAEA_LIVE3D_CHECK.\{0,160\}' "$OUT/$n.live.log" | head -1)"; status=1; continue; }
  d=$(diff <(cut -d' ' -f2 "$OUT/$n.program.txt") <(cut -d' ' -f2 "$OUT/$n.live.txt") | grep -c '^<')
  if [ "$d" = 0 ]; then echo "PASS $f ($(wc -l < "$OUT/$n.live.txt" | tr -d ' ') frames)"; else echo "FAIL $f: $d frames differ"; status=1; fi
done
exit $status
