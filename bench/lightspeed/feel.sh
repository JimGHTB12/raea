#!/bin/bash
# The feel rows and the GC rows (docs/lightspeed.feature): F1, F2, F3, F4, F6, R1, R2.
#
#   bench/lightspeed/feel.sh [--seconds 60] [--only games,gc,scrub]
#
#   games  sdk/react/bench/games/gate.sh SECONDS pong platformer arena3d (the 3D one), offscreen at 120 fps with keys
#          pressed: F1 input to present p99 (offscreen; the in-window gate, apps/legacy/shell/bench/gates.py, went with the
#          Dioxus window in 222ed773: void's bevy_ui window measures input to frame through sdk/ui/conformance),
#          F2 work p99 and p99.9, F3 missed frames, F6 hitches (frames past
#          the first second over 2 ms)
#   gc     pong again with JSC_logGC=1 (lib.jq's gc parse): R2 collections a minute and the worst pause, R1's JS layer as
#          the garbage freed a frame (the other layers aren't counted yet: ZeroHeap Z1)
#   scrub  60 shuffled seeks of ts1 (1080x1920) and k4 (4K) from their bakes (play --scrub, as
#          sdk/react/bench/video/gate.sh): F4 scrub p99
. "$(dirname "$0")/lib.sh"
H=bench/lightspeed/feel.sh
REACT=$ROOT/sdk/react

seconds=60; only=games,gc,scrub
while [ $# -gt 0 ]; do
  case $1 in
    --seconds) seconds=$2; shift ;;
    --only) only=$2; shift ;;
    *) echo "usage: bench/lightspeed/feel.sh [--seconds S] [--only games,gc,scrub]" >&2; exit 2 ;;
  esac
  shift
done
# Python's float: the harness passed str(float) on (60 -> 60.0)
seconds=$(pyf "$seconds")
has() { case ",$only," in *",$1,"*) return 0 ;; esac; return 1; }

