#!/usr/bin/env bash
# RR0 (docs/rerender.feature): the "one note" case. A note changes shot 4's grade and light (film.tsx's NOIR line), and
# this times what it takes to have the whole film again:
#   chunks  today's way by hand: shot 4's chunk rendered again and the six chunks joined again (ffmpeg -c copy, or
#           raea media concat where there's no ffmpeg); needs bench.sh's chunks (KEEP=1) in OUT
#   film    the film as one render again (what a render with no shot cache costs; before RR1 its picture isn't the film)
# The note is made on a copy of film.tsx in OUT, never on the checkout. Writes OUT/change.json.
#   bench/rerender/change.sh OUT      (OUT: a bench.sh run's folder, made with KEEP=1; RAEA_BIN as bench.sh)
set -u
cd "$(dirname "$0")/../.." || exit 2
ROOT=$PWD
EXE=raea; case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) EXE=raea.exe ;; esac
BIN=${RAEA_BIN:-$ROOT/target/release/$EXE}
OUT=${1:?usage: bench/rerender/change.sh OUT (a bench.sh run made with KEEP=1)}
for k in 1 2 3 4 5 6; do [ -f "$OUT/shot-$k.mp4" ] || { echo "change: no $OUT/shot-$k.mp4: run KEEP=1 bench/rerender/bench.sh $OUT first"; exit 2; }; done
command -v jq > /dev/null || { echo "change: needs jq"; exit 2; }
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
now() { perl -MTime::HiRes=time -e 'printf "%.3f", time'; }
span() { echo "$1 $2" | awk '{printf "%.3f", $2 - $1}'; }

# the note: shot 4 warmer, a little colour back, harder, under a stronger sun
NOTE=$OUT/note
mkdir -p "$NOTE"
sed 's/^export const NOIR = .*/export const NOIR = { saturation: 0.25, contrast: 1.5, temperature: 0.04, sun: 45000, ambient: 250 };/' "$ROOT/bench/rerender/film.tsx" > "$NOTE/film.tsx"
grep -q 'saturation: 0.25, contrast: 1.5' "$NOTE/film.tsx" || { echo "change: film.tsx's NOIR line wasn't found to change"; exit 1; }

status=0
# today's way by hand: the changed chunk again, then the join again
t0=$(now)
"$BIN" render "$NOTE/film.tsx" --composition shot-4 -o "$NOTE/shot-4.mp4" > "$NOTE/shot-4.log" 2>&1 && grep -q '^APPLIED' "$NOTE/shot-4.log" || { echo "change: shot 4 failed: $(grep -E 'REFUSED|error' "$NOTE/shot-4.log" | head -2)"; status=1; }
t1=$(now)
: > "$NOTE/list.txt"
for k in 1 2 3 4 5 6; do
  f=$OUT/shot-$k.mp4; [ $k = 4 ] && f=$NOTE/shot-4.mp4
  echo "file '$f'" >> "$NOTE/list.txt"
done
if command -v ffmpeg > /dev/null; then
  joiner="ffmpeg -f concat -c copy"
  ffmpeg -y -loglevel error -f concat -safe 0 -i "$NOTE/list.txt" -c copy "$NOTE/film-chunked.mp4" || status=1
else
  joiner="raea media concat"
  "$BIN" media concat $(sed "s/^file '//; s/'$//" "$NOTE/list.txt") -o "$NOTE/film-chunked.mp4" > "$NOTE/concat.log" 2>&1 || status=1
fi
t2=$(now)
# no shot cache: the whole film again
"$BIN" render "$NOTE/film.tsx" --composition film -o "$NOTE/film-one.mp4" > "$NOTE/film.log" 2>&1 && grep -q '^APPLIED' "$NOTE/film.log" || { echo "change: the film failed"; status=1; }
t3=$(now)
jq -n --argjson chunk "$(span "$t0" "$t1")" --argjson join "$(span "$t1" "$t2")" --argjson film "$(span "$t2" "$t3")" --arg joiner "$joiner" \
  '{note: "shot 4 (150 of 900 frames): grade and sun", chunks: {shot_s: $chunk, join_s: $join, total_s: ($chunk + $join), joiner: $joiner},
    film: {total_s: $film}}' > "$OUT/change.json"
echo "change: shot 4 again $(span "$t0" "$t1") s + join $(span "$t1" "$t2") s; the whole film again $(span "$t2" "$t3") s"
[ "${KEEP:-0}" = 1 ] || rm -f "$NOTE"/*.mp4
exit $status
