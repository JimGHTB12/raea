#!/bin/bash
# P26's gate (docs/ts-native.feature P26a): each baseline frame (manifest.jsonl, the last Blitz build's) rendered again
# by raea on the Blitz page (the only page since B5d; P26_PAGE names it in results.jsonl,
# docs/blitz-bevy.feature B2b) and compared, SSIM >= 0.98 a frame.
#
#   bench/p26-baseline/compare.sh [RAEA_BIN]      (default: target/release/raea)
#
# The frames go to $P26_STORE/bevy/frames/<slug>/<frame>.png beside the baseline's ($P26_STORE: ~/Library/Caches/raea/
# p26-baseline), each frame's SSIM (ffmpeg's, over RGB) to $P26_STORE/bevy/results.jsonl; a composition that didn't
# render is a failure of all its frames. ONLY=substring limits the compositions. MIN=0.98 is the bar. Prints the pass
# rate and the worst frames, and exits 1 when any frame is under the bar (CI's p26 gate reads the exit). One render at
# a time (the GPU is shared).
#
# The floor (10-26, one binary, each row rendered twice): rows repeat at SSIM >= 0.99998 run to run. Most are
# byte-identical or 1/255 apart; look/town and kitchen/wreck differ in about 200 pixels, on house-kit's siding, where
# the outside and inside skins have coincident faces and which one draws on top follows Bevy's draw order for equal
# depths, likely the asset-load order (spawning the models' scenes in a fixed order didn't change it). Coincident faces
# z-fight in any engine: whoever next touches those houses, offset the inner skin.
set -uo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/lib.sh"
# pixel checks: no Content Credentials or TrustMark id in the frames (a still's mark changes its pixels each run)
export RAEA_SIGN=0 RAEA_MARK=0
ROOT=$(cd "$HERE/../.." && pwd)
STORE=${P26_STORE:-$HOME/Library/Caches/raea/p26-baseline}
BIN=${1:-$ROOT/target/release/raea}
MIN=${MIN:-0.98}
# a composition whose baseline came from a plugin build (raea-bevy-sdk: hanabi, avian) renders with one: P26_PLUGIN_BIN
# (default sdk/raea/bench/bevy-plugins/motion.sh; it must take `render` as raea does)
PLUGIN_BIN=${P26_PLUGIN_BIN:-$ROOT/sdk/raea/bench/bevy-plugins/motion.sh}
[ -x "$BIN" ] || { echo "compare.sh: no raea at $BIN" >&2; exit 2; }
[ -d "$STORE/frames" ] || { echo "compare.sh: no baseline frames at $STORE/frames" >&2; exit 2; }
# P26_OUT: where this run writes (its frames and results.jsonl), so two runs at once don't write over each other
PAGE=${P26_PAGE:-blitz}
OUT=${P26_OUT:-$STORE/$PAGE}
WORK=$(mktemp -d "${TMPDIR:-/tmp}/p26-compare.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
# the run's own copy of raea (and its bundle library): a rebuild of target/release while it runs doesn't mix two
# binaries into one run's rows
mkdir -p "$WORK/bin" && cp "$BIN" "$WORK/bin/raea" || exit 2
[ -f "$(dirname "$BIN")/libraea_bundle.dylib" ] && cp "$(dirname "$BIN")/libraea_bundle.dylib" "$WORK/bin/"
BIN=$WORK/bin/raea
mkdir -p "$OUT/frames" "$OUT/logs"
RES=$OUT/results.jsonl
: > "$RES"
for rel in $(jq -r '.composition' "$HERE/manifest.jsonl" | awk '!seen[$0]++'); do
  [ -n "${ONLY:-}" ] && [[ "$rel" != *"$ONLY"* ]] && continue
  p26_src "$rel"
  bin=$BIN
  jq -r --arg c "$rel" 'select(.composition == $c) | .binary' "$HERE/manifest.jsonl" | grep -qE 'bevy-plugins|bin-avian' && bin=$PLUGIN_BIN
  frames=$(jq -r --arg c "$rel" 'select(.composition == $c) | .frame' "$HERE/manifest.jsonl" | sort -n | paste -sd, -)
  # rendered at the same nine frames the baseline's render took (capture.sh): a render's 3D rows are keyed by the
  # frames it samples, so the kept frames alone would draw another animation (a box springing from scale 0 at frame 0
  # was still at 0 at frame 59 when only those two were sampled, in Blitz as here)
  total=$(jq -r --arg c "$rel" 'select(.composition == $c) | .frames' "$HERE/manifest.jsonl" | head -1)
  last=$((total - 1))
  cands=$(jq -r --arg c "$rel" 'select(.composition == $c) | .cands // empty' "$HERE/manifest.jsonl" | head -1)
  [ -n "$cands" ] || cands=$(for i in 0 1 2 3 4 5 6 7 8; do echo $((last * i / 8)); done | awk '!seen[$0]++' | paste -sd, -)
  rm -rf "$WORK/r" && mkdir -p "$WORK/r" "$OUT/frames/$slug"
  # each row's render log kept at $OUT/logs/<slug>.log (a frame that came out black and was redrawn says so there)
  if ! (cd "$dir" && RAEA_PAGE=$PAGE "$bin" render "$src" ${copt[@]+"${copt[@]}"} --at "$cands" -o "$WORK/r/") > "$WORK/log" 2>&1; then
    cp "$WORK/log" "$OUT/logs/$slug.log"
    why=$(grep -E 'REFUSED|error|Error|panicked' "$WORK/log" | head -1 | cut -c1-300)
    for f in ${frames//,/ }; do
      jq -nc --arg c "$rel" --argjson f "$f" --arg why "$why" '{composition: $c, frame: $f, ssim: null, pass: false, why: $why}' >> "$RES"
    done
    echo "MISS $rel: $why"
    continue
  fi
  cp "$WORK/log" "$OUT/logs/$slug.log"
  IFS=, read -ra list <<< "$cands"
  for f in ${frames//,/ }; do
    idx=-1
    for i in "${!list[@]}"; do [ "${list[$i]}" = "$f" ] && idx=$i; done
    got=$(printf "%s/r/frame-%05d.png" "$WORK" "$idx")
    ref=$STORE/frames/$slug/$f.png
    if [ ! -f "$got" ]; then
      jq -nc --arg c "$rel" --argjson f "$f" '{composition: $c, frame: $f, ssim: null, pass: false, why: "no frame came out"}' >> "$RES"
      continue
    fi
    cp "$got" "$OUT/frames/$slug/$f.png"
    s=$(ffmpeg -hide_banner -nostats -i "$ref" -i "$got" -lavfi "[0:v]format=rgb24[a];[1:v]format=rgb24[b];[a][b]ssim" -f null - 2>&1 | grep -o 'All:[0-9.]*' | cut -d: -f2)
    [ -n "$s" ] || s=0
    pass=$(awk -v s="$s" -v m="$MIN" 'BEGIN { print (s >= m) ? "true" : "false" }')
    jq -nc --arg c "$rel" --argjson f "$f" --argjson s "$s" --argjson p "$pass" '{composition: $c, frame: $f, ssim: $s, pass: $p}' >> "$RES"
  done
  echo "ok $rel"
done
n=$(wc -l < "$RES" | tr -d ' ')
ok=$(jq -s '[.[] | select(.pass)] | length' "$RES")
echo "P26: $ok of $n frames at SSIM >= $MIN ($(jq -s '[.[] | .composition] | unique | length' "$RES") compositions)"
echo "worst:"
jq -s -r 'sort_by(.ssim // -1) | .[:15][] | "  \(.ssim // "none")  \(.composition) @\(.frame)\(if .why then "  " + .why else "" end)"' "$RES"
[ "$ok" = "$n" ]
