#!/usr/bin/env bash
# The conformance suite's fixture workspace, generated (nothing large is committed):
#   sdk/ui/conformance/fixture/make.sh OUT
# Laid out as apps/legacy/shell/spaces/feed/scan.sh reads a workspace:
#   intro/renders/v2-a.mp4 .. v2-g.mp4   seven takes of one render (2 s, 1920x1080 h264): take folding
#   intro/renders/i1-room.png            a still
#   outro/renders/s3.mp4, s3b.mp4        two takes of another group's render
#   media/gen/concept-1.jpg, -2.jpg      concepts
#   refs/swipe-1.png                     the swipe file
#   storyboard/board.md                  a twelve-shot storyboard (it scrolls, and has a bottom)
#   .spare/live.mp4                      not in the feed (hidden): the live cases copy it into a renders/ folder
#   library/still-01.png .. still-40.png forty small stills, not in the feed (no renders/): the left panel's Assets
#                                        list overflows at every size, so the left-list scroll cases have a range
#   .home/Studio/internal/projects/raea-play/pong/   the world's home (the runner moves .home there): the play
#                                        item's game, pong.tsx, as links to the repo's own bench Pong (and
#                                        .home/films/raea-play, a link to it, for older raea-feed)
# The runner copies OUT into each case's world and dates the files by times.txt. conformance runs this itself (once
# per version of this folder, cached under $TMPDIR/conformance/fixture-<hash>).
set -euo pipefail
out=${1:?usage: make.sh OUT}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"/{intro/renders,outro/renders,media/gen,refs,storyboard,.spare,library}
ff() { ffmpeg -nostdin -v error -y "$@"; }
# a 2 s 1920x1080 h264 clip, tinted by a hue (degrees)
clip() { ff -f lavfi -i "testsrc2=s=1920x1080:r=30:d=2" -vf "hue=h=$2,format=yuv420p" -c:v libx264 -preset ultrafast -crf 40 -movflags +faststart "$1"; }
hue=0
for t in a b c d e f g; do clip "$out/intro/renders/v2-$t.mp4" $hue; hue=$((hue + 45)); done
clip "$out/outro/renders/s3.mp4" 200
clip "$out/outro/renders/s3b.mp4" 260
clip "$out/.spare/live.mp4" 120
ff -f lavfi -i "testsrc2=s=1920x1080:r=1:d=1" -vf "hue=h=90" -frames:v 1 "$out/intro/renders/i1-room.png"
ff -f lavfi -i "mandelbrot=s=1280x720:r=1" -frames:v 1 -q:v 4 "$out/media/gen/concept-1.jpg"
ff -f lavfi -i "mandelbrot=s=1280x720:r=1:start_scale=1.5" -frames:v 1 -q:v 4 "$out/media/gen/concept-2.jpg"
ff -f lavfi -i "life=s=1080x1350:r=1:mold=10:ratio=0.4:death_color=#101820:life_color=#e0c060" -frames:v 1 "$out/refs/swipe-1.png"
cp "$here/board.md" "$out/storyboard/board.md"
ff -f lavfi -i "testsrc2=s=160x90:r=1:d=1" -frames:v 1 "$out/library/still-01.png"
for i in $(seq -w 2 40); do cp "$out/library/still-01.png" "$out/library/still-$i.png"; done
repo=$(cd "$here/../../../.." && pwd)
# the play item's source is ~/Studio/internal/projects/raea-play/pong (raea-feed from 40a27b4) or, in older
# raea-feed, ~/films/raea-play/pong: the fixture's home has both
pong=$out/.home/Studio/internal/projects/raea-play/pong
mkdir -p "$pong" "$out/.home/films"
for f in "$repo"/sdk/react/bench/games/pong/*; do ln -s "$f" "$pong/$(basename "$f")"; done
ln -s "$repo/sdk/react/bench/games/pong/edit.tsx" "$pong/pong.tsx"
ln -s ../Studio/internal/projects/raea-play "$out/.home/films/raea-play"
du -sh "$out" | awk '{print "fixture: " $1 " in " $2}'
