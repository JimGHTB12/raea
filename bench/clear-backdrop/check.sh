#!/bin/bash
# A composition that paints no background is black behind its content in every output, as a PNG of it always was. A
# video took the fed path, whose program has no camera, and runtime/view's camera_bg gave raea's old dark blue
# (13,10,26) for that; the same frame as a PNG was black (#13's sweep met it). The composition, white text on nothing,
# is rendered to an mp4 and to a PNG: the corner of each must be black (within 3 of 0, the mp4's YUV rounding) and
# the two must agree. Exits 1 on a failure.
#   bench/clear-backdrop/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-clear-backdrop.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
cat > "$OUT/clear.tsx" <<'TSX'
import { AbsoluteFill, Composition, registerRoot } from "raea";
const Shot = () => (<AbsoluteFill><div style={{ color: "#fff", fontSize: 24 }}>x</div></AbsoluteFill>);
registerRoot(() => <Composition id="clear" component={Shot} width={160} height={160} fps={30} durationInFrames={2} />);
TSX
status=0
corner() { magick "$1" -format "%[fx:int(255*p{120,120}.r+0.5)] %[fx:int(255*p{120,120}.g+0.5)] %[fx:int(255*p{120,120}.b+0.5)]" info:; }
black() { # PNG WHAT
  read -r r g b <<< "$(corner "$1")"
  (( r <= 3 && g <= 3 && b <= 3 )) && echo "ok   $2: black ($r,$g,$b)" || { echo "FAIL $2: $r,$g,$b behind the content, wanted black"; status=1; }
}
if "$BIN" render "$OUT/clear.tsx" -o "$OUT/clear.mp4" > "$OUT/mp4.log" 2>&1 && ffmpeg -loglevel error -i "$OUT/clear.mp4" -frames:v 1 -y "$OUT/mp4.png"; then
  black "$OUT/mp4.png" "an mp4"
else echo "FAIL an mp4: the render failed"; sed 's/^/    /' "$OUT/mp4.log" | tail -4; status=1; fi
if "$BIN" render "$OUT/clear.tsx" --at 0 -o "$OUT/png" > "$OUT/png.log" 2>&1 && [ -f "$OUT/png/frame-00000.png" ]; then
  black "$OUT/png/frame-00000.png" "a PNG (--at 0)"
else echo "FAIL a PNG: the render failed"; sed 's/^/    /' "$OUT/png.log" | tail -4; status=1; fi
[ $status = 0 ] && echo "PASS a clear composition is black in an mp4 and a PNG"
exit $status
