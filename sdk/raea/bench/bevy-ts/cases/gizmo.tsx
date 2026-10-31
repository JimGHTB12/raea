// gap: G11
// A debug line across the frame (immediate-mode gizmo): red through the middle row.
// probe: 32 32 r>150
// probe: 10 32 r>150
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Gizmo } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Gizmo lines={[[[-3, 0, 0], [3, 0, 0]]]} color="#ff0000" width={6} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
