// gap: control
// morph.tsx at weight 0: the card's right edge stays at 0.5 m.
// probe: 32 32 r>150
// probe: 52 32 r<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Model } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 1.0471975511966 } }} />
        <Model src="morph.gltf" morphs={{ wide: 0 }} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
