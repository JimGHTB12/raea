#!/bin/bash
# A page whose first draw is past raea's Vello bump buffers (shards.tsx): the mp4's frame 0 must be the page, not black.
# The fed render (pages handed to raea-render) never checked its draws, so a skipped draw went into the film as it was:
# a black first frame, or a layer gone for a frame (#23). It now checks before the frame goes to the encoder and
# paints the page again at Vello's own sizes. Exits 1 on a failure.
#   bench/vello-skip/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-vello-skip.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
"$BIN" render bench/vello-skip/shards.tsx -o "$OUT/v.mp4" > "$OUT/v.log" 2>&1 || { echo "FAIL: the render failed"; tail -3 "$OUT/v.log"; exit 1; }
grep -q "past raea's Vello buffers" "$OUT/v.log" || echo "note: the first draw fit raea's buffers this time (nothing skipped to check)"
ffmpeg -loglevel error -y -i "$OUT/v.mp4" -frames:v 1 "$OUT/f0.png" || { echo "FAIL: no frame 0"; exit 1; }
mean=$(magick "$OUT/f0.png" -colorspace gray -format "%[fx:int(mean*255)]" info:)
[ "$mean" -ge 15 ] || { echo "FAIL: frame 0's mean luma is $mean: the skipped first draw went out (black is 0, the page about 22)"; exit 1; }
echo "PASS: a skipped first draw is painted again before it's encoded (frame 0's mean luma $mean)"
