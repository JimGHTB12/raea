#!/bin/sh
# Every lightspeed harness in turn (docs/lightspeed.feature), each timing step in its own turn at the shared lock on a
# quiet machine (bench/lightspeed/lib.sh), then the scoreboard. The iPhone rows need the phone unlocked; they're
# skipped with a note when it isn't.
#   bench/lightspeed/all.sh [quick]      quick: 30 s game runs and 2 repeats instead of 60 s and 3
# Build first: raea (sdk/react/) and void (apps/legacy/shell/), release.
cd "$(dirname "$0")"
secs=60; runs=3
[ "${1:-}" = quick ] && { secs=30; runs=2; }
status=0
bash startup.sh --runs $((runs + 2)) || status=1
bash feel.sh --seconds $secs || status=1
bash jsheap.sh pong pong-stepfold pong-view --seconds 12 --runs $runs || status=1
bash export.sh --runs $runs || status=1
bash quality.sh || status=1
bash gpumem.sh || status=1
bash energy.sh --seconds 30 || status=1
bash ../scoreboard.sh
exit $status
