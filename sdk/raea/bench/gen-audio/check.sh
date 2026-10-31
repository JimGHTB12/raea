#!/bin/bash
# The generated-audio check (sdk/video media.tsx data-options; sdk/react audio.rs `generated_params`, motion.rs
# `generated`): a generated <Audio> is made with every setting its generator takes, so two renders that differ only in a
# TokenSynth note's seed, or only in its pitch, sound different, and the same note rendered again from nothing is the
# same samples. Each case renders in a fresh project (its own asset store, so nothing is reused between them):
#   c4-seed1 twice   the same audio, sample for sample
#   c4-seed2         differs from c4-seed1 (the seed reached TokenSynth)
#   e4-seed1         differs from c4-seed1 (the pitch reached TokenSynth)
# Local TokenSynth only (CPU, no spend).
#
#   sdk/raea/bench/gen-audio/check.sh     (RAEA_BIN, default raea)
# Prints a line per check (PASS or MISS) and exits 1 when any misses.
set -u
export RAEA_SIGN=0 RAEA_MARK=0
HERE=$(cd "$(dirname "$0")" && pwd)
BIN=${RAEA_BIN:-raea}
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-gen-audio.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
bad=0
# the case's audio as rendered, decoded to 48 kHz stereo s16, hashed (empty when it has none)
sound() {
  local run=$1 c=$2
  echo "  rendering $run ($c): a TokenSynth note made from nothing in a fresh project, on the CPU" >&2
  mkdir -p "$OUT/$run" && cp "$HERE/$c.tsx" "$OUT/$run/"
  if ! (cd "$OUT/$run" && "$BIN" render "$c.tsx" -o "$OUT/$run/out.mp4") > "$OUT/$run.log" 2>&1; then
    echo "MISS $run: the render failed: $(grep -m1 -E 'REFUSED|error|wasn.t generated' "$OUT/$run.log" | cut -c1-200)" >&2; return
  fi
  grep -m1 "wasn't generated" "$OUT/$run.log" >&2
  ffmpeg -nostdin -v error -i "$OUT/$run/out.mp4" -vn -ac 2 -ar 48000 -f s16le - 2>/dev/null | shasum | cut -c1-16
}
a=$(sound a c4-seed1) b=$(sound b c4-seed1) s=$(sound s c4-seed2) p=$(sound p e4-seed1)
silent=$(head -c 192000 /dev/zero | shasum | cut -c1-16)
for x in "$a" "$s" "$p"; do [ -n "$x" ] && [ "$x" != "$silent" ] || { echo "MISS a case has no audio (or is silent): $a $s $p"; bad=1; break; }; done
[ -n "$a" ] && [ "$a" = "$b" ] && echo "PASS same seed, made again from nothing: the same samples ($a)" || { echo "MISS same seed: $a vs $b"; bad=1; }
[ -n "$s" ] && [ "$s" != "$a" ] && echo "PASS another seed: other samples ($s vs $a)" || { echo "MISS another seed: the same audio ($s)"; bad=1; }
[ -n "$p" ] && [ "$p" != "$a" ] && echo "PASS another pitch: other samples ($p vs $a)" || { echo "MISS another pitch: the same audio ($p)"; bad=1; }
# what each note was made with, from the store's index
for r in a s p; do
  m=$(ls "$OUT/$r"/assets/made.jsonl "$OUT/$r"/*/made.jsonl 2>/dev/null | head -1)
  [ -n "$m" ] && echo "     $r made with: $(jq -c '.origin | {seed, params}' "$m" | head -1)"
done
exit $bad
