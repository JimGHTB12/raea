#!/bin/bash
# #20: the same HTML page rendered again gives the same frames. index.html is rendered RUNS times (8) to frame hashes
# (.txt, the NV12 bytes before any encoder) and every run must match the first. Before Vello's fine stage summed a
# tile's coverage in fixed point, about one run in three differed from the first, on a frame or a few.
#   bench/html-repeat/check.sh      (RAEA_BIN: the raea to run, default target/release/raea; RUNS: how many runs)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
RUNS=${RUNS:-8}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-html-repeat.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
status=0
for i in $(seq 1 "$RUNS"); do
  "$BIN" render bench/html-repeat/index.html -o "$OUT/$i.txt" > "$OUT/$i.log" 2>&1 || { echo "FAIL: run $i failed"; tail -5 "$OUT/$i.log"; exit 1; }
  [ "$i" = 1 ] && continue
  d=$(diff "$OUT/1.txt" "$OUT/$i.txt" | awk '/^>/ {print $2}' | tr '\n' ' ')
  [ -z "$d" ] || { echo "FAIL: run $i differs from run 1 at frames $d"; status=1; }
done
[ $status = 0 ] && echo "PASS: $RUNS runs, every frame the same"
exit $status
