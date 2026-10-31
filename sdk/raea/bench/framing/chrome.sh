#!/bin/bash
# The framing check against Chrome: a <Video framing="face"> frame as raea draws it, against the same crop written out
# as CSS and drawn by Chrome (P26's reference). Chrome can't read a face, so the reference takes the engine's own face
# track (TAKE.faces.json, which the render keeps beside the take) and its crop (raea-page fit_rect: the eyes held 38%
# down and centred, the cover scale times punchIn), as an <img> placed absolutely in an overflow:hidden box the size of
# the frame. Beside it, a plain centred cover of the same take shows what framing changed.
#
#   sdk/raea/bench/framing/chrome.sh COMPOSITION TAKE FRAMES [PUNCH]     (RAEA_BIN, default raea)
#     COMPOSITION  a P26 row (examples/parts/brand/face-frame.tsx: its project made as compare.sh makes it, props
#                  laid over) or a .tsx path rendered in its own folder
#     TAKE         the take as the composition plays it (its faces.json is beside it: TAKE minus its extension
#                  plus .faces.json)
#     FRAMES       composition frames, comma-separated (e.g. 0,29,59)
#     PUNCH        the Video's punchIn (default 1)
# The video is assumed to fill the frame from its start (no startFrom, rate 1), as framed.tsx and FaceFrame do.
# Prints a line per frame: SSIM against Chrome's framed crop and against a plain centred cover, both at the crop's own
# resolution. Exits 1 when a frame isn't closer to the framed crop than to the plain cover by 0.2 (framed.tsx at
# 0,30,59: 0.995/0.996/0.996 against the framed crop, 0.25-0.27 against plain).
# FRAMING_KEEP=DIR keeps the pictures (Chrome's framed and plain, and raea's).
set -u
export RAEA_SIGN=0 RAEA_MARK=0
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
BIN=${RAEA_BIN:-raea}
[ $# -ge 3 ] || { sed -n '9,16p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
COMP=$1 TAKE=$2 FRAMES=$3 PUNCH=${4:-1}
CHROME=$(bash "$ROOT/sdk/ui/parity/chrome.sh" | tail -1)
[ -x "$CHROME" ] || { echo "MISS no chrome-headless-shell (sdk/ui/parity/chrome.sh)"; exit 1; }
WORK=$(mktemp -d "${TMPDIR:-/tmp}/raea-framing-chrome.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
if [ -f "$COMP" ] && [[ "$COMP" = /* ]]; then
  src=$COMP dir=$(dirname "$COMP")
else
  HERE_P26=$ROOT/bench/p26-baseline
  # lib.sh's p26_src reads HERE (the P26 folder), ROOT and WORK, and sets src and dir
  (HERE=$HERE_P26; source "$HERE_P26/lib.sh"; p26_src "$COMP"; echo "$src"; echo "$dir") > "$WORK/where" || exit 1
  src=$(sed -n 1p "$WORK/where") dir=$(sed -n 2p "$WORK/where")
fi
(cd "$dir" && "$BIN" render "$src" --at "$FRAMES" -o "$WORK/r/") > "$WORK/render.log" 2>&1 || { echo "MISS the render failed: $(grep -m1 -E 'REFUSED|error' "$WORK/render.log" | cut -c1-200)"; exit 1; }
FACES=${TAKE%.*}.faces.json
[ -f "$FACES" ] || { echo "MISS no face track at $FACES (does the composition frame the take with framing=\"face\"?)"; exit 1; }
fps=$(grep -ho 'fps={[0-9.]*}' "$src" "$dir"/*.tsx 2>/dev/null | head -1 | tr -dc '0-9.'); fps=${fps:-30}
read -r W H < <(ffprobe -v error -show_entries stream=width,height -of csv=p=0 "$WORK/r/frame-00000.png" | tr , " ")
read -r FW FH < <(ffprobe -v error -select_streams v -show_entries stream=width,height -of csv=p=0 "$TAKE" | tr , " ")
# SSIM at the take's own resolution (both scaled to the crop's size, WxH): neither picture holds more than the take's pixels,
# and blown up (a 320x180 take's face crop is 4.6x) raea's and Chrome's resampling filters would be most of the
# difference
ssim() { ffmpeg -nostdin -hide_banner -nostats -i "$1" -i "$2" -lavfi "[0:v]format=rgb24,scale=$3:flags=area[a];[1:v]format=rgb24,scale=$3:flags=area[b];[a][b]ssim" -f null - 2>&1 | grep -o 'All:[0-9.]*' | cut -d: -f2; }
shot() { "$CHROME" --headless --disable-gpu --hide-scrollbars --force-device-scale-factor=1 --window-size="$W,$H" --screenshot="$2" "file://$1" >/dev/null 2>&1; }
page() { # FILE SRC LEFT TOP WIDTH HEIGHT
  printf '<html><body style="margin:0;background:#000"><div style="position:relative;width:%spx;height:%spx;overflow:hidden"><img src="%s" style="position:absolute;left:%spx;top:%spx;width:%spx;height:%spx"></div></body></html>\n' "$W" "$H" "$2" "$3" "$4" "$5" "$6" > "$1"
}
bad=0 k=0
IFS=, read -ra list <<< "$FRAMES"
for f in "${list[@]}"; do
  t=$(awk -v f="$f" -v r="$fps" 'BEGIN { printf "%.6f", f / r }')
  # the eyes at t as sdk/react runtime/page/src/faces.rs eyes_at takes them: between the two samples around t when both found
  # a face, else the one that did, else the nearest sample that did
  eyes=$(jq -r --argjson t "$t" '.every as $e | .eyes as $E | ([$t / $e, 0] | max) as $i | ($i | floor) as $a | ($i | ceil) as $b
    | if $E[$a] != null and $E[$b] != null then ($i - $a) as $k | [$E[$a][0] + ($E[$b][0] - $E[$a][0]) * $k, $E[$a][1] + ($E[$b][1] - $E[$a][1]) * $k]
      elif $E[$a] != null then $E[$a] elif $E[$b] != null then $E[$b]
      else ([$a, ($E | length) - 1] | min) as $at | [range(0; $E | length) | select($E[.] != null)] | min_by(. - $at | fabs) as $n | $E[$n] end
    | "\(.[0]) \(.[1])"' "$FACES")
  # raea-page fit_rect: the source rect [x, y, w, h] stretched to the box, as left/top/size of the whole take
  read -r x y w h L T IW IH < <(awk -v bw="$W" -v bh="$H" -v fw="$FW" -v fh="$FH" -v p="$PUNCH" -v e="$eyes" 'function r(v) { return int(v + 0.5) }
    function cl(v, a, b) { return v < a ? a : v > b ? b : v }
    BEGIN { split(e, E, " "); s = (bw / fw > bh / fh ? bw / fw : bh / fh) * (p > 1 ? p : 1)
      w = cl(r(bw / s), 1, fw); h = cl(r(bh / s), 1, fh)
      x = cl(r(E[1] * fw - 0.5 * w), 0, fw - w); y = cl(r(E[2] * fh - 0.38 * h), 0, fh - h)
      printf "%d %d %d %d %.4f %.4f %.4f %.4f\n", x, y, w, h, -x * bw / w, -y * bh / h, fw * bw / w, fh * bh / h }')
  # without the gAMA and cHRM chunks ffmpeg writes from the take's BT.709 tags: Chrome colour-manages a PNG by them
  # (gamma 0.4545 drew face.mp4's frame 30 at grey 36.6 where its pixels are 46.2), and raea draws the decoded pixels
  ffmpeg -nostdin -loglevel error -y -i "$TAKE" -ss "$t" -frames:v 1 -f image2pipe -vcodec ppm - | magick ppm:- "$WORK/src-$f.png"
  page "$WORK/framed-$f.html" "src-$f.png" "$L" "$T" "$IW" "$IH"
  # a plain cover, centred: the take scaled to cover the box
  cs=$(awk -v bw="$W" -v bh="$H" -v fw="$FW" -v fh="$FH" 'BEGIN { s = (bw / fw > bh / fh ? bw / fw : bh / fh); printf "%.4f %.4f %.4f %.4f", (bw - fw * s) / 2, (bh - fh * s) / 2, fw * s, fh * s }')
  page "$WORK/plain-$f.html" "src-$f.png" $cs
  shot "$WORK/framed-$f.html" "$WORK/chrome-framed-$f.png"
  shot "$WORK/plain-$f.html" "$WORK/chrome-plain-$f.png"
  got=$(printf "%s/r/frame-%05d.png" "$WORK" "$k")
  a=$(ssim "$got" "$WORK/chrome-framed-$f.png" "${w}x$h") b=$(ssim "$got" "$WORK/chrome-plain-$f.png" "${w}x$h")
  line="frame $f: SSIM $a against Chrome's framed crop, $b against a plain cover, at the crop's ${w}x$h (eyes $eyes, crop $x,$y ${w}x$h of ${FW}x$FH)"
  if awk -v a="${a:-0}" -v b="${b:-1}" 'BEGIN { exit !(a >= b + 0.2) }'; then echo "PASS $line"; else echo "MISS $line"; bad=1; fi
  [ -n "${FRAMING_KEEP:-}" ] && mkdir -p "$FRAMING_KEEP" && cp "$WORK/chrome-framed-$f.png" "$WORK/chrome-plain-$f.png" "$FRAMING_KEEP/" && cp "$got" "$FRAMING_KEEP/raea-$f.png"
  k=$((k + 1))
done
exit $bad
