#!/bin/bash
# #33: a render that panics as a frame draws ends with an error naming the frame and the panic, and keeps the frames
# drawn before it. RAEA_PANIC_FAULT=100 panics as frame 100 draws; each render below must exit 1 (not a panic's 101),
# say "panicked drawing frame 100", and leave 100 frames: an HTML page (bench/html-repeat) to .mp4 and .txt, the 2D page
# here to .mp4 and .txt, and a 3D scene (bench/at-3d) to .mp4 and a folder of PNGs. Exits 1 on a failure.
#   bench/frame-panic/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0 RAEA_PANIC_FAULT=100
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-frame-panic.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
frames() {
  if [ -d "$1" ]; then ls "$1" | grep -c '\.png$'
  elif [[ $1 == *.txt ]]; then wc -l < "$1" | tr -d ' '
  elif [ -f "$1" ]; then ffprobe -v error -count_frames -show_entries stream=nb_read_frames -of csv=p=0 "$1"
  else echo 0; fi
}
status=0
while read -r src out; do
  "$BIN" render "$src" -o "$OUT/$out" > "$OUT/$out.log" 2>&1
  rc=$?
  said=$(grep -c "panicked drawing frame 100: RAEA_PANIC_FAULT" "$OUT/$out.log")
  n=$(frames "$OUT/$out")
  if [ "$rc" = 1 ] && [ "$said" -ge 1 ] && [ "$n" = 100 ]; then echo "PASS $src -> $out: exit 1, frame 100 named, 100 frames kept"
  else echo "FAIL $src -> $out: exit $rc, frame named ${said}x, $n frames kept (want 1, 1, 100)"; status=1; fi
done <<LIST
bench/html-repeat/index.html html.mp4
bench/html-repeat/index.html html.txt
bench/frame-panic/page.tsx page.mp4
bench/frame-panic/page.tsx page.txt
bench/at-3d/scene.tsx scene.mp4
bench/at-3d/scene.tsx scene
LIST
exit $status
