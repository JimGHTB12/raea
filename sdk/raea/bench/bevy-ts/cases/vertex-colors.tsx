// gap: G9
// Vertex colours (<BufferMesh colors>, Bevy's ATTRIBUTE_COLOR): a 3 m quad, its left vertices green and right ones
// blue, under a white unlit material that multiplies by them; the colour runs across it.
// probe: 8 32 g>150 b<90 r<90
// probe: 56 32 b>150 g<90 r<90
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
        <Mesh3d name="quad">
          <BufferMesh positions={[-1.5, -1, 0, 1.5, -1, 0, 1.5, 1, 0, -1.5, 1, 0]} indices={[0, 1, 2, 0, 2, 3]} colors={["#00ff00", "#0000ff", "#0000ff", "#00ff00"]} />
          <StandardMaterial base_color="#ffffff" unlit />
        </Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
