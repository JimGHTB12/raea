// Bevy v0.20.0 examples/2d/sprite.rs in raea's TSX (docs/control.feature FC0a): a Camera2d and Bevy's bird
// (branding/bevy_bird_dark.png) as a Sprite at its own size in the middle, on Bevy's default clear colour, in Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D, Sprite } from "raea/3d";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Sprite src={staticFile("branding/bevy_bird_dark.png")} x={640} y={360} />
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
