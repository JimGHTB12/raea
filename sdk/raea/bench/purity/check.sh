#!/bin/bash
# The frame-purity check (sdk/raea/src/pure.ts, wired by sdk/react motion.rs's entries): Math.random and Date in a
# render are the frame's, so a frame is the same however it's reached, and frames still differ from one another.
#   frame 10 alone, twice           the same numbers and time, and the same picture
#   frame 10 after frame 9          the same numbers and picture as frame 10 alone (nothing carried over from the frame before)
#   frame 11                        another picture (its own numbers)
#   frame 10's text                 its Date is 2026-01-01T00:00:00.333Z (the epoch plus 10 frames at 30 fps)
#   a game's steps (game.tsx)       frame 30 alone holds the same draws for steps 0-19 as frame 20 alone (a step is
#                                   seeded by its own frame, not by whichever frame's render ran the fold), and its
#                                   steps differ from one another
#   two live sessions (raea play --offscreen): another picture at 0.5 s (each its own seed)
#   session 1's input log rendered at frame 15: session 1's picture at 0.5 s (the log keeps its seed)
#
#   sdk/raea/bench/purity/check.sh     (RAEA_BIN, default raea)
# Prints a line per check (PASS or MISS) and exits 1 when any misses.
set -u
export RAEA_SIGN=0 RAEA_MARK=0
HERE=$(cd "$(dirname "$0")" && pwd)
BIN=${RAEA_BIN:-raea}
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-purity.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
bad=0
r() { (cd "$HERE" && RAEA_PAGE_DUMP="$OUT/$1" "$BIN" render pure.tsx --at "$2" -o "$OUT/$1/") > "$OUT/$1.log" 2>&1 || { echo "MISS $1: the render failed: $(grep -m1 -E 'REFUSED|error' "$OUT/$1.log" | cut -c1-200)"; bad=1; }; }
for x in "a 10" "b 10" "c 9,10" "d 11"; do set -- $x; mkdir -p "$OUT/$1"; r "$1" "$2"; done
h() { [ -f "$1" ] && shasum "$1" | cut -c1-16; }
# what a frame drew: its numbers and time as the page held them (RAEA_PAGE_DUMP), and pixels alike within the GPU's
# own run-to-run noise (a glyph pixel 1/255 apart now and then: SSIM >= 0.9999)
nums() { grep -o 'data-pure="[^"]*"' "$OUT/$1/page.html" 2>/dev/null | head -1 | sed 's/^data-pure="//; s/"$//; s/&quot;/"/g'; }
alike() { [ -f "$1" ] && [ -f "$2" ] && awk -v s="$(ffmpeg -nostdin -hide_banner -nostats -i "$1" -i "$2" -lavfi "[0:v]format=rgb24[a];[1:v]format=rgb24[b];[a][b]ssim" -f null - 2>&1 | grep -o 'All:[0-9.]*' | cut -d: -f2)" 'BEGIN { exit !(s >= 0.9999) }'; }
a=$(h "$OUT/a/frame-00000.png") b=$(h "$OUT/b/frame-00000.png") c=$(h "$OUT/c/frame-00001.png") d=$(h "$OUT/d/frame-00000.png")
[ -n "$(nums a)" ] && [ "$(nums a)" = "$(nums b)" ] && alike "$OUT/a/frame-00000.png" "$OUT/b/frame-00000.png" && echo "PASS frame 10 twice: the same numbers and picture" || { echo "MISS frame 10 twice: $(nums a) vs $(nums b)"; bad=1; }
[ -n "$(nums a)" ] && [ "$(nums a)" = "$(nums c)" ] && alike "$OUT/a/frame-00000.png" "$OUT/c/frame-00001.png" && echo "PASS frame 10 after frame 9: the same numbers and picture as alone" || { echo "MISS frame 10 after frame 9: $(nums c) vs $(nums a)"; bad=1; }
[ -n "$d" ] && [ "$d" != "$a" ] && echo "PASS frame 11: another picture ($d)" || { echo "MISS frame 11: the same picture as frame 10"; bad=1; }
said=$(grep -o 'data-pure="[^"]*"' "$OUT/a/page.html" 2>/dev/null | head -1 | sed 's/^data-pure="//; s/"$//; s/&quot;/"/g')
now=$(jq -r '.now' <<< "$said" 2>/dev/null)
want=$(( $(date -j -u -f "%Y-%m-%d %H:%M:%S" "2026-01-01 00:00:00" +%s) * 1000 + 333 ))
[ "$now" = "$want" ] && echo "PASS frame 10's Date.now: $now (2026-01-01T00:00:00.333Z)" || { echo "MISS frame 10's Date.now: ${now:-none}, want $want"; bad=1; }
echo "     frame 10 drew: $said"
# a game's fold: each step's draw is its own frame's, however the fold got there
for x in "g20 20" "g30 30"; do set -- $x; mkdir -p "$OUT/$1"; (cd "$HERE" && RAEA_PAGE_DUMP="$OUT/$1" "$BIN" render game.tsx --at "$2" -o "$OUT/$1/") > "$OUT/$1.log" 2>&1 || { echo "MISS game frame $2: the render failed: $(grep -m1 -E 'REFUSED|error' "$OUT/$1.log" | cut -c1-200)"; bad=1; }; done
g20=$(nums g20 | jq -c '.drawn' 2>/dev/null) g30=$(nums g30 | jq -c '.drawn[:20]' 2>/dev/null)
[ "$(jq length <<< "${g20:-[]}")" = 20 ] && [ "$g20" = "$g30" ] && [ "$(jq 'unique | length' <<< "$g20")" = 20 ] && echo "PASS a game's steps: frame 30 alone drew the same for steps 0-19 as frame 20 alone, 20 different numbers" || { echo "MISS a game's steps: frame 20 ${g20:-none}, frame 30's first 20 ${g30:-none}"; bad=1; }
# live play: a seed of its own each session, kept in its input log, so a render of the log draws what was played
# (1 s: the play reaches its snapshot frame, 15, however loaded the machine, as play keeps stepping to it)
for k in 1 2; do
  (cd "$HERE" && "$BIN" play pure.tsx --offscreen --seconds 1 --input-log "$OUT/log$k.json" --snapshot 0.5 "$OUT/snap$k.png") > "$OUT/play$k.log" 2>&1 || { echo "MISS play $k failed: $(grep -m1 -E 'error|REFUSED' "$OUT/play$k.log" | cut -c1-200)"; bad=1; }
done
(cd "$HERE" && "$BIN" render pure.tsx --input "$OUT/log1.json" --at 15 -o "$OUT/replay/") > "$OUT/replay.log" 2>&1 || { echo "MISS the replay failed"; bad=1; }
p1=$(h "$OUT/snap1.png") p2=$(h "$OUT/snap2.png") rp=$(h "$OUT/replay/frame-00000.png")
[ -n "$p1" ] && [ -n "$p2" ] && [ "$p1" != "$p2" ] && echo "PASS two live sessions: other pictures ($p1, $p2)" || { echo "MISS two live sessions: $p1 vs $p2"; bad=1; }
alike "$OUT/replay/frame-00000.png" "$OUT/snap1.png" && echo "PASS session 1's log rendered: its picture (SSIM >= 0.9999) (session $(jq -c '.["0"].session' "$OUT/log1.json"))" || { echo "MISS session 1's log rendered: $rp vs $p1"; bad=1; }
exit $bad
