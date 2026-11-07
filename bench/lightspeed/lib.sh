# Shared by lightspeed's harnesses (docs/lightspeed.feature), sourced: the quiet-machine turn at the shared measurement
# lock, the load recorded beside every number, and one results log the scoreboard reads.
#
# Results: ~/Library/Logs/raea/lightspeed/results.jsonl, one line per measured row:
#   {"date", "commit", "row", "value", "unit", "detail", "load": {"start", "end"}, "quiet", "others", "harness"}
# "others": other agents' busy antsy jobs seen during the run (any makes it NOT QUIET).
# RAEA_LIGHTSPEED_MOTION: the raea to measure (default target/release/raea under this checkout).
# Bash 3.2 (macOS's /bin/bash, what launchd's CI runs); jq for JSON (lib.jq: Python's rounding, so rows read as the
# Python harnesses' did), awk for arithmetic, and `raea ci` (apps/ci) for what a shell can't time or
# read: milliseconds from a spawn, a process's energy, footprint(1)'s graphics memory.

LS_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$LS_DIR/../.." && pwd)
MOTION=${RAEA_LIGHTSPEED_MOTION:-$ROOT/target/release/raea}
LOCK=/tmp/raea-measure.lock
LOGDIR=$HOME/Library/Logs/raea/lightspeed
RESULTS=$LOGDIR/results.jsonl
FIXTURES=${RAEA_LIGHTSPEED_FIXTURES:-$HOME/Library/Caches/raea-lightspeed/fixtures}
QUIET=${RAEA_CI_QUIET_LOAD:-6}
# another agent's job counts against a run once it has used this much memory: idle waiters and lock polls (a few MB)
# don't, a build, render, export or model run does
BUSY_MB=100
# set in a turn's environment, so every process the harness starts carries it, even one that leaves the parent chain
# (a helper that detaches is re-parented to launchd): one once marked its run NOT QUIET
MARK=RAEA_LIGHTSPEED_TURN

LS_JQ() { jq -L "$LS_DIR" "$@"; }

say() { echo "$*"; }

# say_tail PREFIX N FILE: PREFIX and the file's last N characters, as print(prefix + text[-N:]) writes them (characters,
# not bytes; the text's own trailing newline kept)
say_tail() { LS_JQ -rn --rawfile t "$3" --argjson n "$2" --arg p "$1" '$p + ($t | .[-$n:])'; }

# a decimal as Python's str(float(...)) writes it: 2.00 -> 2.0, 6 -> 6.0, 2.10 -> 2.1
pyf() {
  case $1 in
    *.*) echo "$1" | sed -e 's/\(\.[0-9]*[1-9]\)0*$/\1/' -e 's/\.0*$/.0/' ;;
    '') echo None ;;
    *) echo "$1.0" ;;
  esac
}

load1() { sysctl -n vm.loadavg | awk '{print $2}'; }

commit() { git -C "$ROOT" rev-parse --short=9 HEAD 2>/dev/null; }

# Other agents' jobs running now, one name a line: from `antsy board`, those that reached BUSY_MB, leaving out this
# harness's own job (ANTSY_JOB); and raea's own engines run outside antsy (an agent's quick A/B of raea plays):
# raea, raea-game and raea-page-host processes this harness didn't start (not its descendants, not carrying its turn's
# mark) that are busy (BUSY_MB resident, or a quarter of a core). An idle one (a `shell-serve` an exited app left behind,
# ~45 MB at 2-3% of a core) doesn't count.
other_jobs() {
  antsy board 2>/dev/null | awk -v me="${ANTSY_JOB:-}" -v busy=$BUSY_MB '
    match($0, /^[ \t]*job [^ \t]+ \[[^]]*\] /) {
      split(substr($0, 1, RLENGTH), w, " "); name = w[2]
      rest = substr($0, RLENGTH + 1)
      if (!match(rest, / peak [0-9.]+ (MB|GB)/)) next
      split(substr(rest, RSTART + 1, RLENGTH - 1), p, " ")
      mb = p[2] * (p[3] == "GB" ? 1024 : 1)
      if (name != me && mb >= busy) print name
    }'
  ps -Ao pid=,ppid=,rss=,pcpu=,comm= 2>/dev/null | awk -v me=$$ -v busy=$BUSY_MB '
    $1 ~ /^[0-9]+$/ && $2 ~ /^[0-9]+$/ && NF >= 5 {
      n++; pid[n] = $1; ppid[n] = $2; rss[n] = ($3 ~ /^[0-9]+$/) ? $3 : 0; cpu[n] = $4 + 0
      c = $0; for (i = 1; i <= 4; i++) sub(/^[ \t]*[^ \t]+/, "", c); sub(/^[ \t]+/, "", c)
      k = split(c, parts, "/"); comm[n] = parts[k]
    }
    END {
      mine[me] = 1
      do { grew = 0; for (i = 1; i <= n; i++) if (!(pid[i] in mine) && (ppid[i] in mine)) { mine[pid[i]] = 1; grew = 1 } } while (grew)
      for (i = 1; i <= n; i++)
        if ((comm[i] == "raea" || comm[i] == "raea-game" || comm[i] == "raea-page-host") && !(pid[i] in mine) && (rss[i] / 1024 >= busy || cpu[i] >= 25))
          print pid[i], comm[i]
    }' | while read -r pid name; do ours "$pid" || echo "$name pid $pid"; done
}

