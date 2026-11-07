#!/bin/bash
# R6, energy (docs/lightspeed.feature): watts while playing pong and playing a timeline (the legacy app's idle
# stage retired with apps/legacy).
#
#   bench/lightspeed/energy.sh [--seconds 30]
#
# Without root (powermetrics needs it), from two counters (`raea ci energy`):
#   process watts  the process's own energy (proc_pid_rusage's ri_energy_nj, rusage_info_v6: the CPU energy the kernel
#                  attributes to it) over the run past its first 5 s. GPU energy isn't attributed per process, so this is
#                  the CPU's part; it's the row's value
#   system watts   the whole Mac's input power (the battery controller's SystemPowerIn, mW) sampled each second: context
#                  only, since everything else running counts too; with the load recorded beside it
#   pong      raea play pong --offscreen at its 120 fps, keys pressed as gate.sh presses them
#   timeline  titles played from its bake (--baked), 30 fps
. "$(dirname "$0")/lib.sh"
H=bench/lightspeed/energy.sh
set -o pipefail
GAMES=$ROOT/sdk/react/bench/games
TIMELINES=$ROOT/sdk/react/bench/timelines

seconds=30
while [ $# -gt 0 ]; do
  case $1 in
    --seconds) seconds=$2; shift ;;
    *) echo "usage: bench/lightspeed/energy.sh [--seconds S]" >&2; exit 2 ;;
  esac
  shift
done
seconds=$(pyf "$seconds")
# the play runs 2 s past the watch (Python: str(seconds + 2))
secs2=$(pyf "$(awk -v s="$seconds" 'BEGIN { print s + 2 }')")

# the watch's raw numbers rounded as the rows keep them
rounded() { LS_JQ -c 'include "lib"; {process_w: (.process_w | pyround(3)), system_w: (.system_w | if . == null then null else pyround(1) end), span_s: (.span_s | pyround(1))}'; }

# motion EDIT MODE: play it (MODE --input or --baked) and watch its energy
motion() {
  (cd "$ROOT/sdk/react" && RAEA_AUDIO_MUTE=1 RAEA_INPUT_KEYS="" "$MOTION" ci energy --seconds "$seconds" -- "$MOTION" play "$1" \
    --offscreen --size 1280x720 --seconds "$secs2" "$2") | rounded
}

trap cleanup_turn EXIT

turn_begin energy
pong=$(motion "$GAMES/pong/edit.tsx" --input) || { turn_end; exit 1; }
timeline=$(motion "$TIMELINES/titles/edit.tsx" --baked) || { turn_end; exit 1; }
turn_end
res=$(jq -nc --argjson p "$pong" --argjson t "$timeline" '{pong: $p, timeline: $t}')
record R6 "$(echo "$res" | LS_JQ -c 'include "lib"; "\(.pong.process_w | pyf) / \(.timeline.process_w | pyf)"')" \
  "W (CPU, process) pong / timeline" "$res" $H
