// gap: PL8
// bevy_hanabi through raea/plugins/hanabi (docs/plugins.feature PL8): one fountain of opaque orange GPU particles
// rising from a pedestal against a black frame. By frame 20 the particles fill a column above it through the frame's
// lower middle (orange: red high, blue low) while the frame's top corners stay black.
// build: plugins
// frame: 20
// probe: 32 42 r>150 b<120
// probe: 2 2 r<20 g<20 b<20
// probe: 61 2 r<20 g<20 b<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, ToneMapping } from "raea/3d";
import { Particles } from "raea/plugins/hanabi";
import { StandardMaterial } from "raea/pbr";
import { ConicalFrustum, Mesh3d } from "raea/mesh";
import { AmbientLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 1.5, 5).lookingAt([0, 1.5, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <ToneMapping mode="none" />
        <AmbientLight brightness={0} />
        <Mesh3d transform={Transform.fromXyz(0, 0, 0)}>
          <ConicalFrustum radius_top={0.25} radius_bottom={0.35} height={0.3} />
          <StandardMaterial base_color="#000000" />
          <Particles value={{ rate: 2000, lifetime: 1.6, speed: 4, spread: 0.15, size: 0.12, color: [1, 0.55, 0.1, 1], gravity: 5, seed: 7, opaque: true }} />
        </Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={24} fps={30} width={64} height={64} />);
