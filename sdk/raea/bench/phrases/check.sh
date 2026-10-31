#!/bin/bash
# The Phrases check (examples/parts/reel: layout.ts `spanOf`, index.tsx `unsaid`): a <Phrases> places itself by the lines it says.
# words.json beside the cases is a made-up take, one word every 10 frames ("then" at 60, "your" at 80, "rock" at 90):
#   cue-less.tsx  a Hook, then a Phrases with no cue: the Hook is up at frame 30, "then name" at 65, "your rock" at 95
#                 (before, the Phrases started at 0 with the Hook, the Hook was cut to one frame, frame 30 was empty)
#   early.tsx     a Phrases from={0} seconds={2} whose first line is said at frame 60: refused, naming frame 60 and
#                 saying to start the Phrases there
#   unsaid.tsx    a Phrases whose line isn't said at all: refused, saying so
#
#   sdk/raea/bench/phrases/check.sh     (RAEA_BIN, default raea)
# Prints a line per check (PASS or MISS) and exits 1 when any misses.
set -u
export RAEA_SIGN=0 RAEA_MARK=0
HERE=$(cd "$(dirname "$0")" && pwd)
BIN=${RAEA_BIN:-raea}
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-phrases.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
bad=0
# each case renders in a copy of its own (with words.json), so what a render makes (the parts' sounds) stays out of the tree
# and the reel's parts as `raea part add reel` copies them (examples/parts/reel, without its tests)
REEL=$(cd "$HERE/../../../../examples/parts/reel" && pwd)
render() { mkdir -p "$OUT/p/$1/components/reel" && cp "$HERE/$1.tsx" "$HERE/words.json" "$OUT/p/$1/" && (cd "$REEL" && cp $(ls | grep -v '\.test\.') "$OUT/p/$1/components/reel/") && (cd "$OUT/p/$1" && "$BIN" render "$1.tsx" --at "$2" -o "$OUT/$1/") > "$OUT/$1.log" 2>&1; }
# the top band's mean luma: 16 when it's empty (black, limited range), above 20 with a part's white or accent type on it
band() { ffmpeg -nostdin -hide_banner -nostats -i "$1" -vf "crop=iw:ih/5:0:0,signalstats,metadata=print:key=lavfi.signalstats.YAVG" -f null - 2>&1 | grep -o 'YAVG=[0-9.]*' | cut -d= -f2; }
if render cue-less 30,65,95; then
  for k in "0 30 the Hook" "1 65 then name" "2 95 your rock"; do
    set -- $k
    y=$(band "$OUT/cue-less/frame-0000$1.png")
    awk -v y="${y:-0}" 'BEGIN { exit !(y > 20) }' && echo "PASS cue-less frame $2: ${*:3} on the band (brightness $y)" || { echo "MISS cue-less frame $2: the band is empty (brightness ${y:-none})"; bad=1; }
  done
else
  echo "MISS cue-less: the render failed: $(grep -m1 -E 'REFUSED|error' "$OUT/cue-less.log" | cut -c1-240)"; bad=1
fi
if render early 30; then echo "MISS early: rendered"; bad=1
elif grep -q 'is said at frame 60, outside the part.s window (frames 0–60): start the Phrases at frame 60' "$OUT/early.log"; then
  echo "PASS early: refused, $(grep -m1 -o 'line "then name" is said at frame 60[^)]*)' "$OUT/early.log")"
else echo "MISS early: refused without naming frame 60: $(grep -m1 -E 'REFUSED' "$OUT/early.log" | cut -c1-240)"; bad=1
fi
if render unsaid 30; then echo "MISS unsaid: rendered"; bad=1
elif grep -q "isn't said in the take" "$OUT/unsaid.log"; then echo "PASS unsaid: refused, the line isn't said in the take"
else echo "MISS unsaid: refused without saying why: $(grep -m1 -E 'REFUSED' "$OUT/unsaid.log" | cut -c1-240)"; bad=1
fi
exit $bad
