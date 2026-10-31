// gap: G13b
// A procedural sky (<Atmosphere>, Bevy 0.20's scattering atmosphere): looking at the horizon under a high sun, the sky
// above is blue (more blue than red) where without it the clear colour is black; the ground plane below stays dark.
// probe: 32 6 b>60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Atmosphere, Canvas3D } from "raea/3d";
import { DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 1, 0).lookingAt([0, 1.2, -10])} projection={{ Perspective: { fov: 1.0471975511966 } }} />
        <DirectionalLight transform={Transform.fromXyz(0.3, 1, 0.2).lookingAt([0, 0, 0])} illuminance={32000} />
        <Atmosphere />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
