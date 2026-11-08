#!/bin/bash
# raea/text loadFont: a composition that asks for a Google Fonts family by name gets it, as a stylesheet link gets it
# in Chrome (#13: snapcn's components load theirs by name). "Hello raea" in Rubik Mono One, a
# family no Mac has: with loadFont its frame must differ from the same text without it (which falls back as it did),
# and a second render offline (RAEA_OFFLINE=1, from raea's URL cache) must match the first exactly. The first render
# needs the network (fonts.googleapis.com). Exits 1 on a failure.
#   bench/load-font/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-load-font.XXXXXX")
export RAEA_CACHE=$OUT/cache
trap 'rm -rf "$OUT"' EXIT
comp() { # NAME LINES
  cat > "$OUT/$1.tsx" <<TSX
import { AbsoluteFill, Composition, registerRoot } from "raea";
$2
const Shot = () => (
  <AbsoluteFill style={{ backgroundColor: "#fff", color: "#000", fontFamily, fontSize: 64, padding: 20 }}>Hello raea</AbsoluteFill>
);
registerRoot(() => <Composition id="f" component={Shot} width={640} height={160} fps={30} durationInFrames={1} />);
TSX
}
frame() { # NAME OUT
  "$BIN" render "$OUT/$1.tsx" -o "$OUT/$2.mp4" > "$OUT/$2.log" 2>&1 || { echo "FAIL $2: the render failed"; sed 's/^/    /' "$OUT/$2.log" | tail -6; exit 1; }
  ffmpeg -loglevel error -i "$OUT/$2.mp4" -frames:v 1 -y "$OUT/$2.png" || { echo "FAIL $2: no frame"; exit 1; }
}
comp on 'import { loadFont } from "raea/text";
const { fontFamily } = loadFont("Rubik Mono One");'
comp off 'const fontFamily = "Rubik Mono One";'
frame on on
frame off off
RAEA_OFFLINE=1 frame on again
status=0
diff=$(magick compare -metric AE "$OUT/on.png" "$OUT/off.png" null: 2>&1 | cut -d' ' -f1)
(( ${diff%.*} > 1000 )) && echo "ok   loadFont changes the text's face ($diff px differ)" || { echo "FAIL loadFont: only $diff px differ from the fallback"; status=1; }
same=$(magick compare -metric AE "$OUT/on.png" "$OUT/again.png" null: 2>&1 | cut -d' ' -f1)
[ "${same%.*}" = 0 ] && echo "ok   offline, from the cache: the same frame" || { echo "FAIL offline: $same px differ"; status=1; }
[ $status = 0 ] && echo "PASS loadFont"
exit $status
