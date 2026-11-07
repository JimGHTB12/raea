#!/bin/bash
# P26 recapture: baseline frames made again from the bevy_ui page, each with the reason. Chrome is P26's reference
# (Shane, 10-23: "it renders how typescript/react expect"): where the bevy_ui page differs from a Blitz frame and matches
# Chrome, that frame is retaken from the bevy_ui page and its row says why. Only the frames named are touched.
# P26_RECAPTURE_PAGE=blitz retakes them from the Blitz page instead, for a reference whose bevy_ui page disagrees with
# Chrome where the Blitz page agrees (the reason names the check against Chrome).
#
#   RECAPTURED="why" [P26_RECAPTURE_PAGE=blitz] bench/p26-baseline/recapture.sh RAEA_MOTION ENTRY...
#
# ENTRY is a composition as manifest.jsonl names it (file.tsx or file.tsx#ID), alone (every frame of it) or with
# @F,G (those frames). Each composition renders as compare.sh renders it (the bevy_ui page, at the baseline's
# candidate frames, the plugin build for a plugin entry), so a recaptured frame is the one compare.sh will draw.
# RAEA_MOTION (with its libraea_bundle.dylib beside it) is copied under $P26_STORE/bin-bevy-<sha> and the rows name
# that copy; the frame each replaces is kept under $P26_STORE/frames-blitz/ (the first one only). A row gets the new
# PNG's sha256, the binary's, its page ("bevy" or "blitz") and "recaptured": why (the date first).
# RAEA_MOTION is built without incremental (`lane build -- env CARGO_INCREMENTAL=0 cargo build --release -p raea-cli`;
# lane's builds are incremental otherwise): an incremental build moves a few frames by up to 5 pixels (Cargo.toml's
# [profile.release] note), and a baseline is a reference.
set -uo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/lib.sh"
# pixel checks: no Content Credentials or TrustMark id in the frames (a still's mark changes its pixels each run)
export RAEA_SIGN=0 RAEA_MARK=0
ROOT=$(cd "$HERE/../.." && pwd)
STORE=${P26_STORE:-$HOME/Library/Caches/raea/p26-baseline}
[ $# -ge 2 ] && [ -n "${RECAPTURED:-}" ] || { echo "usage: RECAPTURED=\"why\" recapture.sh RAEA_MOTION ENTRY[@F,G]..." >&2; exit 2; }
SRC_BIN=$1; shift
[ -x "$SRC_BIN" ] || { echo "recapture.sh: no raea at $SRC_BIN" >&2; exit 2; }
SHA=$(shasum -a 256 "$SRC_BIN" | cut -d' ' -f1)
KEEP=$STORE/bin-bevy-${SHA:0:12}
if [ ! -x "$KEEP/raea-motion" ]; then
  mkdir -p "$KEEP" && cp "$SRC_BIN" "$KEEP/raea-motion"
  [ -f "$(dirname "$SRC_BIN")/libraea_bundle.dylib" ] && cp "$(dirname "$SRC_BIN")/libraea_bundle.dylib" "$KEEP/"
fi
BIN=$KEEP/raea-motion
BIN_SHOWN="~${BIN#"$HOME"}"; [ "${BIN#"$HOME"}" = "$BIN" ] && BIN_SHOWN=$BIN
PLUGIN_BIN=${P26_PLUGIN_BIN:-$ROOT/sdk/raea/bench/bevy-plugins/motion.sh}
PAGE=${P26_RECAPTURE_PAGE:-blitz}
case $PAGE in bevy) SHOWN_PAGE="bevy_ui page" ;; blitz) SHOWN_PAGE="Blitz page" ;; *) echo "recapture.sh: P26_RECAPTURE_PAGE is bevy or blitz" >&2; exit 2 ;; esac
WHY="$(date +%m-%d) $SHOWN_PAGE: $RECAPTURED"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/p26-recapture.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
M=$HERE/manifest.jsonl
n=0
for entry in "$@"; do
  rel=${entry%%@*}
  want=""; [[ "$entry" == *@* ]] && want=${entry#*@}
  rows=$(jq -c --arg c "$rel" 'select(.composition == $c)' "$M")
  [ -n "$rows" ] || { echo "recapture.sh: $rel isn't in manifest.jsonl" >&2; exit 1; }
  [ -n "$want" ] || want=$(echo "$rows" | jq -r .frame | paste -sd, -)
  p26_src "$rel"
  bin=$BIN; shown=$BIN_SHOWN
  # a plugin entry stays one (compare.sh picks the plugin build by its row's binary)
  echo "$rows" | jq -r .binary | grep -qE 'bevy-plugins|bin-avian' && bin=$PLUGIN_BIN && shown=$PLUGIN_BIN
  cands=$(echo "$rows" | jq -r '.cands // empty' | head -1)
  if [ -z "$cands" ]; then
    last=$(( $(echo "$rows" | jq -r .frames | head -1) - 1 ))
    cands=$(for i in 0 1 2 3 4 5 6 7 8; do echo $((last * i / 8)); done | awk '!seen[$0]++' | paste -sd, -)
  fi
  rm -rf "$WORK/r" && mkdir -p "$WORK/r"
  if ! (cd "$dir" && RAEA_PAGE=$PAGE "$bin" render "$src" ${copt[@]+"${copt[@]}"} --at "$cands" -o "$WORK/r/") > "$WORK/log" 2>&1; then
    echo "recapture.sh: $rel didn't render: $(grep -E 'REFUSED|error|Error|panicked' "$WORK/log" | head -1 | cut -c1-300)" >&2; exit 1
  fi
  IFS=, read -ra list <<< "$cands"
  for f in ${want//,/ }; do
    file=$(echo "$rows" | jq -r --argjson f "$f" 'select(.frame == $f) | .file')
    [ -n "$file" ] || { echo "recapture.sh: $rel has no frame $f in manifest.jsonl" >&2; exit 1; }
    idx=-1
    for i in "${!list[@]}"; do [ "${list[$i]}" = "$f" ] && idx=$i; done
    png=$(printf "%s/r/frame-%05d.png" "$WORK" "$idx")
    [ $idx -ge 0 ] && [ -f "$png" ] || { echo "recapture.sh: $rel frame $f wasn't rendered" >&2; exit 1; }
    mkdir -p "$(dirname "$STORE/frames-blitz/${file#frames/}")"
    [ -f "$STORE/frames-blitz/${file#frames/}" ] || cp "$STORE/$file" "$STORE/frames-blitz/${file#frames/}" 2>/dev/null
    cp "$png" "$STORE/$file"
    psha=$(shasum -a 256 "$png" | cut -d' ' -f1)
    jq -c --arg c "$rel" --argjson f "$f" --arg p "$psha" --arg s "$SHA" --arg b "$shown" --arg w "$WHY" --arg pg "$PAGE" \
      'if .composition == $c and .frame == $f then .png_sha256 = $p | .sha256 = $s | .binary = $b | .page = $pg | .recaptured = $w else . end' \
      "$M" > "$WORK/m" && cat "$WORK/m" > "$M"
    echo "recaptured $rel@$f"
    n=$((n + 1))
  done
done
echo "P26 recapture: $n frames"
