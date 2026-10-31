#!/bin/bash
# The Bevy-to-TypeScript map's cases (docs/bevy-ts.feature): one small TSX scene per gap, rendered headless at its own
# 64 x 64 and probed. A case passes when its render succeeds and every probe holds; a gap is closed only when its case
# passes. Each case says what it checks in its header comments:
#   // gap: G2                     the gap it belongs to (docs/bevy-ts.feature, "The gaps")
#   // probe: X Y r>200 g<80 ...    the pixel at (X, Y) of frame 0: each channel (r, g, b) against a number (< or >)
#   // rows: REGEX                  the scene's rows (RAEA_SCENE_ROWS=1) must match this extended regex
#   // pending: WHY                 not checkable by a still yet: counted open, with the reason
#   // build: plugins               render with the plugin build (sdk/raea/bench/bevy-plugins/motion.sh)
#   // frame: N                     probe frame N instead of 0
#   // input: FILE.json             the input timeline the render plays (render --input; next to the case)
#   // play: FILE.json SECONDS      live play instead of a render: `play --offscreen` with FILE as its --input-script
#                                   (marked keys from each frame on, the live wire), probed at a snapshot at SECONDS
#
#   sdk/raea/bench/bevy-ts/check.sh [CASE ...]     (default: every case; a CASE is a name or a path)
# Prints a line per case and a tally; exits 1 when any case is open.
set -u
# pixel checks: no Content Credentials or TrustMark id in the frames (a still's mark changes its pixels each run)
export RAEA_SIGN=0 RAEA_MARK=0
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
BIN=${RAEA_BIN:-raea}
OUT=${TMPDIR:-/tmp}/raea-bevy-ts
mkdir -p "$OUT"
cases=()
if [ $# -eq 0 ]; then
  for f in "$HERE"/cases/*.tsx; do cases+=("$f"); done
else
  for c in "$@"; do [ -f "$c" ] && cases+=("$c") || cases+=("$HERE/cases/$c.tsx"); done
fi
pass=0
open=0
for edit in "${cases[@]}"; do
  name=$(basename "$edit" .tsx)
  gap=$(sed -n 's|^// gap: *||p' "$edit" | head -1)
  pending=$(sed -n 's|^// pending: *||p' "$edit" | head -1)
  if [ -n "$pending" ]; then
    echo "$name ($gap): OPEN  pending: $pending"; open=$((open + 1)); continue
  fi
  bin=$BIN
  grep -q '^// build: plugins' "$edit" && bin="$ROOT/sdk/raea/bench/bevy-plugins/motion.sh"
  png="$OUT/$name.png"
  log="$OUT/$name.log"
  rm -f "$png"
  frame=$(sed -n 's|^// frame: *||p' "$edit" | head -1)
  extra=()
  input=$(sed -n 's|^// input: *||p' "$edit" | head -1)
  [ -n "$input" ] && extra=(--input "$(dirname "$edit")/$input")
  play=$(sed -n 's|^// play: *||p' "$edit" | head -1)
  if [ -n "$play" ]; then
    set -- $play
    run=("$bin" play "$edit" --offscreen --no-vsync --size 64x64 --seconds "$(echo "$2 + 0.3" | bc)" --input-script "$(dirname "$edit")/$1" --snapshot "$2" "$png")
  else
    run=("$bin" render "$edit" --frame "${frame:-0}" ${extra[@]+"${extra[@]}"} -o "$png")
  fi
  if ! RAEA_SCENE_ROWS=1 nice -n 10 "${run[@]}" > "$log" 2>&1 || [ ! -f "$png" ]; then
    why=$(grep -iE "error|refused|not |no " "$log" | grep -v "global logger" | tail -1 | cut -c1-160)
    echo "$name ($gap): OPEN  the render failed: ${why:-see $log}"; open=$((open + 1)); continue
  fi
  bad=""
  while IFS= read -r re; do
    grep -Eq "$re" "$log" || bad="$bad rows don't match /$re/;"
  done < <(sed -n 's|^// rows: *||p' "$edit")
  while IFS= read -r line; do
    set -- $line
    x=$1; y=$2; shift 2
    rgb=$(magick "$png" -format "%[fx:int(255*p{$x,$y}.r+0.5)] %[fx:int(255*p{$x,$y}.g+0.5)] %[fx:int(255*p{$x,$y}.b+0.5)]" info:)
    read -r r g b <<< "$rgb"
    for t in "$@"; do
      ch=${t:0:1}; op=${t:1:1}; want=${t:2}
      case $ch in r) have=$r ;; g) have=$g ;; b) have=$b ;; *) bad="$bad bad probe $t;"; continue ;; esac
      if { [ "$op" = ">" ] && [ "$have" -gt "$want" ]; } || { [ "$op" = "<" ] && [ "$have" -lt "$want" ]; }; then :; else
        bad="$bad ($x,$y) is rgb($r,$g,$b), wants $t;"
      fi
    done
  done < <(sed -n 's|^// probe: *||p' "$edit")
  if [ -n "$bad" ]; then
    echo "$name ($gap): OPEN $bad"; open=$((open + 1))
  else
    echo "$name ($gap): PASS"; pass=$((pass + 1))
  fi
done
echo "bevy-ts: $pass pass, $open open (frames and logs in $OUT)"
[ $open -eq 0 ]
