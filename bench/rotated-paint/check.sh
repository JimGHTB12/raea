#!/bin/bash
# #18: edit.tsx's "sliced" and "whole" pictures under a -4deg rotation must match, but for the seams between slices
# (each slice's edge antialiased against the next): at most 200 pixels may differ by more than 10%. When a background's
# position, its tiles or a box shadow's offset move in the page's axes instead of the element's, about 7,800 do.
#   bench/rotated-paint/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
# pixel checks: no Content Credentials or TrustMark id in the frames
export RAEA_SIGN=0 RAEA_MARK=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-rotated-paint.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
for c in sliced whole; do
  "$BIN" render bench/rotated-paint/edit.tsx --composition $c -o "$OUT/$c.png" > "$OUT/$c.log" 2>&1 || { echo "FAIL: the $c render failed"; tail -5 "$OUT/$c.log"; exit 1; }
done
d=$(magick compare -metric AE -fuzz 10% "$OUT/sliced.png" "$OUT/whole.png" null: 2>&1 | cut -d' ' -f1 | cut -d. -f1)
[ -n "$d" ] && [ "$d" -le 200 ] || { echo "FAIL: ${d:-?} pixels differ between the sliced and whole pictures (at most 200)"; exit 1; }
echo "PASS: $d pixels differ between the sliced and whole pictures (the seams)"
