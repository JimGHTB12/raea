#!/bin/bash
# #26/#38: every composition in corpus.txt rendered 30 frames to per-frame hashes on both 3D routes, RAEA_3D=program
# and the default (live, RAEA_LIVE3D_CHECK=1), compared. A line each: SAME, DIFF (frames that differ), ERR (a route's
# exit codes: a render that failed) or SKIP (why it isn't a composition to compare), then the counts. So that a failure
# here is a real gap, not the harness:
#   - a .tsx without registerRoot is a module (a copy-and-own component, a scene an edit imports), not a composition;
#   - a case with `// build: plugins` or an import of raea/plugins/... or @raea/plugins renders with the plugin build
#     (sdk/raea/bench/bevy-plugins/motion.sh, or RAEA_PLUGIN_BIN), as sdk/raea/bench/bevy-ts/check.sh does;
#   - EXPECTED below lists the compositions left out on purpose, each with its reason.
#   bench/live-3d/corpus.sh [OUTDIR]      (RAEA_BIN: the raea to run, default target/release/raea; RAEA_PLUGIN_BIN)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
PLUGIN_BIN=${RAEA_PLUGIN_BIN:-$PWD/sdk/raea/bench/bevy-plugins/motion.sh}
OUT=${1:-$(mktemp -d "${TMPDIR:-/tmp}/raea-live-3d-corpus.XXXXXX")}
mkdir -p "$OUT"
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
expected() { case $1 in
  examples/games/animated-character.tsx) echo "its media/RobotExpressive.glb was deleted" ;;
  examples/playables/brick-rush.tsx) echo "a playable: it needs the playable host's raea_playable (sdk/playable)" ;;
  sdk/raea/bench/bevy-parity/3d/rotate_environment_map.tsx) echo "clearcoat textures need raea-sys-material's multi-layer feature, off by default (as bench/bevy-parity/runs.tsv leaves it out)" ;;
esac; }
SAME=0 DIFF=0 ERR=0 SKIP=0
while read -r f; do
  [ -n "$f" ] || continue
  why=$(expected "$f")
  [ -z "$why" ] && ! grep -q registerRoot "$f" && why="a module (no registerRoot), not a composition"
  if [ -n "$why" ]; then echo "SKIP	$f	($why)"; SKIP=$((SKIP + 1)); continue; fi
  bin=$BIN
  grep -qE '^// build: plugins|from "(raea/plugins/|@raea/plugins)' "$f" && bin=$PLUGIN_BIN
  name=$(echo "$f" | tr / _ | sed 's/\.tsx$//')
  for m in program live; do
    env $([ $m = program ] && echo RAEA_3D=program || echo RAEA_LIVE3D_CHECK=1) "$bin" render "$f" --frames 30 -o "$OUT/$name.$m.txt" > "$OUT/$name.$m.log" 2>&1
    eval "r_$m=$?"
  done
  if [ "$r_program" != 0 ] || [ "$r_live" != 0 ]; then v=ERR; d="program $r_program, live $r_live: $(grep -vE '^\s*$' "$OUT/$name.live.log" | tail -1 | cut -c1-160)"
  elif cmp -s "$OUT/$name.program.txt" "$OUT/$name.live.txt"; then v=SAME; d=""
  else v=DIFF; d="$(diff <(cut -d' ' -f2 "$OUT/$name.program.txt") <(cut -d' ' -f2 "$OUT/$name.live.txt") | grep -c '^<') frames"; fi
  echo "$v	$f	$d"; eval "$v=\$(($v + 1))"
done < bench/live-3d/corpus.txt
echo "corpus: $SAME same, $DIFF differ, $ERR failed, $SKIP skipped (logs in $OUT)"
[ "$DIFF" = 0 ] && [ "$ERR" = 0 ]
