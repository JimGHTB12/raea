// Bevy v0.20.0 examples/2d/rotate_to_cursor.rs in raea's TSX (docs/control.feature FC0a): the ship
// (textures/simplespace/ship_C.png) as a Sprite in the middle, on Bevy's default clear colour, in Bevy's default
// 1280 x 720 window. It turns to face the cursor, and the harness's window has none, so it stays unturned on both sides.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D, Sprite } from "raea/3d";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Sprite src={staticFile("textures/simplespace/ship_C.png")} x={640} y={360} />
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
