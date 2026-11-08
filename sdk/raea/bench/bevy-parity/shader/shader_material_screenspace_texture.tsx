// Bevy v0.20.0 examples/shader/shader_material_screenspace_texture.rs in raea's TSX (docs/control.feature FC0a, FC2a):
// a 5 m plane of srgb(0.3, 0.5, 0.3), a default PointLight at (4, 8, 4), and a 1 m cube at (0, 0.5, 0) with the
// example's own Material, assets/shaders/custom_material_screenspace_texture.wesl as written (it samples the
// FlightHelmet's lens texture at the fragment's viewport uv, view.viewport from mesh_view_bindings); the camera starts
// at (4, 2.5, 4) and turns about the origin at 45 degrees a second (rotate_camera, no delta on the first update).
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

function Shot() {
  const a = ((45 * Math.PI) / 180) * (useCurrentFrame() / 60);
  // Transform::rotate_around(ZERO, rotation about Y by a) of (4, 2.5, 4)
  const [x, z] = [4 * Math.cos(a) + 4 * Math.sin(a), -4 * Math.sin(a) + 4 * Math.cos(a)];
  return (
    <>
      <Mesh3d transform={Transform.IDENTITY}>
        <Plane3d half_size={[2.5, 2.5]} />
        <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
      </Mesh3d>
      <PointLight transform={Transform.fromXyz(4, 8, 4)} />
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <Material
          fragment={staticFile("shaders/custom_material_screenspace_texture.wesl")}
          textures={{ texture: staticFile("models/FlightHelmet/FlightHelmet_Materials_LensesMat_OcclusionRoughMetal.png") }}
        />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(x, 2.5, z).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