games() {
  local tmp out
  tmp=$(mktemp -d "${TMPDIR:-/tmp}/lightspeed-feel-XXXXXX")
  turn_begin feel:games
  out=$(cd "$REACT" && TMPDIR=$tmp sh bench/games/gate.sh "$seconds" pong platformer arena3d 2> /dev/null)
  turn_end
  # Python's str.strip()
  say "$(printf '%s' "$out" | awk '{ l[NR] = $0 } END { s = 1; while (s <= NR && l[s] ~ /^[ \t\r]*$/) s++; e = NR; while (e >= s && l[e] ~ /^[ \t\r]*$/) e--
    for (i = s; i <= e; i++) { x = l[i]; if (i == s) sub(/^[ \t\r]+/, "", x); if (i == e) sub(/[ \t\r]+$/, "", x); print x } }')"
  local files=()
  for g in pong platformer arena3d; do
    [ -f "$tmp/raea-games-gate/$g.json" ] && files+=("$g" "$tmp/raea-games-gate/$g.json")
  done
  local per rows
  per=$(for ((i = 0; i < ${#files[@]}; i += 2)); do jq -c --arg g "${files[i]}" '{($g): .}' "${files[i + 1]}"; done | LS_JQ -sc 'include "lib";
    add // {} | map_values(
      {input_p99_ms: (.input_to_present_ms.p99 | pyround(2)), input_p99_frames: (.input_to_present_ms.p99_frames | pyround(2)),
       work_p99_ms: (.work_ms.p99 | pyround(2)), work_p999_ms: ((.work_ms.p999 // 0) | pyround(2)),
       missed: .missed_vs_display, presented: .presented,
       work_p50_ms: (.work_ms.p50 | pyround(2)), hitches: (.hitches_over_2ms // {} | .count),
       hitch_max_ms: ((.hitches_over_2ms // {} | .max // 0) | pyround(2)),
       first_hitches: ((.hitches_over_2ms // {} | .first // [])[:6])})')
  if [ "$per" = "{}" ]; then echo "feel: no game ran (gate.sh: $out)" >&2; rm -rf "$tmp"; return 1; fi
  # a hitch is a frame over 2 ms in a game whose frames are well under it; a game whose median frame is already near
  # 2 ms (arena3d's 3D frame, ~3 ms) is F2's steady cost, not first-use stalls, so it's listed, not counted
  rows=$(echo "$per" | LS_JQ -c --arg s "$seconds" 'include "lib";
    def worst($k): [.[][$k]] | max;
    (with_entries(select((.value.work_p50_ms // 0) < 1.5))) as $first
    | ([keys[] as $k | select($first | has($k) | not) | $k] | sort) as $steady
    | ([$first[].hitch_max_ms] | if length == 0 then 0 else max end) as $hmax
    | {F1: (worst("input_p99_ms") | pyf),
       F2: ("\(worst("work_p99_ms") | pyf) / \(worst("work_p999_ms") | pyf)" | tojson),
       F3: ([.[].missed] | add | tostring),
       F3u: "missed frames in \($s | tonumber | fixed(0)) s x \(length) games",
       F6: ("\([$first[].hitches] | add // 0) (max \(if ($first | length) == 0 then "0" else $hmax | pyf end) ms)"
            + (if ($steady | length) > 0 then "; \($steady | join(", ")) steady over 2 ms, in F2" else "" end) | tojson)}')
  rm -rf "$tmp"
  record F1 "$(echo "$rows" | jq -r .F1)" "ms input to present p99, offscreen 120 fps (worst game)" "$per" $H
  record F2 "$(echo "$rows" | jq -r .F2)" "ms work p99 / p99.9 (worst game)" "$per" $H
  record F3 "$(echo "$rows" | jq -r .F3)" "$(echo "$rows" | jq -r .F3u)" "$per" $H
  record F6 "$(echo "$rows" | jq -r .F6)" "hitches over 2 ms past the first second" "$per" $H
}

gc() {
  local s=$1 tmp detail
  tmp=$(mktemp -d "${TMPDIR:-/tmp}/lightspeed-gc-XXXXXX")
  turn_begin feel:gc
  (cd "$REACT" && JSC_logGC=1 RAEA_AUDIO_MUTE=1 "$MOTION" play bench/games/pong/edit.tsx --input --offscreen --size 1280x720 \
    --seconds "$s" --json "$tmp/pong.json" > "$tmp/out" 2> "$tmp/err")
  turn_end
  cat "$tmp/out" "$tmp/err" > "$tmp/all"
  detail=$(LS_JQ -c --rawfile t "$tmp/all" 'include "lib";
    [$t | match("START \\w (\\d+)kb => (\\w+),.*?=> (\\d+)kb, p=([0-9.]+)ms \\(max [0-9.]+\\), cycle ([0-9.]+)ms END\\]"; "g")
      | [.captures[].string]] as $gcs
    | [$gcs[][3] | tonumber] as $pauses
    | ([$gcs[] | (.[0] | tonumber) - (.[2] | tonumber)] | add // 0) as $freed
    | {collections: ($gcs | length), per_minute: (($gcs | length) * 60 / .seconds | pyround(1)),
       pause_max_ms: ($pauses | max // 0), pause_total_ms: ($pauses | add // 0 | pyround(1)),
       freed_kb_per_frame: ($freed / ([.presented, 1] | max) | pyround(1)),
       kinds: ([$gcs[][1]] | unique), frames: .presented, seconds: .seconds}' "$tmp/pong.json") || { rm -rf "$tmp"; return 1; }
  rm -rf "$tmp"
  local r2 r1
  r2=$(echo "$detail" | LS_JQ -c 'include "lib"; "\(.per_minute | pyf)/min (max pause \(if .collections > 0 then .pause_max_ms | pyf else "0" end) ms)"')
  r1=$(echo "$detail" | LS_JQ -c 'include "lib"; "JS \(.freed_kb_per_frame | pyf) KB/frame"')
  record R2 "$r2" "JSC collections, pong" "$detail" $H
  record R1 "$r1" "garbage a frame, pong (JS layer; others uncounted)" "$detail" $H
}

scrub() {
  local res="" name size edit t
  t=$(mktemp "${TMPDIR:-/tmp}/lightspeed-scrub.XXXXXX")
  turn_begin feel:scrub
  for spec in "ts1 1080x1920" "k4 2160x3840"; do
    set -- $spec; name=$1; size=$2
    edit=$FIXTURES/$name/edit.tsx
    if [ ! -f "$edit" ]; then say "F4: $edit missing; skipped"; continue; fi
    (cd "$REACT" && RAEA_AUDIO_MUTE=1 "$MOTION" play "$edit" --baked --offscreen --size "$size" --scrub 60 > "$t.out" 2> "$t.err")
    cat "$t.out" "$t.err" > "$t"
    # the JSON after SCRUBBED: from the first { after it to the text's last }
    local j
    if ! j=$(jq -c --rawfile t "$t" -n --arg n "$name" '
        ($t | split("SCRUBBED")) as $p | if ($p | length) < 2 then error("no SCRUBBED") else
        ($p[1:] | join("SCRUBBED")) as $rest | ($rest | split("{")) as $q
        | ("{" + ($q[1:] | join("{"))) as $from | ($from | split("}")) as $r
        | ($r[:-1] | join("}") + "}") | fromjson end' 2> /dev/null); then
      say_tail "F4 $name: the scrub failed: " 400 "$t"; continue
    fi
    res="$res$(echo "$j" | LS_JQ -c --arg n "$name" 'include "lib"; {($n): {scrub_p50_ms: (.scrub_ms.p50 | pyround(1)),
      scrub_p99_ms: (.scrub_ms.p99 | pyround(1)), scrub_max_ms: (.scrub_ms.max | pyround(1)), first_frame_ms: .first_frame_ms,
      seeks: .seeks}}')"$'\n'
  done
  turn_end
  rm -f "$t" "$t.out" "$t.err"
  [ -n "$res" ] || return 0
  res=$(printf '%s' "$res" | jq -sc add)
  record F4 "$(echo "$res" | LS_JQ -c 'include "lib"; [to_entries[] | "\(.key) \(.value.scrub_p99_ms | pyf)"] | join(" / ")')" \
    "ms scrub p99 (1080p / 4K)" "$res" $H
}

status=0
if has games; then games || status=1; fi
# Python's min(seconds, 60): the float up to 60, the int 60 past it
if has gc; then gc "$(awk -v s="$seconds" 'BEGIN { print (s + 0 <= 60 ? s : "60") }')" || status=1; fi
if has scrub; then scrub || status=1; fi
exit $status
