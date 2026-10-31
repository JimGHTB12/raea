// gap: G17
// An Entity field by SDK name: a bare <Entity> made a child of the mesh named "anchor" with Bevy's ChildOf (its
// parent field an entity, given as that mesh's name), so the green cube sits where the anchor is (left), not at the
// origin (the centre).
// probe: 14 32 g>150 r<100
// probe: 32 32 r<30 g<30
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const green = { Srgba: { red: 0, green: 1, blue: 0, alpha: 1 } };

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <Mesh3d transform={Transform.fromXyz(-1.2, 0, 0)} name="anchor"><Cuboid half_size={[0.025, 0.025, 0.025]} /><StandardMaterial base_color="#000000" unlit /></Mesh3d>
        <Entity name="kid">
          <BevyComponent type="bevy_ecs::hierarchy::ChildOf" value="anchor" />
          <BevyComponent type="bevy_mesh::components::Mesh3d" value="shape:box(0.8, 0.8, 0.8)" />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: green, unlit: true }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
