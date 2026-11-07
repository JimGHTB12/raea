#!/bin/bash
# Render = play (docs/handoff/blitz-bridge.md, done-when 3): each row of rows.tsv is played live offscreen with its
# input, snapshotted at its moments, and the input the play logged (--input-log) is rendered at the same frames
# (render --input); the two pictures are compared by SSIM. A row passes at 1.000000 on every frame, or, when rows.tsv
# explains it, at >= 0.999. One line a row (PASS, EXPLAINED or FAIL, each frame's SSIM), then a tally; exits 1 on a FAIL.
#   bench/render-play/run.sh [ROW-SUBSTRING...]     (RAEA_BIN: the raea to run, default target/release/raea)
# A play keeps stepping to its snapshots' frames however loaded the machine (ovw), so an unreached frame is a failure.
cd "$(dirname "$0")/../.." || exit 2
ROOT=$PWD
# pixel checks: no Content Credentials or TrustMark id in the frames (a still's mark moves its pixels)
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_AUDIO_MUTE=1
BIN=${RAEA_BIN:-$ROOT/target/release/raea}
OUT=${RAEA_RP_OUT:-$(mktemp -d "${TMPDIR:-/tmp}/raea-render-play.XXXXXX")}
mkdir -p "$OUT"
# the composition's size and fps from its source: <Composition width height fps>, else the CW/VW and CH/VH constants
# (the games), else 1920x1080 at 30
dim() { grep '<Composition' "$1" | grep -oE "$2=\{[0-9]+\}" | head -1 | tr -dc 0-9; }
const() { grep -oE "const ($2) = [0-9]+" "$1" | head -1 | grep -oE '[0-9]+$'; }
pass=0 explained=0 fail=0
while IFS=$'\t' read -r path input secs why; do
  [[ -z $path || $path == \#* ]] && continue
  if [ $# -gt 0 ]; then hit=0; for s in "$@"; do [[ $path == *"$s"* ]] && hit=1; done; [ $hit = 1 ] || continue; fi
  src=$ROOT/$path
  slug=$(echo "$path" | tr '/' '_' | sed 's/\.tsx$//')
  w=$(dim "$src" width); w=${w:-$(const "$src" 'CW|VW')}; w=${w:-1920}
  h=$(dim "$src" height); h=${h:-$(const "$src" 'CH|VH')}; h=${h:-1080}
  fps=$(dim "$src" fps); fps=${fps:-30}
  last=$(echo $secs | awk '{ m = $1; for (i = 2; i <= NF; i++) if ($i + 0 > m + 0) m = $i; printf "%.17g", m + 0.25 }')
  snaps=()
  for s in $secs; do snaps+=(--snapshot "$s" "$OUT/$slug-play-$s.png"); done
  rm -f "$OUT/$slug"-play-*.png "$OUT/$slug"-render-*.png
  how=(); keys=""
  case $input in
    -) ;;
    keys:*) keys=${input#keys:}; how=(--input) ;;
    script:*) how=(--input-script "$ROOT/${input#script:}") ;;
  esac
  if ! (cd "$(dirname "$src")" && RAEA_INPUT_KEYS="$keys" "$BIN" play "$src" --offscreen --size "${w}x$h" --seconds "$last" \
      ${how[@]+"${how[@]}"} --input-log "$OUT/$slug-input.json" "${snaps[@]}") > "$OUT/$slug-play.log" 2>&1; then
    echo "FAIL $path: the play failed: $(grep -m1 -E 'REFUSED|error|panicked' "$OUT/$slug-play.log" | cut -c1-200)"; fail=$((fail + 1)); continue
  fi
  line="" worst=1 broke=""
  # the run rendered, frames 0 to the last compared, as play played it and an export renders it: a frame of a
  # composition that keeps state in its module (a World ask sent once, at frame 3) is the run's frame, which a still of
  # that frame alone, its page jumping there, isn't (frame purity, motion.feature: the author's to keep)
  lastf=$(for s in $secs; do awk -v s="$s" -v fps="$fps" 'BEGIN { printf "%.0f\n", s * fps }'; done | sort -n | tail -1)
  rm -rf "$OUT/$slug-render" && mkdir -p "$OUT/$slug-render"
  (cd "$(dirname "$src")" && RAEA_READBACK=rgba "$BIN" render "$src" --at "$(seq 0 "$lastf" | paste -sd, -)" --input "$OUT/$slug-input.json" -o "$OUT/$slug-render/") > "$OUT/$slug-render.log" 2>&1
  for s in $secs; do
    f=$(awk -v s="$s" -v fps="$fps" 'BEGIN { printf "%.0f", s * fps }')
    if [ ! -f "$OUT/$slug-play-$s.png" ]; then line="$line $f:unreached"; broke=1; continue; fi
    if [ ! -f "$OUT/$slug-render/$(printf 'frame-%05d.png' "$f")" ]; then line="$line $f:render-failed"; broke=1; continue; fi
    cp "$OUT/$slug-render/$(printf 'frame-%05d.png' "$f")" "$OUT/$slug-render-$s.png"
    v=$(ffmpeg -nostdin -hide_banner -nostats -i "$OUT/$slug-play-$s.png" -i "$OUT/$slug-render-$s.png" \
      -lavfi "[0:v]format=rgb24[a];[1:v]format=rgb24[b];[a][b]ssim" -f null - 2>&1 | grep -o 'All:[0-9.]*' | cut -d: -f2)
    line="$line $f:${v:-none}"
    worst=$(awk -v w="$worst" -v v="${v:-0}" 'BEGIN { print (v + 0 < w + 0) ? v : w }')
  done
  if [ -z "$broke" ] && awk -v w="$worst" 'BEGIN { exit !(w + 0 >= 1) }'; then
    echo "PASS $path:$line"; pass=$((pass + 1))
  elif [ -z "$broke" ] && [ -n "$why" ] && awk -v w="$worst" 'BEGIN { exit !(w + 0 >= 0.999) }'; then
    echo "EXPLAINED $path:$line ($why)"; explained=$((explained + 1))
  else
    echo "FAIL $path:$line (worst $worst; play and renders in $OUT)"; fail=$((fail + 1))
  fi
done < bench/render-play/rows.tsv
echo "render = play: $pass at 1.000000, $explained explained, $fail failed (out: $OUT)"
[ $fail = 0 ]
