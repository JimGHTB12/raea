// Bevy v0.20.0 examples/2d/transparency_2d.rs in raea's TSX (docs/control.feature FC0a): Bevy's icon (branding/icon.png) three times, 100 px apart:
// plain, tinted blue at 0.7 alpha and tinted green at 0.3 alpha, each drawn over the last (z 0, 0.1, 0.2), on Bevy's
// default clear colour, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D, Sprite } from "raea/3d";

const ICON = staticFile("branding/icon.png");

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Sprite src={ICON} x={540} y={360} z={0} />
        <Sprite src={ICON} x={640} y={360} z={0.1} color="#0000ff" opacity={0.7} />
        <Sprite src={ICON} x={740} y={360} z={0.2} color="#00ff00" opacity={0.3} />
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
