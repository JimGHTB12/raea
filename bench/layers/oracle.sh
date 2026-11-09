#!/bin/bash
# The layer pass's oracle (docs/review.feature R2e): where each layer of bench/layers/conformance.tsx shows, found
# without the layer pass. Each layer is hidden in turn (RAEA_HIDE_LAYERS: its element and all inside it, or its 3D part)
# and the frame rendered again at three moments; the pixels that change are where it shows. Written as quarter-size
# masks, sampled as NAME.layers' masks are (mask px (x, y) is frame px (4x+2, 4y+2)): OUT/<moment>/<n>.pgm, 255 where
# it changed, with OUT/keys.txt (n, then the key) and OUT/<moment>/base.png. Exits 1 when a layer's key hides nothing
# (a key that names no layer, or a hide that doesn't work).
#   bench/layers/oracle.sh OUT     (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
OUT=${1:?OUT folder}
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0 RAEA_SOURCE_STAMPS=1 RAEA_AUDIO_MUTE=1
MOMENTS=(10 45 80)
mkdir -p "$OUT/src"
cp bench/layers/conformance.tsx bench/layers/badge.tsx "$OUT/src/"
magick -size 96x64 gradient:'#ffd400-#ff2d55' "$OUT/src/pic.png"
read -r W H <<< "$(sed -n 's/.*width={\([0-9]*\)} height={\([0-9]*\)}.*/\1 \2/p' bench/layers/conformance.tsx)"
MW=$(( (W + 3) / 4 )); MH=$(( (H + 3) / 4 ))

# file:line:col of the nth (1-based) "<Tag" in a file (ASCII sources: a byte column is TypeScript's UTF-16 one)
at() { awk -v t="<$2" -v n="$3" -v f="$1" '{ s = $0; c = 0; while ((i = index(s, t)) > 0) { c += i; nx = substr(s, i + length(t), 1); if (nx ~ /[ >\/]/ && ++k == n) { print f ":" NR ":" c; exit } s = substr(s, i + length(t)); c += length(t) - 1 } }' "$OUT/src/$1"; }
B1=$(at conformance.tsx Badge 1); B2=$(at conformance.tsx Badge 2); LI=$(at conformance.tsx li 1)
# not the AbsoluteFill: its background over 3D is the 3D's clear (raea-runtime-export program.rs), behind every layer
# and none of them, so hiding it changes nothing and the layer pass gives no layer there
KEYS=(
  "$(at conformance.tsx Mesh3d 1)" "$(at conformance.tsx Mesh3d 2)"
  "$(at badge.tsx div 1)|$B1" "$(at badge.tsx span 1)|$B1" "$(at badge.tsx div 1)|$B2" "$(at badge.tsx span 1)|$B2"
  "$(at conformance.tsx div 1)" "$(at conformance.tsx h1 1)" "$(at conformance.tsx Img 1)" "$(at conformance.tsx div 2)"
  "$(at conformance.tsx ul 1)" "$LI#one" "$LI#two" "$LI#three"
)
: > "$OUT/keys.txt"
for i in "${!KEYS[@]}"; do echo "$i ${KEYS[$i]}" >> "$OUT/keys.txt"; done

at_list=$(IFS=,; echo "${MOMENTS[*]}")
# --at's frames come out in order as frame-00000.png ...: moment m is the m'th
render() { "$BIN" render "$OUT/src/conformance.tsx" --at "$at_list" -o "$1" > "$1.log" 2>&1 && [ -f "$1/frame-00002.png" ] || { echo "FAIL the render ($2) failed"; tail -4 "$1.log" | sed 's/^/    /'; exit 1; }; }
render "$OUT/base" base
quarter() { magick "$1" -crop +1+1 +repage -background black -extent "${W}x${H}" -sample "${MW}x${MH}" "$2"; }
status=0
for i in "${!KEYS[@]}"; do
  RAEA_HIDE_LAYERS="${KEYS[$i]}" render "$OUT/hide$i" "hiding ${KEYS[$i]}"
  total=0
  for m in 0 1 2; do
    mkdir -p "$OUT/${MOMENTS[$m]}"
    cp "$OUT/base/frame-0000$m.png" "$OUT/${MOMENTS[$m]}/base.png"
    # changed: some channel moved more than 10 of 255 (the encoder-free PNGs are exact; this is for dithered gradients)
    magick "$OUT/base/frame-0000$m.png" "$OUT/hide$i/frame-0000$m.png" -compose difference -composite -separate -evaluate-sequence max -threshold 4% "$OUT/full.png"
    quarter "$OUT/full.png" "$OUT/${MOMENTS[$m]}/$i.pgm"
    n=$(magick "$OUT/${MOMENTS[$m]}/$i.pgm" -format "%[fx:int(mean*w*h+0.5)]" info:)
    total=$((total + n))
  done
  rm -rf "${OUT:?}/hide$i" "$OUT/hide$i.log"
  if (( total == 0 )); then echo "FAIL layer $i (${KEYS[$i]}): hiding it changed nothing"; status=1
  else echo "ok   layer $i (${KEYS[$i]}): $total quarter px over ${#MOMENTS[@]} moments"; fi
done
rm -f "$OUT/full.png"
exit $status
