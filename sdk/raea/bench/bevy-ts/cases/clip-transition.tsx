// gap: G8
// A transition between two clips: left's weight goes 1 to 0 and right's 0 to 1 over frames 0 to 8 (interpolate), so
// the chip starts on the left and ends on the right.
// frame: 8
// probe: 53 32 r>150
// probe: 11 32 r<60
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D, Model } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 1.0471975511966 } }} />
        <Model src="clips.gltf" animations={[{ clip: "left", weight: interpolate(f, [0, 8], [1, 0], { extrapolateRight: "clamp" }) }, { clip: "right", weight: interpolate(f, [0, 8], [0, 1], { extrapolateRight: "clamp" }) }]} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={10} fps={30} width={64} height={64} />);
