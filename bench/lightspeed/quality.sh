#!/bin/bash
# The rows CI's gates answer, measured on this tree and quiet (docs/lightspeed.feature): Q1, Q4, T3.
#
#   bench/lightspeed/quality.sh [--only replay,timelines,audio]
#
# CI's history can be many commits old or taken on a busy machine; this runs the same gates on HEAD's build, each in its
# own turn at the lock, and reads their numbers with CI's own parser (bench/ci/record.jq):
#   replay     sdk/react/bench/games/replay.sh: pong and platformer played with logged input, frames against renders
#   timelines  sdk/react/bench/timelines/check.sh titles: render = baked play (SSIM), bake ms, incremental re-bake ms (T3)
#   audio      sdk/react/bench/games/audio.sh: tap = bounce, loudness and true peak, clicks (Q4)
# Q1 is the worst SSIM over replay and timelines.
. "$(dirname "$0")/lib.sh"
H=bench/lightspeed/quality.sh
REACT=$ROOT/sdk/react

only=replay,timelines,audio
while [ $# -gt 0 ]; do
  case $1 in
    --only) only=$2; shift ;;
    *) echo "usage: bench/lightspeed/quality.sh [--only replay,timelines,audio]" >&2; exit 2 ;;
  esac
  shift
done

TMP=$(mktemp -d "${TMPDIR:-/tmp}/lightspeed-quality.XXXXXX")
trap 'cleanup_turn; rm -rf "$TMP"' EXIT

# gate NAME: runs it in its own turn; leaves $TMP/NAME.json ({status, numbers, lines, st: the turn's load and quiet})
gate() {
  local name=$1 tmp log t0 secs status rc
  tmp=$(mktemp -d "${TMPDIR:-/tmp}/lightspeed-$name-XXXXXX")
  log=$tmp/$name.log
  turn_begin "quality:$name"
  t0=$(date +%s)
  case $name in
    replay) (cd "$REACT" && TMPDIR=$tmp bash bench/games/replay.sh > "$log.out" 2> "$log.err") ;;
    timelines) (cd "$REACT" && TMPDIR=$tmp sh bench/timelines/check.sh titles > "$log.out" 2> "$log.err") ;;
    audio) (cd "$REACT" && TMPDIR=$tmp bash bench/games/audio.sh > "$log.out" 2> "$log.err") ;;
    *) echo "quality: no gate $name" >&2; turn_end; return 1 ;;
  esac
  rc=$?
  secs=$(( $(date +%s) - t0 ))
  turn_end
  cat "$log.out" "$log.err" > "$log"
  status=PASS; [ $rc = 0 ] || status=FAIL
  CI_DATE=$(date +%Y-%m-%dT%H:%M:%S) CI_COMMIT="" CI_TIER=lightspeed \
    jq -nc --arg gate "$name" --arg status $status --arg secs "$secs" --arg log "$log" --rawfile text "$log" \
      -f "$ROOT/bench/ci/record.jq" > "$TMP/$name.rec"
  jq -c --arg s "$ST_START" --arg e "$ST_END" --argjson q "$ST_QUIET" --argjson o "$ST_OTHERS" \
    '{status, numbers: (.numbers // {}), lines: (.lines // []), st: {start: $s, end: $e, quiet: $q, others: $o}}' "$TMP/$name.rec" > "$TMP/$name.json"
  say "$name: $status  $(jq -r '.lines[:4] | join(" | ")' "$TMP/$name.json")"
  rm -rf "$tmp"
}

# use_st NAME: the turn of gate NAME is the one a row records
use_st() {
  ST_START=$(jq -r .st.start "$TMP/$1.json"); ST_END=$(jq -r .st.end "$TMP/$1.json")
  ST_QUIET=$(jq -c .st.quiet "$TMP/$1.json"); ST_OTHERS=$(jq -c .st.others "$TMP/$1.json")
}

IFS=, read -r -a gates <<< "$only"
for g in "${gates[@]}"; do gate "$g" || exit 1; done
got() { [ -f "$TMP/$1.json" ]; }

if got replay || got timelines; then
  # every gate run, in the order run (the detail's status lists them all)
  q1=$(for g in "${gates[@]}"; do got "$g" && jq -c --arg g "$g" '{($g): .}' "$TMP/$g.json"; done | LS_JQ -sc 'include "lib"; add as $got
    | [("replay", "timelines") as $g | select($got[$g]) | $got[$g].numbers | to_entries[]
       | select((.value | type) == "object" and (.value | has("worst_ssim"))) | [.value.worst_ssim, "\($g):\(.key)"]] as $ss
    | if ($ss | length) == 0 then null else ($ss | min) as $w
      | {value: ("\($w[0] | fixed(6)) (\($w[1]))"), detail: {all: $ss, status: ($got | map_values(.status))}} end')
  if [ "$q1" != null ]; then
    if got replay; then use_st replay; else use_st timelines; fi
    record Q1 "$(echo "$q1" | jq -c .value)" "worst SSIM, replay + timelines on HEAD" "$(echo "$q1" | jq -c .detail)" $H
  fi
fi
if got timelines; then
  use_st timelines
  record T3 "$(LS_JQ -c 'include "lib"; "\(.numbers.titles.bake_ms | pystr) / \(.numbers.titles_edited.incremental.ms | pystr)"' "$TMP/timelines.json")" \
    "ms bake / incremental re-bake (titles)" "$(jq -c '{numbers, lines}' "$TMP/timelines.json")" $H
fi
if got audio; then
  use_st audio
  record Q4 "$(LS_JQ -c 'include "lib"; "\(.status) (pong live \(.numbers.pong.live_lufs | pystr) / render \(.numbers.pong.render_lufs | pystr) LUFS, clicks \(.numbers.tones.live_clicks | pystr))"' "$TMP/audio.json")" \
    "audio gate on HEAD" "$(jq -c '{numbers, lines}' "$TMP/audio.json")" $H
fi
exit 0
