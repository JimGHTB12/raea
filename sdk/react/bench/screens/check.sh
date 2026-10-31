#!/bin/bash
# Flat <Screen>s (sdk/raea/src/screen.tsx) drawn as their pages: bench/screens/edit.tsx rendered as a folder of stills
# (--at, the path that drew every screen as bare glass until 10-22), as one still, and played live, then checked:
#   - frame 0's page at its own colours (#ff3030 under no tone curve), not black or the glass's grey-green
#   - frames 2 and 3: a see-through page (#ffffff00) shows the blue screen behind it, its white card stays white, also
#     under DepthOfField
#   - the single still and live play's snapshots equal the folder's frames (SSIM >= 0.99)
#   sdk/react/bench/screens/check.sh
# Build first: lane build -- cargo build --release -p raea-cli -j 6 (target/release/raea; RAEA_BIN another).
set -u
cd "$(dirname "$0")/../.."
BIN=${RAEA_BIN:-../../target/release/raea}
EDIT=bench/screens/edit.tsx
OUT=${TMPDIR:-/tmp}/raea-screens-check
rm -rf "$OUT" && mkdir -p "$OUT"
status=0
fail() { echo "FAIL  $*"; status=1; }

# one pixel of a PNG as "r g b"
px() { ffmpeg -v error -i "$1" -vf "crop=1:1:$2:$3,format=rgb24" -f rawvideo - | od -An -tu1 | tr -s ' ' | sed 's/^ //; s/ $//'; }
# PASS when "r g b" is within 6 of each of the wanted channels
near() { awk -v a="$1" -v b="$2" 'BEGIN { split(a, x, " "); split(b, y, " "); for (i = 1; i <= 3; i++) { d = x[i] - y[i]; if (d < -6 || d > 6) exit 1 } }'; }
ssim() { ffmpeg -v error -i "$1" -i "$2" -filter_complex "[0]format=rgb24[a];[1]format=rgb24[b];[a][b]ssim=stats_file=-" -f null - 2>&1 | grep -o 'All:[0-9.]*' | tail -1 | cut -d: -f2; }
at_least() { awk -v s="${1:-0}" -v m="$2" 'BEGIN { exit !(s + 0 >= m) }'; }

"$BIN" render "$EDIT" --at 0,1,2,3 -o "$OUT/at" > "$OUT/render.log" 2>&1 || { echo "FAIL  the folder render failed (see $OUT/render.log)"; exit 1; }
# the screen's middle, the card on the front screen's left, the see-through right of it
check() {
  local got; got=$(px "$OUT/at/frame-0000$1.png" "$2" "$3")
  if near "$got" "$4"; then echo "ok    frame $1 ($2,$3) $got: $5"; else fail "frame $1 ($2,$3) is $got, wanted $4: $5"; fi
}
check 0 480 330 "255 48 48" "the red page at its own colour"
check 2 330 300 "255 255 255" "the front page's white card"
check 2 600 300 "32 64 255" "the blue page behind, through the see-through front"
check 3 330 300 "255 255 255" "the white card under DepthOfField"
check 3 600 300 "32 64 255" "see-through under DepthOfField"

"$BIN" render "$EDIT" --frame 2 -o "$OUT/still2.png" > "$OUT/still.log" 2>&1 || fail "the still failed (see $OUT/still.log)"
s=$(ssim "$OUT/at/frame-00002.png" "$OUT/still2.png")
at_least "$s" 0.99 && echo "ok    the still equals the folder's frame 2 (SSIM $s)" || fail "the still against the folder's frame 2: SSIM $s"

snaps=""
for f in 0 1 2 3; do snaps="$snaps --snapshot $(awk -v f=$f 'BEGIN { printf "%.7f", f / 30 }') $OUT/play-$f.png"; done
RAEA_AUDIO_MUTE=1 "$BIN" play "$EDIT" --offscreen --seconds 0.3 $snaps > "$OUT/play.log" 2>&1 || fail "the play failed (see $OUT/play.log)"
for f in 0 1 2 3; do
  s=$(ssim "$OUT/at/frame-0000$f.png" "$OUT/play-$f.png")
  at_least "$s" 0.99 && echo "ok    play equals the render at frame $f (SSIM $s)" || fail "play against the render at frame $f: SSIM $s"
done
[ $status = 0 ] && echo "screens: PASS" || echo "screens: FAIL (frames in $OUT)"
exit $status
