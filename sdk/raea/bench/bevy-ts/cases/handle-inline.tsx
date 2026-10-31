// gap: G17
// A bare <Entity> given Bevy's own Mesh3d and MeshMaterial3d<StandardMaterial> through the TypeRegistry: the mesh's
// handle from a Manifold shape ("shape:box(...)"), the material's from an inline asset (its fields as JSON), each
// added to Assets<T> by the reflect deserializer's handle processor.
// probe: 32 32 r>200 g<60
// probe: 4 4 r<30 g<30
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const red = { Srgba: { red: 1, green: 0, blue: 0, alpha: 1 } };

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <Entity name="cube">
          <BevyComponent type="bevy_mesh::components::Mesh3d" value="shape:box(1.5, 1.5, 1.5)" />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: red, unlit: true }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
