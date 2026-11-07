// Bevy v0.20.0 examples/2d/move_sprite.rs in raea's TSX (docs/control.feature FC0a): Bevy's icon (branding/icon.png) as a Sprite
// moving along X at 150 px/s from the middle, turning back past +-200 px, on Bevy's default clear colour, in Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D, Sprite } from "raea/3d";

/** sprite_movement over the updates up to frame f: each moves by 150 x its delta_secs (none on the first), then turns */
function xAt(f: number): number {
  let x = 0;
  let right = true;
  for (let k = 0; k <= f; k++) {
    // Bevy's delta: a Duration of 16_666_667 ns as f32 seconds (0.016666668), which tips x past 200 an update sooner
    const dt = k === 0 ? 0 : Math.fround(16_666_667e-9);
    x = Math.fround(x + (right ? 1 : -1) * Math.fround(150 * dt));
    if (x > 200) right = false;
    else if (x < -200) right = true;
  }
  return x;
}

function Shot() {
  const x = xAt(useCurrentFrame());
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Sprite src={staticFile("branding/icon.png")} x={640 + x} y={360} />
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={121} fps={60} width={1280} height={720} />);