# whether process PID was started inside this harness's turn: its environment carries the turn's mark (ps -E shows the
# environment of the person's own processes)
ours() {
  local mark
  eval "mark=\${$MARK:-}"
  [ -n "$mark" ] || return 1
  ps -Ewwo command= -p "$1" 2>/dev/null | grep -qF "$MARK=$mark"
}

# Other agents' busy jobs seen during a run: at the start, every 5 s, and at the end (a 65 s export that began and
# finished inside a game's run once put a 19 ms step on the platformer the load average never showed)
watch_start() {
  WATCH_FILE=$(mktemp "${TMPDIR:-/tmp}/lightspeed-watch.XXXXXX")
  other_jobs > "$WATCH_FILE"
  # the poller stops at its next tick once WATCH_FILE.stop exists (a poll under way finishes, as the Python thread's did)
  ( trap 'exit 0' TERM; while [ ! -e "$WATCH_FILE.stop" ]; do sleep 5 & wait $!; [ -e "$WATCH_FILE.stop" ] || other_jobs >> "$WATCH_FILE"; done ) &
  WATCH_PID=$!
}

# the names seen, as a JSON array: a job's name without antsy's run suffix (stress-r5-9da29 -> stress-r5), sorted, once
watch_end() {
  if [ -n "${WATCH_PID:-}" ]; then
    : > "$WATCH_FILE.stop"; pkill -TERM -x -P "$WATCH_PID" sleep 2>/dev/null; wait "$WATCH_PID" 2>/dev/null
    WATCH_PID=
  fi
  other_jobs >> "$WATCH_FILE"
  sed -E 's/-[0-9a-f]{5}$//' "$WATCH_FILE" | LC_ALL=C sort -u | jq -Rsc 'split("\n") | map(select(. != ""))'
  rm -f "$WATCH_FILE" "$WATCH_FILE.stop"
}

HELD_LOCK=0
cleanup_turn() {
  [ -n "${WATCH_PID:-}" ] && { : > "$WATCH_FILE.stop"; kill -TERM "$WATCH_PID" 2>/dev/null; pkill -TERM -x -P "$WATCH_PID" sleep 2>/dev/null; rm -f "$WATCH_FILE" "$WATCH_FILE.stop"; }
  [ "$HELD_LOCK" = 1 ] && rmdir "$LOCK" 2>/dev/null
  HELD_LOCK=0
}
# a harness stopped from outside (antsy, CI's timeout) still gives the lock back
trap 'cleanup_turn; exit 143' TERM
trap 'cleanup_turn; exit 130' INT
trap cleanup_turn EXIT

