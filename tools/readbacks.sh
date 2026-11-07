#!/bin/bash
# The GPU-to-CPU copies in a run (docs/blitz-bevy.feature B1d): every copy into a buffer the CPU maps (MAP_READ), read
# from the shared device's wgpu trace. Take one with RAEA_WGPU_TRACE=DIR on any raea program, then:
#   tools/readbacks.sh DIR            one line per destination buffer's label: copies, bytes; then the total
#   tools/readbacks.sh DIR --count    the total copies alone
# A copy whose buffer label is in RAEA_READBACKS_EXCEPT (comma separated, e.g. the MP4 encoder's) is counted apart.
set -euo pipefail
dir=${1:?usage: tools/readbacks.sh TRACE_DIR [--count]}
trace=$dir/trace.ron
[ -f "$trace" ] || { echo "readbacks: no $trace (run with RAEA_WGPU_TRACE=$dir)" >&2; exit 2; }
awk -v count_only="${2:-}" -v except="${RAEA_READBACKS_EXCEPT:-}" '
  BEGIN { n = split(except, ex, ","); for (i = 1; i <= n; i++) skip[ex[i]] = 1 }
  # CreateBuffer(PointerId(ID), ( label: Some("L"), size: N, usage: "...", ...
  /^CreateBuffer\(PointerId\(/ { match($0, /PointerId\([0-9]+\)/); id = substr($0, RSTART, RLENGTH); want = 1; label = "(no label)"; next }
  want && /label: Some\(/ { l = $0; sub(/.*label: Some\("/, "", l); sub(/"\).*/, "", l); label = l; next }
  want && /usage:/ { if ($0 ~ /MAP_READ/) { read[id] = label }; want = 0; next }
  # CopyTextureToBuffer( src: (...), dst: ( buffer: PointerId(ID), ...  and  CopyBufferToBuffer( ..., dst: PointerId(ID),
  /CopyTextureToBuffer\(/ { t2b = 1; next }
  t2b && /buffer: PointerId\(/ { match($0, /PointerId\([0-9]+\)/); hit(substr($0, RSTART, RLENGTH)); t2b = 0; next }
  /CopyBufferToBuffer\(/ { b2b = 1; next }
  b2b && /dst: PointerId\(/ { match($0, /PointerId\([0-9]+\)/); hit(substr($0, RSTART, RLENGTH)); b2b = 0; next }
  function hit(id) { if (id in read) { by[read[id]]++ } }
  END {
    total = 0; apart = 0
    for (l in by) { if (l in skip) apart += by[l]; else total += by[l] }
    if (count_only == "--count") { print total; exit }
    for (l in by) printf "%6d  %s%s\n", by[l], l, (l in skip) ? "  (excepted)" : ""
    printf "readbacks: %d", total; if (apart) printf " (+%d excepted)", apart; print ""
  }
' "$trace"
