#!/bin/bash
# lightspeed's scoreboard (docs/lightspeed.feature): every row's newest value, target, gap and trend, ranked by gap
# weighted by feel. The harnesses that fill it are bench/lightspeed/*.sh (bench/lightspeed/all.sh runs them all); the
# table itself is bench/lightspeed/scoreboard.jq.
#   bench/scoreboard.sh [--json]
ROOT=$(cd "$(dirname "$0")/.." && pwd)
RESULTS=$HOME/Library/Logs/raea/lightspeed/results.jsonl
CI=$HOME/Library/Logs/raea/ci/history.jsonl
json=0
for a in "$@"; do [ "$a" = --json ] && json=1; done
EMPTY=$(mktemp "${TMPDIR:-/tmp}/scoreboard.XXXXXX")
trap 'rm -f "$EMPTY"' EXIT
[ -f "$RESULTS" ] || RESULTS=$EMPTY
[ -f "$CI" ] || CI=$EMPTY

# Q6, the legacy app's eval runs, retired with apps/legacy (F2b): no rows
evals=null

# how many commits HEAD is past each CI gate's newest quiet commit (STALE past 10)
behind=$(jq -Rsr '[split("\n")[] | fromjson? | select(type == "object" and .quiet != false)]
    | reduce .[] as $r ({}; .[$r.gate] = $r) | .[] | (.commit // "")[:9] | select(. != "")' "$CI" | sort -u |
  while read -r c; do
    n=$(git -C "$ROOT" rev-list --count "$c..HEAD" 2> /dev/null) && echo "$c $n"
  done | jq -Rsc '[split("\n")[] | select(. != "") | split(" ") | {(.[0]): (.[1] | tonumber)}] | add // {}')

if [ $json = 1 ]; then out=(--indent 1); else out=(-r); fi
jq -n "${out[@]}" -L "$ROOT/bench/lightspeed" --rawfile results "$RESULTS" --rawfile ci "$CI" --argjson evals "$evals" \
  --argjson behind "$behind" --arg json $json -f "$ROOT/bench/lightspeed/scoreboard.jq"