# turn_begin NAME [WAIT_S] ... turn_end: a measurement's turn. Sets ST_START, then ST_END, ST_OTHERS (a JSON array) and
# ST_QUIET (under QUIET at both ends and no other agent's busy job seen). The user, 2026-10-18: nobody waits for a quiet
# machine; a row is marked with the load it was measured under, and levers are judged by paired A/Bs and by counts (CPU
# time, allocations, frames), which hold under load. So by default there's no wait and no lock. RAEA_LIGHTSPEED_QUIET=1
# (the nightly pass in bench/ci.sh) waits up to WAIT_S (600) for a quiet machine and takes the shared lock;
# RAEA_LIGHTSPEED_LOCK=1 takes the lock alone (a short paired A/B).
turn_begin() {
  TURN_NAME=$1
  local wait_s=${2:-600} t=0
  if [ "${RAEA_LIGHTSPEED_QUIET:-}" = 1 ] || [ "${RAEA_LIGHTSPEED_LOCK:-}" = 1 ]; then
    while [ "${RAEA_LIGHTSPEED_QUIET:-}" = 1 ] && awk -v l="$(load1)" -v q="$QUIET" 'BEGIN { exit !(l + 0 >= q + 0) }' && [ $t -lt "$wait_s" ]; do
      [ $((t % 60)) = 0 ] && say "$TURN_NAME: waiting for a load under $(pyf "$QUIET") (now $(pyf "$(load1)"), $t s of $wait_s)"
      sleep 15; t=$((t + 15))
    done
    t=0
    until mkdir "$LOCK" 2>/dev/null; do
      [ $((t % 60)) = 0 ] && say "$TURN_NAME: waiting for the measurement lock ($t s)"
      sleep 5; t=$((t + 5))
    done
    HELD_LOCK=1
  fi
  TURN_HELD=$(date +%s)
  ST_START=$(load1)
  export $MARK="$$-$(date +%s)-$RANDOM"
  watch_start
}

turn_end() {
  unset $MARK
  if [ "$HELD_LOCK" = 1 ]; then rmdir "$LOCK" 2>/dev/null; HELD_LOCK=0; local locked=1; else local locked=0; fi
  ST_END=$(load1)
  ST_OTHERS=$(watch_end)
  # quiet only when the load stayed under the line at both ends and no other agent's busy job ran meanwhile: a run the
  # machine got busy during isn't clean
  if [ "$ST_OTHERS" = "[]" ] && awk -v a="$ST_START" -v b="$ST_END" -v q="$QUIET" 'BEGIN { exit !((a > b ? a : b) + 0 < q + 0) }'; then
    ST_QUIET=true
  else
    ST_QUIET=false
  fi
  local secs=$(( $(date +%s) - TURN_HELD ))
  [ $locked = 1 ] && [ $secs -gt 600 ] && say "$TURN_NAME: held the measurement lock $secs s (over a 10 minute turn)"
  return 0
}

# record ROW VALUE UNIT DETAIL HARNESS: one row to results.jsonl, and its line on stdout. VALUE is JSON as Python's str()
# writes it (12.0, "a string", null); DETAIL is JSON. The turn's ST_START, ST_END (empty: null), ST_QUIET, ST_OTHERS.
record() {
  local row=$1 value=$2 unit=$3 detail=$4 harness=$5
  mkdir -p "$LOGDIR"
  # the loads as Python's floats (sysctl's 2.00 -> 2.0)
  local start=null end=null
  [ -n "${ST_START:-}" ] && start=$(pyf "$ST_START")
  [ -n "${ST_END:-}" ] && end=$(pyf "$ST_END")
  LS_JQ -nc --arg date "$(date +%Y-%m-%dT%H:%M:%S)" --arg commit "$(commit)" --arg row "$row" --argjson value "$value" \
    --arg unit "$unit" --argjson detail "$detail" --argjson start "$start" --argjson end "$end" \
    --argjson quiet "${ST_QUIET:-null}" --argjson others "${ST_OTHERS:-[]}" --arg harness "$harness" \
    '{date: $date, commit: $commit, row: $row, value: $value, unit: $unit, detail: $detail, load: {start: $start, end: $end},
      quiet: $quiet, others: $others, harness: $harness}' >> "$RESULTS"
  local shown q=""
  shown=$(LS_JQ -rn --argjson v "$value" 'include "lib"; if ($v | type) == "string" then $v else $v | pystr end')
  if [ "${ST_QUIET:-}" != true ]; then
    q=$(jq -rn --argjson o "${ST_OTHERS:-[]}" 'if ($o | length) > 0 then "  (NOT QUIET: other jobs " + ($o | join(", ")) + ")" else "  (NOT QUIET)" end')
  fi
  say "$row: $shown $unit  load $( [ "$start" = null ] && echo None || pyf "$start") -> $( [ "$end" = null ] && echo None || pyf "$end")$q"
}
