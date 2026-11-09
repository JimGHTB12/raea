// Bevy v0.20.0 examples/shader_advanced/specialized_mesh_pipeline.rs in raea's TSX (docs/control.feature FC2d): meshes
// with no material drawn only by a SpecializedMeshPipeline of assets/shaders/specialized_mesh_pipeline.wesl as written
// (positions at location 0, compressed or not, and colours at 1 under VERTEX_COLORS when the mesh has them), depth
// tested and written GreaterEqual, back faces culled, under the view's MSAA: a <MeshPass depth> in each mesh. Three
// copies of one triangle ((-0.5, -0.5), (0.5, -0.5), (0, 0.25), coloured red, green and blue) at (-0.5, -0.25),
// (0, 0.5) and (0.5, -0.25), and a 0.5 m Rectangle with no normals or uvs, compressed (MeshCompressionArgs::regular()),
// at (0, -1/3, -1), each a <Mesh3d material={false}> (no MeshMaterial3d); the camera at (0, 0, 3) looking at the
// origin, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { BufferMesh, MeshPass } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d } from "raea/mesh";
import { Transform } from "raea/transform";

const SHADER = staticFile("shaders/specialized_mesh_pipeline.wesl");

function Shot() {
  return (
    <>
      {([[-0.5, -0.25], [0, 0.5], [0.5, -0.25]] as const).map(([x, y]) => (
        <Mesh3d key={x} transform={Transform.fromXyz(x, y, 0)} material={false}>
          <BufferMesh positions={[-0.5, -0.5, 0, 0.5, -0.5, 0, 0, 0.25, 0]} indices={[0, 1, 2]} colors={[1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 1]} />
          <MeshPass shader={SHADER} depth />
        </Mesh3d>
      ))}
      {/* a Rectangle is Plane3d facing +Z, its positions the same; the shader reads only them */}
      <Mesh3d transform={Transform.fromXyz(0, -1 / 3, -1)} compressed material={false}>
        <Plane3d normal={[0, 0, 1]} half_size={[0.25, 0.25]} />
        <MeshPass shader={SHADER} depth />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
