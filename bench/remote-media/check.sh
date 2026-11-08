#!/bin/bash
# #57: a <Video> whose src is an http(s) URL plays that clip, fetched once into raea's URL cache (raea-io-net's
# cache.rs, keyed by the URL), never read as a path beside the composition ("<dir>/https:/…: no video track"). A
# local server serves a red clip; the render's centre must be red, a second render (offline, RAEA_OFFLINE=1) must play it
# from the cache, an uncached URL offline must refuse saying so, and a 404 must refuse with its status. Exits 1 on a
# failure.
#   bench/remote-media/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-remote-media.XXXXXX")
export RAEA_CACHE=$OUT/cache
mkdir -p "$OUT/www" "$OUT/comp"
ffmpeg -loglevel error -f lavfi -i color=c=red:s=64x64:r=30 -t 1 -pix_fmt yuv420p "$OUT/www/red.mp4" || { echo "FAIL: ffmpeg"; exit 2; }
PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$OUT/www" > "$OUT/http.log" 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null; wait $SRV 2>/dev/null; rm -rf "$OUT"' EXIT
for _ in $(seq 1 100); do curl -sf -o /dev/null "http://127.0.0.1:$PORT/red.mp4" && break; perl -e 'select(undef,undef,undef,0.05)'; done
comp() { # NAME URL
  cat > "$OUT/comp/$1.tsx" <<TSX
import { AbsoluteFill, Composition, Video, registerRoot } from "raea";
const Shot = () => (
  <AbsoluteFill style={{ backgroundColor: "#000" }}>
    <Video src="$2" style={{ position: "absolute", left: 0, top: 0, width: 160, height: 160, objectFit: "cover" }} />
  </AbsoluteFill>
);
registerRoot(() => <Composition id="remote" component={Shot} width={160} height={160} fps={30} durationInFrames={2} />);
TSX
}
status=0
red() { # PNG: the centre's red and green
  magick "$1" -format "%[fx:int(255*p{80,80}.r+0.5)] %[fx:int(255*p{80,80}.g+0.5)]" info:
}
plays() { # NAME WHAT
  rm -f "$OUT/$1.mp4" "$OUT/$1.png"
  "$BIN" render "$OUT/comp/$1.tsx" -o "$OUT/$1.mp4" > "$OUT/$1.log" 2>&1 || { echo "FAIL $2: the render failed"; sed 's/^/    /' "$OUT/$1.log" | tail -8; status=1; return; }
  local png=$OUT/$1.png
  ffmpeg -loglevel error -i "$OUT/$1.mp4" -frames:v 1 -y "$png" && [ -f "$png" ] || { echo "FAIL $2: no frame"; status=1; return; }
  read -r r g <<< "$(red "$png")"
  (( r > 200 && g < 60 )) && echo "ok   $2 (centre $r,$g)" || { echo "FAIL $2: centre $r,$g, wanted red"; status=1; }
}
refuses() { # NAME WANT WHAT
  if "$BIN" render "$OUT/comp/$1.tsx" -o "$OUT/$1.mp4" > "$OUT/$1.log" 2>&1; then echo "FAIL $3: rendered"; status=1; return; fi
  if grep -q "no video track" "$OUT/$1.log"; then echo "FAIL $3: read the URL as a path"; sed 's/^/    /' "$OUT/$1.log" | tail -4; status=1; return; fi
  grep -q "$2" "$OUT/$1.log" && echo "ok   $3" || { echo "FAIL $3: no \"$2\" in its refusal"; sed 's/^/    /' "$OUT/$1.log" | tail -4; status=1; }
}
comp live "http://127.0.0.1:$PORT/red.mp4"
plays live "an http src plays"
kill $SRV; wait $SRV 2>/dev/null
RAEA_OFFLINE=1 plays live "the same src, offline, from the cache"
comp uncached "http://127.0.0.1:$PORT/uncached.mp4"
RAEA_OFFLINE=1 refuses uncached "not in raea's cache" "an uncached src offline refuses"
python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$OUT/www" > "$OUT/http.log" 2>&1 &
SRV=$!
for _ in $(seq 1 100); do curl -sf -o /dev/null "http://127.0.0.1:$PORT/red.mp4" && break; perl -e 'select(undef,undef,undef,0.05)'; done
comp gone "http://127.0.0.1:$PORT/gone.mp4"
refuses gone "404" "a 404 refuses with its status"
[ $status = 0 ] && echo "PASS remote media srcs"
exit $status
