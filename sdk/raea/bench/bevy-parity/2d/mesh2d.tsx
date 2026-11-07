// Bevy v0.20.0 examples/2d/mesh2d.rs in raea's TSX (docs/control.feature FC0a): a Camera2d and a default Rectangle
// scaled to 128 px with a PURPLE ColorMaterial, on Bevy's default clear colour, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { ColorMaterial, Mesh2d, Rectangle } from "raea/sprite";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Mesh2d transform={Transform.IDENTITY.withScale([128, 128, 1])}>
          <Rectangle />
          <ColorMaterial color="#800080" />
        </Mesh2d>
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
