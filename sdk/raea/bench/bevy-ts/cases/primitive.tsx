// gap: G9
// One of Bevy's own primitive shapes as a Mesh handle ({ primitive: "Torus", ...its fields }, Mesh::from): a red ring
// of radius 1 m stood up to face the camera; its hole shows the black behind.
// probe: 50 32 r>150
// probe: 32 32 r<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Entity name="ring" rotation={[Math.PI / 2, 0, 0]}>
          <BevyComponent type="bevy_mesh::components::Mesh3d" value={{ primitive: "Torus", minor_radius: 0.25, major_radius: 1 }} />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: { Srgba: { red: 1, green: 0, blue: 0, alpha: 1 } }, unlit: true }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
