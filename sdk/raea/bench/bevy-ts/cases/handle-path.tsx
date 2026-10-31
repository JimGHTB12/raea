// gap: G17
// A Handle field by asset path: an inline StandardMaterial whose base_color_texture is "red.png" (next to the edit),
// loaded through the AssetServer and waited for before the frame draws. White base colour, so the red is the texture.
// probe: 32 32 r>200 g<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <Entity name="tile">
          <BevyComponent type="bevy_mesh::components::Mesh3d" value="shape:box(1.5, 1.5, 1.5)" />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color_texture: "red.png", unlit: true }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
