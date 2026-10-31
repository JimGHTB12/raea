#!/bin/bash
# The framing check (<Video framing="face" punchIn>: sdk/react page.rs video_fit, raea-page fit_rect, the face track
# runtime/page/src/faces.rs). The stock talking take (engine/cli/conformance/fixtures/face.mp4, 320x180, the face left of centre
# and high) in a 360x640 frame:
#   plain.tsx   objectFit cover, centred: the reference
#   framed.tsx  framing="face" punchIn={1.3}: must differ from plain, and the eyes raea-perceive finds in it (their
#               midpoint, `raea-perceive eyes`) 38% +-2% down the frame and centred across within 5%
#   framed.tsx again in its project, raea-perceive pointing at nothing: framed as before, from the kept faces.json
#   framed.tsx with RAEA_PERCEIVE pointing at nothing: the render is refused, saying it needs raea-perceive
# The projects are made in a temp dir with the take linked into media/, so its faces.json stays out of the tree.
#
#   sdk/raea/bench/framing/check.sh     (RAEA_BIN, default raea; RAEA_PERCEIVE_BIN, default target/release's)
# Prints a line per check (PASS or MISS) and exits 1 when any misses.
set -u
export RAEA_SIGN=0 RAEA_MARK=0
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
BIN=${RAEA_BIN:-raea}
PERCEIVE=${RAEA_PERCEIVE_BIN:-$ROOT/target/release/raea-perceive}
TAKE=$ROOT/engine/cli/conformance/fixtures/face.mp4
[ -f "$TAKE" ] || { echo "MISS no fixture take at $TAKE"; exit 1; }
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-framing.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
project() { mkdir -p "$OUT/$1/media" && cp "$HERE/$2.tsx" "$OUT/$1/" && ln -s "$TAKE" "$OUT/$1/media/face.mp4"; }
bad=0
for c in plain framed; do
  project "$c" "$c"
  if ! (cd "$OUT/$c" && RAEA_PERCEIVE=$PERCEIVE "$BIN" render "$c.tsx" --at 30 -o "$OUT/$c/r/") > "$OUT/$c.log" 2>&1; then
    echo "MISS $c: the render failed: $(grep -m1 -E 'REFUSED|error' "$OUT/$c.log" | cut -c1-200)"; bad=1
  fi
done
p=$OUT/plain/r/frame-00000.png f=$OUT/framed/r/frame-00000.png
if [ -f "$p" ] && [ -f "$f" ]; then
  s=$(ffmpeg -nostdin -hide_banner -nostats -i "$p" -i "$f" -lavfi "[0:v]format=rgb24[a];[1:v]format=rgb24[b];[a][b]ssim" -f null - 2>&1 | grep -o 'All:[0-9.]*' | cut -d: -f2)
  if awk -v s="${s:-1}" 'BEGIN { exit !(s < 0.95) }'; then echo "PASS differs from plain: SSIM $s"; else echo "MISS framed is plain's picture: SSIM ${s:-none}"; bad=1; fi
  eyes=$("$PERCEIVE" eyes "$f" | jq -c .eyes)
  if [ -z "$eyes" ] || [ "$eyes" = null ]; then echo "MISS no face found in the framed frame"; bad=1
  else
    read -r ex ey <<< "$(jq -r '"\(.[0]) \(.[1])"' <<< "$eyes")"
    if awk -v x="$ex" -v y="$ey" 'BEGIN { exit !(y >= 0.36 && y <= 0.40 && x >= 0.45 && x <= 0.55) }'; then
      echo "PASS eyes at $(awk -v y="$ey" 'BEGIN { printf "%.1f%%", y * 100 }') down, $(awk -v x="$ex" 'BEGIN { printf "%.1f%%", x * 100 }') across"
    else
      echo "MISS eyes at y $ey, x $ex: not 38% +-2% down and centred"; bad=1
    fi
  fi
  [ -f "$OUT/framed/media/face.faces.json" ] && echo "PASS face track kept beside the take: $(jq -c '{every, samples: (.eyes | length), found: ([.eyes[] | select(. != null)] | length)}' "$OUT/framed/media/face.faces.json")" \
    || { echo "MISS no face.faces.json beside the take"; bad=1; }
  # the kept track read back: rendered again without raea-perceive, the frame is framed as before
  if (cd "$OUT/framed" && RAEA_PERCEIVE=$OUT/no-such-perceive "$BIN" render framed.tsx --at 30 -o "$OUT/framed/again/") > "$OUT/again.log" 2>&1; then
    s=$(ffmpeg -nostdin -hide_banner -nostats -i "$f" -i "$OUT/framed/again/frame-00000.png" -lavfi "[0:v]format=rgb24[a];[1:v]format=rgb24[b];[a][b]ssim" -f null - 2>&1 | grep -o 'All:[0-9.]*' | cut -d: -f2)
    if awk -v s="${s:-0}" 'BEGIN { exit !(s >= 0.9999) }'; then echo "PASS the kept track read back: the same framing without raea-perceive (SSIM $s)"; else echo "MISS the kept track read back: another picture (SSIM ${s:-none})"; bad=1; fi
  else
    echo "MISS the kept track read back: refused: $(grep -m1 -E 'REFUSED|error|raea:' "$OUT/again.log" | cut -c1-200)"; bad=1
  fi
fi
# no raea-perceive: refused, never centred
project none framed
if (cd "$OUT/none" && RAEA_PERCEIVE=$OUT/no-such-perceive "$BIN" render framed.tsx --at 30 -o "$OUT/none/r/") > "$OUT/none.log" 2>&1; then
  echo "MISS without raea-perceive: rendered"; bad=1
elif grep -q "needs raea-perceive" "$OUT/none.log"; then
  echo "PASS without raea-perceive: refused ($(grep -m1 -o 'framing="face" needs raea-perceive[^(]*' "$OUT/none.log"))"
else
  echo "MISS without raea-perceive: failed without saying why: $(grep -m1 -E 'REFUSED|error' "$OUT/none.log" | cut -c1-200)"; bad=1
fi
exit $bad
