#!/bin/bash
# The encode fixture's checks (edit.tsx beside this: flat SVG shapes and text moving at 4K30). Frame 45 of each .mp4,
# decoded, against the lossless PNG of that frame (--at 45), whole frame and a crop of small type:
#   default   the bitrate from the size and frame rate (raea_media::default_kbps, 80 Mbit/s asked at 4K30): text-crop
#             PSNR >= 47 dB and SSIM >= 0.998 (the old flat 10 Mbit/s gave 43.2 dB and 0.9975: rough type and edges)
#   master    --quality master (VideoToolbox constant quality 0.95): text-crop PSNR >= 54 dB
#   kbps      --kbps 10000 is taken (the old default, printed for comparison) and changes the file's size
#   bash sdk/raea/bench/encode/check.sh [RAEA_BIN]     exits 1 on any failure
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
BIN=${1:-${RAEA_BIN:-$ROOT/target/release/raea}}
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-encode-check.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
export RAEA_EXPORT_CACHE=0
status=0
fail() { echo "FAIL $*"; status=1; }

render() { # NAME OUT ARGS...
  local n=$1 o=$2; shift 2
  "$BIN" render "$HERE/edit.tsx" "$@" -o "$o" > "$OUT/$n.log" 2>&1 || { echo "render $n failed (log below)"; tail -20 "$OUT/$n.log"; exit 1; }
}
render png "$OUT/png/" --at 45
REF="$OUT/png/frame-00000.png"
CROP=crop=1600:500:200:1060
metric() { # A B FILTER(ssim|psnr) [CROP]
  local c=${4:+$4,}
  ffmpeg -nostats -i "$1" -i "$2" -lavfi "[0]${c}format=rgb24[a];[1]${c}format=rgb24[b];[a][b]$3" -f null - 2>&1 | grep -oE '(All|average):[0-9.]+' | cut -d: -f2
}
measure() { # NAME ARGS...: render, then "mbps ssim textpsnr"
  local n=$1; shift
  render "$n" "$OUT/$n.mp4" "$@"
  ffmpeg -v error -y -i "$OUT/$n.mp4" -vf "select=eq(n\,45)" -frames:v 1 "$OUT/$n-45.png"
  local mbps; mbps=$(ffprobe -v error -show_entries format=bit_rate -of csv=p=0 "$OUT/$n.mp4" | awk '{printf "%.1f", $1/1e6}')
  echo "$mbps $(metric "$OUT/$n-45.png" "$REF" ssim) $(metric "$OUT/$n-45.png" "$REF" psnr $CROP)"
}
read -r m s p <<< "$(measure default)"; echo "default        ${m} Mbit/s  SSIM $s  text PSNR $p dB"
[ "$(echo "${p:-0} >= 47 && ${s:-0} >= 0.998" | bc)" = 1 ] || fail "default: text PSNR ${p:-none}, SSIM ${s:-none}"
read -r m2 s p <<< "$(measure master --quality master)"; echo "quality master ${m2} Mbit/s  SSIM $s  text PSNR $p dB"
[ "$(echo "${p:-0} >= 54" | bc)" = 1 ] || fail "master: text PSNR ${p:-none}"
read -r m3 s p <<< "$(measure kbps --kbps 10000)"; echo "kbps 10000     ${m3} Mbit/s  SSIM $s  text PSNR $p dB"
[ "$m3" != "$m" ] || fail "kbps: --kbps 10000 made the default's ${m} Mbit/s"
[ $status = 0 ] && echo "encode: all checks pass"
exit $status
