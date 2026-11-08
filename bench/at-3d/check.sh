#!/bin/bash
# #26: a 3D composition's --at frames step only themselves and their neighbours, and draw as the whole timeline stepped
# draws them. scene.tsx (360 frames) is drawn at four frames with --at and must report the sparse stepping ("N of M
# frames stepped"); frozen.tsx draws the same four frames as a 4-frame composition (each the scene under <Freeze>, so
# its rows come from stepping every frame), and each pair must be pixel-identical. Exits 1 on a failure.
#   bench/at-3d/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
# pixel checks: no Content Credentials or TrustMark id in the frames
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-at-3d.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
RAEA_STARTUP_TRACE=1 "$BIN" render bench/at-3d/scene.tsx --at 30,150,270,330 -o "$OUT/at" > "$OUT/at.log" 2>&1 || { echo "FAIL: the --at render failed"; tail -5 "$OUT/at.log"; exit 1; }
"$BIN" render bench/at-3d/frozen.tsx -o "$OUT/frozen" > "$OUT/frozen.log" 2>&1 || { echo "FAIL: the frozen render failed"; tail -5 "$OUT/frozen.log"; exit 1; }
status=0
stepped=$(grep -o "[0-9]* of [0-9]* frames stepped" "$OUT/at.log")
[ "$stepped" = "12 of 332 frames stepped" ] || { echo "FAIL: the --at render stepped '${stepped:-every frame}', not 12 of 332"; status=1; }
for k in 0 1 2 3; do
  f=$(printf 'frame-%05d.png' $k)
  d=$(magick compare -metric AE "$OUT/at/$f" "$OUT/frozen/$f" null: 2>&1 | cut -d' ' -f1)
  [ "$d" = 0 ] || { echo "FAIL: $f differs from the frozen render's in ${d:-?} pixels"; status=1; }
done
[ $status = 0 ] && echo "PASS: $stepped, the four frames pixel-identical to the frozen render's"
exit $status
