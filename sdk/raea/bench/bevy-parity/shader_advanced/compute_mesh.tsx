// Bevy v0.20.0 examples/shader_advanced/compute_mesh.rs in raea's TSX (docs/control.feature FC2d): a mesh filled on
// the GPU. Its empty mesh is 50 positions, normals and uvs of zero and 50 indices of zero (a BufferMesh with indices
// and normals, Bevy's indexed Mesh); assets/shaders/compute_mesh.wesl as written, a <Compute file> dispatched one
// workgroup before the cameras draw, writes a 3 m cube's 24 vertices and 36 indices into the mesh's vertex and index
// slabs (Bevy's MeshAllocator, storage buffers too, as the example's plugin sets them), at the range its uniform
// `data_range` gives (the slab's start in floats, its vertex start times 8, and its index start). The example
// dispatches once its pipeline is ready; the writes are the same each time, so here every frame. Two parts hold the
// mesh: the one the compute names, srgb RED_400 at (-2.5, 1.5, 0), and SKY_400 at (2.5, 1.5, 0). A white circle of
// radius 4 lying flat, a shadow-casting PointLight at (4, 8, 4) and the camera at (-2.5, 4.5, 9) looking at the
// origin, over a black clear colour, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { BevyComponent, Canvas3D, Compute, Entity } from "raea/3d";
import { Transform, Quat, eulerOf } from "raea/transform";
import { Camera3d } from "raea/camera";
import { BufferMesh, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

const zeros = (n: number) => new Array<number>(n).fill(0);
// the empty mesh: as many spots as the compute writes or more
const EMPTY = { positions: zeros(150), normals: zeros(150), uvs: zeros(100), indices: zeros(50) };
const RED_400 = "rgb(248, 113, 113)";
const SKY_400 = "rgb(56, 189, 248)";

function Shot() {
  return (
    <Canvas3D layered clearColor="#000000">
      <Compute
        file={staticFile("shaders/compute_mesh.wesl")}
        workgroups={[1, 1, 1]}
        entry="main"
        buffers={{ vertex_data: { mesh: "generated", slab: "vertices" }, index_data: { mesh: "generated", slab: "indices" } }}
        uniforms={{ data_range: { vertex_start: { start: "vertex_data" }, vertex_end: { end: "vertex_data" }, index_start: { start: "index_data" }, index_end: { end: "index_data" } } }}
      />
      <Mesh3d name="generated" transform={Transform.fromXyz(-2.5, 1.5, 0)}>
        <BufferMesh {...EMPTY} />
        <StandardMaterial base_color={RED_400} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(2.5, 1.5, 0)}>
        <BufferMesh {...EMPTY} />
        <StandardMaterial base_color={SKY_400} />
      </Mesh3d>
      {/* a Circle in 3D: raea's typed Circle only goes in a Mesh2d (conventions C15), so through reflection */}
      <Entity rotation={eulerOf(Quat.fromRotationX(-Math.PI / 2))}>
        <BevyComponent type="bevy_mesh::components::Mesh3d" value={{ primitive: "Circle", radius: 4 }} />
        <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: { Srgba: { red: 1, green: 1, blue: 1, alpha: 1 } } }} />
      </Entity>
      <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4)} />
      <Camera3d transform={Transform.fromXyz(-2.5, 4.5, 9).lookingAt([0, 0, 0])} />
    </Canvas3D>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
