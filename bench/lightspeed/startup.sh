#!/bin/bash
# F5, open to first frame (docs/lightspeed.feature; its legacy-app rows and R3 retired with apps/legacy).
#
#   bench/lightspeed/startup.sh [--runs 5]
#
# Every time is measured from the moment the harness spawns the process (or sends the open) to the end of the first
# frame's present, on a quiet machine at the shared lock:
#   motion open, game      raea play pong --offscreen: spawn to frame 0 presented (play's first_frame_unix_ms
#                          less `raea ci time`'s clock just before the spawn)
#   motion open, timeline  the same for titles played from its bake (--baked), so the bake is inside it
#   timeline bake          the baked play's open_ms alone (React stepped through every frame, the bake written)
. "$(dirname "$0")/lib.sh"
H=bench/lightspeed/startup.sh
GAMES=$ROOT/sdk/react/bench/games
TIMELINES=$ROOT/sdk/react/bench/timelines

runs=5
while [ $# -gt 0 ]; do
  case $1 in
    --runs) runs=$2; shift ;;
    *) echo "usage: bench/lightspeed/startup.sh [--runs N]" >&2; exit 2 ;;
  esac
  shift
done

TMP=$(mktemp -d "${TMPDIR:-/tmp}/lightspeed-startup.XXXXXX")
trap 'cleanup_turn; rm -rf "$TMP"' EXIT

# motion_first_frame EDIT BAKED: prints "first_ms open_ms"
motion_first_frame() {
  local edit=$1 baked=$2 out=$TMP/play.json clock=$TMP/clock.json
  rm -f "$out"
  local extra=""
  [ "$baked" = 1 ] && extra=--baked
  RAEA_AUDIO_MUTE=1 "$MOTION" ci time --to "$clock" -- "$MOTION" play "$edit" $extra --offscreen --size 1280x720 --seconds 0.5 \
    --json "$out" > "$TMP/play.log" 2> "$TMP/play.err"
  if [ $? != 0 ] || [ ! -f "$out" ]; then
    say_tail "$edit: play failed: " 400 "$TMP/play.err" >&2; exit 1
  fi
  jq -r --slurpfile c "$clock" 'if .first_frame_unix_ms then "\(.first_frame_unix_ms - $c[0].start_unix_ms) \(.open_ms)"
    else "none" end' "$out" | { read -r a b; [ "$a" = none ] && { echo "raea has no first_frame_unix_ms: rebuild it" >&2; exit 1; }; echo "$a $b"; } || exit 1
}

# summary: {"p50", "max", "runs"} of the numbers on stdin, one a line
summary() { LS_JQ -sc 'include "lib"; {p50: (pct(0.5) | pyround(1)), max: (max | pyround(1)), runs: map(pyround(1))}'; }

pong=$GAMES/pong/edit.tsx; titles=$TIMELINES/titles/edit.tsx
r=$(motion_first_frame "$pong" 0) || exit 1  # warm the file cache and the shader cache: F5 is the warm open
: > "$TMP/game"; : > "$TMP/tl"
turn_begin startup:motion
for _ in $(seq 1 "$runs"); do r=$(motion_first_frame "$pong" 0) || exit 1; echo "${r% *}" >> "$TMP/game"; done
for _ in $(seq 1 "$runs"); do r=$(motion_first_frame "$titles" 1) || exit 1; echo "$r" >> "$TMP/tl"; done
turn_end
game=$(summary < "$TMP/game")
tl=$(awk '{print $1}' "$TMP/tl" | summary)
bake=$(awk '{print $2}' "$TMP/tl" | summary)
p50() { LS_JQ -r 'include "lib"; .p50 | pyf'; }
record F5.game "$(echo "$game" | p50)" ms "$game" $H
record F5.timeline "$(echo "$tl" | p50)" ms "$tl" $H
record F5.bake "$(echo "$bake" | p50)" ms "$bake" $H
