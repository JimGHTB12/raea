// Bevy v0.20.0 examples/2d/sprite_tile.rs in raea's TSX (docs/control.feature FC0a): Bevy's icon tiled every 128 px
// (SpriteImageMode::Tiled, stretch_value 0.5) across a square sprite whose size bounces between 128 and 512 px at
// 50 px/s, on Bevy's default clear colour, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D, Sprite } from "raea/3d";

/** `animate` over the updates up to frame f, in f32 as Bevy has it: turn at either end, then grow by speed x delta
 *  (none on the first update; then Bevy's 16_666_667 ns as f32 seconds) */
function sizeAt(f: number): number {
  const dt = Math.fround(16_666_667e-9);
  let [current, speed] = [128, 50];
  for (let k = 0; k <= f; k++) {
    if (current >= 512 || current <= 128) speed = -speed;
    current = Math.fround(current + Math.fround(speed * (k === 0 ? 0 : dt)));
  }
  return current;
}

function Shot() {
  const s = sizeAt(useCurrentFrame());
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Sprite src={staticFile("branding/icon.png")} x={640} y={360} width={s} height={s} tileX tileY tileStretch={0.5} />
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={301} fps={60} width={1280} height={720} />);
