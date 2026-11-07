#!/bin/bash
# R1's JS layer and R2 (docs/lightspeed.feature): the garbage a frame and the collections, per edit, on the live
# runtime (sdk/react/js/lite: every frame rendered in JS, as a game plays).
#
#   bench/lightspeed/jsheap.sh [--seconds 20] [--runs 3] [--label NAME] [EDIT.tsx ...]
#
# Each edit is played offscreen with JSC_logGC=1 (no bake: timelines step their JS each frame too) and its GC log read:
# garbage freed a frame (KB), collections a minute, the worst pause, and JS time a frame (p50, p99). Defaults: the
# animate fixture, its interpolate twin, titles and pong, and pong's Z3 variants. The medians of RUNS go in the rows.
. "$(dirname "$0")/lib.sh"
H=bench/lightspeed/jsheap.sh

# NAME -> edit; Z3's fixtures: pong's step and fold alone (its view renders nothing), its view alone over a frozen state
edit_of() {
  case $1 in
    animate) echo sdk/react/bench/timelines/animate/edit.tsx ;;
    twin) echo bench/lightspeed/animate-twin/edit.tsx ;;
    titles) echo sdk/react/bench/timelines/titles/edit.tsx ;;
    pong) echo sdk/react/bench/games/pong/edit.tsx ;;
    pong-stepfold) echo sdk/react/bench/games/pong/edit.stepfold.tsx ;;
    pong-view) echo sdk/react/bench/games/pong/edit.view.tsx ;;
    *) echo "$1" ;;
  esac
}

seconds=20; runs=3; label=""; names=()
while [ $# -gt 0 ]; do
  case $1 in
    --seconds) seconds=$2; shift ;;
    --runs) runs=$2; shift ;;
    --label) label=$2; shift ;;
    -*) echo "usage: bench/lightspeed/jsheap.sh [--seconds S] [--runs N] [--label NAME] [EDIT ...]" >&2; exit 2 ;;
    *) names+=("$1") ;;
  esac
  shift
done
seconds=$(pyf "$seconds")
[ ${#names[@]} = 0 ] && names=(animate twin titles pong pong-stepfold pong-view)

TMP=$(mktemp -d "${TMPDIR:-/tmp}/lightspeed-jsheap.XXXXXX")
trap 'cleanup_turn; rm -rf "$TMP"' EXIT

# once EDIT: one run's numbers, a JSON line
once() {
  local edit=$1 input=""
  case $edit in *games*) input=--input ;; esac
  (cd "$ROOT/sdk/react" && JSC_logGC=1 RAEA_AUDIO_MUTE=1 RAEA_ZEROHEAP=1 "$MOTION" play "$edit" $input --offscreen --size 1280x720 \
    --seconds "$seconds" --json "$TMP/run.json" > "$TMP/out" 2> "$TMP/err")
  cat "$TMP/out" "$TMP/err" > "$TMP/all"
  jq -c --rawfile t "$TMP/all" '
    [$t | match("START \\w (\\d+)kb => (\\w+),.*?=> (\\d+)kb, p=([0-9.]+)ms \\(max [0-9.]+\\), cycle ([0-9.]+)ms END\\]"; "g")
      | [.captures[].string]] as $gcs
    | ([.presented, 1] | max) as $frames
    | {kb_frame: (([$gcs[] | (.[0] | tonumber) - (.[2] | tonumber)] | add // 0) / $frames),
       gc_min: (($gcs | length) * 60 / ([.seconds, 0.1] | max)),
       pause_max: ([$gcs[][3] | tonumber] | max // 0.0), js_p50: .js_ms.p50, js_p99: .js_ms.p99,
       work_p99: .work_ms.p99, frames: .presented,
       # ZeroHeap Z1: Rust allocations a frame applying the ops (raea counting allocator)
       rust_apply: (.rust_allocs.apply_per_frame // 0.0), rust_apply_kb: ((.rust_allocs.apply_bytes_per_frame // 0.0) / 1024)}' "$TMP/run.json"
  local rc=$?
  rm -f "$TMP/run.json"
  return $rc
}

res=""
turn_begin jsheap
for n in "${names[@]}"; do
  edit=$(edit_of "$n"); case $edit in /*) ;; *) edit=$ROOT/$edit ;; esac
  : > "$TMP/runs"
  for _ in $(seq 1 "$runs"); do once "$edit" >> "$TMP/runs" || { turn_end; say_tail "$n: the play failed: " 400 "$TMP/err" >&2; exit 1; }; done
  med=$(LS_JQ -sc 'include "lib"; . as $runs | reduce ($runs[0] | keys_unsorted[]) as $k ({}; .[$k] = ([$runs[][$k]] | median | pyround(3)))
    | .runs_kb_frame = [$runs[].kb_frame | pyround(1)]' "$TMP/runs")
  say "$(echo "$med" | LS_JQ -r --arg n "$n" 'include "lib"; "\($n): \(.kb_frame | fixed(1)) KB/frame (\(.runs_kb_frame | pylist)), \(.gc_min | fixed(0)) GC/min, max pause \(.pause_max | fixed(2)) ms, js p50 \(.js_p50 | fixed(3)) / p99 \(.js_p99 | fixed(3)) ms"')"
  res="$res$(echo "$med" | jq -c --arg n "$n" '{($n): .}')"$'\n'
done
turn_end
res=$(printf '%s' "$res" | jq -sc add)
lab=""; [ -n "$label" ] && lab=" ($label)"
record R1.js "$(echo "$res" | LS_JQ -c 'include "lib"; [to_entries[] | "\(.key) \(.value.kb_frame | fixed(0))"] | join(" / ")')" \
  "KB JS garbage a frame$lab" "$res" $H
record R1.rust "$(echo "$res" | LS_JQ -c 'include "lib"; [to_entries[] | "\(.key) \(.value.rust_apply | fixed(0))"] | join(" / ")')" \
  "Rust allocations a frame applying ops$lab" "$res" $H
