#!/bin/bash
# A still late in a long 3D composition costs about what one early in it does (#77). A still at --at F stepped React
# only near F, but held every frame before it as copies, keyed each changing column on all of them and evaluated and
# kept every one in the timeline; and the engine drew every frame before F (catch_up: the plugin clock, installed in
# every build, though nothing in the scene was a plugin's): a film's --at 2448 took 234 s and 7.1 GB against 9 s and
# 3.1 GB at --at 30. 200 turning cubes over 3618 frames, with Bevy's own components on them, are drawn at --at 30 and
# --at 2448. The peak memory at 2448 must stay under 1.6x the one at 30 (the #60 build: 2.3x); the engine must step no
# frame to reach 2448 (RAEA_PLUGIN_TRACE; was 2447); and the 2448 still must match the same scene drawn at frame 30
# with its clock moved on 2418 frames (within 1 level, as frames are pure), so the shortcut changes no pixel. A scene
# a plugin steps is still stepped: runtime/plugins' only_rows_... test. Exits 1 on a failure.
#   bench/at-cost/check.sh      (RAEA_BIN: the raea to run, default target/release/raea)
cd "$(dirname "$0")/../.." || exit 2
BIN=${RAEA_BIN:-$PWD/target/release/raea}
export RAEA_SIGN=0 RAEA_MARK=0 RAEA_EXPORT_CACHE=0
OUT=$(mktemp -d "${TMPDIR:-/tmp}/raea-at-cost.XXXXXX")
trap 'rm -rf "$OUT"' EXIT
scene() { # FILE OFFSET
  cat > "$1" <<TSX
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { DirectionalLight } from "raea/light";
import { Transform, Quat } from "raea/transform";
import { Camera3d } from "raea/camera";
import { BevyComponent } from "raea/3d";
function Shot() {
  const f = useCurrentFrame() + $2;
  const parts = Array.from({ length: 200 }, (_, i) => (
    <Mesh3d key={i} transform={Transform.fromXyz((i % 20) - 10, Math.floor(i / 20) - 5, -8).withRotation(Quat.fromEuler("XYZ", 0, f * 0.01 + i, 0))}>
      <Cuboid half_size={[0.3, 0.3, 0.3]} />
      <StandardMaterial base_color="#ff7a1a" />
      {i === 0 ? <BevyComponent type="bevy_light::NotShadowCaster" value={{}} /> : null}
    </Mesh3d>
  ));
  return (
    <AbsoluteFill style={{ backgroundColor: "#1b2230" }}>
      <Camera3d transform={Transform.fromXyz(0, 0, 8).lookingAt([0, 0, 0])} />
      <DirectionalLight illuminance={8000} transform={Transform.fromXyz(3, 5, 4).lookingAt([0, 0, 0])} />
      {parts}
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="c" component={Shot} width={640} height={360} fps={30} durationInFrames={3618} />);
TSX
}
scene "$OUT/late.tsx" 0
scene "$OUT/moved.tsx" 2418
status=0
peak() { # NAME TSX AT: the render's peak resident bytes on stdout
  /usr/bin/time -l "$BIN" render "$2" --at "$3" -o "$OUT/$1" > "$OUT/$1.log" 2>&1 && [ -f "$OUT/$1/frame-00000.png" ] || { echo "FAIL $1: the render failed" >&2; tail -4 "$OUT/$1.log" | sed 's/^/    /' >&2; return 1; }
  awk '/maximum resident set size/ { print $1 }' "$OUT/$1.log"
}
early=$(peak early "$OUT/late.tsx" 30) && late=$(peak late "$OUT/late.tsx" 2448) && moved=$(peak moved "$OUT/moved.tsx" 30) || exit 1
ratio=$(echo "scale=2; $late / $early" | bc)
if (( late * 10 < early * 16 )); then echo "ok   --at 2448 peaks at ${ratio}x --at 30 ($((late >> 20)) MB, $((early >> 20)) MB)"
else echo "FAIL --at 2448 peaks at ${ratio}x --at 30 ($((late >> 20)) MB against $((early >> 20)) MB), wanted under 1.6x"; status=1; fi
steps=$(RAEA_PLUGIN_TRACE=1 "$BIN" render "$OUT/late.tsx" --at 2448 -o "$OUT/trace" 2>&1 | grep -c "^plugin clock: .* step$")
if (( steps == 0 )); then echo "ok   --at 2448 draws no frame before it"
else echo "FAIL --at 2448 drew $steps frames before it, wanted none (the scene has nothing a plugin carries)"; status=1; fi
max=$(magick "$OUT/late/frame-00000.png" "$OUT/moved/frame-00000.png" -compose difference -composite -format "%[fx:int(maxima*255+0.5)]" info:)
if (( max <= 1 )); then echo "ok   frame 2448 matches the scene moved on to it at frame 30 (largest difference $max)"
else echo "FAIL frame 2448 differs from the scene moved on to it at frame 30 by up to $max levels"; status=1; fi
[ $status = 0 ] && echo "PASS a late still costs about what an early one does, and draws the same frame"
exit $status
