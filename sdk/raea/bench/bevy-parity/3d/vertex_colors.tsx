// Bevy v0.20.0 examples/3d/vertex_colors.rs in raea's TSX (docs/control.feature FC0a): a 5 m green plane and a unit
// cube whose vertices are coloured from their positions ((1 - p) / 2 a channel, linear), under a shadowed point light.
// The cube is Bevy's Cuboid mesh vertex for vertex (bevy_mesh cuboid.rs: 24 vertices, each face two triangles split
// on the same diagonal), as a BufferMesh with ATTRIBUTE_COLOR, so the colours interpolate across the same triangles.
import { Composition, registerRoot } from "raea";
import { Camera3d } from "raea/camera";
import { PointLight } from "raea/light";
import { BufferMesh, Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const [n, x] = [-0.5, 0.5];
// position, normal, uv: Bevy's order (front, back, right, left, top, bottom)
const V: [number[], number[], number[]][] = [
  [[n, n, x], [0, 0, 1], [0, 0]], [[x, n, x], [0, 0, 1], [1, 0]], [[x, x, x], [0, 0, 1], [1, 1]], [[n, x, x], [0, 0, 1], [0, 1]],
  [[n, x, n], [0, 0, -1], [1, 0]], [[x, x, n], [0, 0, -1], [0, 0]], [[x, n, n], [0, 0, -1], [0, 1]], [[n, n, n], [0, 0, -1], [1, 1]],
  [[x, n, n], [1, 0, 0], [0, 0]], [[x, x, n], [1, 0, 0], [1, 0]], [[x, x, x], [1, 0, 0], [1, 1]], [[x, n, x], [1, 0, 0], [0, 1]],
  [[n, n, x], [-1, 0, 0], [1, 0]], [[n, x, x], [-1, 0, 0], [0, 0]], [[n, x, n], [-1, 0, 0], [0, 1]], [[n, n, n], [-1, 0, 0], [1, 1]],
  [[x, x, n], [0, 1, 0], [1, 0]], [[n, x, n], [0, 1, 0], [0, 0]], [[n, x, x], [0, 1, 0], [0, 1]], [[x, x, x], [0, 1, 0], [1, 1]],
  [[x, n, x], [0, -1, 0], [0, 0]], [[n, n, x], [0, -1, 0], [1, 0]], [[n, n, n], [0, -1, 0], [1, 1]], [[x, n, n], [0, -1, 0], [0, 1]],
];
const positions = V.flatMap((v) => v[0]);
const normals = V.flatMap((v) => v[1]);
const uvs = V.flatMap((v) => v[2]);
const colors = V.flatMap(([[r, g, b]]) => [(1 - r) / 2, (1 - g) / 2, (1 - b) / 2, 1]);
const indices = [0, 1, 2, 3, 4, 5].flatMap((f) => [0, 1, 2, 2, 3, 0].map((i) => f * 4 + i));

function Shot() {
  return (
    <>
      <Mesh3d>
        <Plane3d half_size={[2.5, 2.5]} />
        <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <BufferMesh positions={positions} normals={normals} uvs={uvs} colors={colors} indices={indices} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
      <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 5, 4).lookingAt([0, 0, 0])} />
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
