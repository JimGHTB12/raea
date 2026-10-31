// gap: G9
// A mesh from its own vertices: one big red triangle over the middle of the frame.
// probe: 32 36 r>200 g<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BufferMesh, Canvas3D } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Mesh3d name="tri"><BufferMesh positions={[-1.5, -1, 0, 1.5, -1, 0, 0, 1.5, 0]} indices={[0, 1, 2]} /><StandardMaterial base_color="#ff0000" unlit /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
