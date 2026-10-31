// gap: G17
// A picture made in code as a Handle<Image> ({ width, height, pixels, nearest }): two pixels, red then green, on a
// 3 m square (Bevy's Rectangle primitive) through an unlit StandardMaterial's base_color_texture, sampled nearest.
// probe: 20 32 r>150 g<80
// probe: 44 32 g>150 r<80
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Entity name="card">
          <BevyComponent type="bevy_mesh::components::Mesh3d" value={{ primitive: "Rectangle", half_size: [1.5, 1.5] }} />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color_texture: { width: 2, height: 1, pixels: ["#ff0000", "#00ff00"], nearest: true }, unlit: true }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
