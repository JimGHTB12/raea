// Bevy v0.20.0 examples/shader_advanced/mesh_shader_intro.rs in raea's TSX (docs/control.feature FC2d): a mesh-shader
// pipeline drawn after the main opaque pass, a <MeshShaderPass> on the camera with assets/shaders/mesh_shader_intro's
// task.wesl, mesh.wesl and fragment.wesl as written (each importing bindings.wesl beside it), dispatched as one
// workgroup: the task shader asks for u32(globals.time) workgroups on each axis, and each makes a cube of 0.1 m half
// size at its id, coloured by its id over their count and culled where sin(time * 10 + x + y + z) > 0.8. A 0.5 m cube of
// srgb(124, 144, 255) at (0, 0.5, 0), a shadow-casting PointLight at (4, 8, 4), and the camera at (-9, 1, -9) looking
// at (9, 4, 9) with Msaa off, in Bevy's default 1280 x 720 window. The example needs a GPU with mesh shaders.
import { Composition, registerRoot, staticFile } from "raea";
import { BevyComponent, MeshShaderPass } from "raea/3d";
import { Camera3d } from "raea/camera";
import { PointLight } from "raea/light";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

const DIR = "shaders/mesh_shader_intro";

function Shot() {
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <Cuboid half_size={[0.25, 0.25, 0.25]} />
        <StandardMaterial base_color="rgb(124, 144, 255)" />
      </Mesh3d>
      <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4)} />
      <Camera3d transform={Transform.fromXyz(-9, 1, -9).lookingAt([9, 4, 9])}>
        <BevyComponent type="bevy_render::view::Msaa" value="Off" />
        <MeshShaderPass task={staticFile(`${DIR}/task.wesl`)} mesh={staticFile(`${DIR}/mesh.wesl`)} fragment={staticFile(`${DIR}/fragment.wesl`)} />
      </Camera3d>
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={301} fps={60} width={1280} height={720} />);
