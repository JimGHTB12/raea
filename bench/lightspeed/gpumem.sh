#!/bin/bash
# R4, GPU memory per stage (docs/lightspeed.feature).
#
#   bench/lightspeed/gpumem.sh
#
# On Apple silicon the GPU's memory is the process's own: footprint(1) files Metal's buffers and textures under its
# "(graphics)" categories (owned, unmapped graphics memory and IOAccelerator). Each stage is played offscreen and read
# 5 s in, once it's steady (`raea ci graphics`):
#   pong      live, 1280x720
#   titles    from its bake, 1080x1920
#   ts1       from its bake, 1080x1920 (footage: the decoder's surfaces count)
# The value is each stage's graphics memory (dirty, MB). Not timed, so it takes no lock; the load is still recorded.
. "$(dirname "$0")/lib.sh"
H=bench/lightspeed/gpumem.sh

ST_START=$(load1)
res=""
for spec in "pong $ROOT/sdk/react/bench/games/pong/edit.tsx 1280x720 0" \
            "titles $ROOT/sdk/react/bench/timelines/titles/edit.tsx 1080x1920 1" \
            "ts1 $FIXTURES/ts1/edit.tsx 1080x1920 1"; do
  set -- $spec; name=$1; edit=$2; size=$3; baked=$4
  if [ ! -f "$edit" ]; then say "R4 $name: $edit missing; skipped"; continue; fi
  mode=--input; [ "$baked" = 1 ] && mode=--baked
  g=$(cd "$ROOT/sdk/react" && RAEA_AUDIO_MUTE=1 "$MOTION" ci graphics --after 5 -- "$MOTION" play "$edit" --offscreen --size "$size" --seconds 9 $mode) || exit 1
  res="$res$(echo "$g" | LS_JQ -c --arg n "$name" 'include "lib";
    {($n): {graphics_mb: (.graphics_mb | pyround(1)), footprint_mb: (.footprint_mb | if . == null then null else pyround(1) end),
            parts: (reduce .parts[] as $p ({}; .[$p[0]] = ($p[1] | pyround(1))))}}')"$'\n'
done
# memory, not time: the load doesn't change it much
ST_END=$(load1); ST_QUIET=true; ST_OTHERS='[]'
res=$(printf '%s' "$res" | jq -sc 'add // {}')
record R4 "$(echo "$res" | LS_JQ -c 'include "lib"; [to_entries[] | "\(.key) \(.value.graphics_mb | fixed(0))"] | join(" / ")')" \
  "MB GPU (graphics footprint) per stage" "$res" $H
