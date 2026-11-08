// Bevy v0.20.0 examples/shader/shader_material.rs in raea's TSX (docs/control.feature FC0a, FC2a): a 1 m cube at
// (0, 0.5, 0) with the example's own Material, assets/shaders/custom_material.wesl as written (its own bindings at 0, 1
// and 2 of the material's group, and `import super::custom_material_import` from beside it), LinearRgba::BLUE, the
// Bevy logo as its texture and AlphaMode::Blend; the camera at (-2, 2.5, 5), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

function Shot() {
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <Material
          fragment={staticFile("shaders/custom_material.wesl")}
          uniforms={{ material_color: [0, 0, 1, 1] }}
          textures={{ material_color_texture: staticFile("branding/icon.png") }}
          alpha_mode="Blend"
        />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={31} fps={60} width={1280} height={720} />);
