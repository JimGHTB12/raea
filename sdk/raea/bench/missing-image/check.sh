#!/bin/bash
# The missing-image check (sdk/react page.rs `page_props`): an <Img> whose src isn't there must fail the render and
# name the file (footage.md, motion.feature "Img and staticFile"), never draw nothing and say ok.
#   present.tsx  media/red.png filling the frame: renders, and frame 0's centre is red
#   missing.tsx  media/missing.png, not there: the render fails, and says "missing image: .../missing.png"
#
#   sdk/raea/bench/missing-image/check.sh     (RAEA_BIN, default raea)
# Prints a line per case (PASS or MISS) and exits 1 when any misses.
set -u
export RAEA_SIGN=0 RAEA_MARK=0
HERE=$(cd "$(dirname "$0")" && pwd)
BIN=${RAEA_BIN:-raea}
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-missing-image.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
bad=0
if ! (cd "$HERE" && "$BIN" render present.tsx --at 0 -o "$OUT/present/") > "$OUT/present.log" 2>&1; then
  echo "MISS present: the render failed: $(grep -m1 -E 'missing image|REFUSED|error' "$OUT/present.log" | cut -c1-200)"; bad=1
else
  rgb=$(ffmpeg -nostdin -v error -i "$OUT/present/frame-00000.png" -vf "crop=1:1:160:90,format=rgb24" -f rawvideo - | od -An -tu1 | xargs)
  set -- $rgb
  if [ "${1:-0}" -gt 200 ] && [ "${2:-255}" -lt 60 ] && [ "${3:-255}" -lt 60 ]; then echo "PASS present: drawn, centre rgb $rgb"
  else echo "MISS present: centre rgb ${rgb:-none}, not the image's red"; bad=1; fi
fi
if (cd "$HERE" && "$BIN" render missing.tsx --at 0 -o "$OUT/missing/") > "$OUT/missing.log" 2>&1; then
  echo "MISS missing: rendered ok$([ -f "$OUT/missing/frame-00000.png" ] && echo ", a frame written")"; bad=1
elif said=$(grep -m1 -o "missing image: [^ ]*missing.png" "$OUT/missing.log"); then
  echo "PASS missing: failed, $said"
else
  echo "MISS missing: failed without naming it: $(grep -m1 -E 'error|REFUSED' "$OUT/missing.log" | cut -c1-200)"; bad=1
fi
exit $bad
