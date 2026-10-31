// gap: G7
// A camera rendering into a texture: a camera on a bare <Entity> (order -1) draws a green cube 20 m off into the image
// "render:screen:64x64" (its RenderTarget), and a screen in front of the main camera shows that image as its
// base_color_texture (an inline StandardMaterial, unlit). The main camera sees only the screen, so the green comes
// through the texture.
// probe: 32 32 g>150 r<80
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Entity name="feed" position={[20, 0, 3]}>
          <BevyComponent type="bevy_camera::components::Camera3d" value={{}} />
          <BevyComponent type="bevy_camera::camera::Camera" value={{ order: -1, clear_color: { Custom: { Srgba: { red: 0, green: 0, blue: 0, alpha: 1 } } } }} />
          <BevyComponent type="bevy_camera::camera::RenderTarget" value={{ Image: { handle: "render:screen:64x64", scale_factor: 1 } }} />
        </Entity>
        <Mesh3d transform={Transform.fromXyz(20, 0, 0)} name="green"><Cuboid half_size={[1.5, 1.5, 0.1]} /><StandardMaterial base_color="#00ff00" unlit /></Mesh3d>
        <Entity name="screen">
          <BevyComponent type="bevy_mesh::components::Mesh3d" value={"shape:box(3, 3, 0.05)"} />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color_texture: "render:screen", unlit: true }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={3} fps={30} width={64} height={64} />);
