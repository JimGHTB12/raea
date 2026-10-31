// gap: G8
// An authored animation graph (<Model graph>): an additive node over clips.gltf's "left" (x -1.5 m) and half of
// "right" (+1.5 m) adds them, -1.5 + 0.75 = -0.75 m; graph-blend.tsx, the same two under a blend node, normalises the
// weights to -0.5 m. The chip's left edge tells them apart.
// probe: 19 32 r>150
// probe: 29 32 r<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Model } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 1.0471975511966 } }} />
        <Model src="clips.gltf" graph={{ add: [{ clip: "left" }, { clip: "right", weight: 0.5 }] }} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
