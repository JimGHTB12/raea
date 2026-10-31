// gap: G8
// graph-additive.tsx's clips under a blend node: weights 1 and 0.5 normalised, (-1.5 + 0.75) / 1.5 = -0.5 m.
// probe: 28 32 r>150
// probe: 18 32 r<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Model } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 1.0471975511966 } }} />
        <Model src="clips.gltf" graph={{ blend: [{ clip: "left" }, { clip: "right", weight: 0.5 }] }} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
