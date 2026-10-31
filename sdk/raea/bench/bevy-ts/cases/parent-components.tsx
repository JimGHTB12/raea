// gap: M5
// A parent entity's own components: <Entity transform> at the left carries a red box as its own Mesh3d and material
// (<BevyComponent> children), and holds a green Mesh3d placed 2.4 m to its right, so red at the left, green at the
// right, black between (docs/sdk.feature M5, the old Group as Bevy's parent entity).
// probe: 14 32 r>150 g<100
// probe: 50 32 g>150 r<100
// probe: 32 32 r<30 g<30
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const red = { Srgba: { red: 1, green: 0, blue: 0, alpha: 1 } };

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <Entity transform={Transform.fromXyz(-1.2, 0, 0)}>
          <BevyComponent type="bevy_mesh::components::Mesh3d" value="shape:box(0.8, 0.8, 0.8)" />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: red, unlit: true }} />
          <Mesh3d transform={Transform.fromXyz(2.4, 0, 0)}><Cuboid half_size={[0.4, 0.4, 0.4]} /><StandardMaterial base_color="#00ff00" unlit /></Mesh3d>
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
