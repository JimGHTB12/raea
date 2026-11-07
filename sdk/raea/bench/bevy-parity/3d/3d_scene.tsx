// Bevy v0.20.0 examples/3d/3d_scene.rs in raea's TSX (docs/control.feature FC0a): a white circle of radius 4 lying
// flat, a 1 m cube of srgb(124, 144, 255) on it, a shadow-casting PointLight at (4, 8, 4) with Bevy's default
// 1 000 000 lm, and the camera at (-2.5, 4.5, 9) looking at the origin, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot } from "raea";
import { BevyComponent, Entity } from "raea/3d";
import { Transform, Quat, eulerOf } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

function Shot() {
  return (
    <>
      {/* a Circle in 3D: raea's typed Circle only goes in a Mesh2d (conventions C15), so through reflection */}
      <Entity rotation={eulerOf(Quat.fromRotationX(-Math.PI / 2))}>
        <BevyComponent type="bevy_mesh::components::Mesh3d" value={{ primitive: "Circle", radius: 4 }} />
        <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: { Srgba: { red: 1, green: 1, blue: 1, alpha: 1 } } }} />
      </Entity>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color="rgb(124, 144, 255)" />
      </Mesh3d>
      <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4)} />
      <Camera3d transform={Transform.fromXyz(-2.5, 4.5, 9).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
