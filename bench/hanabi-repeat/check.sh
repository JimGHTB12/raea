#!/bin/bash
# #52: GPU particles rendered again give the same frames. sdk/raea/bench/bevy-plugins/hanabi.tsx (two seeded fountains,
# one opaque and one alpha-blended) is rendered RUNS times (4), all 120 frames to hashes, and every run must match the
# first. Before vendor/bevy_hanabi, its draw list and its recycled slots were taken with GPU atomics, in an order that
# changed from run to run: the blended fountain differed from frame 7 or so (draw order), and once particles died both
# could (a new particle was seeded by the slot it took). Needs the plugin build.
#   bench/hanabi-repeat/check.sh    (RAEA_PLUGIN_BIN: the plugin build, default sdk/raea/bench/bevy-plugins/motion.sh;
#                                    RUNS: how many runs)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_PLUGIN_BIN:-$PWD/sdk/raea/bench/bevy-plugins/motion.sh}
RUNS=${RUNS:-4}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0 RAEA_3D=program
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-hanabi-repeat.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
status=0
for i in $(seq 1 "$RUNS"); do
  "$BIN" render sdk/raea/bench/bevy-plugins/hanabi.tsx -o "$OUT/$i.txt" > "$OUT/$i.log" 2>&1 || { echo "FAIL: run $i failed"; tail -5 "$OUT/$i.log"; exit 1; }
  [ "$i" = 1 ] && continue
  d=$(diff "$OUT/1.txt" "$OUT/$i.txt" | grep '^<' | cut -d' ' -f2 | head -5 | tr '\n' ' ')
  [ -z "$d" ] || { echo "FAIL: run $i differs from run 1 at frames ${d}($(diff "$OUT/1.txt" "$OUT/$i.txt" | grep -c '^<') of $(wc -l < "$OUT/1.txt"))"; status=1; }
done
[ $status = 0 ] && echo "PASS: $RUNS runs of hanabi, every frame the same"
exit $status
