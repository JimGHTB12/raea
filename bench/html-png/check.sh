#!/bin/bash
# #32: an HTML page renders to PNGs as a TSX composition does. index.html is a half-opaque red box sliding over a canvas
# with nothing else on it. Checked:
#   -o DIR --at 0,29        two PNGs, frame-00000.png and frame-00029.png, the box moved between them
#   -o DIR again            into the folder that's there now (it was "Is a directory")
#   -o OUT.png --at 29      one PNG, byte-identical to the folder's frame-00029.png
#   -o OUT.png              frame 0
#   the pixels              without --alpha the canvas is white and the box pink (255,128,128); with --alpha the canvas
#                           is clear (alpha 0) and the box straight red at half alpha (255,0,0,~128)
#   --alpha -o OUT.mp4      refused
# Exits 1 on a failure.
#   bench/html-png/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-html-png.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
PAGE=bench/html-png/index.html
status=0
ok() { echo "PASS $1"; }
bad() { echo "FAIL $1"; status=1; }
px() { ffmpeg -v error -i "$1" -vf "crop=1:1:$2:$3" -f rawvideo -pix_fmt rgba - | od -An -tu1 | tr -s ' ' | sed 's/^ //; s/ *$//'; }
near() { # "R G B A" "R G B A": each within 3
  read -r a b c d <<<"$1"; read -r e f g h <<<"$2"
  for p in "$a $e" "$b $f" "$c $g" "$d $h"; do set -- $p; (( ${1:-999} - $2 <= 3 && $2 - ${1:-999} <= 3 )) || return 1; done
}
"$BIN" render $PAGE -o "$OUT/dir" --at 0,29 > "$OUT/log" 2>&1
n=$(ls "$OUT/dir" 2>/dev/null | grep -c '^frame-000\(00\|29\)\.png$')
[ "$n" = 2 ] && ok "-o DIR --at 0,29: two PNGs" || { bad "-o DIR: $n PNGs (want 2)"; cat "$OUT/log"; }
[ "$(px "$OUT/dir/frame-00000.png" 60 60)" != "$(px "$OUT/dir/frame-00029.png" 60 60)" ] && ok "the box moved between frames 0 and 29" || bad "frames 0 and 29 are the same at (60,60)"
"$BIN" render $PAGE -o "$OUT/dir" --at 29 > "$OUT/log" 2>&1 && ok "-o DIR into a folder that's there" || { bad "-o DIR into a folder that's there"; cat "$OUT/log"; }
"$BIN" render $PAGE -o "$OUT/one.png" --at 29 > "$OUT/log" 2>&1
cmp -s "$OUT/one.png" "$OUT/dir/frame-00029.png" && ok "-o OUT.png --at 29 is the folder's frame 29" || { bad "-o OUT.png --at 29 differs from the folder's frame 29"; cat "$OUT/log"; }
"$BIN" render $PAGE -o "$OUT/first.png" > "$OUT/log" 2>&1
cmp -s "$OUT/first.png" "$OUT/dir/frame-00000.png" && ok "-o OUT.png is frame 0" || { bad "-o OUT.png isn't frame 0"; cat "$OUT/log"; }
# frame 0: the box at x 40..140, y 40..140
canvas=$(px "$OUT/first.png" 5 5); box=$(px "$OUT/first.png" 90 90)
near "$canvas" "255 255 255 255" && near "$box" "255 128 128 255" && ok "opaque: white canvas ($canvas), pink box ($box)" || bad "opaque: canvas $canvas (want 255 255 255 255), box $box (want 255 128 128 255)"
"$BIN" render $PAGE -o "$OUT/alpha.png" --alpha > "$OUT/log" 2>&1
canvas=$(px "$OUT/alpha.png" 5 5); box=$(px "$OUT/alpha.png" 90 90)
[ "${canvas##* }" = 0 ] && near "$box" "255 0 0 128" && ok "--alpha: clear canvas ($canvas), straight red at half alpha ($box)" || { bad "--alpha: canvas $canvas (want alpha 0), box $box (want 255 0 0 128)"; cat "$OUT/log"; }
"$BIN" render $PAGE -o "$OUT/a.mp4" --alpha > "$OUT/log" 2>&1 && bad "--alpha to an MP4 wasn't refused" || ok "--alpha to an MP4 is refused"
exit $status
