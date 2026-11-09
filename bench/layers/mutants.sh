#!/bin/bash
# The layer pass's mutants (docs/review.feature R2e): the pass broken five ways on purpose (RAEA_LAYERS_MUTANT, in
# raea-sys-layers): two layers' keys swapped, the 3D 2 px to the right, the 3D's occlusion ignored, the page drawn
# under the 3D, a skinned mesh in its bind pose. Each must fail compare.sh against the oracle; the unbroken pass must
# pass. Exits 1 when one doesn't.
#   bench/layers/mutants.sh ORACLE OUT     (ORACLE: oracle.sh's folder; RAEA_BIN: the raea to run)
cd "$(dirname "$0")/../.." || exit 2
ORACLE=${1:?ORACLE: oracle.sh OUT folder}
OUT=${2:?OUT folder}
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0 RAEA_SOURCE_STAMPS=1 RAEA_AUDIO_MUTE=1 RAEA_LAYERS=1
mkdir -p "$OUT/src"
cp bench/layers/conformance.tsx bench/layers/badge.tsx "$ORACLE/src/pic.png" "$ORACLE/src/clip.mp4" "$OUT/src/"
mkdir -p "$OUT/src/media" && cp "$ORACLE/src/media/RobotExpressive.glb" "$OUT/src/media/"
status=0
for m in none swap shift occlusion under bind; do
  rm -rf "${OUT:?}/$m"
  RAEA_LAYERS_MUTANT=$m "$BIN" render "$OUT/src/conformance.tsx" --at 10,45,80 -o "$OUT/$m" > "$OUT/$m.log" 2>&1 || { echo "FAIL the render ($m) failed"; tail -3 "$OUT/$m.log"; exit 1; }
  bench/layers/compare.sh "$ORACLE" "$OUT/$m/conformance.layers" > "$OUT/$m.compare" 2>&1
  got=$?
  if [ "$m" = none ]; then
    if (( got == 0 )); then echo "ok   the pass itself passes"; else echo "FAIL the pass itself fails:"; grep FAIL "$OUT/$m.compare" | sed 's/^/    /'; status=1; fi
  elif (( got == 1 )); then echo "ok   mutant $m fails: $(grep -c FAIL "$OUT/$m.compare") layers"
  else echo "FAIL mutant $m passes the oracle"; status=1; fi
done
exit $status
