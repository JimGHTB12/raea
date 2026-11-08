#!/bin/bash
# A TSX composition drawn with --at 0 (the still a harness or a bench asks for): fill.tsx, one #246 fill, must write
# frame-00000.png at 160 x 160 with every pixel #246, its last row too. The frame tag below the frame is drawn by its
# own camera (raea-sys-frame spawn_tag): when the frame's camera drew it and that camera was given a w x h Viewport,
# the tag was clipped and --at refused every frame ("no two readbacks with its tag agreed"). Exits 1 on a failure.
#   bench/at-0/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-at-0.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
"$BIN" render bench/at-0/fill.tsx --at 0 -o "$OUT/at" > "$OUT/at.log" 2>&1 || { echo "FAIL: the --at 0 render failed"; grep -v WARN "$OUT/at.log" | tail -3; exit 1; }
f="$OUT/at/frame-00000.png"
[ -f "$f" ] || { echo "FAIL: no frame-00000.png"; exit 1; }
got=$(magick "$f" -format "%wx%h %k %[pixel:p{0,159}]" info:)
[ "$got" = "160x160 1 srgba(34,68,102,1)" ] || { echo "FAIL: frame 0 is '$got', not 160x160 1 srgba(34,68,102,1) (size, colours, last row)"; exit 1; }
echo "PASS: --at 0 drew frame 0, every pixel #246"
