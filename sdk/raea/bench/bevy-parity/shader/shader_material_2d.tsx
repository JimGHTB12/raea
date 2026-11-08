// Bevy v0.20.0 examples/shader/shader_material_2d.rs in raea's TSX (docs/control.feature FC0a, FC2a): a Camera2d and a
// default Rectangle scaled to 128 px with the example's own Material2d, assets/shaders/custom_material_2d.wesl as
// written (its bindings at 0, 1 and 2 of group 2, `import super::custom_material_import` from beside it),
// LinearRgba::BLUE, the Bevy logo as its texture and AlphaMode2d::Mask(0.5), on Bevy's default clear colour, in Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { Mesh2d, Rectangle } from "raea/sprite";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Mesh2d transform={Transform.IDENTITY.withScale([128, 128, 1])}>
          <Rectangle />
          <Material
            fragment={staticFile("shaders/custom_material_2d.wesl")}
            uniforms={{ material_color: [0, 0, 1, 1] }}
            textures={{ base_color_texture: staticFile("branding/icon.png") }}
            alpha_mode={{ Mask: 0.5 }}
          />
        </Mesh2d>
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
