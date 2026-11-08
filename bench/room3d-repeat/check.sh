#!/bin/bash
# #48: a 3D game rendered again gives the same frames. sdk/react/bench/games/room3d (a room whose walls meet its floor,
# and glTF trees outside) is rendered RUNS times (8) at frames where a wall's foot and the floor tie in depth, and
# every run must match the first. Before await_models held the views off while models loaded, the render world binned
# each mesh at the update its load landed on, so the multidraw's order, and which surface won the tie, changed from
# run to run: about one run in three differed from the first, by one pixel a frame on the floor's seam.
#   bench/room3d-repeat/check.sh      (RAEA_BIN: the raea to run, default target/release/raea; RUNS: how many runs)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
RUNS=${RUNS:-8}
AT=35,90,210,258,278,295
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-room3d-repeat.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
status=0
for i in $(seq 1 "$RUNS"); do
  "$BIN" render sdk/react/bench/games/room3d/edit.tsx --at "$AT" -o "$OUT/$i/" > "$OUT/$i.log" 2>&1 || { echo "FAIL: run $i failed"; tail -5 "$OUT/$i.log"; exit 1; }
  [ "$i" = 1 ] && continue
  d=$(for f in "$OUT/1"/*.png; do cmp -s "$f" "$OUT/$i/$(basename "$f")" || printf '%s ' "$(basename "$f" .png)"; done)
  [ -z "$d" ] || { echo "FAIL: run $i differs from run 1 at $d(of --at $AT)"; status=1; }
done
[ $status = 0 ] && echo "PASS: $RUNS runs of room3d, every frame the same"
exit $status
